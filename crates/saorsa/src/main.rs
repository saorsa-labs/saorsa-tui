//! saorsa: The AI coding agent application.

use std::io::Write;
use std::time::Duration;

use anyhow::Context;
use crossterm::event::EventStream;
use futures::StreamExt;
use tracing::debug;

use saorsa_agent::{
    AgentConfig, AgentEvent, AgentLoop, AuthConfig, EventReceiver, Message, SessionStorage,
    Settings, TurnEndReason, default_tools, ensure_config_dir, event_channel,
    find_last_active_session, find_session_by_prefix, restore_session,
};
use saorsa_ai::{ProviderConfig, ProviderKind, ProviderRegistry, determine_provider};
use saorsa_core::render_context::RenderContext;
use saorsa_core::terminal::{CrosstermBackend, Terminal};

use saorsa::app::{AppState, AppStatus, OverlayMode};
use saorsa::cli::Cli;
use saorsa::commands::{self, CommandResult};
use saorsa::input::{InputAction, handle_event};
use saorsa::render_throttle::RenderThrottle;
use saorsa::ui;

/// Type alias for session loading result.
type SessionLoadResult = Result<Option<(String, Vec<Message>)>, Box<dyn std::error::Error>>;

/// Resolve the API key from CLI, auth.json, or environment variable.
///
/// Resolution order:
/// 1. CLI `--api-key` argument
/// 2. `auth.json` lookup by provider display name (lowercase)
/// 3. Environment variable for the provider kind
fn resolve_api_key(
    cli_key: Option<&str>,
    auth_config: &AuthConfig,
    provider_kind: ProviderKind,
) -> anyhow::Result<String> {
    // 1. CLI argument wins.
    if let Some(key) = cli_key {
        return Ok(key.to_string());
    }

    // 2. Lookup in auth.json by lowercase provider display name.
    let provider_name = provider_kind.display_name().to_lowercase();
    if let Ok(key) = saorsa_agent::config::auth::get_key(auth_config, &provider_name) {
        return Ok(key);
    }

    // 3. Environment variable fallback.
    let env_name = provider_kind.env_var_name();
    if let Ok(key) = std::env::var(env_name) {
        return Ok(key);
    }

    Err(anyhow::anyhow!(
        "No API key found for {}. Set {} env var, use --api-key, \
         or add '{}' to ~/.saorsa/auth.json",
        provider_kind.display_name(),
        env_name,
        provider_name,
    ))
}

/// Parse a provider name string into a [`ProviderKind`].
fn parse_provider_kind(name: &str) -> Option<ProviderKind> {
    match name.to_lowercase().as_str() {
        "anthropic" => Some(ProviderKind::Anthropic),
        "openai" => Some(ProviderKind::OpenAi),
        "gemini" | "google" => Some(ProviderKind::Gemini),
        "ollama" => Some(ProviderKind::Ollama),
        "openai-compatible" | "openai_compatible" => Some(ProviderKind::OpenAiCompatible),
        "lmstudio" | "lm-studio" | "lm_studio" => Some(ProviderKind::LmStudio),
        "vllm" => Some(ProviderKind::Vllm),
        "openrouter" => Some(ProviderKind::OpenRouter),
        _ => None,
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing.
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "warn".into()),
        )
        .init();

    let cli = Cli::parse_args();

    // Ensure ~/.saorsa/ exists.
    let config_dir = ensure_config_dir().map_err(|e| anyhow::anyhow!("{e}"))?;

    // Load configurations.
    let auth_config = saorsa_agent::config::auth::load(&config_dir.join("auth.json"))
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    let settings = saorsa_agent::config::settings::load(&config_dir.join("settings.json"))
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    // Resolve model: CLI --model > settings default_model > hardcoded default.
    let model = if cli.model != "claude-sonnet-4-5-20250929" {
        // User explicitly provided --model (not the default value).
        cli.model.clone()
    } else if let Some(ref default_model) = settings.default_model {
        default_model.clone()
    } else {
        cli.model.clone()
    };

    // Resolve provider kind: CLI --provider > determine_provider(model) > OpenAiCompatible.
    let provider_kind = if let Some(ref provider_name) = cli.provider {
        parse_provider_kind(provider_name)
            .ok_or_else(|| anyhow::anyhow!("Unknown provider: {provider_name}"))?
    } else {
        determine_provider(&model).unwrap_or(ProviderKind::OpenAiCompatible)
    };

    // Resolve API key.
    let api_key = resolve_api_key(cli.api_key(), &auth_config, provider_kind)?;

    // Show models mode: list all known models and exit.
    if cli.show_models {
        return run_show_models();
    }

    // Print mode: single prompt, no TUI.
    if let Some(prompt) = &cli.print {
        return run_print_mode(&cli, provider_kind, &api_key, &model, prompt).await;
    }

    // Interactive mode.
    run_interactive(&cli, &settings, provider_kind, &api_key, &model).await
}

