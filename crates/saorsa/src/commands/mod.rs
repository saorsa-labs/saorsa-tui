//! Application commands for the saorsa AI agent.
//!
//! Commands are invoked by typing `/command [args]` in the input field.
//! The [`dispatch`] function parses the input and routes to the appropriate
//! handler.

pub mod agents;
pub mod bookmark;
pub mod clear;
pub mod compact;
pub mod cost;
pub mod export;
pub mod fork;
pub mod help;
pub mod hotkeys;
pub mod login;
pub mod logout;
pub mod model;
pub mod providers;
pub mod settings;
pub mod share;
pub mod skills;
pub mod status;
pub mod thinking;
pub mod tree;

pub use bookmark::BookmarkCommand;
pub use export::ExportCommand;
pub use fork::ForkCommand;
pub use tree::TreeCommand;

use crate::app::AppState;

/// Result of executing a slash command.
#[derive(Debug)]
pub enum CommandResult {
    /// Display a system message.
    Message(String),
    /// Clear the message history and display a confirmation.
    ClearMessages(String),
}

/// Try to dispatch a slash command from user input.
///
/// Returns `None` if the input is not a slash command (doesn't start with `/`).
/// Returns `Some(CommandResult)` with the command output.
pub fn dispatch(input: &str, state: &mut AppState) -> Option<CommandResult> {
    let trimmed = input.trim();
    if !trimmed.starts_with('/') {
        return None;
    }

    // Split into command name and arguments.
    let (cmd, args) = match trimmed.find(char::is_whitespace) {
        Some(pos) => (&trimmed[..pos], trimmed[pos..].trim()),
        None => (trimmed, ""),
    };

    let result: anyhow::Result<String> = match cmd {
        "/help" | "/h" | "/?" => help::execute(args),
        "/clear" => {
            return Some(CommandResult::ClearMessages("Conversation cleared.".into()));
        }
        "/model" | "/m" => dispatch_model(args, state),
        "/compact" => compact::execute(args, state),
        "/thinking" | "/think" => thinking::execute(args, state),
        "/hotkeys" | "/keys" | "/keybindings" => hotkeys::execute(args),
        "/settings" | "/config" => settings::execute(args, state),
        "/tree" => tree::TreeCommand::execute(args).map_err(|e| anyhow::anyhow!("{e}")),
        "/bookmark" | "/bm" => dispatch_bookmark(args),
        "/export" => Ok("Usage: /export (session export not yet integrated)".into()),
        "/share" => share::execute(args),
        "/fork" => Ok("Usage: /fork (conversation fork not yet integrated)".into()),
        "/providers" => providers::execute(args),
        "/cost" => cost::execute(args, &state.cost_tracker),
        "/agents" | "/tools" => agents::execute(args),
        "/skills" => skills::execute(args),
        "/status" => status::execute(args, state),
        "/login" => login::execute(args),
        "/logout" => logout::execute(args),
        _ => Ok(format!(
            "Unknown command: {cmd}. Type /help for available commands."
        )),
    };

    match result {
        Ok(text) => Some(CommandResult::Message(text)),
        Err(e) => Some(CommandResult::Message(format!("Error: {e}"))),
    }
}

/// Handle /bookmark with list support.
fn dispatch_bookmark(args: &str) -> anyhow::Result<String> {
    let sub = args.trim();
    if sub.is_empty() || sub == "list" {
        match bookmark::BookmarkCommand::list() {
            Ok(bookmarks) => {
                if bookmarks.is_empty() {
                    Ok("No bookmarks. Use /bookmark add <name> to create one.".into())
                } else {
                    let mut text = "Bookmarks:".to_string();
                    for b in &bookmarks {
                        text.push_str(&format!("\n  {} → {}", b.name, b.session_id.prefix()));
                    }
                    Ok(text)
                }
            }
            Err(e) => Ok(format!("Bookmark error: {e}")),
        }
    } else {
        Ok("Usage: /bookmark [list]\n  (add/remove/rename not yet integrated)".into())
    }
}

