//! fae-agent: AI coding agent runtime.
//!
//! Provides the agent loop, built-in tools (bash, read, write, edit, grep, find, ls),
//! event system for UI integration, and tool registry.

pub mod agent;
pub mod config;
/// Context engineering (AGENTS.md, SYSTEM.md, compaction, skills, templates).
pub mod context;
pub mod error;
pub mod event;
/// Session management for conversation history and persistence.
pub mod session;
pub mod tool;
pub mod tools;

pub use agent::{AgentLoop, default_tools};
pub use config::AgentConfig;
pub use context::{AgentsContext, ContextBuilder, ContextBundle, ContextDiscovery, SystemContext};
pub use error::{FaeAgentError, Result};
pub use event::{AgentEvent, EventReceiver, EventSender, TurnEndReason, event_channel};
pub use session::{
    Bookmark, BookmarkManager, Message, SessionId, SessionMetadata, SessionNode, SessionStorage,
    TreeNode, TreeRenderOptions, auto_fork_on_edit, build_session_tree, export_to_html,
    find_in_tree, find_last_active_session, find_session_by_prefix, fork_session, render_tree,
    restore_session,
};
pub use tool::{Tool, ToolRegistry};
pub use tools::{BashTool, EditTool, FindTool, GrepTool, LsTool, ReadTool, WriteTool};