/// Run in print mode: send a single prompt and print the response.
async fn run_print_mode(
    cli: &Cli,
    provider_kind: ProviderKind,
    api_key: &str,
    model: &str,
    prompt: &str,
) -> anyhow::Result<()> {
    let registry = ProviderRegistry::default();
    let provider_config = ProviderConfig::new(provider_kind, api_key, model);
    let provider = registry
        .create(provider_config)
        .map_err(|e| anyhow::anyhow!("{e}"))
        .context("Failed to create provider")?;

    let agent_config = AgentConfig::new(model)
        .system_prompt(&cli.system_prompt)
        .max_turns(cli.max_turns)
        .max_tokens(cli.max_tokens);

    let tools = if let Ok(cwd) = std::env::current_dir() {
        default_tools(cwd)
    } else {
        default_tools(std::path::PathBuf::from("."))
    };

    let (event_tx, mut event_rx) = event_channel(256);

    let mut agent = AgentLoop::new(provider, agent_config, tools, event_tx);

    // Spawn event consumer that prints to stdout.
    let print_handle = tokio::spawn(async move {
        let mut stdout = std::io::stdout();
        while let Some(event) = event_rx.recv().await {
            match event {
                AgentEvent::TextDelta { text } => {
                    let _ = write!(stdout, "{text}");
                    let _ = stdout.flush();
                }
                AgentEvent::TurnEnd { .. } => {}
                AgentEvent::Error { message } => {
                    eprintln!("\nError: {message}");
                }
                _ => {}
            }
        }
        let _ = writeln!(stdout);
    });

    let result = agent.run(prompt).await;

    // Drop agent to close event channel.
    drop(agent);
    let _ = print_handle.await;

    result.map(|_| ()).map_err(|e| anyhow::anyhow!("{e}"))
}

/// Run in show-models mode: list all known models and exit.
fn run_show_models() -> anyhow::Result<()> {
    use saorsa_ai::models::all_models;

    println!("Known models:\n");
    println!(
        "  {:<45} {:<20} {:>8}  {:>6}  {:>6}",
        "Model", "Provider", "Context", "Tools", "Vision"
    );
    println!("  {}", "-".repeat(95));

    for model in all_models() {
        let tools = if model.supports_tools { "yes" } else { "no" };
        let vision = if model.supports_vision { "yes" } else { "no" };
        println!(
            "  {:<45} {:<20} {:>8}  {:>6}  {:>6}",
            model.name,
            model.provider.display_name(),
            model.context_window,
            tools,
            vision,
        );
    }
    println!("\n  Total: {} models", all_models().len());
    Ok(())
}

