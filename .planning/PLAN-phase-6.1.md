# Phase 6.1: Additional Providers

**Milestone**: 6 — Full Agent Features
**Phase**: 6.1 — Additional Providers
**Goal**: Multi-provider LLM support — OpenAI, Gemini, Ollama, and OpenAI-compatible providers.

## Architecture

Current state: `AnthropicProvider` implements `Provider` + `StreamingProvider` traits.
Strategy: Add parallel provider modules, each implementing the same traits.
Provider-specific request/response conversion happens inside each implementation.

Key design decisions:
- Each provider gets its own module file (e.g., `openai.rs`, `gemini.rs`, `ollama.rs`)
- Shared types (`Message`, `ContentBlock`, `CompletionRequest/Response`, `StreamEvent`) remain unchanged
- Provider factory function for easy construction from config
- `ProviderConfig` enhanced with provider-type awareness
- Token estimation updated for all provider model families

## Tasks

### Task 1: Provider Registry & Factory
**Files**: `crates/fae-ai/src/provider.rs`, `crates/fae-ai/src/lib.rs`

Enhance the provider system with:
- `ProviderKind` enum: `Anthropic`, `OpenAi`, `Gemini`, `Ollama`, `OpenAiCompatible`
- `ProviderRegistry` struct that maps provider kinds to factory functions
- `create_provider(kind, config) -> Box<dyn StreamingProvider>` factory
- Update `ProviderConfig` with optional `provider_kind` field
- Provider-specific default base URLs
- Tests for registry creation, factory dispatch, config defaults

### Task 2: OpenAI Provider — Non-Streaming
**Files**: `crates/fae-ai/src/openai.rs`, `crates/fae-ai/src/lib.rs`

Implement `OpenAiProvider` with `Provider` trait (non-streaming first):
- `OpenAiProvider` struct with `ProviderConfig` + `reqwest::Client`
- Auth via `Authorization: Bearer {api_key}` header
- Map `CompletionRequest` → OpenAI Chat Completions API JSON format
- Map OpenAI response → `CompletionResponse`
- Handle differences: `role` mapping, `tool_calls` format, `finish_reason` → `StopReason`
- Error mapping: 401→Auth, 429→RateLimit, etc.
- Base URL default: `https://api.openai.com`
- Tests for request serialization, response deserialization, error mapping

### Task 3: OpenAI Provider — Streaming
**Files**: `crates/fae-ai/src/openai.rs`

Add `StreamingProvider` impl for `OpenAiProvider`:
- SSE parsing for OpenAI's `data: [DONE]` format
- Map OpenAI streaming chunks → `StreamEvent` variants
- Handle `delta.content`, `delta.tool_calls` incremental updates
- Handle `[DONE]` terminal event
- Channel-based output matching Anthropic pattern
- Tests for SSE parsing, stream event mapping, error handling during stream

### Task 4: Gemini Provider
**Files**: `crates/fae-ai/src/gemini.rs`, `crates/fae-ai/src/lib.rs`

Implement `GeminiProvider` with both traits:
- Auth via `?key={api_key}` query parameter (Google Gemini API style)
- Map `CompletionRequest` → Gemini `generateContent` format
  - `contents` array with `parts` (text, functionCall, functionResponse)
  - `generationConfig` for temperature, maxOutputTokens
  - `tools` → `functionDeclarations`
- Map Gemini response → `CompletionResponse`
- Streaming via SSE on `streamGenerateContent` endpoint
- Map streaming chunks → `StreamEvent`
- Base URL default: `https://generativelanguage.googleapis.com/v1beta`
- Tests for request/response mapping, streaming, error handling

### Task 5: Ollama Provider
**Files**: `crates/fae-ai/src/ollama.rs`, `crates/fae-ai/src/lib.rs`

Implement `OllamaProvider` for local inference:
- No auth required (local server)
- Map `CompletionRequest` → Ollama Chat API (`/api/chat`) format
- Streaming via NDJSON (newline-delimited JSON), not SSE
- Map Ollama response fields → `CompletionResponse`
- Map streaming chunks → `StreamEvent`
- Handle Ollama-specific fields: `done`, `done_reason`
- Base URL default: `http://localhost:11434`
- Tests for NDJSON parsing, request/response mapping, stream events

### Task 6: OpenAI-Compatible Provider
**Files**: `crates/fae-ai/src/openai_compat.rs`, `crates/fae-ai/src/lib.rs`

Generic provider for OpenAI-compatible APIs (Azure, Groq, Cerebras, xAI, OpenRouter, Mistral, etc.):
- Reuse OpenAI request/response mapping via shared helper module
- `OpenAiCompatProvider` wraps core OpenAI logic with configurable:
  - Custom base URL (required)
  - Custom auth header name/format (optional, defaults to Bearer)
  - Custom model name mapping (optional)
  - Extra headers (e.g., Azure `api-version`, OpenRouter `HTTP-Referer`)
- Pre-configured factories: `azure_openai()`, `groq()`, `openrouter()`, `mistral()`, etc.
- Tests for custom URL, custom auth, extra headers, factory functions

### Task 7: Token Estimation & Model Registry
**Files**: `crates/fae-ai/src/tokens.rs`, `crates/fae-ai/src/models.rs` (new)

Extend token estimation and model awareness across all providers:
- New `models.rs` module with `ModelInfo` struct: name, provider, context_window, supports_tools, supports_vision
- Model registry with known models for each provider
  - OpenAI: gpt-4o, gpt-4o-mini, gpt-4-turbo, o1, o3-mini
  - Gemini: gemini-2.0-flash, gemini-1.5-pro, gemini-1.5-flash
  - Anthropic: claude-4-opus, claude-4-sonnet, claude-3.5-haiku (existing)
  - Ollama: llama3, codellama, mistral, etc. (default context windows)
- `lookup_model(name) -> Option<ModelInfo>` function
- Update `context_window()` and `estimate_tokens()` to use registry
- Tests for model lookup, context windows, provider detection

### Task 8: Integration Tests & Module Wiring
**Files**: `crates/fae-ai/src/lib.rs`, integration tests

Wire everything together:
- Register all providers in lib.rs exports
- Integration test: create each provider type via factory
- Integration test: serialize/deserialize round-trip for each provider's request format
- Integration test: parse sample streaming responses from each provider
- Integration test: provider registry with all providers registered
- Verify zero warnings, all tests pass
- Update doc comments on all public items

## File Summary

| File | Action | Task |
|------|--------|------|
| `crates/fae-ai/src/provider.rs` | Modify | T1 |
| `crates/fae-ai/src/openai.rs` | Create | T2, T3 |
| `crates/fae-ai/src/gemini.rs` | Create | T4 |
| `crates/fae-ai/src/ollama.rs` | Create | T5 |
| `crates/fae-ai/src/openai_compat.rs` | Create | T6 |
| `crates/fae-ai/src/models.rs` | Create | T7 |
| `crates/fae-ai/src/tokens.rs` | Modify | T7 |
| `crates/fae-ai/src/lib.rs` | Modify | T1-T8 |

## Quality Requirements

- Zero clippy warnings
- Zero compilation warnings
- No `.unwrap()` or `.expect()` in production code
- Doc comments on all public items
- Tests for each provider's request/response mapping
- Tests for each provider's SSE/streaming parsing
- All existing 32 fae-ai tests continue to pass
