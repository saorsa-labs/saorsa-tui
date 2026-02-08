//! saorsa: The AI coding agent application.

use std::io::Write;

use anyhow::Context;
use crossterm::event::EventStream;
use futures::StreamExt;
use tracing::debug;

use saorsa_agent::{
    AgentConfig, AgentEvent, AgentLoop, AuthConfig, Message, SessionStorage, Settings,
    TurnEndReason, default_tools, ensure_config_dir, event_channel, find_last_active_session,
    find_session_by_prefix, restore_session,
};
use saorsa_ai::{ProviderConfig, ProviderKind, ProviderRegistry, determine_provider};
use saorsa_core::render_context::RenderContext;
use saorsa_core::terminal::{CrosstermBackend, Terminal};

use saorsa::app::{AppState, AppStatus};
use saorsa::cli::Cli;
use saorsa::input::{InputAction, handle_event};
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

    // Initial render.
    render_ui(&state, &mut ctx, &mut backend);

    let mut event_stream = EventStream::new();

    // Store provider config for creating agents per interaction.
    let mut api_key = api_key.to_string();
    let mut model = initial_model.to_string();
    let system_prompt = cli.system_prompt.clone();
    let max_turns = cli.max_turns;
    let max_tokens = cli.max_tokens;

    loop {
        tokio::select! {
            maybe_event = event_stream.next() => {
                let Some(Ok(ct_event)) = maybe_event else {
                    break;
                };

                let event = saorsa_core::event::Event::from(ct_event);
                let action = handle_event(&mut state, &event);

                match action {
                    InputAction::Quit => break,
                    InputAction::Submit(text) => {
                        state.add_user_message(&text);
                        state.status = AppStatus::Thinking;
                        render_ui(&state, &mut ctx, &mut backend);

                        // Run agent interaction.
                        run_agent_interaction(
                            &mut state,
                            &mut ctx,
                            &mut backend,
                            provider_kind,
                            &api_key,
                            &model,
                            &system_prompt,
                            max_turns,
                            max_tokens,
                            &text,
                        ).await;

                        state.status = AppStatus::Idle;
                        state.streaming_text.clear();
                        render_ui(&state, &mut ctx, &mut backend);
                    }
                    InputAction::Redraw => {
                        render_ui(&state, &mut ctx, &mut backend);
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
                        render_ui(&state, &mut ctx, &mut backend);
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
                        render_ui(&state, &mut ctx, &mut backend);
                    }
                    InputAction::None => {}
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

/// Run a single agent interaction (user sends message, agent responds).
#[allow(clippy::too_many_arguments)]
async fn run_agent_interaction(
    state: &mut AppState,
    ctx: &mut RenderContext,
    backend: &mut CrosstermBackend,
    provider_kind: ProviderKind,
    api_key: &str,
    model: &str,
    system_prompt: &str,
    max_turns: u32,
    max_tokens: u32,
    prompt: &str,
) {
    let registry = ProviderRegistry::default();
    let provider_config = ProviderConfig::new(provider_kind, api_key, model);
    let provider = match registry.create(provider_config) {
        Ok(p) => p,
        Err(e) => {
            state.add_system_message(format!("Provider error: {e}"));
            return;
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

    let (event_tx, mut event_rx) = event_channel(256);

    let mut agent = AgentLoop::new(provider, agent_config, tools, event_tx);

    let prompt = prompt.to_string();

    // Spawn agent task.
    let agent_handle = tokio::spawn(async move { agent.run(&prompt).await });

    // Process events as they arrive.
    while let Some(event) = event_rx.recv().await {
        match event {
            AgentEvent::TextDelta { text } => {
                state.streaming_text.push_str(&text);
                render_ui(state, ctx, backend);
            }
            AgentEvent::ToolCall { name, .. } => {
                state.status = AppStatus::ToolRunning {
                    tool_name: name.clone(),
                };
                render_ui(state, ctx, backend);
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
                render_ui(state, ctx, backend);
            }
            AgentEvent::TextComplete { text } => {
                state.streaming_text.clear();
                state.add_assistant_message(text);
                render_ui(state, ctx, backend);
            }
            AgentEvent::TurnEnd {
                reason: TurnEndReason::MaxTurns,
                ..
            } => {
                state.add_system_message("(max turns reached)");
            }
            AgentEvent::Error { message } => {
                state.add_system_message(format!("Error: {message}"));
                render_ui(state, ctx, backend);
            }
            _ => {}
        }
    }

    // Wait for agent to finish.
    match agent_handle.await {
        Ok(Ok(_)) => {}
        Ok(Err(e)) => {
            state.add_system_message(format!("Agent error: {e}"));
        }
        Err(e) => {
            state.add_system_message(format!("Agent task error: {e}"));
        }
    }
}

/// Render the UI and flush to terminal.
fn render_ui(state: &AppState, ctx: &mut RenderContext, backend: &mut dyn Terminal) {
    ctx.begin_frame();
    ui::render(state, ctx.buffer_mut());
    if let Err(e) = ctx.end_frame(backend) {
        debug!(error = %e, "Render error");
    }
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
