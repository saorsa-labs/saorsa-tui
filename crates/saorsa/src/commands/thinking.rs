//! `/thinking` command — show or set the extended-thinking level.

use crate::app::AppState;

/// Show or set the extended-thinking level.
///
/// Without arguments, displays the current level.
/// With an argument (`off`, `low`, `medium`, `high`), sets the level.
///
/// # Errors
///
/// Returns an error if the argument is not a recognised level.
pub fn execute(args: &str, state: &mut AppState) -> anyhow::Result<String> {
    let trimmed = args.trim();
    if trimmed.is_empty() {
        return Ok(format!("Thinking level: {}", state.thinking_level));
    }

    let level: saorsa_agent::ThinkingLevel = trimmed
        .parse()
        .map_err(|e: saorsa_agent::ParseThinkingLevelError| anyhow::anyhow!("{e}"))?;
    state.thinking_level = level;
    Ok(format!("Thinking level set to: {}", state.thinking_level))
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use saorsa_agent::ThinkingLevel;

    #[test]
    fn show_current_level() {
        let mut state = AppState::new("test");
        let text = execute("", &mut state).expect("should succeed");
        assert!(text.contains("off"));
    }

    #[test]
    fn set_level_high() {
        let mut state = AppState::new("test");
        let text = execute("high", &mut state).expect("should succeed");
        assert!(text.contains("high"));
        assert_eq!(state.thinking_level, ThinkingLevel::High);
    }

    #[test]
    fn set_level_medium() {
        let mut state = AppState::new("test");
        let text = execute("medium", &mut state).expect("should succeed");
        assert!(text.contains("medium"));
        assert_eq!(state.thinking_level, ThinkingLevel::Medium);
    }

    #[test]
    fn invalid_level_returns_error() {
        let mut state = AppState::new("test");
        let result = execute("extreme", &mut state);
        assert!(result.is_err());
    }
}
