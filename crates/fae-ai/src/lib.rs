//! fae-ai: Unified multi-provider LLM API.
//!
//! Provides a common interface for streaming completions, tool calling,
//! and authentication across multiple LLM providers.

pub mod error;

pub use error::{FaeAiError, Result};
