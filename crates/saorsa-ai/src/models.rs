//! Model registry for known LLM models.
//!
//! Provides a lookup table of known models with context window sizes,
//! capability flags, and provider associations.

use crate::provider::ProviderKind;

/// Information about a known LLM model.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModelInfo {
    /// Model identifier (e.g., "gpt-4o", "claude-sonnet-4").
    pub name: &'static str,
    /// Which provider this model belongs to.
    pub provider: ProviderKind,
    /// Context window size in tokens.
    pub context_window: u32,
    /// Whether this model supports tool/function calling.
    pub supports_tools: bool,
    /// Whether this model supports vision/image inputs.
    pub supports_vision: bool,
}

impl ModelInfo {
    /// Create a new model info entry.
    pub const fn new(
        name: &'static str,
        provider: ProviderKind,
        context_window: u32,
        supports_tools: bool,
        supports_vision: bool,
    ) -> Self {
        Self {
            name,
            provider,
            context_window,
            supports_tools,
            supports_vision,
        }
    }
}

/// Known models registry.
const KNOWN_MODELS: &[ModelInfo] = &[
    // ── Anthropic ──
    ModelInfo {
        name: "claude-opus-4",
        provider: ProviderKind::Anthropic,
        context_window: 200_000,
        supports_tools: true,
        supports_vision: true,
    },
    ModelInfo {
        name: "claude-sonnet-4",
        provider: ProviderKind::Anthropic,
        context_window: 200_000,
        supports_tools: true,
        supports_vision: true,
    },
    ModelInfo {
        name: "claude-haiku-4",
        provider: ProviderKind::Anthropic,
        context_window: 200_000,
        supports_tools: true,
        supports_vision: true,
    },
    ModelInfo {
        name: "claude-3-5-sonnet",
        provider: ProviderKind::Anthropic,
        context_window: 200_000,
        supports_tools: true,
        supports_vision: true,
    },
    ModelInfo {
        name: "claude-3-5-haiku",
        provider: ProviderKind::Anthropic,
        context_window: 200_000,
        supports_tools: true,
        supports_vision: true,
    },
    ModelInfo {
        name: "claude-3-opus",
        provider: ProviderKind::Anthropic,
        context_window: 200_000,
        supports_tools: true,
        supports_vision: true,
    },
    // ── OpenAI ──
    ModelInfo {
        name: "gpt-4o",
        provider: ProviderKind::OpenAi,
        context_window: 128_000,
        supports_tools: true,
        supports_vision: true,
    },
    ModelInfo {
        name: "gpt-4o-mini",
        provider: ProviderKind::OpenAi,
        context_window: 128_000,
        supports_tools: true,
        supports_vision: true,
    },
    ModelInfo {
        name: "gpt-4-turbo",
        provider: ProviderKind::OpenAi,
        context_window: 128_000,
        supports_tools: true,
        supports_vision: true,
    },
    ModelInfo {
        name: "o1",
        provider: ProviderKind::OpenAi,
        context_window: 200_000,
        supports_tools: true,
        supports_vision: true,
    },
    ModelInfo {
        name: "o3-mini",
        provider: ProviderKind::OpenAi,
        context_window: 200_000,
        supports_tools: true,
        supports_vision: false,
    },
    // ── Google Gemini ──
    ModelInfo {
        name: "gemini-2.0-flash",
        provider: ProviderKind::Gemini,
        context_window: 1_048_576,
        supports_tools: true,
        supports_vision: true,
    },
    ModelInfo {
        name: "gemini-1.5-pro",
        provider: ProviderKind::Gemini,
        context_window: 2_097_152,
        supports_tools: true,
        supports_vision: true,
    },
    ModelInfo {
        name: "gemini-1.5-flash",
        provider: ProviderKind::Gemini,
        context_window: 1_048_576,
        supports_tools: true,
        supports_vision: true,
    },
    // ── Ollama ──
    ModelInfo {
        name: "llama3",
        provider: ProviderKind::Ollama,
        context_window: 8_192,
        supports_tools: true,
        supports_vision: false,
    },
    ModelInfo {
        name: "llama3.1",
        provider: ProviderKind::Ollama,
        context_window: 131_072,
        supports_tools: true,
        supports_vision: false,
    },
    ModelInfo {
        name: "codellama",
        provider: ProviderKind::Ollama,
        context_window: 16_384,
        supports_tools: false,
        supports_vision: false,
    },
    ModelInfo {
        name: "mistral",
        provider: ProviderKind::Ollama,
        context_window: 32_768,
        supports_tools: true,
        supports_vision: false,
    },
    ModelInfo {
        name: "mixtral",
        provider: ProviderKind::Ollama,
        context_window: 32_768,
        supports_tools: true,
        supports_vision: false,
    },
    ModelInfo {
        name: "llava",
        provider: ProviderKind::Ollama,
        context_window: 4_096,
        supports_tools: false,
        supports_vision: true,
    },
];

/// Look up a model by exact name.
pub fn lookup_model(name: &str) -> Option<ModelInfo> {
    KNOWN_MODELS.iter().find(|m| m.name == name).copied()
}

/// Look up a model by prefix match.
///
/// Useful for versioned model names (e.g., "claude-sonnet-4-5-20250929"
/// starts with "claude-sonnet-4"). Returns the first match.
pub fn lookup_model_by_prefix(name: &str) -> Option<ModelInfo> {
    KNOWN_MODELS
        .iter()
        .find(|m| name.starts_with(m.name))
        .copied()
}