/// Run in interactive TUI mode.
async fn run_interactive(
    cli: &Cli,
    settings: &Settings,
    mut provider_kind: ProviderKind,
    api_key: &str,
    initial_model: &str,
) -> anyhow::Result<()> {
    let mut state = AppState::new(initial_model);

    // Populate enabled_models from settings.
    state.enabled_models = settings.enabled_models.clone();

    // Set model_index to the position of the current model in the list, or 0.
    state.model_index = state
        .enabled_models
        .iter()
        .position(|m| m == initial_model)
        .unwrap_or(0);

    // Handle session continuation/resumption
    if !cli.ephemeral {
        if let Some(resume_prefix) = &cli.resume {
            // Resume specific session by prefix
            match load_session_by_prefix(resume_prefix) {
                Ok((session_id, messages)) => {
                    for msg in messages {
                        add_message_to_state(&mut state, &msg);
                    }
                    state.add_system_message(format!(
                        "Resumed session {} ({} messages loaded)",
                        session_id,
                        state.messages.len()
                    ));
                }
                Err(e) => {
                    state.add_system_message(format!("Failed to resume session: {}", e));
                }
            }
        } else if cli.continue_session {
            // Continue most recent session
            match load_last_active_session() {
                Ok(Some((session_id, messages))) => {
                    for msg in messages {
                        add_message_to_state(&mut state, &msg);
                    }
                    state.add_system_message(format!(
                        "Continued session {} ({} messages loaded)",
                        session_id,
                        state.messages.len()
                    ));
                }
                Ok(None) => {
                    state.add_system_message("No previous sessions found. Starting new session.");
                }
                Err(e) => {
                    state.add_system_message(format!("Failed to continue session: {}", e));
                }
            }
        } else {
            state.add_system_message(format!(
                "Connected to {} ({}). Type a message to start.",
                state.model,
                provider_kind.display_name(),
            ));
        }
    } else {
        state.add_system_message(format!(
            "Connected to {} ({}, ephemeral mode). Type a message to start.",
            state.model,
            provider_kind.display_name(),
        ));
    }

    // Set up terminal.
    let mut backend = CrosstermBackend::new();
    let mut ctx = RenderContext::new(&backend).context("Failed to initialize render context")?;
    backend
        .enter_raw_mode()
        .context("Failed to enter raw mode")?;
    backend.enable_mouse().context("Failed to enable mouse")?;

    // Render throttle: cap at 30fps to reduce CPU during streaming.
    let mut throttle = RenderThrottle::default_fps();

    // Initial render (always allowed — throttle starts ready).
    render_ui(&state, &mut ctx, &mut backend);
    throttle.mark_rendered();

    let mut event_stream = EventStream::new();

    // Tick interval for flushing batched stream text and rendering.
    let mut tick_interval = tokio::time::interval(Duration::from_millis(33));
    // Don't pile up ticks while busy processing events.
    tick_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    // Store provider config for creating agents per interaction.
    let mut api_key = api_key.to_string();
    let mut model = initial_model.to_string();
    let system_prompt = cli.system_prompt.clone();
    let max_turns = cli.max_turns;
    let max_tokens = cli.max_tokens;

    // Autocomplete provider.
    let autocomplete = saorsa::autocomplete::Autocomplete::new();

    // Active agent session (None when idle, Some when agent is running).
    let mut agent_rx: Option<EventReceiver> = None;
    let mut agent_handle: Option<tokio::task::JoinHandle<saorsa_agent::Result<String>>> = None;

    loop {
        tokio::select! {
            // Periodic tick: flush batched stream text and render if dirty.
            _ = tick_interval.tick() => {
                state.flush_stream_text();
                maybe_render_ui(&mut state, &mut ctx, &mut backend, &mut throttle);
            }

            // Agent events (only when an agent is running).
            maybe_agent_event = async {
                if let Some(rx) = agent_rx.as_mut() {
                    rx.recv().await
                } else {
                    // No agent running — pend forever (never completes).
                    std::future::pending::<Option<AgentEvent>>().await
                }
            } => {
                match maybe_agent_event {
                    Some(event) => {
                        handle_agent_event(&mut state, &mut ctx, &mut backend, &mut throttle, event);
                    }
                    None => {
                        // Channel closed — agent finished.
                        state.flush_stream_text();
                        state.streaming_text.clear();
                        state.status = AppStatus::Idle;
                        state.mark_dirty();

                        // Collect agent result.
                        if let Some(handle) = agent_handle.take() {
                            match handle.await {
                                Ok(Ok(_)) => {}
                                Ok(Err(e)) => {
                                    state.add_system_message(format!("Agent error: {e}"));
                                }
                                Err(e) => {
                                    state.add_system_message(format!("Agent task error: {e}"));
                                }
                            }
                        }
                        agent_rx = None;

                        // Force immediate render to show Idle status.
                        render_ui(&state, &mut ctx, &mut backend);
                        throttle.mark_rendered();
                    }
                }
            }

            maybe_event = event_stream.next() => {
                let Some(Ok(ct_event)) = maybe_event else {
                    break;
                };

                let event = saorsa_core::event::Event::from(ct_event);
                let action = handle_event(&mut state, &event);

                match action {
                    InputAction::Quit => break,
                    InputAction::Submit(text) => {
                        // Only allow submit when idle (no agent running).
                        if agent_rx.is_some() {
                            continue;
                        }

                        // Try slash command dispatch first.
                        if let Some(cmd_result) = commands::dispatch(&text, &mut state) {
                            state.add_user_message(&text);
                            match cmd_result {
                                CommandResult::Message(msg) => {
                                    state.add_system_message(msg);
                                }
                                CommandResult::ClearMessages(msg) => {
                                    state.messages.clear();
                                    state.scroll_to_bottom();
                                    state.add_system_message(msg);
                                }
                            }
                            state.mark_dirty();
                            render_ui(&state, &mut ctx, &mut backend);
                            throttle.mark_rendered();
                            continue;
                        }

                        // Not a command — send to the AI agent.
                        state.add_user_message(&text);
                        state.status = AppStatus::Thinking;
                        state.mark_dirty();
                        // Force immediate render so user sees "Thinking" instantly.
                        render_ui(&state, &mut ctx, &mut backend);
                        throttle.mark_rendered();

                        // Start agent (non-blocking).
                        let (rx, handle) = start_agent(
                            provider_kind,
                            &api_key,
                            &model,
                            &system_prompt,
                            max_turns,
                            max_tokens,
                            &text,
                        );
                        agent_rx = Some(rx);
                        agent_handle = Some(handle);
                    }
                    InputAction::Redraw => {
                        state.mark_dirty();
                        // Redraw is explicit — bypass throttle.
                        render_ui(&state, &mut ctx, &mut backend);
                        throttle.mark_rendered();
                    }
                    InputAction::CycleModel => {
                        if let Some(new_model) = state.cycle_model_forward() {
                            let new_model = new_model.to_string();
                            let new_kind = determine_provider(&new_model)
                                .unwrap_or(ProviderKind::OpenAiCompatible);
                            // Re-resolve API key for the new provider.
                            // Load auth config fresh in case it changed.
                            if let Ok(config_dir) = ensure_config_dir()
                                .map_err(|e| anyhow::anyhow!("{e}"))
                            {
                                let auth_config = saorsa_agent::config::auth::load(
                                    &config_dir.join("auth.json"),
                                ).ok().unwrap_or_default();
                                if let Ok(key) = resolve_api_key(None, &auth_config, new_kind) {
                                    api_key = key;
                                }
                            }
                            provider_kind = new_kind;
                            model = new_model.clone();
                            state.model = new_model.clone();
                            state.add_system_message(format!(
                                "Switched to {} ({})",
                                new_model,
                                provider_kind.display_name(),
                            ));
                        } else {
                            state.add_system_message(
                                "No other models configured. Add models to ~/.saorsa/settings.json"
                            );
                        }
                        maybe_render_ui(&mut state, &mut ctx, &mut backend, &mut throttle);
                    }
                    InputAction::CycleModelBackward => {
                        if let Some(new_model) = state.cycle_model_backward() {
                            let new_model = new_model.to_string();
                            let new_kind = determine_provider(&new_model)
                                .unwrap_or(ProviderKind::OpenAiCompatible);
                            if let Ok(config_dir) = ensure_config_dir()
                                .map_err(|e| anyhow::anyhow!("{e}"))
                            {
                                let auth_config = saorsa_agent::config::auth::load(
                                    &config_dir.join("auth.json"),
                                ).ok().unwrap_or_default();
                                if let Ok(key) = resolve_api_key(None, &auth_config, new_kind) {
                                    api_key = key;
                                }
                            }
                            provider_kind = new_kind;
                            model = new_model.clone();
                            state.model = new_model.clone();
                            state.add_system_message(format!(
                                "Switched to {} ({})",
                                new_model,
                                provider_kind.display_name(),
                            ));
                        } else {
                            state.add_system_message(
                                "No other models configured. Add models to ~/.saorsa/settings.json"
                            );
                        }
                        maybe_render_ui(&mut state, &mut ctx, &mut backend, &mut throttle);
                    }
                    InputAction::TabComplete => {
                        let suggestions = autocomplete.suggest(&state.input);
                        if suggestions.len() == 1 {
                            // Single match: replace input with suggestion.
                            state.input = suggestions[0].text.clone();
                            state.cursor = state.input.len();
                            state.mark_dirty();
                        } else if suggestions.len() > 1 {
                            // Multiple matches: find common prefix and complete to that.
                            let common = common_prefix(
                                &suggestions.iter().map(|s| s.text.as_str()).collect::<Vec<_>>(),
                            );
                            if common.len() > state.input.len() {
                                state.input = common;
                                state.cursor = state.input.len();
                            }
                            // Show available completions as a system message.
                            let list: Vec<&str> = suggestions.iter()
                                .map(|s| s.text.as_str())
                                .collect();
                            state.add_system_message(format!("Completions: {}", list.join("  ")));
                            state.mark_dirty();
                        }
                        render_ui(&state, &mut ctx, &mut backend);
                        throttle.mark_rendered();
                    }
                    InputAction::OpenModelSelector => {
                        state.overlay_mode = OverlayMode::ModelSelector;
                        state.add_system_message(
                            "Model selector opened. (Widget integration pending — use /model to switch.)"
                        );
                        state.mark_dirty();
                        render_ui(&state, &mut ctx, &mut backend);
                        throttle.mark_rendered();
                    }
                    InputAction::ScrollUp(lines) => {
                        state.scroll_up(lines);
                        maybe_render_ui(&mut state, &mut ctx, &mut backend, &mut throttle);
                    }
                    InputAction::ScrollDown(lines) => {
                        state.scroll_down(lines);
                        maybe_render_ui(&mut state, &mut ctx, &mut backend, &mut throttle);
                    }
                    InputAction::None => {
                        // Input events (typing) mark dirty via AppState methods.
                        // Let the tick handle rendering.
                    }
                }

                if state.should_quit {
                    break;
                }
            }
        }
    }

    // Restore terminal.
    backend.disable_mouse().ok();
    backend.exit_raw_mode().ok();

    Ok(())
}

