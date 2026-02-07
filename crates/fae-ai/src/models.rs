//! Model information and registry.
//!
//! Provides metadata about known LLM models including context windows,
//! capability flags, and provider information.

use crate::provider::ProviderKind;

/// Information about a specific LLM model.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ModelInfo {
    /// Model identifier (e.g., "gpt-4o", "claude-4-sonnet")
    pub name: &'static str,
    /// Which provider this model belongs to
    pub provider: ProviderKind,
    /// Maximum context window size in tokens
    pub context_window: u32,
    /// Whether this model supports tool/function calling
    pub supports_tools: bool,
    /// Whether this model supports vision/image inputs
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

/// Look up model information by name.
///
/// Returns `Some` if the model is in the registry, `None` otherwise.
pub fn lookup_model(name: &str) -> Option<ModelInfo> {
    MODEL_REGISTRY.iter().find(|m| m.name == name).copied()
}

/// Get the context window size for a known model.
///
/// Returns `Some` if the model is registered, `None` otherwise.
pub fn get_context_window(name: &str) -> Option<u32> {
    lookup_model(name).map(|m| m.context_window)
}

/// Check if a model supports tool calling.
///
/// Returns `Some(bool)` if the model is known, `None` if unknown.
pub fn supports_tools(name: &str) -> Option<bool> {
    lookup_model(name).map(|m| m.supports_tools)
}

/// Check if a model supports vision inputs.
///
/// Returns `Some(bool)` if the model is known, `None` if unknown.
pub fn supports_vision(name: &str) -> Option<bool> {
    lookup_model(name).map(|m| m.supports_vision)
}

// ============================================================================
// Model Registry
// ============================================================================

