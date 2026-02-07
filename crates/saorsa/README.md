# saorsa

An AI coding agent TUI application built on the saorsa framework.

[![Crates.io](https://img.shields.io/crates/v/saorsa.svg)](https://crates.io/crates/saorsa)
[![Documentation](https://docs.rs/saorsa/badge.svg)](https://docs.rs/saorsa)
[![License](https://img.shields.io/crates/l/saorsa.svg)](LICENSE)
[![MSRV](https://img.shields.io/badge/MSRV-1.88-blue.svg)](#minimum-supported-rust-version)

## Overview

**saorsa** is a full-featured AI coding agent that runs in your terminal. It combines the saorsa TUI framework, multi-provider LLM support, and an agent runtime with tool execution into a single interactive application.

- **Interactive TUI** - Real-time streaming chat with retained-mode rendering
- **Print mode** - Non-interactive mode for scripting and piping
- **7 built-in tools** - bash, read, write, edit, grep, find, ls
- **Session management** - Tree-structured sessions with branching, forking, and resume
- **13 slash commands** - Model switching, session tree, export, settings, and more
- **Model selector** - Fuzzy search with favorites and provider metadata
- **Configurable keybindings** - Customize keyboard shortcuts

## Installation

```bash
cargo install saorsa
```

Or build from the workspace:

```bash
git clone https://github.com/saorsa-labs/saorsa-tui.git
cd saorsa-tui
cargo build --release -p saorsa
```

## Usage

### Interactive Mode (default)

```bash
export ANTHROPIC_API_KEY=sk-ant-...
saorsa
```

### Print Mode (scripting)

```bash
saorsa --print "Explain Rust lifetimes" | tee output.txt
```

### Session Management

```bash
saorsa                    # New session
saorsa -c                 # Continue most recent session
saorsa -r abc123          # Resume session by ID prefix
saorsa --ephemeral        # No session persistence
```

## CLI Arguments

| Argument | Default | Description |
|----------|---------|-------------|
| `--model <NAME>` | `claude-sonnet-4-5-20250929` | LLM model to use |
| `--api-key <KEY>` | `$ANTHROPIC_API_KEY` | API key (or set env var) |
| `--system-prompt <TEXT>` | `"You are a helpful AI coding assistant."` | System prompt |
| `--max-tokens <N>` | `4096` | Max output tokens per response |
| `--max-turns <N>` | `10` | Max agent turns per interaction |
| `-p, --print <PROMPT>` | - | Print mode: single prompt, no TUI |
| `-c, --continue-session` | - | Continue most recent session |
| `-r, --resume <PREFIX>` | - | Resume session by ID prefix |
| `--ephemeral` | - | Disable session persistence |

## UI Layout

```
┌─────────────────────────────────────────────────┐
│ saorsa-tui │ claude-sonnet-4 │ Ready             │  Header
├─────────────────────────────────────────────────┤
│                                                  │
│ > How do I read a file in Rust?                  │
│                                                  │
│ Here's how to read a file in Rust:              │  Messages
│ ```rust                                          │
│ let content = std::fs::read_to_string("f.txt")?; │
│ ```                                              │
│                                                  │
├─────────────────────────────────────────────────┤
│ ╭ Type a message ───────────────────────────────╮│
│ │ █                                             ││  Input
│ ╰───────────────────────────────────────────────╯│
└─────────────────────────────────────────────────┘
```

**Message colors:**
- **User** - Green, bold, `>` prefix
- **Assistant** - Cyan
- **Tool** - Yellow, dim, `[tool_name]` prefix
- **System** - Magenta, italic

## Slash Commands

| Command | Description |
|---------|-------------|
| `/help` | Show available commands |
| `/model <name>` | Switch LLM model |
| `/thinking` | Toggle thinking mode (show model reasoning) |
| `/compact` | Toggle compact UI mode |
| `/tree` | Show session tree hierarchy |
| `/fork [title]` | Fork conversation at current point |
| `/bookmark [name]` | Add/remove/list/jump to bookmarks |
| `/export <path>` | Export conversation to HTML |
| `/share` | Generate shareable link |
| `/login <provider>` | Authenticate with API provider |
| `/logout` | Clear credentials |
| `/settings` | Open settings screen |
| `/hotkeys` | Show keybinding reference |
| `/clear` | Clear conversation |

## Keybindings

### General

| Key | Action |
|-----|--------|
| `Enter` | Submit message |
| `Ctrl+C` | Quit |
| `Ctrl+D` | Quit (on empty input) |
| `Escape` | Clear input |
| `Left` / `Right` | Move cursor |
| `Home` / `End` | Jump to start/end |
| `Backspace` | Delete character |

### Configurable

| Action | Default | Description |
|--------|---------|-------------|
| Send | `Ctrl+Enter` | Submit message |
| Cancel | `Escape` | Cancel current action |
| New chat | `Ctrl+N` | Start new conversation |
| Model selector | `Ctrl+L` | Open model picker / cycle favorites |
| Settings | `Ctrl+,` | Open settings |
| Queue | `Ctrl+Q` | Open message queue |
| Save | `Ctrl+S` | Save current session |

### Model Selector

| Key | Action |
|-----|--------|
| `Up` / `Down` | Navigate models |
| `Enter` | Select model |
| `f` | Toggle favorite |
| `Ctrl+L` | Cycle through favorites |
| `Escape` | Close selector |

### Settings Screen

| Key | Action |
|-----|--------|
| `Tab` / `Shift+Tab` | Next/previous tab |
| `S` | Save settings |
| `C` / `Escape` | Cancel and close |

## Operating Modes

| Mode | Activation | Description |
|------|------------|-------------|
| **Interactive** | Default | Full TUI with event loop and session persistence |
| **Print** | `--print <prompt>` | Single prompt, streams response to stdout, exits |
| **JSON** | (planned) | JSON Lines structured event output |
| **RPC** | (planned) | JSON-RPC 2.0 over stdio for editor integration |

## Architecture

```
CLI Parsing → Mode Selection
                  │
          ┌───────┴───────┐
          │               │
     Interactive       Print Mode
          │               │
    ┌─────┴─────┐    Stream to stdout
    │           │
  Event Loop  AgentLoop
    │           │
  tokio::select!
    │           │
  Terminal    Agent Events
  Events      (text, tools)
    │           │
    └─────┬─────┘
          │
     UI Render
          │
     ScreenBuffer → Terminal
```

**Crate dependencies:**
- `saorsa-core` - TUI framework (widgets, layout, rendering)
- `saorsa-ai` - LLM provider abstraction
- `saorsa-agent` - Agent loop, tools, sessions, context

## Development

```bash
# Run tests
cargo test -p saorsa

# Run the application
cargo run -p saorsa

# Run in print mode
cargo run -p saorsa -- --print "Hello"
```

## Minimum Supported Rust Version

The MSRV is **1.88** (Rust Edition 2024). This is enforced in CI.

## License

Licensed under either of:

- [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0)
- [MIT license](http://opensource.org/licenses/MIT)

at your option.

## Contributing

Part of the [saorsa-tui](https://github.com/saorsa-labs/saorsa-tui) workspace. See the workspace root for contribution guidelines.
