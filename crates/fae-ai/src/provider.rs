//! Provider trait for LLM backends.

use std::collections::HashMap;

use crate::error::{FaeAiError, Result};
use crate::types::{CompletionRequest, CompletionResponse, StreamEvent};

/// Identifies which LLM provider to use.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ProviderKind {
    /// Anthropic (Claude) — Messages API.
    Anthropic,
    /// OpenAI — Chat Completions API.
    OpenAi,
    /// Google Gemini — GenerateContent API.
    Gemini,
    /// Ollama — local inference server.
    Ollama,
    /// Any OpenAI-compatible API (Azure, Groq, Mistral, xAI, OpenRouter, etc.).
    OpenAiCompatible,
}

impl ProviderKind {
    /// Returns the default base URL for this provider.
    ///
    /// `OpenAiCompatible` returns an empty string because a custom URL is required.
    #[must_use]
    pub fn default_base_url(self) -> &'static str {
        match self {
            Self::Anthropic => "https://api.anthropic.com",
            Self::OpenAi => "https://api.openai.com",
            Self::Gemini => "https://generativelanguage.googleapis.com/v1beta",
            Self::Ollama => "http://localhost:11434",
            Self::OpenAiCompatible => "",
        }
    }

    /// Returns a human-readable display name for this provider.
    #[must_use]
    pub fn display_name(self) -> &'static str {
        match self {
            Self::Anthropic => "Anthropic",
            Self::OpenAi => "OpenAI",
            Self::Gemini => "Google Gemini",
            Self::Ollama => "Ollama",
            Self::OpenAiCompatible => "OpenAI-Compatible",
        }
    }
}

/// Configuration for an LLM provider.
#[derive(Clone, Debug)]
pub struct ProviderConfig {
    /// Which provider this configuration targets.
    pub kind: ProviderKind,
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
    /// Create a new provider config for the given kind.
    ///
    /// The `base_url` defaults to the provider's standard endpoint.
    /// Use [`with_base_url`](Self::with_base_url) to override.
    pub fn new(kind: ProviderKind, api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            base_url: kind.default_base_url().to_string(),
            kind,
            api_key: api_key.into(),
            model: model.into(),
            max_tokens: 4096,
        }
    }

    /// Override the base URL.
    #[must_use]
    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    /// Set the max tokens.
    #[must_use]
    pub fn with_max_tokens(mut self, max: u32) -> Self {
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

/// Factory function type for creating providers.
type ProviderFactory =
    Box<dyn Fn(ProviderConfig) -> Result<Box<dyn StreamingProvider>> + Send + Sync>;

/// Registry of available LLM providers.
///
/// Maps [`ProviderKind`] values to factory functions that create provider instances.
pub struct ProviderRegistry {
    factories: HashMap<ProviderKind, ProviderFactory>,
}

impl ProviderRegistry {
    /// Create an empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            factories: HashMap::new(),
        }
    }

    /// Register a provider factory for the given kind.
    pub fn register<F>(&mut self, kind: ProviderKind, factory: F)
    where
        F: Fn(ProviderConfig) -> Result<Box<dyn StreamingProvider>> + Send + Sync + 'static,
    {
        self.factories.insert(kind, Box::new(factory));
    }

    /// Create a provider instance from the config.
    ///
    /// Returns an error if no factory is registered for `config.kind`.
    pub fn create(&self, config: ProviderConfig) -> Result<Box<dyn StreamingProvider>> {
        let factory = self
            .factories
            .get(&config.kind)
            .ok_or_else(|| FaeAiError::Provider {
                provider: config.kind.display_name().to_string(),
                message: "no factory registered for this provider".to_string(),
            })?;
        factory(config)
    }

    /// Check whether a factory is registered for the given kind.
    #[must_use]
    pub fn has_provider(&self, kind: ProviderKind) -> bool {
        self.factories.contains_key(&kind)
    }
}