/// Start an agent interaction and return the event receiver and join handle.
///
/// The agent runs in a background tokio task. Events arrive via the returned
/// receiver and should be processed by the main event loop using
/// [`handle_agent_event`].
#[allow(clippy::too_many_arguments)]
fn start_agent(
    provider_kind: ProviderKind,
    api_key: &str,
    model: &str,
    system_prompt: &str,
    max_turns: u32,
    max_tokens: u32,
    prompt: &str,
) -> (
    EventReceiver,
    tokio::task::JoinHandle<saorsa_agent::Result<String>>,
) {
    let registry = ProviderRegistry::default();
    let provider_config = ProviderConfig::new(provider_kind, api_key, model);
    let provider = match registry.create(provider_config) {
        Ok(p) => p,
        Err(e) => {
            // Send error through channel so main loop handles it normally.
            let (tx, rx) = event_channel(1);
            let err_msg = e.to_string();
            let handle = tokio::spawn(async move {
                let _ = tx
                    .send(AgentEvent::Error {
                        message: err_msg.clone(),
                    })
                    .await;
                Err(saorsa_agent::SaorsaAgentError::Internal(err_msg))
            });
            return (rx, handle);
        }
    };

    let agent_config = AgentConfig::new(model)
        .system_prompt(system_prompt)
        .max_turns(max_turns)
        .max_tokens(max_tokens);

    let tools = if let Ok(cwd) = std::env::current_dir() {
        default_tools(cwd)
    } else {
        default_tools(std::path::PathBuf::from("."))
    };

    let (event_tx, event_rx) = event_channel(256);

    let mut agent = AgentLoop::new(provider, agent_config, tools, event_tx);
    let prompt = prompt.to_string();

    let handle = tokio::spawn(async move { agent.run(&prompt).await });

    (event_rx, handle)
}

