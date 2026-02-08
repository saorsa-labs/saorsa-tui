//! CLI argument parsing for saorsa.

use clap::Parser;

/// saorsa - AI coding agent for the terminal.
#[derive(Parser, Debug)]
#[command(name = "saorsa", version, about)]
pub struct Cli {
    /// LLM model to use.
    #[arg(long, default_value = "claude-sonnet-4-5-20250929")]
    pub model: String,

    /// API key (or use ~/.saorsa/auth.json config).
    #[arg(long)]
    pub api_key: Option<String>,

    /// Provider to use (auto-detected from model if omitted).
    #[arg(long)]
    pub provider: Option<String>,

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

    /// Get the API key from CLI arguments, if provided.
    pub fn api_key(&self) -> Option<&str> {
        self.api_key.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_defaults() {
        // Note: api_key may be set via ANTHROPIC_API_KEY env var.
        let cli = Cli::parse_from(["saorsa"]);
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
        let cli = Cli::parse_from(["saorsa", "--model", "claude-opus-4-20250514"]);
        assert_eq!(cli.model, "claude-opus-4-20250514");
    }

    #[test]
    fn cli_print_mode() {
        let cli = Cli::parse_from(["saorsa", "--print", "Hello"]);
        assert_eq!(cli.print.as_deref(), Some("Hello"));
    }

    #[test]
    fn cli_api_key_from_arg() {
        let cli = Cli::parse_from(["saorsa", "--api-key", "sk-test"]);
        assert_eq!(cli.api_key(), Some("sk-test"));
    }

    #[test]
    fn cli_api_key_missing_returns_none() {
        let cli = Cli {
            model: "test".into(),
            api_key: None,
            provider: None,
            system_prompt: "test".into(),
            max_tokens: 4096,
            max_turns: 10,
            print: None,
            continue_session: false,
            resume: None,
            ephemeral: false,
        };
        assert!(cli.api_key().is_none());
    }

    #[test]
    fn cli_provider_flag() {
        let cli = Cli::parse_from(["saorsa", "--provider", "openai"]);
        assert_eq!(cli.provider.as_deref(), Some("openai"));
    }

    #[test]
    fn cli_provider_defaults_to_none() {
        let cli = Cli::parse_from(["saorsa"]);
        assert!(cli.provider.is_none());
    }

    #[test]
    fn cli_continue_session() {
        let cli = Cli::parse_from(["saorsa", "-c"]);
        assert!(cli.continue_session);
    }

    #[test]
    fn cli_continue_session_long_form() {
        let cli = Cli::parse_from(["saorsa", "--continue-session"]);
        assert!(cli.continue_session);
    }

    #[test]
    fn cli_resume_session() {
        let cli = Cli::parse_from(["saorsa", "--resume", "abc123"]);
        assert_eq!(cli.resume.as_deref(), Some("abc123"));
    }

    #[test]
    fn cli_ephemeral() {
        let cli = Cli::parse_from(["saorsa", "--ephemeral"]);
        assert!(cli.ephemeral);
    }
}
