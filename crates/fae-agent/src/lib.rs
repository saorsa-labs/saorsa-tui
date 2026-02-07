//! fae-agent: AI coding agent runtime.
//!
//! Provides the agent loop, built-in tools (bash, read, write, edit, grep, find, ls),
//! event system for UI integration, and tool registry.
//!
//! # Architecture Overview
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                     UI Layer (fae-app)                      │
//! │  Sends user input, receives AgentEvent stream               │
//! └─────────────────────────────────────────────────────────────┘
//!                              │
//!                              ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │                  AgentLoop (async runtime)                  │
//! │  User message → Context → LLM request → Tool execution      │
//! │  → Follow-up → ... → Final response → TurnEnd               │
//! └─────────────────────────────────────────────────────────────┘
//!          │                   │                   │
//!          ▼                   ▼                   ▼
//!    ┌──────────┐      ┌──────────────┐    ┌──────────────┐
//!    │ Context  │      │  Provider    │    │ ToolRegistry │
//!    │ Builder  │      │  (fae-ai)    │    │              │
//!    └──────────┘      └──────────────┘    └──────────────┘
//!         │                   │                   │
//!         ▼                   ▼                   ▼
//! ┌────────────────┐  ┌────────────────┐  ┌────────────────┐
//! │ AGENTS.md      │  │ Stream events  │  │ Bash, Read,    │
//! │ SYSTEM.md      │  │ Tool calls     │  │ Write, Edit,   │
//! │ Project files  │  │ Content delta  │  │ Grep, Find, Ls │
//! └────────────────┘  └────────────────┘  └────────────────┘
//!                              │
//!                              ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │              SessionStorage (persistence)                   │
//! │  Messages, tool results, tree structure, bookmarks          │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Core Subsystems
//!
//! - **AgentLoop**: Main async runtime coordinating LLM interaction and tool execution
//! - **Context Engineering**: Loads AGENTS.md, SYSTEM.md, project files into LLM context
//! - **Tool Registry**: Built-in tools (bash, file ops, search) + extension tools
//! - **Session Management**: Conversation history with tree-based branching and bookmarks
//! - **Event System**: Async channel for UI updates (thinking, tool execution, streaming)
//! - **Skills System**: On-demand capabilities loaded from `~/.claude/skills/`
//! - **Extension System**: Plugins for commands, keybindings, tools, and widgets
//!
//! ## Agent Execution Flow
//!
//! 1. **User Input** → ContextBuilder assembles prompt with context files
//! 2. **LLM Request** → Provider streams back content and tool calls
//! 3. **Tool Execution** → Tools run in parallel, results added to conversation
//! 4. **Follow-up** → If tools were called, LLM gets results and continues
//! 5. **Turn End** → Final response emitted, session persisted
//!
//! ## Key Types
//!
//! - `AgentLoop`: Main agent runtime (run_turn, execute_tools)
//! - `Tool`: Trait for executable tools with JSON schema parameters
//! - `ContextBuilder`: Assembles system prompt with AGENTS.md, project files
//! - `SessionStorage`: Persists conversation history and tree structure
//! - `AgentEvent`: UI update events (thinking, tool execution, streaming, turn end)

pub mod agent;
pub mod config;
/// Context engineering (AGENTS.md, SYSTEM.md, compaction, skills, templates).
pub mod context;
pub mod error;
pub mod event;
/// Extension system for plugins and custom functionality.
pub mod extension;
/// Session management for conversation history and persistence.
pub mod session;
/// Skills system for on-demand capabilities.
pub mod skills;
/// Prompt template system.
pub mod templates;
pub mod tool;
pub mod tools;

pub use agent::{AgentLoop, default_tools};
pub use config::AgentConfig;
pub use context::{AgentsContext, ContextBuilder, ContextBundle, ContextDiscovery, SystemContext};
pub use error::{FaeAgentError, Result};
pub use event::{AgentEvent, EventReceiver, EventSender, TurnEndReason, event_channel};
pub use extension::{
    CommandDefinition, CommandHandler, CommandRegistry, Extension, ExtensionMetadata,
    ExtensionPackage, ExtensionRegistry, KeybindingDefinition, KeybindingHandler,
    KeybindingRegistry, OverlayConfig, PackageManager, SharedExtensionRegistry,
    ToolDefinition as ExtensionToolDefinition, ToolHandler as ExtensionToolHandler,
    ToolParameter as ExtensionToolParameter, ToolRegistry as ExtensionToolRegistry, WidgetFactory,
    WidgetRegistry, shared_registry,
};
pub use session::{
    Bookmark, BookmarkManager, Message, SessionId, SessionMetadata, SessionNode, SessionStorage,
    TreeNode, TreeRenderOptions, auto_fork_on_edit, build_session_tree, export_to_html,
    find_in_tree, find_last_active_session, find_session_by_prefix, fork_session, render_tree,
    restore_session,
};
pub use skills::{Skill, SkillRegistry};
pub use templates::{TemplateContext, TemplateEngine, get_builtin, list_builtins, render_simple};
pub use tool::{Tool, ToolRegistry};
pub use tools::{BashTool, EditTool, FindTool, GrepTool, LsTool, ReadTool, WriteTool};
