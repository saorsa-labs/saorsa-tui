//! `/compact` command — toggle compact display mode.

use crate::app::AppState;

/// Toggle compact display mode.
///
/// Compact mode reduces visual chrome for a denser conversation view.
pub fn execute(_args: &str, state: &mut AppState) -> anyhow::Result<String> {
    state.compact_mode = !state.compact_mode;
    let label = if state.compact_mode { "on" } else { "off" };
    Ok(format!("Compact mode: {label}"))
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn toggle_on() {
        let mut state = AppState::new("test");
        assert!(!state.compact_mode);
        let text = execute("", &mut state).expect("should succeed");
        assert!(text.contains("on"));
        assert!(state.compact_mode);
    }

    #[test]
    fn toggle_off_after_on() {
        let mut state = AppState::new("test");
        execute("", &mut state).expect("should succeed");
        let text = execute("", &mut state).expect("should succeed");
        assert!(text.contains("off"));
        assert!(!state.compact_mode);
    }
}
