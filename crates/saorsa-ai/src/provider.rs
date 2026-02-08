//! Provider trait for LLM backends.

use std::collections::HashMap;

use crate::error::{Result, SaorsaAiError};
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
    /// Any OpenAI-compatible API (Azure, Groq, Mistral, xAI, etc.).
    OpenAiCompatible,
    /// LM Studio — local inference server.
    LmStudio,
    /// vLLM — high-performance local inference.
    Vllm,
    /// OpenRouter — multi-model API gateway.
    OpenRouter,
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
            Self::LmStudio => "http://localhost:1234/v1",
            Self::Vllm => "http://localhost:8000/v1",
            Self::OpenRouter => "https://openrouter.ai/api",
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
            Self::LmStudio => "LM Studio",
            Self::Vllm => "vLLM",
            Self::OpenRouter => "OpenRouter",
        }
    }

    /// Returns the standard environment variable name for this provider's API key.
    #[must_use]
    pub fn env_var_name(self) -> &'static str {
        match self {
            Self::Anthropic => "ANTHROPIC_API_KEY",
            Self::OpenAi => "OPENAI_API_KEY",
            Self::Gemini => "GOOGLE_API_KEY",
            Self::Ollama => "OLLAMA_API_KEY",
            Self::OpenAiCompatible => "OPENAI_API_KEY",
            Self::LmStudio => "LMSTUDIO_API_KEY",
            Self::Vllm => "VLLM_API_KEY",
            Self::OpenRouter => "OPENROUTER_API_KEY",
        }
    }
}