impl Default for ProviderRegistry {
    /// Create a registry pre-loaded with available providers.
    fn default() -> Self {
        let mut reg = Self::new();
        reg.register(ProviderKind::Anthropic, |config| {
            let provider = crate::anthropic::AnthropicProvider::new(config)?;
            Ok(Box::new(provider))
        });
        reg.register(ProviderKind::OpenAi, |config| {
            let provider = crate::openai::OpenAiProvider::new(config)?;
            Ok(Box::new(provider))
        });
        reg
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_kind_default_base_url() {
        assert_eq!(
            ProviderKind::Anthropic.default_base_url(),
            "https://api.anthropic.com"
        );
        assert_eq!(
            ProviderKind::OpenAi.default_base_url(),
            "https://api.openai.com"
        );
        assert_eq!(
            ProviderKind::Gemini.default_base_url(),
            "https://generativelanguage.googleapis.com/v1beta"
        );
        assert_eq!(
            ProviderKind::Ollama.default_base_url(),
            "http://localhost:11434"
        );
        assert_eq!(ProviderKind::OpenAiCompatible.default_base_url(), "");
    }

    #[test]
    fn provider_kind_display_name() {
        assert_eq!(ProviderKind::Anthropic.display_name(), "Anthropic");
        assert_eq!(ProviderKind::OpenAi.display_name(), "OpenAI");
        assert_eq!(ProviderKind::Gemini.display_name(), "Google Gemini");
        assert_eq!(ProviderKind::Ollama.display_name(), "Ollama");
        assert_eq!(
            ProviderKind::OpenAiCompatible.display_name(),
            "OpenAI-Compatible"
        );
    }

    #[test]
    fn provider_config_defaults_from_kind() {
        let config = ProviderConfig::new(
            ProviderKind::Anthropic,
            "sk-test",
            "claude-sonnet-4-5-20250929",
        );
        assert_eq!(config.base_url, "https://api.anthropic.com");
        assert_eq!(config.max_tokens, 4096);
        assert_eq!(config.kind, ProviderKind::Anthropic);

        let config = ProviderConfig::new(ProviderKind::OpenAi, "sk-test", "gpt-4o");
        assert_eq!(config.base_url, "https://api.openai.com");

        let config = ProviderConfig::new(ProviderKind::Ollama, "", "llama3");
        assert_eq!(config.base_url, "http://localhost:11434");
    }

    #[test]
    fn provider_config_custom_base_url() {
        let config = ProviderConfig::new(ProviderKind::Anthropic, "key", "model")
            .with_base_url("https://custom.api.com");
        assert_eq!(config.base_url, "https://custom.api.com");
    }

    #[test]
    fn provider_config_builder() {
        let config = ProviderConfig::new(
            ProviderKind::Anthropic,
            "sk-test",
            "claude-sonnet-4-5-20250929",
        )
        .with_base_url("https://custom.api.com")
        .with_max_tokens(8192);
        assert_eq!(config.api_key, "sk-test");
        assert_eq!(config.model, "claude-sonnet-4-5-20250929");
        assert_eq!(config.base_url, "https://custom.api.com");
        assert_eq!(config.max_tokens, 8192);
    }

    #[test]
    fn registry_has_provider() {
        let reg = ProviderRegistry::default();
        assert!(reg.has_provider(ProviderKind::Anthropic));
        assert!(reg.has_provider(ProviderKind::OpenAi));
        assert!(!reg.has_provider(ProviderKind::Gemini));
        assert!(!reg.has_provider(ProviderKind::Ollama));
        assert!(!reg.has_provider(ProviderKind::OpenAiCompatible));
    }

    #[test]
    fn registry_create_anthropic() {
        let reg = ProviderRegistry::default();
        let config = ProviderConfig::new(
            ProviderKind::Anthropic,
            "sk-test",
            "claude-sonnet-4-5-20250929",
        );
        let result = reg.create(config);
        assert!(result.is_ok());
    }

    #[test]
    fn registry_create_openai() {
        let reg = ProviderRegistry::default();
        let config = ProviderConfig::new(ProviderKind::OpenAi, "sk-test", "gpt-4o");
        let result = reg.create(config);
        assert!(result.is_ok());
    }

    #[test]
    fn registry_create_unknown_returns_error() {
        let reg = ProviderRegistry::default();
        let config = ProviderConfig::new(ProviderKind::Gemini, "key", "gemini-2.0-flash");
        let result = reg.create(config);
        assert!(result.is_err());
    }

    #[test]
    fn registry_custom_factory() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicBool, Ordering};

        let called = Arc::new(AtomicBool::new(false));
        let called_clone = called.clone();

        let mut reg = ProviderRegistry::new();
        reg.register(ProviderKind::Anthropic, move |config| {
            called_clone.store(true, Ordering::Relaxed);
            crate::anthropic::AnthropicProvider::new(config)
                .map(|p| Box::new(p) as Box<dyn StreamingProvider>)
        });

        let config = ProviderConfig::new(
            ProviderKind::Anthropic,
            "sk-test",
            "claude-sonnet-4-5-20250929",
        );
        let result = reg.create(config);
        assert!(result.is_ok());
        assert!(called.load(Ordering::Relaxed));
    }
}
