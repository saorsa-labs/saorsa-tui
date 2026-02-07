//! Context engineering system for the fae AI agent.
//!
//! This module provides:
//! - Discovery of AGENTS.md and SYSTEM.md files across multiple locations
//! - Loading and merging context files with precedence rules
//! - Context compaction strategies for managing token limits
//! - Skills system for on-demand capabilities
//! - Prompt templates

pub mod discovery;

pub use discovery::ContextDiscovery;
