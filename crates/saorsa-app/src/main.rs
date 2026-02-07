//! saorsa-app: The AI coding agent application.

use std::io::Write;

use anyhow::Context;
use crossterm::event::EventStream;
use futures::StreamExt;
use tracing::debug;

use saorsa_agent::{
    AgentConfig, AgentEvent, AgentLoop, BashTool, Message, SessionStorage, ToolRegistry,
    TurnEndReason, event_channel, find_last_active_session, find_session_by_prefix,
    restore_session,
};
use saorsa_ai::{AnthropicProvider, ProviderConfig, ProviderKind};
use saorsa_core::render_context::RenderContext;
use saorsa_core::terminal::{CrosstermBackend, Terminal};

use saorsa_app::app::{AppState, AppStatus};
use saorsa_app::cli::Cli;
use saorsa_app::input::{InputAction, handle_event};
use saorsa_app::ui;

/// Type alias for session loading result
type SessionLoadResult = Result<Option<(String, Vec<Message>)>, Box<dyn std::error::Error>>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing.
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "warn".into()),
        )
        .init();

    let cli = Cli::parse_args();

    let api_key = cli
        .api_key()
        .map_err(|e| anyhow::anyhow!("{e}"))?
        .to_string();

    // Print mode: single prompt, no TUI.
    if let Some(prompt) = &cli.print {
        return run_print_mode(&cli, &api_key, prompt).await;
    }

    // Interactive mode.
    run_interactive(&cli, &api_key).await
}

/// Run in print mode: send a single prompt and print the response.
async fn run_print_mode(cli: &Cli, api_key: &str, prompt: &str) -> anyhow::Result<()> {
    let provider_config = ProviderConfig::new(ProviderKind::Anthropic, api_key, &cli.model);
    let provider = AnthropicProvider::new(provider_config).context("Failed to create provider")?;

    let agent_config = AgentConfig::new(&cli.model)
        .system_prompt(&cli.system_prompt)
        .max_turns(cli.max_turns)
        .max_tokens(cli.max_tokens);

    let mut tools = ToolRegistry::new();
    tools.register(Box::new(BashTool::new(
        std::env::current_dir().context("Failed to get current directory")?,
    )));

    let (event_tx, mut event_rx) = event_channel(256);

    let mut agent = AgentLoop::new(Box::new(provider), agent_config, tools, event_tx);

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
async fn run_interactive(cli: &Cli, api_key: &str) -> anyhow::Result<()> {
    let mut state = AppState::new(&cli.model);

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
                "Connected to {}. Type a message to start.",
                cli.model
            ));
        }
    } else {
        state.add_system_message(format!(
            "Connected to {} (ephemeral mode). Type a message to start.",
            cli.model
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
    let api_key = api_key.to_string();
    let model = cli.model.clone();
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
    api_key: &str,
    model: &str,
    system_prompt: &str,
    max_turns: u32,
    max_tokens: u32,
    prompt: &str,
) {
    let provider_config = ProviderConfig::new(ProviderKind::Anthropic, api_key, model);
    let provider = match AnthropicProvider::new(provider_config) {
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

    let mut tools = ToolRegistry::new();
    if let Ok(cwd) = std::env::current_dir() {
        tools.register(Box::new(BashTool::new(cwd)));
    }

    let (event_tx, mut event_rx) = event_channel(256);

    let mut agent = AgentLoop::new(Box::new(provider), agent_config, tools, event_tx);

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
