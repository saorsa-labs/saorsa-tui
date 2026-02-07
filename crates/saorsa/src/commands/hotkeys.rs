//! /hotkeys command - show keybindings help.

/// Handle the /hotkeys command.
///
/// Shows available keybindings.
pub fn execute(_args: &str) -> anyhow::Result<String> {
    Ok("Keybindings:\n\
        Ctrl+Enter - Send message\n\
        Ctrl+N - New chat\n\
        Ctrl+L - Model selector\n\
        Ctrl+, - Settings\n\
        Escape - Cancel"
        .to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hotkeys_shows_help() {
        let result = execute("");
        assert!(result.is_ok());
        match result {
            Ok(output) => {
                assert!(output.contains("Ctrl+Enter"));
                assert!(output.contains("Escape"));
            }
            Err(_) => unreachable!(),
        }
    }
}
