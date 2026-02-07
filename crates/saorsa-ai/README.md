# saorsa-ai

A unified, multi-provider LLM API for Rust with streaming, tool calling, and model metadata.

[![Crates.io](https://img.shields.io/crates/v/saorsa-ai.svg)](https://crates.io/crates/saorsa-ai)
[![Documentation](https://docs.rs/saorsa-ai/badge.svg)](https://docs.rs/saorsa-ai)
[![License](https://img.shields.io/crates/l/saorsa-ai.svg)](LICENSE)
[![MSRV](https://img.shields.io/badge/MSRV-1.88-blue.svg)](#minimum-supported-rust-version)

## Overview

**saorsa-ai** provides a single, vendor-agnostic API for interacting with large language models from multiple providers:

- **Anthropic** (Claude) - Messages API with native tool use
- **OpenAI** - Chat Completions API with function calling
- **Google Gemini** - GenerateContent API with function calling
- **Ollama** - Local inference via NDJSON chat API
- **OpenAI-compatible** - Azure OpenAI, Groq, Mistral, OpenRouter, xAI, Cerebras, and any other OpenAI-compatible endpoint

All providers share the same request/response types, streaming events, and tool calling interface. Switch providers by changing a config value - no code changes required.

## Quick Start

Add saorsa-ai to your `Cargo.toml`:

```toml
[dependencies]
saorsa-ai = "0.1"
tokio = { version = "1", features = ["full"] }
```

### Non-Streaming Completion

```rust
use saorsa_ai::{
    CompletionRequest, Message, Provider, ProviderConfig, ProviderKind, ProviderRegistry,
};

#[tokio::main]
async fn main() -> saorsa_ai::Result<()> {
    let config = ProviderConfig::new(
        ProviderKind::Anthropic,
        std::env::var("ANTHROPIC_API_KEY").expect("set ANTHROPIC_API_KEY"),
        "claude-sonnet-4",
    );

    let registry = ProviderRegistry::default();
    let provider = registry.create(config)?;

    let request = CompletionRequest::new(
        "claude-sonnet-4",
        vec![Message::user("What is the capital of France?")],
        1024,
    );

    let response = provider.complete(request).await?;
    for block in &response.content {
        if let saorsa_ai::ContentBlock::Text { text } = block {
            println!("{text}");
        }
    }

    Ok(())
}
```

### Streaming Completion

```rust
use saorsa_ai::{
    CompletionRequest, ContentDelta, Message, ProviderConfig, ProviderKind,
    ProviderRegistry, StreamEvent, StreamingProvider,
};

#[tokio::main]
async fn main() -> saorsa_ai::Result<()> {
    let config = ProviderConfig::new(
        ProviderKind::OpenAi,
        std::env::var("OPENAI_API_KEY").expect("set OPENAI_API_KEY"),
        "gpt-4o",
    );

    let registry = ProviderRegistry::default();
    let provider = registry.create(config)?;

    let request = CompletionRequest::new(
        "gpt-4o",
        vec![Message::user("Explain async/await in Rust")],
        2048,
    ).system("You are a helpful programming tutor.");

    let mut rx = provider.stream(request).await?;

    while let Some(event) = rx.recv().await {
        match event? {
            StreamEvent::ContentBlockDelta {
                delta: ContentDelta::TextDelta { text }, ..
            } => print!("{text}"),
            StreamEvent::MessageDelta { stop_reason, .. } => {
                if stop_reason.is_some() {
                    println!();
                }
            }
            _ => {}
        }
    }

    Ok(())
}
```

## Provider Catalog

### Anthropic (Claude)

| Detail | Value |
|--------|-------|
| **Endpoint** | `https://api.anthropic.com/v1/messages` |
| **Auth** | `x-api-key` header |
| **Streaming** | Server-Sent Events (SSE) |
| **API version** | `2023-06-01` |

**Models:**

| Model | Context | Tools | Vision |
|-------|---------|-------|--------|
| `claude-opus-4` | 200k | Yes | Yes |
| `claude-sonnet-4` | 200k | Yes | Yes |
| `claude-haiku-4` | 200k | Yes | Yes |
| `claude-3-5-sonnet` | 200k | Yes | Yes |
| `claude-3-5-haiku` | 200k | Yes | Yes |
| `claude-3-opus` | 200k | Yes | Yes |

```rust
let config = ProviderConfig::new(
    ProviderKind::Anthropic,
    "sk-ant-...",
    "claude-sonnet-4",
);
```

### OpenAI

| Detail | Value |
|--------|-------|
| **Endpoint** | `https://api.openai.com/v1/chat/completions` |
| **Auth** | `Authorization: Bearer` |
| **Streaming** | Server-Sent Events (SSE) |

**Models:**

| Model | Context | Tools | Vision |
|-------|---------|-------|--------|
| `gpt-4o` | 128k | Yes | Yes |
| `gpt-4o-mini` | 128k | Yes | Yes |
| `gpt-4-turbo` | 128k | Yes | Yes |
| `o1` | 200k | Yes | Yes |
| `o3-mini` | 200k | Yes | No |

```rust
let config = ProviderConfig::new(
    ProviderKind::OpenAi,
    "sk-...",
    "gpt-4o",
);
```

### Google Gemini

| Detail | Value |
|--------|-------|
| **Endpoint** | `https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent` |
| **Auth** | `x-goog-api-key` header |
| **Streaming** | SSE via `streamGenerateContent?alt=sse` |

**Models:**

| Model | Context | Tools | Vision |
|-------|---------|-------|--------|
| `gemini-2.0-flash` | 1M | Yes | Yes |
| `gemini-1.5-pro` | 2M | Yes | Yes |
| `gemini-1.5-flash` | 1M | Yes | Yes |

```rust
let config = ProviderConfig::new(
    ProviderKind::Gemini,
    "AIza...",
    "gemini-2.0-flash",
);
```

### Ollama (Local)

| Detail | Value |
|--------|-------|
| **Endpoint** | `http://localhost:11434/api/chat` |
| **Auth** | Optional Bearer token |
| **Streaming** | Newline-delimited JSON (NDJSON) |

**Models:**

| Model | Context | Tools | Vision |
|-------|---------|-------|--------|
| `llama3` | 8k | No | No |
| `llama3.1` | 131k | Yes | No |
| `codellama` | 16k | No | No |
| `mistral` | 32k | Yes | No |
| `mixtral` | 32k | Yes | No |
| `llava` | 4k | No | Yes |

```rust
let config = ProviderConfig::new(
    ProviderKind::Ollama,
    "", // No API key needed for local
    "llama3.1",
).with_base_url("http://localhost:11434");
```

### OpenAI-Compatible Providers

For any service that implements the OpenAI API format. Factory functions are provided for popular services:

```rust
use saorsa_ai::openai_compat;

// Azure OpenAI
let provider = openai_compat::azure_openai(
    "your-api-key",
    "https://your-resource.openai.azure.com",
    "your-deployment",
    "2024-02-01",
)?;

// Groq
let provider = openai_compat::groq("gsk_...", "llama-3.1-70b-versatile")?;

// Mistral
let provider = openai_compat::mistral("your-key", "mistral-large-latest")?;

// OpenRouter
let provider = openai_compat::openrouter("sk-or-...", "anthropic/claude-3.5-sonnet")?;

// xAI (Grok)
let provider = openai_compat::xai("xai-...", "grok-2")?;

// Cerebras
let provider = openai_compat::cerebras("csk-...", "llama3.1-70b")?;
```

For custom endpoints, use the builder:

```rust
use saorsa_ai::openai_compat::OpenAiCompatProvider;

let provider = OpenAiCompatProvider::builder(config)
    .url_path("/v2/chat/completions")  // Custom API path
    .auth_header("X-Custom-Key")       // Custom auth header
    .extra_header("X-Project-Id", "my-project")
    .build()?;
```

## Streaming

All providers return a unified stream of `StreamEvent` values via a tokio `mpsc::Receiver`:

```rust
let mut rx = provider.stream(request).await?;

while let Some(event) = rx.recv().await {
    match event? {
        StreamEvent::MessageStart { id, model, usage } => {
            // Stream started
        }
        StreamEvent::ContentBlockStart { index, content_block } => {
            // New content block (text or tool use)
        }
        StreamEvent::ContentBlockDelta { index, delta } => {
            match delta {
                ContentDelta::TextDelta { text } => {
                    // Incremental text
                }
                ContentDelta::InputJsonDelta { partial_json } => {
                    // Incremental tool call JSON
                }
            }
        }
        StreamEvent::ContentBlockStop { index } => {
            // Content block complete
        }
        StreamEvent::MessageDelta { stop_reason, usage } => {
            // Final metadata (stop reason, token usage)
        }
        StreamEvent::MessageStop => {
            // Stream complete
        }
        StreamEvent::Ping => {
            // Keepalive
        }
        StreamEvent::Error { message } => {
            // Stream error
        }
    }
}
```

Each provider translates its native streaming format (SSE or NDJSON) into the same event sequence. A background tokio task handles the parsing.

## Tool Calling

Define tools using JSON Schema and handle tool use/result cycles:

```rust
use saorsa_ai::{
    CompletionRequest, ContentBlock, Message, StopReason, ToolDefinition,
};

// 1. Define a tool
let tool = ToolDefinition::new(
    "get_weather",
    "Get the current weather for a city",
    serde_json::json!({
        "type": "object",
        "properties": {
            "city": {
                "type": "string",
                "description": "City name"
            }
        },
        "required": ["city"]
    }),
);

// 2. Send request with tools
let request = CompletionRequest::new("claude-sonnet-4", messages, 1024)
    .tools(vec![tool]);

let response = provider.complete(request).await?;

// 3. Handle tool use
if response.stop_reason == Some(StopReason::ToolUse) {
    for block in &response.content {
        if let ContentBlock::ToolUse { id, name, input } = block {
            // Execute the tool (your logic here)
            let result = execute_tool(name, input);

            // 4. Send result back
            messages.push(Message::tool_result(id, result));
        }
    }

    // 5. Continue the conversation with tool results
    let followup = CompletionRequest::new("claude-sonnet-4", messages, 1024);
    let final_response = provider.complete(followup).await?;
}
```

Tool calling works identically across all providers - saorsa-ai handles the format translation between Anthropic's native tool blocks, OpenAI's function calling, Gemini's function declarations, and Ollama's format.

## Model Registry

Look up model metadata at runtime:

```rust
use saorsa_ai::models;

// Exact match
if let Some(info) = models::lookup_model("gpt-4o") {
    println!("Context: {} tokens", info.context_window);
    println!("Tools: {}", info.supports_tools);
    println!("Vision: {}", info.supports_vision);
}

// Prefix match (for versioned model IDs)
let info = models::lookup_model_by_prefix("claude-sonnet-4-5-20250929");
// Matches "claude-sonnet-4"

// Individual queries
let ctx = models::get_context_window("gemini-1.5-pro"); // Some(2_000_000)
let tools = models::supports_tools("llama3");            // Some(false)
let vision = models::supports_vision("gpt-4o");          // Some(true)
```

## Token Counting

Estimate token usage for context window management:

```rust
use saorsa_ai::tokens;

// Estimate tokens in text (~4 chars per token)
let count = tokens::estimate_tokens("Hello, world!");

// Estimate message tokens (includes per-message overhead)
let msg_tokens = tokens::estimate_message_tokens(&message);

// Estimate full conversation
let total = tokens::estimate_conversation_tokens(&messages, Some("system prompt"));

// Check if conversation fits within model's context
let fits = tokens::fits_in_context(
    &messages,
    Some("system prompt"),
    "claude-sonnet-4",
    4096, // max output tokens
);
```

Token counting is heuristic-based (~4 characters per token for English). For precise counts, use provider-specific tokenizers.

## Error Handling

All operations return `Result<T, SaorsaAiError>`:

```rust
pub enum SaorsaAiError {
    /// Provider-specific error
    Provider { provider: String, message: String },
    /// Authentication failure (invalid or missing API key)
    Auth(String),
    /// Network error (connection, DNS, timeout)
    Network(String),
    /// Rate limit exceeded
    RateLimit(String),
    /// Invalid request parameters
    InvalidRequest(String),
    /// Streaming error
    Streaming(String),
    /// Token limit exceeded
    TokenLimit(String),
    /// JSON serialization/deserialization error
    Json(serde_json::Error),
    /// I/O error
    Io(std::io::Error),
    /// Internal error
    Internal(String),
}
```

## Core Types Reference

| Type | Description |
|------|-------------|
| `Provider` | Trait for non-streaming completions |
| `StreamingProvider` | Trait extending `Provider` with streaming |
| `ProviderConfig` | Configuration for creating a provider |
| `ProviderKind` | Enum of provider types (`Anthropic`, `OpenAi`, `Gemini`, `Ollama`, `OpenAiCompatible`) |
| `ProviderRegistry` | Factory for creating providers from config |
| `CompletionRequest` | Builder for completion requests |
| `CompletionResponse` | Parsed completion response |
| `Message` | Conversation message (user, assistant, tool result) |
| `Role` | Message role (`User`, `Assistant`) |
| `ContentBlock` | Message content (`Text`, `ToolUse`, `ToolResult`) |
| `ContentDelta` | Streaming delta (`TextDelta`, `InputJsonDelta`) |
| `StreamEvent` | Streaming event (message start/stop, content deltas, errors) |
| `StopReason` | Why generation stopped (`EndTurn`, `MaxTokens`, `StopSequence`, `ToolUse`) |
| `Usage` | Token usage (`input_tokens`, `output_tokens`) |
| `ToolDefinition` | Tool schema for function calling |
| `ModelInfo` | Model metadata (context window, capabilities) |

## Dependencies

| Crate | Purpose |
|-------|---------|
| `reqwest` | HTTP client (rustls-tls) |
| `reqwest-eventsource` | Server-Sent Events parsing |
| `tokio` | Async runtime |
| `futures` | Async stream utilities |
| `async-trait` | Async trait support |
| `serde` / `serde_json` | JSON serialization |
| `tracing` | Structured logging |
| `thiserror` | Error type derivation |

## Minimum Supported Rust Version

The MSRV is **1.88** (Rust Edition 2024). This is enforced in CI.

## License

Licensed under either of:

- [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0)
- [MIT license](http://opensource.org/licenses/MIT)

at your option.

## Contributing

Part of the [saorsa-tui](https://github.com/saorsa-labs/saorsa-tui) workspace. See the workspace root for contribution guidelines.
