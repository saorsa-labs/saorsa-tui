//! CLI argument parsing for fae.

use clap::Parser;

/// fae - AI coding agent for the terminal.
#[derive(Parser, Debug)]
#[command(name = "fae", version, about)]
pub struct Cli {
    /// LLM model to use.
    #[arg(long, default_value = "claude-sonnet-4-5-20250929")]
    pub model: String,

    /// Anthropic API key (or set ANTHROPIC_API_KEY env var).
    #[arg(long, env = "ANTHROPIC_API_KEY")]
    pub api_key: Option<String>,

    /// System prompt for the agent.
    #[arg(long, default_value = "You are a helpful AI coding assistant.")]
    pub system_prompt: String,

    /// Maximum tokens per response.
    #[arg(long, default_value = "4096")]
    pub max_tokens: u32,

    /// Maximum agent turns per interaction.
    #[arg(long, default_value = "10")]
    pub max_turns: u32,

    /// Run in print mode: send a single prompt and print the response.
    #[arg(short, long)]
    pub print: Option<String>,

    /// Continue the most recent session.
    #[arg(short = 'c', long)]
    pub continue_session: bool,

    /// Resume a specific session by ID prefix.
    #[arg(short = 'r', long)]
    pub resume: Option<String>,

    /// Run in ephemeral mode (no session persistence).
    #[arg(long)]
    pub ephemeral: bool,
}

impl Cli {
    /// Parse CLI arguments.
    pub fn parse_args() -> Self {
        Self::parse()
    }

    /// Get the API key from args or environment.
    ///
    /// Returns an error message if no key is found.
    pub fn api_key(&self) -> std::result::Result<&str, &'static str> {
        self.api_key
            .as_deref()
            .ok_or("No API key provided. Set ANTHROPIC_API_KEY or use --api-key")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_defaults() {
        // Note: api_key may be set via ANTHROPIC_API_KEY env var.
        let cli = Cli::parse_from(["fae"]);
        assert_eq!(cli.model, "claude-sonnet-4-5-20250929");
        assert_eq!(cli.max_tokens, 4096);
        assert_eq!(cli.max_turns, 10);
        assert!(cli.print.is_none());
        assert!(!cli.continue_session);
        assert!(cli.resume.is_none());
        assert!(!cli.ephemeral);
    }

    #[test]
    fn cli_custom_model() {
        let cli = Cli::parse_from(["fae", "--model", "claude-opus-4-20250514"]);
        assert_eq!(cli.model, "claude-opus-4-20250514");
    }

    #[test]
    fn cli_print_mode() {
        let cli = Cli::parse_from(["fae", "--print", "Hello"]);
        assert_eq!(cli.print.as_deref(), Some("Hello"));
    }

    #[test]
    fn cli_api_key_from_arg() {
        let cli = Cli::parse_from(["fae", "--api-key", "sk-test"]);
        assert_eq!(cli.api_key(), Ok("sk-test"));
    }

    #[test]
    fn cli_api_key_missing_without_env() {
        // When ANTHROPIC_API_KEY is not set and --api-key not given,
        // api_key() should return an error. We simulate this by
        // checking the behavior of api_key() on a manually constructed CLI.
        let cli = Cli {
            model: "test".into(),
            api_key: None,
            system_prompt: "test".into(),
            max_tokens: 4096,
            max_turns: 10,
            print: None,
            continue_session: false,
            resume: None,
            ephemeral: false,
        };
        assert!(cli.api_key().is_err());
    }

    #[test]
    fn cli_continue_session() {
        let cli = Cli::parse_from(["fae", "-c"]);
        assert!(cli.continue_session);
    }

    #[test]
    fn cli_continue_session_long_form() {
        let cli = Cli::parse_from(["fae", "--continue-session"]);
        assert!(cli.continue_session);
    }

    #[test]
    fn cli_resume_session() {
        let cli = Cli::parse_from(["fae", "--resume", "abc123"]);
        assert_eq!(cli.resume.as_deref(), Some("abc123"));
    }

    #[test]
    fn cli_ephemeral() {
        let cli = Cli::parse_from(["fae", "--ephemeral"]);
        assert!(cli.ephemeral);
    }
}
