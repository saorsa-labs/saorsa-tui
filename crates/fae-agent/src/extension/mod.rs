//! Extension system for fae-agent.
//!
//! The extension system allows dynamically loading plugins that can:
//! - Register custom tools, commands, and keybindings
//! - Add custom UI widgets and overlays
//! - Hook into agent lifecycle events (tool calls, messages, turns)
//!
//! Extensions are trait-based and use dynamic dispatch, allowing for future
//! WASM-backed implementations without adding heavy dependencies now.

pub mod command_registry;
pub mod registry;
pub mod tool_registry;

use crate::error::Result;

pub use command_registry::{CommandDefinition, CommandHandler, CommandRegistry};
pub use registry::{ExtensionRegistry, SharedExtensionRegistry, shared_registry};
pub use tool_registry::{ToolDefinition, ToolHandler, ToolParameter, ToolRegistry};

/// Extension trait defining the lifecycle and capabilities of a plugin.
///
/// Extensions can hook into agent events and provide custom functionality
/// without modifying the core agent runtime.
pub trait Extension: Send + Sync {
    /// Returns the unique name of this extension.
    fn name(&self) -> &str;

    /// Returns the semantic version of this extension.
    fn version(&self) -> &str;

    /// Called when the extension is loaded into the runtime.
    ///
    /// Use this to initialize resources, register tools/commands, etc.
    fn on_load(&mut self) -> Result<()> {
        Ok(())
    }

    /// Called when the extension is unloaded from the runtime.
    ///
    /// Use this to clean up resources, unregister handlers, etc.
    fn on_unload(&mut self) -> Result<()> {
        Ok(())
    }

    /// Called when a tool is invoked by the agent.
    ///
    /// Return `Some(output)` to intercept and handle the tool call,
    /// or `None` to allow normal processing.
    fn on_tool_call(&mut self, _tool: &str, _args: &str) -> Result<Option<String>> {
        Ok(None)
    }

    /// Called when a message is received by the agent.
    ///
    /// Return `Some(response)` to intercept and handle the message,
    /// or `None` to allow normal processing.
    fn on_message(&mut self, _message: &str) -> Result<Option<String>> {
        Ok(None)
    }

    /// Called at the start of each agent turn.
    fn on_turn_start(&mut self) -> Result<()> {
        Ok(())
    }

    /// Called at the end of each agent turn.
    fn on_turn_end(&mut self) -> Result<()> {
        Ok(())
    }
}

/// Metadata describing an extension.
#[derive(Debug, Clone)]
pub struct ExtensionMetadata {
    /// Unique extension name.
    pub name: String,
    /// Semantic version.
    pub version: String,
    /// Human-readable description.
    pub description: String,
    /// Author name or organization.
    pub author: String,
}

impl ExtensionMetadata {
    /// Creates new extension metadata.
    pub fn new(name: String, version: String, description: String, author: String) -> Self {
        Self {
            name,
            version,
            description,
            author,
        }
    }
}