/// Handle /model with awareness of current state.
///
/// Subcommands:
/// - `/model` — list enabled models
/// - `/model <name>` — switch to a model
/// - `/model enable <name>` — add model to Ctrl+P rotation
/// - `/model disable <name>` — remove model from Ctrl+P rotation
fn dispatch_model(args: &str, state: &mut AppState) -> anyhow::Result<String> {
    let trimmed = args.trim();
    if trimmed.is_empty() {
        return model::list_models(state);
    }
    // Check for subcommands.
    if let Some(rest) = trimmed.strip_prefix("enable") {
        return model::enable_model(rest, state);
    }
    if let Some(rest) = trimmed.strip_prefix("disable") {
        return model::disable_model(rest, state);
    }
    model::switch_model(trimmed, state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use saorsa_agent::ThinkingLevel;

    #[test]
    fn dispatch_returns_none_for_regular_text() {
        let mut state = AppState::new("test");
        assert!(dispatch("hello world", &mut state).is_none());
    }

    #[test]
    fn dispatch_returns_none_for_empty() {
        let mut state = AppState::new("test");
        assert!(dispatch("", &mut state).is_none());
    }

    #[test]
    fn dispatch_help_command() {
        let mut state = AppState::new("test");
        let result = dispatch("/help", &mut state);
        match result {
            Some(CommandResult::Message(text)) => {
                assert!(text.contains("/model"));
                assert!(text.contains("/thinking"));
            }
            _ => panic!("Expected Message"),
        }
    }

    #[test]
    fn dispatch_help_alias_h() {
        let mut state = AppState::new("test");
        let result = dispatch("/h", &mut state);
        assert!(matches!(result, Some(CommandResult::Message(_))));
    }

    #[test]
    fn dispatch_help_alias_question() {
        let mut state = AppState::new("test");
        let result = dispatch("/?", &mut state);
        assert!(matches!(result, Some(CommandResult::Message(_))));
    }

    #[test]
    fn dispatch_clear_command() {
        let mut state = AppState::new("test");
        let result = dispatch("/clear", &mut state);
        assert!(matches!(result, Some(CommandResult::ClearMessages(_))));
    }

    #[test]
    fn dispatch_unknown_command() {
        let mut state = AppState::new("test");
        let result = dispatch("/foobar", &mut state);
        match result {
            Some(CommandResult::Message(text)) => {
                assert!(text.contains("Unknown command"));
            }
            _ => panic!("Expected Message"),
        }
    }

    #[test]
    fn dispatch_model_no_args_shows_current() {
        let mut state = AppState::new("claude-sonnet-4");
        state.enabled_models = vec!["claude-sonnet-4".into(), "gpt-4o".into()];
        state.model_index = 0;
        let result = dispatch("/model", &mut state);
        match result {
            Some(CommandResult::Message(text)) => {
                assert!(text.contains("claude-sonnet-4"));
                assert!(text.contains("gpt-4o"));
            }
            _ => panic!("Expected Message"),
        }
    }

    #[test]
    fn dispatch_model_with_args_switches() {
        let mut state = AppState::new("model-a");
        state.enabled_models = vec!["model-a".into(), "model-b".into()];
        let result = dispatch("/model model-b", &mut state);
        match result {
            Some(CommandResult::Message(text)) => {
                assert!(text.contains("model-b"));
            }
            _ => panic!("Expected Message"),
        }
        assert_eq!(state.model, "model-b");
    }

    #[test]
    fn dispatch_model_alias_m() {
        let mut state = AppState::new("test");
        let result = dispatch("/m", &mut state);
        assert!(matches!(result, Some(CommandResult::Message(_))));
    }

    #[test]
    fn dispatch_thinking_shows_level() {
        let mut state = AppState::new("test");
        let result = dispatch("/thinking", &mut state);
        match result {
            Some(CommandResult::Message(text)) => {
                assert!(text.contains("off"));
            }
            _ => panic!("Expected Message"),
        }
    }

    #[test]
    fn dispatch_thinking_sets_level() {
        let mut state = AppState::new("test");
        let result = dispatch("/thinking high", &mut state);
        match result {
            Some(CommandResult::Message(text)) => {
                assert!(text.contains("high"));
            }
            _ => panic!("Expected Message"),
        }
        assert_eq!(state.thinking_level, ThinkingLevel::High);
    }

    #[test]
    fn dispatch_thinking_alias() {
        let mut state = AppState::new("test");
        dispatch("/think medium", &mut state);
        assert_eq!(state.thinking_level, ThinkingLevel::Medium);
    }

    #[test]
    fn dispatch_compact_toggles() {
        let mut state = AppState::new("test");
        assert!(!state.compact_mode);
        dispatch("/compact", &mut state);
        assert!(state.compact_mode);
        dispatch("/compact", &mut state);
        assert!(!state.compact_mode);
    }

    #[test]
    fn dispatch_settings_shows_info() {
        let mut state = AppState::new("test-model");
        let result = dispatch("/settings", &mut state);
        match result {
            Some(CommandResult::Message(text)) => {
                assert!(text.contains("test-model"));
                assert!(text.contains("Thinking:"));
            }
            _ => panic!("Expected Message"),
        }
    }

    #[test]
    fn dispatch_settings_alias_config() {
        let mut state = AppState::new("test");
        let result = dispatch("/config", &mut state);
        assert!(matches!(result, Some(CommandResult::Message(_))));
    }

    #[test]
    fn dispatch_hotkeys() {
        let mut state = AppState::new("test");
        let result = dispatch("/hotkeys", &mut state);
        match result {
            Some(CommandResult::Message(text)) => {
                assert!(text.contains("Ctrl+C"));
            }
            _ => panic!("Expected Message"),
        }
    }

    #[test]
    fn dispatch_hotkeys_alias_keys() {
        let mut state = AppState::new("test");
        let result = dispatch("/keys", &mut state);
        assert!(matches!(result, Some(CommandResult::Message(_))));
    }

    #[test]
    fn dispatch_login() {
        let mut state = AppState::new("test");
        let result = dispatch("/login", &mut state);
        match result {
            Some(CommandResult::Message(text)) => {
                assert!(text.contains("auth.json"));
            }
            _ => panic!("Expected Message"),
        }
    }

    #[test]
    fn dispatch_logout() {
        let mut state = AppState::new("test");
        let result = dispatch("/logout", &mut state);
        match result {
            Some(CommandResult::Message(text)) => {
                assert!(text.contains("auth.json"));
            }
            _ => panic!("Expected Message"),
        }
    }

    #[test]
    fn dispatch_share() {
        let mut state = AppState::new("test");
        let result = dispatch("/share", &mut state);
        match result {
            Some(CommandResult::Message(text)) => {
                assert!(text.contains("not yet available"));
            }
            _ => panic!("Expected Message"),
        }
    }

    #[test]
    fn dispatch_thinking_invalid_shows_error() {
        let mut state = AppState::new("test");
        let result = dispatch("/thinking extreme", &mut state);
        match result {
            Some(CommandResult::Message(text)) => {
                assert!(text.contains("Error:"));
            }
            _ => panic!("Expected Message"),
        }
    }

    #[test]
    fn dispatch_whitespace_trimmed() {
        let mut state = AppState::new("test");
        let result = dispatch("  /help  ", &mut state);
        assert!(matches!(result, Some(CommandResult::Message(_))));
    }

    #[test]
    fn dispatch_bookmark_alias() {
        let mut state = AppState::new("test");
        let result = dispatch("/bm", &mut state);
        assert!(matches!(result, Some(CommandResult::Message(_))));
    }

    #[test]
    fn dispatch_providers() {
        let mut state = AppState::new("test");
        let result = dispatch("/providers", &mut state);
        match result {
            Some(CommandResult::Message(text)) => {
                assert!(text.contains("Anthropic"));
                assert!(text.contains("Providers:"));
            }
            _ => panic!("Expected Message"),
        }
    }

    #[test]
    fn dispatch_cost_empty() {
        let mut state = AppState::new("test");
        let result = dispatch("/cost", &mut state);
        match result {
            Some(CommandResult::Message(text)) => {
                assert!(text.contains("$0.00"));
            }
            _ => panic!("Expected Message"),
        }
    }

    #[test]
    fn dispatch_agents() {
        let mut state = AppState::new("test");
        let result = dispatch("/agents", &mut state);
        match result {
            Some(CommandResult::Message(text)) => {
                assert!(text.contains("bash"));
                assert!(text.contains("read"));
            }
            _ => panic!("Expected Message"),
        }
    }

    #[test]
    fn dispatch_agents_alias_tools() {
        let mut state = AppState::new("test");
        let result = dispatch("/tools", &mut state);
        assert!(matches!(result, Some(CommandResult::Message(_))));
    }

    #[test]
    fn dispatch_skills() {
        let mut state = AppState::new("test");
        let result = dispatch("/skills", &mut state);
        assert!(matches!(result, Some(CommandResult::Message(_))));
    }

    #[test]
    fn dispatch_status() {
        let mut state = AppState::new("my-model");
        let result = dispatch("/status", &mut state);
        match result {
            Some(CommandResult::Message(text)) => {
                assert!(text.contains("my-model"));
                assert!(text.contains("Session status"));
            }
            _ => panic!("Expected Message"),
        }
    }
}