/// Process a single agent event, updating state and rendering as needed.
///
/// TextDelta events are batched via [`AppState::accumulate_stream_text`] and
/// flushed at frame boundaries. Other events (tool calls, completions, errors)
/// trigger immediate renders since they are low-frequency and should be visible
/// instantly.
fn handle_agent_event(
    state: &mut AppState,
    ctx: &mut RenderContext,
    backend: &mut CrosstermBackend,
    throttle: &mut RenderThrottle,
    event: AgentEvent,
) {
    match event {
        AgentEvent::TextDelta { text } => {
            tracing::trace!(len = text.len(), "TextDelta accumulated");
            state.accumulate_stream_text(&text);
        }
        AgentEvent::ToolCall { name, .. } => {
            state.flush_stream_text();
            state.status = AppStatus::ToolRunning {
                tool_name: name.clone(),
            };
            state.mark_dirty();
            render_ui(state, ctx, backend);
            throttle.mark_rendered();
        }
        AgentEvent::ToolResult {
            name,
            output,
            success,
            ..
        } => {
            let display = if output.len() > 200 {
                format!("{}...", &output[..200])
            } else {
                output
            };
            let status = if success { "" } else { " (failed)" };
            state.add_tool_message(&name, format!("{display}{status}"));
            state.status = AppStatus::Thinking;
            state.mark_dirty();
            render_ui(state, ctx, backend);
            throttle.mark_rendered();
        }
        AgentEvent::TextComplete { text } => {
            state.flush_stream_text();
            state.streaming_text.clear();
            state.add_assistant_message(text);
            state.mark_dirty();
            render_ui(state, ctx, backend);
            throttle.mark_rendered();
        }
        AgentEvent::TurnEnd {
            reason: TurnEndReason::MaxTurns,
            ..
        } => {
            state.add_system_message("(max turns reached)");
        }
        AgentEvent::Error { message } => {
            state.add_system_message(format!("Error: {message}"));
            state.mark_dirty();
            render_ui(state, ctx, backend);
            throttle.mark_rendered();
        }
        _ => {}
    }
}

