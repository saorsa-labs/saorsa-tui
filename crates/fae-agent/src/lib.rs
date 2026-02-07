//! fae-agent: AI coding agent runtime.
//!
//! Provides the agent loop, built-in tools (bash, read, write, edit, grep, find, ls),
//! event system for UI integration, and tool registry.

pub mod agent;
pub mod config;
pub mod error;
pub mod event;
/// Session management for conversation history and persistence.
pub mod session;
pub mod tool;
pub mod tools;

pub use agent::{AgentLoop, default_tools};
pub use config::AgentConfig;
pub use error::{FaeAgentError, Result};
pub use event::{AgentEvent, EventReceiver, EventSender, TurnEndReason, event_channel};
pub use session::{
    Message, SessionId, SessionMetadata, SessionNode, SessionStorage, TreeNode,
    TreeRenderOptions, build_session_tree, find_in_tree, find_last_active_session,
    find_session_by_prefix, render_tree, restore_session,
};
pub use tool::{Tool, ToolRegistry};
pub use tools::{BashTool, EditTool, FindTool, GrepTool, LsTool, ReadTool, WriteTool};
