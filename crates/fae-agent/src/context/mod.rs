//! Context engineering system for the fae AI agent.
//!
//! This module provides:
//! - Discovery of AGENTS.md and SYSTEM.md files across multiple locations
//! - Loading and merging context files with precedence rules
//! - Context compaction strategies for managing token limits
//! - Skills system for on-demand capabilities
//! - Prompt templates

pub mod agents;
pub mod discovery;
pub mod system;
pub mod types;

pub use agents::AgentsContext;
pub use discovery::ContextDiscovery;
pub use system::SystemContext;
pub use types::{MergeStrategy, SystemMode};
