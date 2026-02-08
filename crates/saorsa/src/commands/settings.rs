//! `/settings` command — show current settings or change them via subcommands.
//!
//! # Subcommands
//!
//! - `/config` — show current settings with usage hints
//! - `/config model <name>` — switch AI model
//! - `/config thinking <level>` — set thinking level (off/low/medium/high)
//! - `/config compact` — toggle compact display mode
//! - `/config reset` — reset settings to defaults

use crate::app::AppState;
use crate::commands::{compact, model, thinking};

/// Show current settings or execute a settings subcommand.
///
/// Without arguments, displays the current settings with actionable hints.
/// With a subcommand, delegates to the appropriate handler.
pub fn execute(args: &str, state: &mut AppState) -> anyhow::Result<String> {
    let trimmed = args.trim();
    if trimmed.is_empty() {
        return show_settings(state);
    }

    // Split into subcommand and its arguments.
    let (sub, sub_args) = match trimmed.find(char::is_whitespace) {
        Some(pos) => (&trimmed[..pos], trimmed[pos..].trim()),
        None => (trimmed, ""),
    };

    match sub {
        "model" | "m" => {
            if sub_args.is_empty() {
                Ok("Usage: /config model <name>\n\nSwitch to a different AI model.".into())
            } else {
                model::switch_model(sub_args, state)
            }
        }
        "thinking" | "think" => thinking::execute(sub_args, state),
        "compact" => compact::execute(sub_args, state),
        "reset" => reset_settings(state),
        _ => Ok(format!(
            "Unknown config option: {sub}\n\n\
             Available: model, thinking, compact, reset"
        )),
    }
}

/// Display a summary of the current session settings with usage hints.
fn show_settings(state: &AppState) -> anyhow::Result<String> {
    let compact_label = if state.compact_mode { "on" } else { "off" };
    let model_count = state.enabled_models.len();

    Ok(format!(
        "\
Current settings:
  Model:          {model:<20} (change: /config model <name>)
  Thinking:       {thinking:<20} (change: /config thinking high|medium|low|off)
  Compact:        {compact:<20} (change: /config compact)
  Enabled models: {model_count}
  Messages:       {messages}
  Config dir:     ~/.saorsa/",
        model = state.model,
        thinking = state.thinking_level,
        compact = compact_label,
        messages = state.messages.len(),
    ))
}

/// Reset all session settings to their defaults.
fn reset_settings(state: &mut AppState) -> anyhow::Result<String> {
    state.thinking_level = saorsa_agent::ThinkingLevel::default();
    state.compact_mode = false;
    Ok("Settings reset to defaults:\n  Thinking: off\n  Compact: off".into())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn settings_shows_model() {
        let mut state = AppState::new("claude-sonnet-4");
        let text = execute("", &mut state).expect("should succeed");
        assert!(text.contains("claude-sonnet-4"));
    }

    #[test]
    fn settings_shows_thinking() {
        let mut state = AppState::new("test");
        let text = execute("", &mut state).expect("should succeed");
        assert!(text.contains("off"));
    }

    #[test]
    fn settings_shows_compact() {
        let mut state = AppState::new("test");
        let text = execute("", &mut state).expect("should succeed");
        assert!(text.contains("Compact:"));
    }

    #[test]
    fn settings_shows_config_dir() {
        let mut state = AppState::new("test");
        let text = execute("", &mut state).expect("should succeed");
        assert!(text.contains("~/.saorsa/"));
    }

    #[test]
    fn settings_shows_usage_hints() {
        let mut state = AppState::new("test");
        let text = execute("", &mut state).expect("should succeed");
        assert!(text.contains("/config model"));
        assert!(text.contains("/config thinking"));
        assert!(text.contains("/config compact"));
    }

    #[test]
    fn config_model_switches() {
        let mut state = AppState::new("model-a");
        state.enabled_models = vec!["model-a".into(), "model-b".into()];
        let text = execute("model model-b", &mut state).expect("should succeed");
        assert!(text.contains("model-b"));
        assert_eq!(state.model, "model-b");
    }

    #[test]
    fn config_model_no_args_shows_usage() {
        let mut state = AppState::new("test");
        let text = execute("model", &mut state).expect("should succeed");
        assert!(text.contains("Usage:"));
    }

    #[test]
    fn config_thinking_sets_level() {
        let mut state = AppState::new("test");
        let text = execute("thinking high", &mut state).expect("should succeed");
        assert!(text.contains("high"));
        assert_eq!(state.thinking_level, saorsa_agent::ThinkingLevel::High);
    }

    #[test]
    fn config_compact_toggles() {
        let mut state = AppState::new("test");
        assert!(!state.compact_mode);
        execute("compact", &mut state).expect("should succeed");
        assert!(state.compact_mode);
    }

    #[test]
    fn config_reset_restores_defaults() {
        let mut state = AppState::new("test");
        state.thinking_level = saorsa_agent::ThinkingLevel::High;
        state.compact_mode = true;
        let text = execute("reset", &mut state).expect("should succeed");
        assert!(text.contains("reset"));
        assert_eq!(state.thinking_level, saorsa_agent::ThinkingLevel::Off);
        assert!(!state.compact_mode);
    }

    #[test]
    fn config_unknown_subcommand() {
        let mut state = AppState::new("test");
        let text = execute("foobar", &mut state).expect("should succeed");
        assert!(text.contains("Unknown config option"));
    }
}
