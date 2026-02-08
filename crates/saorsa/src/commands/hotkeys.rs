//! `/hotkeys` command — show keyboard shortcuts.

/// Show keyboard shortcuts for the TUI.
pub fn execute(_args: &str) -> anyhow::Result<String> {
    Ok("\
Keyboard shortcuts:
  Enter              Send message
  Ctrl+C             Quit
  Ctrl+D             Quit (empty input)
  Ctrl+P             Next model
  Shift+Ctrl+P       Previous model
  Ctrl+L             Open model selector
  PageUp             Scroll up
  PageDown           Scroll down
  Escape             Clear input / close overlay
  Home / End         Jump to start/end of input
  Left / Right       Move cursor
  Backspace          Delete character"
        .to_string())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn hotkeys_mentions_send() {
        let text = execute("").expect("should succeed");
        assert!(text.contains("Enter"));
        assert!(text.contains("Send message"));
    }

    #[test]
    fn hotkeys_mentions_quit() {
        let text = execute("").expect("should succeed");
        assert!(text.contains("Ctrl+C"));
    }

    #[test]
    fn hotkeys_mentions_model_cycling() {
        let text = execute("").expect("should succeed");
        assert!(text.contains("Ctrl+P"));
        assert!(text.contains("Next model"));
    }
}
