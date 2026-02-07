//! fae-agent: AI coding agent runtime.
//!
//! Provides the agent loop, built-in tools (bash, read),
//! event system for UI integration, and tool registry.

pub mod agent;
pub mod config;
pub mod error;
pub mod event;
pub mod tool;
pub mod tools;

pub use agent::AgentLoop;
pub use config::AgentConfig;
pub use error::{FaeAgentError, Result};
pub use event::{AgentEvent, EventReceiver, EventSender, TurnEndReason, event_channel};
pub use tool::{Tool, ToolRegistry};
pub use tools::{BashTool, EditTool, FindTool, GrepTool, LsTool, ReadTool, WriteTool};