/// Render the UI and flush to terminal.
fn render_ui(state: &AppState, ctx: &mut RenderContext, backend: &mut dyn Terminal) {
    debug!("Rendering frame");
    ctx.begin_frame();
    ui::render(state, ctx.buffer_mut());
    if let Err(e) = ctx.end_frame(backend) {
        debug!(error = %e, "Render error");
    }
}

/// Render the UI if the frame rate limit allows and the state has changed.
///
/// Returns `true` if a render was performed.
fn maybe_render_ui(
    state: &mut AppState,
    ctx: &mut RenderContext,
    backend: &mut CrosstermBackend,
    throttle: &mut RenderThrottle,
) -> bool {
    if throttle.should_render() && state.take_dirty() {
        render_ui(state, ctx, backend);
        throttle.mark_rendered();
        return true;
    }
    debug!("Render skipped (throttled or clean)");
    false
}

/// Load the most recently active session.
fn load_last_active_session() -> SessionLoadResult {
    let storage = SessionStorage::new()?;
    let session_id: saorsa_agent::SessionId = match find_last_active_session(&storage)? {
        Some(id) => id,
        None => return Ok(None),
    };
    let (_metadata, messages) = restore_session(&storage, &session_id)?;
    Ok(Some((session_id.prefix(), messages)))
}

/// Load a session by ID prefix.
fn load_session_by_prefix(
    prefix: &str,
) -> Result<(String, Vec<Message>), Box<dyn std::error::Error>> {
    let storage = SessionStorage::new()?;
    let session_id = find_session_by_prefix(&storage, prefix)?;
    let (_metadata, messages) = restore_session(&storage, &session_id)?;
    Ok((session_id.prefix(), messages))
}

/// Find the longest common prefix among a set of strings.
fn common_prefix(strings: &[&str]) -> String {
    if strings.is_empty() {
        return String::new();
    }
    let first = strings[0];
    let mut len = first.len();
    for s in &strings[1..] {
        len = len.min(s.len());
        for (i, (a, b)) in first.bytes().zip(s.bytes()).enumerate() {
            if a != b {
                len = len.min(i);
                break;
            }
        }
    }
    first[..len].to_string()
}

/// Add a session message to app state.
fn add_message_to_state(state: &mut AppState, msg: &Message) {
    match msg {
        Message::User { content, .. } => {
            state.add_user_message(content);
        }
        Message::Assistant { content, .. } => {
            state.add_assistant_message(content);
        }
        Message::ToolCall { tool_name, .. } => {
            state.add_tool_message(tool_name, "(tool called)");
        }
        Message::ToolResult {
            tool_name, result, ..
        } => {
            let result_str = result.to_string();
            let display = if result_str.len() > 200 {
                format!("{}...", &result_str[..200])
            } else {
                result_str
            };
            state.add_tool_message(tool_name, display);
        }
    }
}
