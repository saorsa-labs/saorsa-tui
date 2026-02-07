//! fae-ai: Unified multi-provider LLM API.
//!
//! Provides a common interface for streaming completions, tool calling,
//! and authentication across multiple LLM providers.

pub mod anthropic;
pub mod error;
pub mod message;
pub mod openai;
pub mod provider;
pub mod tokens;
pub mod types;

pub use anthropic::AnthropicProvider;
pub use error::{FaeAiError, Result};
pub use message::{ContentBlock, Message, Role, ToolDefinition};
pub use openai::OpenAiProvider;
pub use provider::{Provider, ProviderConfig, ProviderKind, ProviderRegistry, StreamingProvider};
pub use types::{
    CompletionRequest, CompletionResponse, ContentDelta, StopReason, StreamEvent, Usage,
};
