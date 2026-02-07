# Phase 1.4: Anthropic Provider (fae-ai)

## Overview
Implement the Anthropic Messages API client in fae-ai with streaming
response handling, tool calling abstraction, and context/message types.

## Tasks

### Task 1: Message & Content Types
- `crates/fae-ai/src/message.rs`
- `Role` enum: User, Assistant, System
- `Message` struct: role, content (Vec<ContentBlock>)
- `ContentBlock` enum: Text(String), ToolUse { id, name, input }, ToolResult { tool_use_id, content }
- `ToolDefinition` struct: name, description, input_schema (serde_json::Value)
- Serialize/Deserialize with serde
- Tests: message construction, serialization roundtrip

### Task 2: Request & Response Types
- `crates/fae-ai/src/types.rs`
- `CompletionRequest`: model, messages, max_tokens, system, tools, stream, temperature, stop_sequences
- `CompletionResponse`: id, content, model, stop_reason, usage
- `StreamEvent` enum: MessageStart, ContentBlockStart, ContentBlockDelta, ContentBlockStop, MessageDelta, MessageStop, Ping, Error
- `Usage` struct: input_tokens, output_tokens
- `StopReason` enum: EndTurn, MaxTokens, StopSequence, ToolUse
- Tests: request building, response parsing

### Task 3: Provider Trait
- `crates/fae-ai/src/provider.rs`
- `Provider` trait (async): complete(&self, request: CompletionRequest) -> Result<CompletionResponse>
- `StreamingProvider` trait: stream(&self, request: CompletionRequest) -> Result<impl Stream<Item=StreamEvent>>
- `ProviderConfig` struct: api_key, base_url, model, max_tokens
- Tests: mock provider impl

### Task 4: Anthropic Provider Implementation
- `crates/fae-ai/src/anthropic.rs`
- `AnthropicProvider` implementing Provider + StreamingProvider
- HTTP client using reqwest
- Request serialization to Anthropic Messages API format
- Response deserialization
- SSE stream parsing for streaming responses
- Error handling: rate limits, auth errors, API errors
- Tests: request serialization, response parsing (with fixture data)

### Task 5: Token Counting
- `crates/fae-ai/src/tokens.rs`
- Basic token estimation (chars / 4 as rough approximation)
- Track usage from responses
- Context window management helpers
- Tests: estimation sanity checks

### Task 6: Wire Up
- Add all modules to fae-ai lib.rs
- Re-export key types
- Add reqwest, tokio, futures dependencies to fae-ai Cargo.toml
- Add reqwest-eventsource for SSE
- Ensure zero warnings

## Dependencies
- Phase 1.3 complete ✅
- reqwest (HTTP client)
- tokio (async runtime)
- futures (Stream trait)
- reqwest-eventsource (SSE parsing)

## Acceptance Criteria
- Message/Content types serialize to Anthropic API format
- Provider trait defines async completion and streaming
- AnthropicProvider can construct valid API requests
- SSE stream events parse correctly
- Token estimation provides rough counts
- All tests pass, zero clippy warnings
