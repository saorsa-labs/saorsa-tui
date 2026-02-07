//! Provider trait for LLM backends.

use crate::error::Result;
use crate::types::{CompletionRequest, CompletionResponse, StreamEvent};

/// Configuration for an LLM provider.
#[derive(Clone, Debug)]
pub struct ProviderConfig {
    /// API key for authentication.
    pub api_key: String,
    /// Base URL for the API.
    pub base_url: String,
    /// Default model identifier.
    pub model: String,
    /// Default max tokens.
    pub max_tokens: u32,
}

impl ProviderConfig {
    /// Create a new provider config.
    pub fn new(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: "https://api.anthropic.com".into(),
            model: model.into(),
            max_tokens: 4096,
        }
    }

    /// Set the base URL.
    #[must_use]
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    /// Set the max tokens.
    #[must_use]
    pub fn max_tokens(mut self, max: u32) -> Self {
        self.max_tokens = max;
        self
    }
}

/// Trait for LLM providers that support non-streaming completion.
#[async_trait::async_trait]
pub trait Provider: Send + Sync {
    /// Send a completion request and wait for the full response.
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse>;
}

/// Trait for LLM providers that support streaming.
#[async_trait::async_trait]
pub trait StreamingProvider: Provider {
    /// Send a completion request and return a stream of events.
    async fn stream(
        &self,
        request: CompletionRequest,
    ) -> Result<tokio::sync::mpsc::Receiver<Result<StreamEvent>>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_config_builder() {
        let config = ProviderConfig::new("sk-test", "claude-sonnet-4-5-20250929")
            .base_url("https://custom.api.com")
            .max_tokens(8192);
        assert_eq!(config.api_key, "sk-test");
        assert_eq!(config.model, "claude-sonnet-4-5-20250929");
        assert_eq!(config.base_url, "https://custom.api.com");
        assert_eq!(config.max_tokens, 8192);
    }

    #[test]
    fn provider_config_defaults() {
        let config = ProviderConfig::new("key", "model");
        assert_eq!(config.base_url, "https://api.anthropic.com");
        assert_eq!(config.max_tokens, 4096);
    }
}