/// Registry of all known models.
const MODEL_REGISTRY: &[ModelInfo] = &[
    // OpenAI Models
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
        name: "gpt-4",
        provider: ProviderKind::OpenAi,
        context_window: 8_192,
        supports_tools: true,
        supports_vision: false,
    },
    ModelInfo {
        name: "o1",
        provider: ProviderKind::OpenAi,
        context_window: 128_000,
        supports_tools: false,
        supports_vision: false,
    },
    ModelInfo {
        name: "o3-mini",
        provider: ProviderKind::OpenAi,
        context_window: 128_000,
        supports_tools: false,
        supports_vision: false,
    },
    // Google Gemini Models
    ModelInfo {
        name: "gemini-2.0-flash",
        provider: ProviderKind::Gemini,
        context_window: 1_000_000,
        supports_tools: true,
        supports_vision: true,
    },
    ModelInfo {
        name: "gemini-1.5-pro",
        provider: ProviderKind::Gemini,
        context_window: 1_000_000,
        supports_tools: true,
        supports_vision: true,
    },
    ModelInfo {
        name: "gemini-1.5-flash",
        provider: ProviderKind::Gemini,
        context_window: 1_000_000,
        supports_tools: true,
        supports_vision: true,
    },
    ModelInfo {
        name: "gemini-pro",
        provider: ProviderKind::Gemini,
        context_window: 32_768,
        supports_tools: true,
        supports_vision: false,
    },
    // Anthropic Claude Models
    ModelInfo {
        name: "claude-4-opus",
        provider: ProviderKind::Anthropic,
        context_window: 200_000,
        supports_tools: true,
        supports_vision: true,
    },
    ModelInfo {
        name: "claude-4-sonnet",
        provider: ProviderKind::Anthropic,
        context_window: 200_000,
        supports_tools: true,
        supports_vision: true,
    },
    ModelInfo {
        name: "claude-3.5-haiku",
        provider: ProviderKind::Anthropic,
        context_window: 200_000,
        supports_tools: true,
        supports_vision: true,
    },
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
    // Ollama Models (approximate context windows)
    ModelInfo {
        name: "llama3",
        provider: ProviderKind::Ollama,
        context_window: 8_192,
        supports_tools: false,
        supports_vision: false,
    },
    ModelInfo {
        name: "llama3:70b",
        provider: ProviderKind::Ollama,
        context_window: 8_192,
        supports_tools: false,
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
        context_window: 8_192,
        supports_tools: false,
        supports_vision: false,
    },
    ModelInfo {
        name: "neural-chat",
        provider: ProviderKind::Ollama,
        context_window: 4_096,
        supports_tools: false,
        supports_vision: false,
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_model_known() {
        let model = lookup_model("gpt-4o");
        assert!(model.is_some());
        match model {
            Some(m) => {
                assert_eq!(m.name, "gpt-4o");
                assert_eq!(m.provider, ProviderKind::OpenAi);
                assert_eq!(m.context_window, 128_000);
                assert!(m.supports_tools);
                assert!(m.supports_vision);
            }
            None => unreachable!(),
        }
    }

    #[test]
    fn lookup_model_unknown() {
        assert_eq!(lookup_model("unknown-model"), None);
    }

    #[test]
    fn lookup_claude_model() {
        let model = lookup_model("claude-4-sonnet");
        assert!(model.is_some());
        match model {
            Some(m) => {
                assert_eq!(m.provider, ProviderKind::Anthropic);
                assert_eq!(m.context_window, 200_000);
            }
            None => unreachable!(),
        }
    }

    #[test]
    fn lookup_gemini_model() {
        let model = lookup_model("gemini-2.0-flash");
        assert!(model.is_some());
        match model {
            Some(m) => {
                assert_eq!(m.provider, ProviderKind::Gemini);
                assert_eq!(m.context_window, 1_000_000);
            }
            None => unreachable!(),
        }
    }

    #[test]
    fn lookup_ollama_model() {
        let model = lookup_model("llama3");
        assert!(model.is_some());
        match model {
            Some(m) => {
                assert_eq!(m.provider, ProviderKind::Ollama);
                assert_eq!(m.context_window, 8_192);
            }
            None => unreachable!(),
        }
    }

    #[test]
    fn get_context_window_known() {
        assert_eq!(get_context_window("gpt-4o"), Some(128_000));
        assert_eq!(get_context_window("claude-4-sonnet"), Some(200_000));
        assert_eq!(get_context_window("gemini-2.0-flash"), Some(1_000_000));
    }

    #[test]
    fn get_context_window_unknown() {
        assert_eq!(get_context_window("unknown-model"), None);
    }

    #[test]
    fn supports_tools_flags() {
        assert_eq!(supports_tools("gpt-4o"), Some(true));
        assert_eq!(supports_tools("o1"), Some(false));
        assert_eq!(supports_tools("llama3"), Some(false));
        assert_eq!(supports_tools("unknown-model"), None);
    }

    #[test]
    fn supports_vision_flags() {
        assert_eq!(supports_vision("gpt-4o"), Some(true));
        assert_eq!(supports_vision("gpt-4"), Some(false));
        assert_eq!(supports_vision("gemini-2.0-flash"), Some(true));
        assert_eq!(supports_vision("llama3"), Some(false));
        assert_eq!(supports_vision("unknown-model"), None);
    }

    #[test]
    fn model_registry_has_entries() {
        assert!(!MODEL_REGISTRY.is_empty());
        assert!(MODEL_REGISTRY.len() > 10);
    }

    #[test]
    fn all_openai_models_found() {
        assert!(lookup_model("gpt-4o").is_some());
        assert!(lookup_model("gpt-4o-mini").is_some());
        assert!(lookup_model("gpt-4-turbo").is_some());
        assert!(lookup_model("o1").is_some());
        assert!(lookup_model("o3-mini").is_some());
    }

    #[test]
    fn all_claude_models_found() {
        assert!(lookup_model("claude-4-opus").is_some());
        assert!(lookup_model("claude-4-sonnet").is_some());
        assert!(lookup_model("claude-3.5-haiku").is_some());
    }

    #[test]
    fn all_gemini_models_found() {
        assert!(lookup_model("gemini-2.0-flash").is_some());
        assert!(lookup_model("gemini-1.5-pro").is_some());
        assert!(lookup_model("gemini-1.5-flash").is_some());
    }
}
