# saorsa-tui

A retained-mode, CSS-styled terminal UI framework in Rust, with a full AI coding agent as the showcase application.

[![CI](https://github.com/saorsa-labs/saorsa-tui/actions/workflows/ci.yml/badge.svg)](https://github.com/saorsa-labs/saorsa-tui/actions/workflows/ci.yml)
[![License](https://img.shields.io/crates/l/saorsa-core.svg)](LICENSE)
[![MSRV](https://img.shields.io/badge/MSRV-1.88-blue.svg)](#minimum-supported-rust-version)

## Overview

**saorsa-tui** is a Rust workspace containing a full-featured TUI framework and an AI coding agent built on top of it.

The framework brings web-like development patterns to the terminal: retained-mode widgets, CSS styling with themes and hot-reload, reactive state management, and a compositor with z-ordering and overlays. The showcase application is a streaming AI coding agent with tool execution, session management, and multi-provider LLM support.

## Workspace Crates

```
saorsa-cli ──→ saorsa ──→ saorsa-core
                        ├→ saorsa-ai
                        └→ saorsa-agent ──→ saorsa-ai
```

| Crate | Type | Description |
|-------|------|-------------|
| [**saorsa-core**](crates/saorsa-core/) | lib | TUI framework: 24+ widgets, TCSS styling, Taffy layout, reactive signals, compositor |
| [**saorsa-ai**](crates/saorsa-ai/) | lib | Unified multi-provider LLM API (Anthropic, OpenAI, Gemini, Ollama, + OpenAI-compatible) |
| [**saorsa-agent**](crates/saorsa-agent/) | lib | Agent runtime: tool execution, sessions, context engineering, extensions |
| [**saorsa**](crates/saorsa/) | bin+lib | AI coding agent application with TUI and print modes |
| [**saorsa-cli**](crates/saorsa-cli/) | bin | Thin CLI entry point |

## Quick Start

### Build from source

```bash
git clone https://github.com/saorsa-labs/saorsa-tui.git
cd saorsa-tui
cargo build --release
```

### Run the AI coding agent

```bash
export ANTHROPIC_API_KEY=sk-ant-...
cargo run -p saorsa
```

### Use as a library

Add individual crates to your `Cargo.toml`:

```toml
[dependencies]
saorsa-core = "0.1"   # TUI framework
saorsa-ai = "0.1"     # LLM providers
saorsa-agent = "0.1"  # Agent runtime
```

## Framework Highlights

### saorsa-core - TUI Framework

- **Retained-mode widgets** - 24+ widgets including DataTable, Tree, TextArea, MarkdownRenderer, DiffView, Modal, Tabs, Sparkline
- **TCSS (Terminal CSS)** - CSS-like styling with selectors, variables, themes (Catppuccin, Dracula, Nord, Solarized), and live hot-reload
- **Layout engine** - Manual split/dock layout plus Taffy-powered CSS Flexbox and Grid
- **Reactive system** - Signal/Computed/Effect primitives with automatic dependency tracking and batch updates
- **Compositor** - Layer-based rendering with z-ordering, clipping, and overlay support
- **Differential rendering** - Double-buffered with SGR-optimized escape sequences
- **Full Unicode** - Grapheme clusters, CJK wide characters, emoji sequences

### saorsa-ai - LLM Providers

- **5 provider families** - Anthropic, OpenAI, Gemini, Ollama, OpenAI-compatible (Azure, Groq, Mistral, OpenRouter, xAI, Cerebras)
- **Unified streaming** - Same `StreamEvent` types across all providers
- **Tool calling** - JSON Schema-based tool definitions with automatic format translation
- **Model registry** - Context windows, capability metadata, prefix matching

### saorsa-agent - Agent Runtime

- **Agent loop** - Turn-based conversation with streaming and automatic tool continuation
- **7 built-in tools** - bash, read, write, edit, grep, find, ls
- **Session management** - Tree-structured sessions with branching, forking, auto-save, resume
- **Context engineering** - AGENTS.md/SYSTEM.md discovery, compaction, merge strategies
- **Extension system** - Lifecycle hooks, custom tools, commands, keybindings, widgets

## Architecture

```text
┌─────────────────────────────────────────────────────────────┐
│                     saorsa (Application)                      │
│  CLI parsing, TUI event loop, slash commands, settings       │
└──────────┬──────────────┬──────────────┬────────────────────┘
           │              │              │
           ▼              ▼              ▼
┌──────────────┐ ┌──────────────┐ ┌──────────────────────────┐
│  saorsa-core │ │  saorsa-ai   │ │     saorsa-agent         │
│              │ │              │ │                            │
│  Widgets     │ │  Providers   │ │  Agent loop               │
│  TCSS Engine │ │  Streaming   │ │  Tools (bash/read/write/  │
│  Layout      │ │  Tool schema │ │    edit/grep/find/ls)     │
│  Compositor  │ │  Model meta  │ │  Sessions & branching     │
│  Renderer    │ │  Token count │ │  Context engineering      │
│  Reactive    │ │              │ │  Skills & templates       │
│  Terminal    │ │              │ │  Extension system         │
└──────────────┘ └──────────────┘ └──────────────────────────┘
```

## Development

### Prerequisites

- Rust 1.88+ (Edition 2024)
- An LLM API key (Anthropic, OpenAI, etc.) for running the application

### Build & Test

```bash
# Check compilation
cargo check --workspace

# Run all tests
cargo test --workspace

# Lint (zero warnings enforced)
cargo clippy --workspace --all-targets -- -D warnings

# Format check
cargo fmt --all -- --check

# Build docs
cargo doc --workspace --no-deps
```

### Benchmarks

```bash
cargo bench -p saorsa-core
```

Benchmarks cover rendering performance, layout computation, and CSS parsing.

### Code Quality Standards

- Zero warnings from rustc and clippy
- Zero test failures
- No `.unwrap()` or `.expect()` in production code (OK in tests)
- No `panic!()`, `todo!()`, `unimplemented!()`
- Doc comments on all public items
- `thiserror` for library error types, `anyhow` for application binaries

## Minimum Supported Rust Version

The MSRV is **1.88** (Rust Edition 2024). This is tested in CI and enforced via `rust-version` in `Cargo.toml`.

## License

Licensed under either of:

- [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0)
- [MIT license](http://opensource.org/licenses/MIT)

at your option.

## Author

David Irvine / [Saorsa Labs](https://github.com/saorsa-labs)
