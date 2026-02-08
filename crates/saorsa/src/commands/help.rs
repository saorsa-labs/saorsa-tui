//! `/help` command — show available commands.

/// Show the list of available slash commands.
pub fn execute(_args: &str) -> anyhow::Result<String> {
    Ok("\
Available commands:
  /help              Show this help
  /model [name]      Show or switch model (Ctrl+P cycles)
  /thinking [level]  Set thinking: off, low, medium, high
  /compact           Toggle compact display mode
  /clear             Clear conversation history
  /hotkeys           Show keyboard shortcuts
  /settings          Show current settings
  /providers         List configured LLM providers
  /cost              Show session cost breakdown
  /agents            List available agent tools
  /skills            List available skills
  /status            Show session information
  /tree              Show conversation tree
  /bookmark [list]   Manage bookmarks
  /export            Export conversation
  /share             Share conversation link
  /fork              Fork conversation
  /login             Configure API keys
  /logout            Remove API keys

Aliases: /h, /?, /m, /think, /keys, /config, /bm, /tools"
        .to_string())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn help_lists_commands() {
        let text = execute("").expect("should succeed");
        assert!(text.contains("/help"));
        assert!(text.contains("/model"));
        assert!(text.contains("/thinking"));
        assert!(text.contains("/clear"));
    }
}
