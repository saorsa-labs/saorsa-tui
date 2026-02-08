//! saorsa-ai: Unified multi-provider LLM API.
//!
//! Provides a common interface for streaming completions, tool calling,
//! and authentication across multiple LLM providers.
//!
//! # Architecture Overview
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                 Application / Agent Layer                   │
//! │  (Sends CompletionRequest, receives StreamEvent stream)     │
//! └─────────────────────────────────────────────────────────────┘
//!                              │
//!                              ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │              ProviderRegistry (Factory)                     │
//! │  ProviderKind → ProviderConfig → Box<dyn Provider>          │
//! └─────────────────────────────────────────────────────────────┘
//!                              │
//!                ┌─────────────┼─────────────┬─────────────┐
//!                ▼             ▼             ▼             ▼
//!       ┌──────────────┬──────────────┬──────────────┬──────────────┐
//!       │  Anthropic   │   OpenAI     │   Gemini     │   Ollama     │
//!       │   Provider   │   Provider   │   Provider   │   Provider   │
//!       └──────────────┴──────────────┴──────────────┴──────────────┘
//!                              │
//!                              ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │         Streaming HTTP (reqwest, Server-Sent Events)        │
//! │  POST /v1/messages → stream of JSON events → StreamEvent    │
//! └─────────────────────────────────────────────────────────────┘
//!                              │
//!                              ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │          Message Protocol (vendor-agnostic types)           │
//! │  Message, ContentBlock, ToolDefinition, ContentDelta        │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Provider Abstraction
//!
//! All providers implement the `Provider` trait:
//!
//! - **`stream_completion`**: Returns `Pin<Box<dyn Stream<Item = Result<StreamEvent>>>>`
//! - **Unified event types**: `StreamEvent::{ContentDelta, ToolUse, Done, Error}`
//! - **Model metadata**: Context windows, tool support, vision capabilities
//!
//! ## Supported Providers
//!
//! - **Anthropic**: Claude models with streaming, tool use, vision
//! - **OpenAI**: GPT models with streaming, function calling, vision
//! - **Gemini**: Google Gemini with streaming and tool use
//! - **Ollama**: Local model hosting with OpenAI-compatible API
//! - **OpenAI-Compatible**: Generic adapter for compatible APIs (Groq, etc.)
//!
//! ## Key Types
//!
//! - `Provider`: Core trait for LLM completion providers
//! - `CompletionRequest`: Vendor-agnostic request (messages, tools, params)
//! - `StreamEvent`: Streaming events (content deltas, tool calls, completion)
//! - `Message`: Conversation message with role and content blocks
//! - `ToolDefinition`: JSON Schema-based tool specification

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
pub use error::{Result, SaorsaAiError};
pub use gemini::GeminiProvider;
pub use message::{ContentBlock, Message, Role, ToolDefinition};
pub use models::{
    ModelInfo, all_models, get_context_window, lookup_by_provider_prefix, lookup_model,
    lookup_model_by_prefix, supports_tools, supports_vision,
};
pub use ollama::OllamaProvider;
pub use openai::OpenAiProvider;
pub use openai_compat::{OpenAiCompatBuilder, OpenAiCompatProvider};
pub use provider::{
    Provider, ProviderConfig, ProviderKind, ProviderRegistry, StreamingProvider, determine_provider,
};
pub use types::{
    CompletionRequest, CompletionResponse, ContentDelta, StopReason, StreamEvent, ThinkingConfig,
    Usage,
};