/// Determine the provider kind for a given model string.
///
/// Resolution order:
/// 1. Exact match in the known models registry.
/// 2. `"provider/model"` prefix format (e.g., `"groq/llama-3.3-70b"`).
/// 3. Prefix match in the known models registry (e.g., versioned names).
pub fn determine_provider(model: &str) -> Option<ProviderKind> {
    // 1. Check exact match in known models
    if let Some(info) = crate::models::lookup_model(model) {
        return Some(info.provider);
    }
    // 2. Handle "provider/model" format before prefix match to avoid
    //    false matches (e.g., "mistral/..." matching Ollama's "mistral" model).
    if let Some((prefix, _)) = model.split_once('/') {
        return match prefix {
            "anthropic" => Some(ProviderKind::Anthropic),
            "openai" => Some(ProviderKind::OpenAi),
            "google" | "gemini" => Some(ProviderKind::Gemini),
            "ollama" => Some(ProviderKind::Ollama),
            "openrouter" => Some(ProviderKind::OpenRouter),
            "lmstudio" | "lm-studio" => Some(ProviderKind::LmStudio),
            "vllm" => Some(ProviderKind::Vllm),
            "groq" | "mistral" | "xai" | "cerebras" | "azure" => {
                Some(ProviderKind::OpenAiCompatible)
            }
            // Unknown prefix with slash: check if any known model matches
            // (handles OpenRouter-style models like "meta-llama/...")
            _ => crate::models::lookup_model_by_prefix(model).map(|info| info.provider),
        };
    }
    // 3. Prefix match for versioned model names (e.g., "claude-sonnet-4-5-20250929")
    crate::models::lookup_model_by_prefix(model).map(|info| info.provider)
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
            .ok_or_else(|| SaorsaAiError::Provider {
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
        reg.register(ProviderKind::Gemini, |config| {
            let provider = crate::gemini::GeminiProvider::new(config)?;
            Ok(Box::new(provider))
        });
        reg.register(ProviderKind::Ollama, |config| {
            let provider = crate::ollama::OllamaProvider::new(config)?;
            Ok(Box::new(provider))
        });
        reg.register(ProviderKind::OpenAiCompatible, |config| {
            let provider = crate::openai_compat::OpenAiCompatProvider::new(config)?;
            Ok(Box::new(provider))
        });
        reg.register(ProviderKind::LmStudio, |config| {
            let provider = crate::openai_compat::OpenAiCompatProvider::new(config)?;
            Ok(Box::new(provider))
        });
        reg.register(ProviderKind::Vllm, |config| {
            let provider = crate::openai_compat::OpenAiCompatProvider::new(config)?;
            Ok(Box::new(provider))
        });
        reg.register(ProviderKind::OpenRouter, |config| {
            let provider = crate::openai_compat::OpenAiCompatProvider::new(config)?;
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
        assert_eq!(
            ProviderKind::LmStudio.default_base_url(),
            "http://localhost:1234/v1"
        );
        assert_eq!(
            ProviderKind::Vllm.default_base_url(),
            "http://localhost:8000/v1"
        );
        assert_eq!(
            ProviderKind::OpenRouter.default_base_url(),
            "https://openrouter.ai/api"
        );
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
        assert_eq!(ProviderKind::LmStudio.display_name(), "LM Studio");
        assert_eq!(ProviderKind::Vllm.display_name(), "vLLM");
        assert_eq!(ProviderKind::OpenRouter.display_name(), "OpenRouter");
    }

    #[test]
    fn provider_kind_env_var_name() {
        assert_eq!(ProviderKind::Anthropic.env_var_name(), "ANTHROPIC_API_KEY");
        assert_eq!(ProviderKind::OpenAi.env_var_name(), "OPENAI_API_KEY");
        assert_eq!(ProviderKind::Gemini.env_var_name(), "GOOGLE_API_KEY");
        assert_eq!(ProviderKind::Ollama.env_var_name(), "OLLAMA_API_KEY");
        assert_eq!(
            ProviderKind::OpenAiCompatible.env_var_name(),
            "OPENAI_API_KEY"
        );
        assert_eq!(ProviderKind::LmStudio.env_var_name(), "LMSTUDIO_API_KEY");
        assert_eq!(ProviderKind::Vllm.env_var_name(), "VLLM_API_KEY");
        assert_eq!(
            ProviderKind::OpenRouter.env_var_name(),
            "OPENROUTER_API_KEY"
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
        assert!(reg.has_provider(ProviderKind::Gemini));
        assert!(reg.has_provider(ProviderKind::Ollama));
        assert!(reg.has_provider(ProviderKind::OpenAiCompatible));
        assert!(reg.has_provider(ProviderKind::LmStudio));
        assert!(reg.has_provider(ProviderKind::Vllm));
        assert!(reg.has_provider(ProviderKind::OpenRouter));
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
    fn registry_create_gemini() {
        let reg = ProviderRegistry::default();
        let config = ProviderConfig::new(ProviderKind::Gemini, "test-key", "gemini-2.0-flash");
        let result = reg.create(config);
        assert!(result.is_ok());
    }

    #[test]
    fn registry_create_ollama() {
        let reg = ProviderRegistry::default();
        let config = ProviderConfig::new(ProviderKind::Ollama, "", "llama3");
        let result = reg.create(config);
        assert!(result.is_ok());
    }

    #[test]
    fn registry_create_unknown_returns_error() {
        let reg = ProviderRegistry::new();
        let config = ProviderConfig::new(ProviderKind::Anthropic, "key", "model");
        let result = reg.create(config);
        assert!(result.is_err());
    }

    #[test]
    fn registry_create_openai_compatible() {
        let reg = ProviderRegistry::default();
        let config = ProviderConfig::new(ProviderKind::OpenAiCompatible, "key", "model")
            .with_base_url("https://api.example.com");
        let result = reg.create(config);
        assert!(result.is_ok());
    }

    #[test]
    fn registry_create_lm_studio() {
        let reg = ProviderRegistry::default();
        let config = ProviderConfig::new(ProviderKind::LmStudio, "", "local-model");
        let result = reg.create(config);
        assert!(result.is_ok());
    }

    #[test]
    fn registry_create_vllm() {
        let reg = ProviderRegistry::default();
        let config = ProviderConfig::new(ProviderKind::Vllm, "", "local-model");
        let result = reg.create(config);
        assert!(result.is_ok());
    }

    #[test]
    fn registry_create_openrouter() {
        let reg = ProviderRegistry::default();
        let config = ProviderConfig::new(ProviderKind::OpenRouter, "key", "openai/gpt-4o");
        let result = reg.create(config);
        assert!(result.is_ok());
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

    #[test]
    fn determine_provider_known_model() {
        assert_eq!(determine_provider("gpt-4o"), Some(ProviderKind::OpenAi));
        assert_eq!(
            determine_provider("claude-sonnet-4"),
            Some(ProviderKind::Anthropic)
        );
        assert_eq!(
            determine_provider("gemini-2.0-flash"),
            Some(ProviderKind::Gemini)
        );
        assert_eq!(determine_provider("llama3"), Some(ProviderKind::Ollama));
    }

    #[test]
    fn determine_provider_prefix_match() {
        assert_eq!(
            determine_provider("claude-sonnet-4-5-20250929"),
            Some(ProviderKind::Anthropic)
        );
        assert_eq!(
            determine_provider("gpt-4o-2024-08-06"),
            Some(ProviderKind::OpenAi)
        );
    }

    #[test]
    fn determine_provider_slash_format() {
        // Unknown model with provider prefix falls through to slash-prefix logic
        assert_eq!(
            determine_provider("anthropic/some-new-model"),
            Some(ProviderKind::Anthropic)
        );
        // "openai/gpt-4o" is a known OpenRouter model, so registry match wins
        assert_eq!(
            determine_provider("openai/gpt-4o"),
            Some(ProviderKind::OpenRouter)
        );
        assert_eq!(
            determine_provider("google/gemini-pro"),
            Some(ProviderKind::Gemini)
        );
        assert_eq!(
            determine_provider("gemini/gemini-pro"),
            Some(ProviderKind::Gemini)
        );
        assert_eq!(
            determine_provider("ollama/llama3"),
            Some(ProviderKind::Ollama)
        );
        assert_eq!(
            determine_provider("openrouter/some-model"),
            Some(ProviderKind::OpenRouter)
        );
        assert_eq!(
            determine_provider("lmstudio/some-model"),
            Some(ProviderKind::LmStudio)
        );
        assert_eq!(
            determine_provider("lm-studio/some-model"),
            Some(ProviderKind::LmStudio)
        );
        assert_eq!(
            determine_provider("vllm/some-model"),
            Some(ProviderKind::Vllm)
        );
        assert_eq!(
            determine_provider("groq/llama-3.3-70b"),
            Some(ProviderKind::OpenAiCompatible)
        );
        assert_eq!(
            determine_provider("mistral/mistral-large"),
            Some(ProviderKind::OpenAiCompatible)
        );
        assert_eq!(
            determine_provider("xai/grok-2"),
            Some(ProviderKind::OpenAiCompatible)
        );
        assert_eq!(
            determine_provider("cerebras/llama3"),
            Some(ProviderKind::OpenAiCompatible)
        );
        assert_eq!(
            determine_provider("azure/gpt-4"),
            Some(ProviderKind::OpenAiCompatible)
        );
    }

    #[test]
    fn determine_provider_unknown() {
        assert_eq!(determine_provider("totally-unknown"), None);
        assert_eq!(determine_provider("unknown-provider/model"), None);
    }

    #[test]
    fn determine_provider_openrouter_models_from_registry() {
        assert_eq!(
            determine_provider("meta-llama/llama-3.1-405b-instruct"),
            Some(ProviderKind::OpenRouter)
        );
        assert_eq!(
            determine_provider("anthropic/claude-sonnet-4"),
            Some(ProviderKind::OpenRouter)
        );
        assert_eq!(
            determine_provider("openai/gpt-4o"),
            Some(ProviderKind::OpenRouter)
        );
        assert_eq!(
            determine_provider("google/gemini-2.0-flash"),
            Some(ProviderKind::OpenRouter)
        );
    }
}
