//! Command registration system for extensions.

use crate::error::{FaeAgentError, Result};
use std::collections::HashMap;
use std::sync::Arc;

/// Type alias for command handler functions.
pub type CommandHandler = Arc<dyn Fn(&[&str]) -> Result<String> + Send + Sync>;

/// Command definition registered by an extension.
pub struct CommandDefinition {
    /// Unique command name.
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// Usage string (e.g., "command \[options\] \<args\>").
    pub usage: String,
    /// Command handler function.
    pub handler: CommandHandler,
}

impl CommandDefinition {
    /// Creates a new command definition.
    pub fn new(name: String, description: String, usage: String, handler: CommandHandler) -> Self {
        Self {
            name,
            description,
            usage,
            handler,
        }
    }
}

/// Registry for extension-provided commands.
pub struct CommandRegistry {
    commands: HashMap<String, CommandDefinition>,
}

impl CommandRegistry {
    /// Creates a new empty command registry.
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
        }
    }

    /// Registers a command.
    ///
    /// Returns an error if a command with the same name is already registered.
    pub fn register_command(&mut self, def: CommandDefinition) -> Result<()> {
        if self.commands.contains_key(&def.name) {
            return Err(FaeAgentError::Extension(format!(
                "command '{}' is already registered",
                def.name
            )));
        }
        self.commands.insert(def.name.clone(), def);
        Ok(())
    }

    /// Unregisters a command by name.
    ///
    /// Returns an error if the command is not found.
    pub fn unregister_command(&mut self, name: &str) -> Result<()> {
        self.commands
            .remove(name)
            .ok_or_else(|| FaeAgentError::Extension(format!("command '{}' not found", name)))?;
        Ok(())
    }

    /// Gets a command definition by name.
    pub fn get_command(&self, name: &str) -> Option<&CommandDefinition> {
        self.commands.get(name)
    }

    /// Lists all registered commands.
    pub fn list_commands(&self) -> Vec<&CommandDefinition> {
        self.commands.values().collect()
    }

    /// Executes a command by name with the given arguments.
    ///
    /// Returns an error if the command is not found or execution fails.
    pub fn execute_command(&self, name: &str, args: &[&str]) -> Result<String> {
        let def = self
            .commands
            .get(name)
            .ok_or_else(|| FaeAgentError::Extension(format!("command '{}' not found", name)))?;
        (def.handler)(args)
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn echo_handler(args: &[&str]) -> Result<String> {
        Ok(format!("echo: {}", args.join(" ")))
    }

    #[test]
    fn register_command() {
        let mut registry = CommandRegistry::new();
        let def = CommandDefinition::new(
            "echo".to_string(),
            "Echo arguments".to_string(),
            "echo <text>".to_string(),
            Arc::new(echo_handler),
        );
        let result = registry.register_command(def);
        assert!(result.is_ok());
        assert!(registry.get_command("echo").is_some());
    }

    #[test]
    fn duplicate_command_fails() {
        let mut registry = CommandRegistry::new();
        let def1 = CommandDefinition::new(
            "echo".to_string(),
            "Echo 1".to_string(),
            "echo".to_string(),
            Arc::new(echo_handler),
        );
        let def2 = CommandDefinition::new(
            "echo".to_string(),
            "Echo 2".to_string(),
            "echo".to_string(),
            Arc::new(echo_handler),
        );
        assert!(registry.register_command(def1).is_ok());
        let result = registry.register_command(def2);
        assert!(result.is_err());
        match result {
            Err(FaeAgentError::Extension(msg)) => {
                assert!(msg.contains("already registered"));
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn unregister_command() {
        let mut registry = CommandRegistry::new();
        let def = CommandDefinition::new(
            "echo".to_string(),
            "Echo".to_string(),
            "echo".to_string(),
            Arc::new(echo_handler),
        );
        assert!(registry.register_command(def).is_ok());
        assert!(registry.unregister_command("echo").is_ok());
        assert!(registry.get_command("echo").is_none());
    }

    #[test]
    fn unregister_nonexistent_fails() {
        let mut registry = CommandRegistry::new();
        let result = registry.unregister_command("nonexistent");
        assert!(result.is_err());
        match result {
            Err(FaeAgentError::Extension(msg)) => {
                assert!(msg.contains("not found"));
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn list_commands() {
        let mut registry = CommandRegistry::new();
        let def1 = CommandDefinition::new(
            "echo".to_string(),
            "Echo".to_string(),
            "echo".to_string(),
            Arc::new(echo_handler),
        );
        let def2 = CommandDefinition::new(
            "test".to_string(),
            "Test".to_string(),
            "test".to_string(),
            Arc::new(echo_handler),
        );
        assert!(registry.register_command(def1).is_ok());
        assert!(registry.register_command(def2).is_ok());
        let list = registry.list_commands();
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn execute_command() {
        let mut registry = CommandRegistry::new();
        let def = CommandDefinition::new(
            "echo".to_string(),
            "Echo".to_string(),
            "echo".to_string(),
            Arc::new(echo_handler),
        );
        assert!(registry.register_command(def).is_ok());
        let result = registry.execute_command("echo", &["hello", "world"]);
        assert!(result.is_ok());
        let output = result.ok().unwrap_or_default();
        assert_eq!(output, "echo: hello world");
    }

    #[test]
    fn execute_nonexistent_fails() {
        let registry = CommandRegistry::new();
        let result = registry.execute_command("nonexistent", &[]);
        assert!(result.is_err());
        match result {
            Err(FaeAgentError::Extension(msg)) => {
                assert!(msg.contains("not found"));
            }
            _ => unreachable!(),
        }
    }
}
