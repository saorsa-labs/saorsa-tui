# saorsa-agent

Agent runtime with tool execution, session management, context engineering, and extension system for building AI coding agents.

[![Crates.io](https://img.shields.io/crates/v/saorsa-agent.svg)](https://crates.io/crates/saorsa-agent)
[![Documentation](https://docs.rs/saorsa-agent/badge.svg)](https://docs.rs/saorsa-agent)
[![License](https://img.shields.io/crates/l/saorsa-agent.svg)](LICENSE)
[![MSRV](https://img.shields.io/badge/MSRV-1.88-blue.svg)](#minimum-supported-rust-version)

## Overview

**saorsa-agent** provides the runtime for AI agents that can execute tools, manage sessions, and integrate with terminal UIs. It builds on `saorsa-ai` for LLM communication and adds:

- **Agent loop** - Turn-based conversation with streaming, tool execution, and automatic continuation
- **7 built-in tools** - bash, read, write, edit, grep, find, ls
- **Session management** - Tree-structured sessions with branching, forking, auto-save, and resume
- **Context engineering** - AGENTS.md/SYSTEM.md discovery, context compaction, merge strategies
- **Skills system** - On-demand capability injection from markdown files
- **Templates** - Prompt templates with variable substitution and conditionals
- **Extension system** - Lifecycle hooks, custom tools, commands, keybindings, and widgets
- **Event system** - Typed events for UI integration (text deltas, tool calls, turn lifecycle)

## Quick Start

```toml
[dependencies]
saorsa-agent = "0.1"
saorsa-ai = "0.1"
tokio = { version = "1", features = ["full"] }
```

Note: `saorsa-agent` is provider-agnostic. Any `Box<dyn saorsa_ai::StreamingProvider>` works,
including in-process providers like `saorsa_ai::MistralrsProvider` (feature-gated behind
`saorsa-ai`'s `mistralrs` feature).

### Running the Agent Loop

```rust
use saorsa_agent::{AgentConfig, AgentLoop, default_tools, event_channel};
use saorsa_ai::{ProviderConfig, ProviderKind, ProviderRegistry};

#[tokio::main]
async fn main() -> saorsa_agent::Result<()> {
    // Create the LLM provider
    let config = ProviderConfig::new(
        ProviderKind::Anthropic,
        std::env::var("ANTHROPIC_API_KEY").expect("set ANTHROPIC_API_KEY"),
        "claude-sonnet-4",
    );
    let registry = ProviderRegistry::default();
    let provider = registry.create(config)?;

    // Set up agent
    let agent_config = AgentConfig::default();
    let tools = default_tools(std::env::current_dir()?);
    let (tx, mut rx) = event_channel(64);

    let mut agent = AgentLoop::new(provider, agent_config, tools, tx);

    // Consume events in a background task
    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            match event {
                saorsa_agent::AgentEvent::TextDelta { text } => {
                    print!("{text}");
                }
                saorsa_agent::AgentEvent::ToolCall { name, .. } => {
                    eprintln!("[calling {name}...]");
                }
                _ => {}
            }
        }
    });

    // Run the agent
    let response = agent.run("List the files in the current directory").await?;
    println!("\nFinal: {response}");

    Ok(())
}
```

## Agent Loop

The `AgentLoop` is the core runtime. It sends messages to an LLM, streams responses, executes tool calls, and loops until the model stops or the turn limit is reached.

### Turn Lifecycle

1. **TurnStart** - Begin a new turn
2. **Stream response** - Receive text deltas and tool call fragments
3. **TextComplete** - Full text assembled
4. **Tool execution** - If `StopReason::ToolUse`, execute tools and add results to history
5. **TurnEnd** - Turn complete, loop if more tools needed

### Configuration

```rust
use saorsa_agent::AgentConfig;

let config = AgentConfig::new("claude-sonnet-4")
    .system_prompt("You are a helpful coding assistant.")
    .max_turns(10)    // Maximum tool-use turns per run()
    .max_tokens(4096); // Max output tokens per completion
```

**Defaults:**
| Setting | Default |
|---------|---------|
| `model` | `claude-sonnet-4-5-20250929` |
| `system_prompt` | `"You are a helpful assistant."` |
| `max_turns` | `10` |
| `max_tokens` | `4096` |

## Built-in Tools

### Tool Trait

All tools implement the async `Tool` trait:

```rust
#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn input_schema(&self) -> serde_json::Value;
    async fn execute(&self, input: serde_json::Value) -> Result<String>;
}
```

### Tool Registry

```rust
use saorsa_agent::{ToolRegistry, default_tools};

// Create registry with all 7 built-in tools
let tools = default_tools("/path/to/working/dir");
assert_eq!(tools.len(), 7);

// Or build a custom registry
let mut registry = ToolRegistry::new();
registry.register(Box::new(my_custom_tool));
```

### Bash Tool

Execute shell commands with timeout and output limits.

```json
{ "command": "cargo test", "working_directory": "/project", "timeout_ms": 60000 }
```

| Limit | Value |
|-------|-------|
| Default timeout | 120 seconds |
| Max output | 100 KB |
| Shell | `/bin/bash -c` |

Captures both stdout and stderr. Output is truncated at safe UTF-8 boundaries if it exceeds the limit.

### Read Tool

Read file contents with optional line ranges.

```json
{ "file_path": "src/main.rs", "line_range": "10-20" }
```

| Feature | Detail |
|---------|--------|
| Line ranges | `10-20`, `5-` (from line 5), `-10` (first 10 lines) |
| Max file size | 10 MB |
| Line numbers | Output includes `N: content` format |

### Write Tool

Write content to files with automatic directory creation and diff display.

```json
{ "file_path": "src/new_file.rs", "content": "fn main() {}" }
```

Creates parent directories automatically. Shows a unified diff when updating existing files. Reports "No changes" if content is identical.

### Edit Tool

Surgical text replacement with ambiguity detection.

```json
{ "file_path": "src/lib.rs", "old_text": "fn old_name()", "new_text": "fn new_name()", "replace_all": false }
```

| Behavior | Detail |
|----------|--------|
| Single match | Replaces the one occurrence |
| Multiple matches | Returns error with match count unless `replace_all: true` |
| No match | Returns error with the search text |

### Grep Tool

Search file contents with regex patterns.

```json
{ "pattern": "fn\\s+\\w+", "path": "src/", "case_insensitive": false }
```

| Feature | Detail |
|---------|--------|
| Pattern | Rust `regex` syntax |
| Scope | Recursive directory search |
| Output | `file:line: content` format |
| Limit | 100 matches max |

### Find Tool

Find files by glob pattern.

```json
{ "pattern": "*.rs", "path": "src/" }
```

| Feature | Detail |
|---------|--------|
| Pattern | Glob syntax (`*.rs`, `test_?.log`, `**/*.toml`) |
| Limit | 100 files max |

### Ls Tool

List directory contents with metadata.

```json
{ "path": "src/", "recursive": true }
```

| Feature | Detail |
|---------|--------|
| Output format | `TYPE SIZE NAME` per entry |
| Types | `FILE`, `DIR`, `LNK` |
| Size format | Human-readable (B, KB, MB, GB) |

## Event System

The agent emits typed events for UI integration:

```rust
pub enum AgentEvent {
    TurnStart { turn: u32 },
    TextDelta { text: String },
    TextComplete { text: String },
    ToolCall { id: String, name: String, input: serde_json::Value },
    ToolResult { id: String, name: String, output: String, success: bool },
    TurnEnd { turn: u32, reason: TurnEndReason },
    Error { message: String },
}

pub enum TurnEndReason {
    EndTurn,    // Model finished naturally
    ToolUse,    // Tools executed, continuing
    MaxTurns,   // Turn limit reached
    MaxTokens,  // Token limit reached
    Error,      // Error occurred
}
```

Events are delivered via a tokio `mpsc` channel:

```rust
let (tx, mut rx) = event_channel(64);
let mut agent = AgentLoop::new(provider, config, tools, tx);

// UI task reads events
while let Some(event) = rx.recv().await {
    match event {
        AgentEvent::TextDelta { text } => { /* stream to display */ }
        AgentEvent::ToolCall { name, input, .. } => { /* show tool activity */ }
        AgentEvent::ToolResult { success, .. } => { /* show result status */ }
        AgentEvent::TurnEnd { reason, .. } => { /* update UI state */ }
        _ => {}
    }
}
```

## Session Management

### Session Storage

Sessions are persisted to disk in a structured format:

```
~/.saorsa/sessions/
  <session-uuid>/
    manifest.json       # SessionMetadata (title, tags, timestamps)
    tree.json           # SessionNode (parent/child relationships)
    messages/
      0-user.json       # Chronological message files
      1-assistant.json
      2-tool_call.json
      3-tool_result.json
```

```rust
use saorsa_agent::{SessionId, SessionMetadata, SessionStorage};

let storage = SessionStorage::new()?;
let id = SessionId::new();

// Save/load metadata
storage.save_manifest(&id, &metadata)?;
let metadata = storage.load_manifest(&id)?;

// Save/load messages
storage.save_message(&id, 0, &message)?;
let messages = storage.load_messages(&id)?;
```

### Tree-Structured Sessions

Sessions form a tree: forking creates a child session that shares history up to the fork point.

```rust
use saorsa_agent::{fork_session, build_session_tree, render_tree, TreeRenderOptions};

// Fork from an existing session
let child_id = fork_session(&storage, &parent_id)?;

// Build and render the session tree
let tree = build_session_tree(&storage)?;
let output = render_tree(&tree, &TreeRenderOptions::default());
println!("{output}");
```

### Resume & Find

```rust
use saorsa_agent::{find_last_active_session, find_session_by_prefix, restore_session};

// Resume the most recent session
let id = find_last_active_session(&storage)?;
let messages = restore_session(&storage, &id)?;

// Find by 8-character prefix
let id = find_session_by_prefix(&storage, "a1b2c3d4")?;
```

### Auto-Save

Sessions auto-save with debouncing and atomic writes (temp file + rename):

```rust
// Auto-fork when editing a message mid-conversation
let forked = auto_fork_on_edit(&storage, &session_id, edit_index)?;
```

### Bookmarks

```rust
use saorsa_agent::{Bookmark, BookmarkManager};

let mut bookmarks = BookmarkManager::new(&storage);
bookmarks.add(Bookmark::new(session_id, "Important conversation"))?;
```

### Export

```rust
use saorsa_agent::export_to_html;

let html = export_to_html(&storage, &session_id)?;
std::fs::write("session.html", html)?;
```

## Context Engineering

### AGENTS.md / SYSTEM.md Discovery

The agent searches for context files in precedence order:

1. Current working directory (highest precedence)
2. Parent directories (walking up to root/home)
3. `~/.saorsa/` (global, lowest precedence)

```rust
use saorsa_agent::ContextDiscovery;

let discovery = ContextDiscovery::new()?;

// Find all AGENTS.md files (highest precedence first)
let agents_files = discovery.discover_agents_md();

// Find all SYSTEM.md files
let system_files = discovery.discover_system_md();
```

### Context Bundle

Combine discovered context into a single bundle:

```rust
use saorsa_agent::ContextBundle;

let context = ContextBundle::builder()
    .agents(agents_context)   // From AGENTS.md
    .system(system_context)   // From SYSTEM.md
    .user("Additional context") // Ad-hoc context
    .build();
```

### SYSTEM.md Modes

| Mode | Behavior |
|------|----------|
| `SystemMode::Replace` | Replace the default system prompt entirely |
| `SystemMode::Append` | Append after the default system prompt (default) |

### Context Compaction

When conversations approach the context window limit:

```rust
use saorsa_agent::{CompactionConfig, CompactionStrategy, compact};

let config = CompactionConfig::default();
let compacted = compact(&messages, &config)?;
```

## Skills System

Skills inject specialized knowledge on demand from markdown files:

```rust
use saorsa_agent::SkillRegistry;

// Discover skills from ~/.saorsa/skills/
let skills = SkillRegistry::discover_skills();

for skill in &skills {
    println!("{}: {}", skill.name, skill.description);
}
```

Skill files are markdown with front matter for metadata (name, description, trigger keywords).

## Templates

Prompt templates with variable substitution:

```rust
use saorsa_agent::{TemplateEngine, render_simple};
use std::collections::HashMap;

// Simple variable substitution
let mut ctx = HashMap::new();
ctx.insert("name".to_string(), "Alice".to_string());
ctx.insert("model".to_string(), "claude-sonnet-4".to_string());

let result = render_simple("Hello {{name}}, using {{model}}!", &ctx)?;
// "Hello Alice, using claude-sonnet-4!"
```

**Template syntax:**
- Variables: `{{name}}`
- Conditionals: `{{#if var}}...{{/if}}`
- Negated: `{{#unless var}}...{{/unless}}`

Built-in templates are available via `get_builtin()` and `list_builtins()`. User templates are loaded from `~/.saorsa/templates/*.md`.

## Extension System

Extensions add custom functionality via lifecycle hooks:

```rust
use saorsa_agent::Extension;

pub trait Extension: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn on_load(&mut self) -> Result<()>;
    fn on_unload(&mut self) -> Result<()>;
    fn on_tool_call(&mut self, tool: &str, args: &str) -> Result<Option<String>>;
    fn on_message(&mut self, message: &str) -> Result<Option<String>>;
    fn on_turn_start(&mut self) -> Result<()>;
    fn on_turn_end(&mut self) -> Result<()>;
}
```

### Extension Registry

```rust
use saorsa_agent::{ExtensionRegistry, shared_registry};

// Thread-safe shared registry
let registry = shared_registry();

// Register an extension
{
    let mut reg = registry.write().unwrap();
    reg.register(Box::new(my_extension))?;
}

// Notify all extensions of events
{
    let mut reg = registry.write().unwrap();
    reg.notify_turn_start()?;
    let responses = reg.notify_tool_call("bash", "{\"command\": \"ls\"}")?;
    reg.notify_turn_end()?;
}
```

### Specialized Registries

| Registry | Purpose |
|----------|---------|
| `CommandRegistry` | Custom slash commands |
| `KeybindingRegistry` | Custom keyboard shortcuts |
| `ExtensionToolRegistry` | Custom agent tools |
| `WidgetRegistry` | Custom UI widgets |

Extensions are loaded from `~/.saorsa/extensions/`.

## Custom Tools

Implement the `Tool` trait to add your own tools:

```rust
use saorsa_agent::Tool;
use async_trait::async_trait;

struct MyTool;

#[async_trait]
impl Tool for MyTool {
    fn name(&self) -> &str { "my_tool" }

    fn description(&self) -> &str {
        "Does something useful"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "query": { "type": "string", "description": "The query" }
            },
            "required": ["query"]
        })
    }

    async fn execute(&self, input: serde_json::Value) -> saorsa_agent::Result<String> {
        let query = input["query"].as_str().unwrap_or("");
        Ok(format!("Result for: {query}"))
    }
}

// Register it
let mut registry = saorsa_agent::ToolRegistry::new();
registry.register(Box::new(MyTool));
```

## Error Handling

```rust
pub enum SaorsaAgentError {
    Tool(String),           // Tool execution error
    Session(String),        // Session storage error
    Context(String),        // Context engineering error
    Provider(SaorsaAiError), // LLM provider error (from saorsa-ai)
    Cancelled(String),      // Operation cancelled
    Io(std::io::Error),     // File I/O error
    Json(serde_json::Error), // Serialization error
    Internal(String),       // Internal error
    Extension(String),      // Extension error
}
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| `saorsa-ai` | LLM provider abstraction |
| `tokio` | Async runtime |
| `async-trait` | Async trait support |
| `serde` / `serde_json` | Serialization |
| `uuid` | Session IDs |
| `chrono` | Timestamps |
| `similar` | Unified diffs (edit/write tools) |
| `regex` | Grep tool patterns |
| `walkdir` | Recursive directory traversal |
| `globset` | Glob pattern matching (find tool) |
| `dirs` | User directory paths |
| `tracing` | Structured logging |
| `thiserror` | Error type derivation |

## Development

```bash
# Run all tests
cargo test -p saorsa-agent

# Run integration tests
cargo test -p saorsa-agent --test tool_integration
cargo test -p saorsa-agent --test integration_tools
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
