//! `/settings` command — show current settings summary.

use crate::app::AppState;

/// Display a summary of the current session settings.
pub fn execute(_args: &str, state: &AppState) -> anyhow::Result<String> {
    let compact_label = if state.compact_mode { "on" } else { "off" };
    let model_count = state.enabled_models.len();

    Ok(format!(
        "\
Current settings:
  Model:          {}
  Thinking:       {}
  Compact:        {compact_label}
  Enabled models: {model_count}
  Messages:       {}
  Config dir:     ~/.saorsa/",
        state.model,
        state.thinking_level,
        state.messages.len(),
    ))
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn settings_shows_model() {
        let state = AppState::new("claude-sonnet-4");
        let text = execute("", &state).expect("should succeed");
        assert!(text.contains("claude-sonnet-4"));
    }

    #[test]
    fn settings_shows_thinking() {
        let state = AppState::new("test");
        let text = execute("", &state).expect("should succeed");
        assert!(text.contains("off"));
    }

    #[test]
    fn settings_shows_compact() {
        let state = AppState::new("test");
        let text = execute("", &state).expect("should succeed");
        assert!(text.contains("Compact:"));
    }

    #[test]
    fn settings_shows_config_dir() {
        let state = AppState::new("test");
        let text = execute("", &state).expect("should succeed");
        assert!(text.contains("~/.saorsa/"));
    }
}
