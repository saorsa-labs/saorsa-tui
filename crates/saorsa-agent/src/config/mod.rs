//! Agent configuration.

pub mod auth;
pub mod import;
pub mod models;
pub mod paths;
pub mod settings;

use crate::context::ContextBundle;

/// Configuration for the agent loop.
#[derive(Clone, Debug)]
pub struct AgentConfig {
    /// The LLM model to use.
    pub model: String,
    /// The system prompt.
    pub system_prompt: String,
    /// Maximum number of agent turns before stopping.
    pub max_turns: u32,
    /// Maximum tokens per response.
    pub max_tokens: u32,
    /// Context bundle (AGENTS.md, SYSTEM.md, user context).
    pub context: ContextBundle,
}

impl AgentConfig {
    /// Create a new agent config with the given model.
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            system_prompt: "You are a helpful assistant.".into(),
            max_turns: 10,
            max_tokens: 4096,
            context: ContextBundle::new(),
        }
    }

    /// Set the system prompt.
    #[must_use]
    pub fn system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = prompt.into();
        self
    }

    /// Set the maximum number of turns.
    #[must_use]
    pub fn max_turns(mut self, max: u32) -> Self {
        self.max_turns = max;
        self
    }

    /// Set the maximum tokens per response.
    #[must_use]
    pub fn max_tokens(mut self, max: u32) -> Self {
        self.max_tokens = max;
        self
    }

    /// Set the context bundle.
    #[must_use]
    pub fn context(mut self, context: ContextBundle) -> Self {
        self.context = context;
        self
    }
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self::new("claude-sonnet-4-5-20250929")
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let config = AgentConfig::default();
        assert_eq!(config.model, "claude-sonnet-4-5-20250929");
        assert_eq!(config.max_turns, 10);
        assert_eq!(config.max_tokens, 4096);
        assert!(!config.system_prompt.is_empty());
    }

    #[test]
    fn builder_pattern() {
        let config = AgentConfig::new("claude-opus-4-20250514")
            .system_prompt("Be concise")
            .max_turns(5)
            .max_tokens(8192);
        assert_eq!(config.model, "claude-opus-4-20250514");
        assert_eq!(config.system_prompt, "Be concise");
        assert_eq!(config.max_turns, 5);
        assert_eq!(config.max_tokens, 8192);
    }

    #[test]
    fn new_custom_model() {
        let config = AgentConfig::new("gpt-4");
        assert_eq!(config.model, "gpt-4");
    }
}
