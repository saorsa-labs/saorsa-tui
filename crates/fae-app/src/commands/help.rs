//! /help command - show command list.

/// Handle the /help command.
///
/// Shows available slash commands.
pub fn execute(_args: &str) -> anyhow::Result<String> {
    Ok("Available commands:\n\
        /model <name> - Switch model\n\
        /thinking - Toggle thinking mode\n\
        /compact - Toggle compact mode\n\
        /tree - Show conversation tree\n\
        /fork - Fork conversation\n\
        /bookmark - Manage bookmarks\n\
        /export - Export conversation\n\
        /share - Share conversation\n\
        /login <provider> - Authenticate\n\
        /logout - Clear credentials\n\
        /settings - Open settings\n\
        /hotkeys - Show keybindings\n\
        /clear - Clear conversation\n\
        /help - Show this help"
        .to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn help_shows_commands() {
        let result = execute("");
        assert!(result.is_ok());
        match result {
            Ok(output) => {
                assert!(output.contains("/model"));
                assert!(output.contains("/thinking"));
                assert!(output.contains("/help"));
            }
            Err(_) => unreachable!(),
        }
    }
}