/// Get the context window size for a model.
///
/// Tries exact match first, then prefix match. Returns `None` for unknown models.
pub fn get_context_window(model: &str) -> Option<u32> {
    lookup_model(model)
        .or_else(|| lookup_model_by_prefix(model))
        .map(|m| m.context_window)
}

/// Check if a model supports tool/function calling.
///
/// Returns `None` for unknown models.
pub fn supports_tools(model: &str) -> Option<bool> {
    lookup_model(model)
        .or_else(|| lookup_model_by_prefix(model))
        .map(|m| m.supports_tools)
}

/// Check if a model supports vision/image inputs.
///
/// Returns `None` for unknown models.
pub fn supports_vision(model: &str) -> Option<bool> {
    lookup_model(model)
        .or_else(|| lookup_model_by_prefix(model))
        .map(|m| m.supports_vision)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_exact_match() {
        let model = lookup_model("gpt-4o");
        assert!(model.is_some());
        if let Some(m) = model {
            assert_eq!(m.provider, ProviderKind::OpenAi);
            assert_eq!(m.context_window, 128_000);
            assert!(m.supports_tools);
            assert!(m.supports_vision);
        }
    }

    #[test]
    fn lookup_claude_exact() {
        let model = lookup_model("claude-sonnet-4");
        assert!(model.is_some());
        if let Some(m) = model {
            assert_eq!(m.provider, ProviderKind::Anthropic);
            assert_eq!(m.context_window, 200_000);
        }
    }

    #[test]
    fn lookup_unknown_returns_none() {
        assert!(lookup_model("nonexistent-model").is_none());
    }

    #[test]
    fn lookup_prefix_match() {
        let model = lookup_model_by_prefix("gpt-4o-2024-08-06");
        assert!(model.is_some());
        if let Some(m) = model {
            assert_eq!(m.name, "gpt-4o");
            assert_eq!(m.context_window, 128_000);
        }
    }

    #[test]
    fn lookup_prefix_claude_versioned() {
        let model = lookup_model_by_prefix("claude-sonnet-4-5-20250929");
        assert!(model.is_some());
        if let Some(m) = model {
            assert_eq!(m.context_window, 200_000);
        }
    }

    #[test]
    fn lookup_prefix_no_match() {
        assert!(lookup_model_by_prefix("nonexistent").is_none());
    }

    #[test]
    fn context_window_exact() {
        assert_eq!(get_context_window("gpt-4o"), Some(128_000));
        assert_eq!(get_context_window("gemini-2.0-flash"), Some(1_048_576));
        assert_eq!(get_context_window("llama3"), Some(8_192));
    }

    #[test]
    fn context_window_prefix_fallback() {
        assert_eq!(
            get_context_window("claude-sonnet-4-5-20250929"),
            Some(200_000)
        );
        assert_eq!(get_context_window("claude-opus-4-20250514"), Some(200_000));
    }

    #[test]
    fn context_window_unknown() {
        assert_eq!(get_context_window("totally-unknown"), None);
    }

    #[test]
    fn supports_tools_check() {
        assert_eq!(supports_tools("gpt-4o"), Some(true));
        assert_eq!(supports_tools("codellama"), Some(false));
        assert_eq!(supports_tools("unknown"), None);
    }

    #[test]
    fn supports_vision_check() {
        assert_eq!(supports_vision("gpt-4o"), Some(true));
        assert_eq!(supports_vision("llama3"), Some(false));
        assert_eq!(supports_vision("llava"), Some(true));
        assert_eq!(supports_vision("unknown"), None);
    }

    #[test]
    fn all_anthropic_models_200k() {
        for model in KNOWN_MODELS
            .iter()
            .filter(|m| m.provider == ProviderKind::Anthropic)
        {
            assert_eq!(
                model.context_window, 200_000,
                "Anthropic model {} should have 200k context",
                model.name
            );
        }
    }

    #[test]
    fn gemini_models_large_context() {
        for model in KNOWN_MODELS
            .iter()
            .filter(|m| m.provider == ProviderKind::Gemini)
        {
            assert!(
                model.context_window >= 1_000_000,
                "Gemini model {} should have 1M+ context, got {}",
                model.name,
                model.context_window
            );
        }
    }

    #[test]
    fn all_models_have_positive_context() {
        for model in KNOWN_MODELS {
            assert!(
                model.context_window > 0,
                "Model {} has zero context window",
                model.name
            );
        }
    }

    #[test]
    fn known_model_count() {
        let anthropic = KNOWN_MODELS
            .iter()
            .filter(|m| m.provider == ProviderKind::Anthropic)
            .count();
        let openai = KNOWN_MODELS
            .iter()
            .filter(|m| m.provider == ProviderKind::OpenAi)
            .count();
        let gemini = KNOWN_MODELS
            .iter()
            .filter(|m| m.provider == ProviderKind::Gemini)
            .count();
        let ollama = KNOWN_MODELS
            .iter()
            .filter(|m| m.provider == ProviderKind::Ollama)
            .count();
        assert!(anthropic >= 3, "Need at least 3 Anthropic models");
        assert!(openai >= 3, "Need at least 3 OpenAI models");
        assert!(gemini >= 2, "Need at least 2 Gemini models");
        assert!(ollama >= 3, "Need at least 3 Ollama models");
    }
}
