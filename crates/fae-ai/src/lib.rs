//! fae-ai: Unified multi-provider LLM API.
//!
//! Provides a common interface for streaming completions, tool calling,
//! and authentication across multiple LLM providers.

pub mod anthropic;
pub mod error;
pub mod gemini;
pub mod message;
pub mod models;
pub mod ollama;
pub mod openai;
pub mod openai_compat;
pub mod provider;
pub mod tokens;
pub mod types;

pub use anthropic::AnthropicProvider;
pub use error::{FaeAiError, Result};
pub use gemini::GeminiProvider;
pub use message::{ContentBlock, Message, Role, ToolDefinition};
pub use models::{
    ModelInfo, get_context_window, lookup_model, lookup_model_by_prefix, supports_tools,
    supports_vision,
};
pub use ollama::OllamaProvider;
pub use openai::OpenAiProvider;
pub use openai_compat::{OpenAiCompatBuilder, OpenAiCompatProvider};
pub use provider::{Provider, ProviderConfig, ProviderKind, ProviderRegistry, StreamingProvider};
pub use types::{
    CompletionRequest, CompletionResponse, ContentDelta, StopReason, StreamEvent, Usage,
};
