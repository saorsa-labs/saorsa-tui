//! `/status` command — show session information.

use crate::app::AppState;

/// Display current session status.
pub fn execute(_args: &str, state: &AppState) -> anyhow::Result<String> {
    let thinking = &state.thinking_level;
    let compact = if state.compact_mode { "on" } else { "off" };
    let msg_count = state.messages.len();
    let enabled_count = state.enabled_models.len();

    let mut text = "Session status:".to_string();
    text.push_str(&format!("\n  Model:          {}", state.model));
    text.push_str(&format!("\n  Thinking:       {thinking}"));
    text.push_str(&format!("\n  Compact:        {compact}"));
    text.push_str(&format!("\n  Messages:       {msg_count}"));
    text.push_str(&format!("\n  Enabled models: {enabled_count}"));
    text.push_str(&format!(
        "\n  Status:         {}",
        format_status(&state.status)
    ));

    Ok(text)
}

/// Format the app status for display.
fn format_status(status: &crate::app::AppStatus) -> &str {
    match status {
        crate::app::AppStatus::Idle => "idle",
        crate::app::AppStatus::Thinking => "thinking",
        crate::app::AppStatus::ToolRunning { .. } => "tool running",
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn shows_session_info() {
        let state = AppState::new("test-model");
        let text = execute("", &state).expect("should succeed");
        assert!(text.contains("test-model"));
        assert!(text.contains("Session status"));
        assert!(text.contains("idle"));
    }

    #[test]
    fn shows_thinking_level() {
        let mut state = AppState::new("test");
        state.thinking_level = saorsa_agent::ThinkingLevel::High;
        let text = execute("", &state).expect("should succeed");
        assert!(text.contains("high"));
    }

    #[test]
    fn shows_compact_mode() {
        let mut state = AppState::new("test");
        state.compact_mode = true;
        let text = execute("", &state).expect("should succeed");
        assert!(text.contains("Compact:        on"));
    }

    #[test]
    fn shows_message_count() {
        let mut state = AppState::new("test");
        state.add_user_message("hello");
        state.add_assistant_message("hi");
        let text = execute("", &state).expect("should succeed");
        assert!(text.contains("Messages:       2"));
    }
}
