//! Tool registration system for extensions.

use crate::error::{FaeAgentError, Result};
use std::collections::HashMap;
use std::sync::Arc;

/// Type alias for tool handler functions.
pub type ToolHandler = Arc<dyn Fn(&str) -> Result<String> + Send + Sync>;

/// Parameter definition for a tool.
#[derive(Debug, Clone)]
pub struct ToolParameter {
    /// Parameter name.
    pub name: String,
    /// Parameter type (e.g., "string", "number", "boolean").
    pub param_type: String,
    /// Human-readable description.
    pub description: String,
    /// Whether this parameter is required.
    pub required: bool,
}

impl ToolParameter {
    /// Creates a new tool parameter definition.
    pub fn new(name: String, param_type: String, description: String, required: bool) -> Self {
        Self {
            name,
            param_type,
            description,
            required,
        }
    }
}

/// Tool definition registered by an extension.
pub struct ToolDefinition {
    /// Unique tool name.
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// Parameter definitions.
    pub parameters: Vec<ToolParameter>,
    /// Tool handler function.
    pub handler: ToolHandler,
}

impl ToolDefinition {
    /// Creates a new tool definition.
    pub fn new(
        name: String,
        description: String,
        parameters: Vec<ToolParameter>,
        handler: ToolHandler,
    ) -> Self {
        Self {
            name,
            description,
            parameters,
            handler,
        }
    }
}

/// Registry for extension-provided tools.
pub struct ToolRegistry {
    tools: HashMap<String, ToolDefinition>,
}

impl ToolRegistry {
    /// Creates a new empty tool registry.
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// Registers a tool.
    ///
    /// Returns an error if a tool with the same name is already registered.
    pub fn register_tool(&mut self, def: ToolDefinition) -> Result<()> {
        if self.tools.contains_key(&def.name) {
            return Err(FaeAgentError::Extension(format!(
                "tool '{}' is already registered",
                def.name
            )));
        }
        self.tools.insert(def.name.clone(), def);
        Ok(())
    }

    /// Unregisters a tool by name.
    ///
    /// Returns an error if the tool is not found.
    pub fn unregister_tool(&mut self, name: &str) -> Result<()> {
        self.tools
            .remove(name)
            .ok_or_else(|| FaeAgentError::Extension(format!("tool '{}' not found", name)))?;
        Ok(())
    }

    /// Gets a tool definition by name.
    pub fn get_tool(&self, name: &str) -> Option<&ToolDefinition> {
        self.tools.get(name)
    }

    /// Lists all registered tools.
    pub fn list_tools(&self) -> Vec<&ToolDefinition> {
        self.tools.values().collect()
    }

    /// Executes a tool by name with the given arguments.
    ///
    /// Returns an error if the tool is not found or execution fails.
    pub fn execute_tool(&self, name: &str, args: &str) -> Result<String> {
        let def = self
            .tools
            .get(name)
            .ok_or_else(|| FaeAgentError::Extension(format!("tool '{}' not found", name)))?;
        (def.handler)(args)
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn echo_handler(args: &str) -> Result<String> {
        Ok(format!("echo: {}", args))
    }

    #[test]
    fn register_tool() {
        let mut registry = ToolRegistry::new();
        let def = ToolDefinition::new(
            "echo".to_string(),
            "Echo arguments".to_string(),
            vec![],
            Arc::new(echo_handler),
        );
        let result = registry.register_tool(def);
        assert!(result.is_ok());
        assert!(registry.get_tool("echo").is_some());
    }

    #[test]
    fn duplicate_tool_fails() {
        let mut registry = ToolRegistry::new();
        let def1 = ToolDefinition::new(
            "echo".to_string(),
            "Echo 1".to_string(),
            vec![],
            Arc::new(echo_handler),
        );
        let def2 = ToolDefinition::new(
            "echo".to_string(),
            "Echo 2".to_string(),
            vec![],
            Arc::new(echo_handler),
        );
        assert!(registry.register_tool(def1).is_ok());
        let result = registry.register_tool(def2);
        assert!(result.is_err());
        match result {
            Err(FaeAgentError::Extension(msg)) => {
                assert!(msg.contains("already registered"));
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn unregister_tool() {
        let mut registry = ToolRegistry::new();
        let def = ToolDefinition::new(
            "echo".to_string(),
            "Echo".to_string(),
            vec![],
            Arc::new(echo_handler),
        );
        assert!(registry.register_tool(def).is_ok());
        assert!(registry.unregister_tool("echo").is_ok());
        assert!(registry.get_tool("echo").is_none());
    }

    #[test]
    fn unregister_nonexistent_fails() {
        let mut registry = ToolRegistry::new();
        let result = registry.unregister_tool("nonexistent");
        assert!(result.is_err());
        match result {
            Err(FaeAgentError::Extension(msg)) => {
                assert!(msg.contains("not found"));
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn list_tools() {
        let mut registry = ToolRegistry::new();
        let def1 = ToolDefinition::new(
            "echo".to_string(),
            "Echo".to_string(),
            vec![],
            Arc::new(echo_handler),
        );
        let def2 = ToolDefinition::new(
            "test".to_string(),
            "Test".to_string(),
            vec![],
            Arc::new(echo_handler),
        );
        assert!(registry.register_tool(def1).is_ok());
        assert!(registry.register_tool(def2).is_ok());
        let list = registry.list_tools();
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn execute_tool() {
        let mut registry = ToolRegistry::new();
        let def = ToolDefinition::new(
            "echo".to_string(),
            "Echo".to_string(),
            vec![],
            Arc::new(echo_handler),
        );
        assert!(registry.register_tool(def).is_ok());
        let result = registry.execute_tool("echo", "hello");
        assert!(result.is_ok());
        let output = result.ok().unwrap_or_default();
        assert_eq!(output, "echo: hello");
    }

    #[test]
    fn execute_nonexistent_fails() {
        let registry = ToolRegistry::new();
        let result = registry.execute_tool("nonexistent", "args");
        assert!(result.is_err());
        match result {
            Err(FaeAgentError::Extension(msg)) => {
                assert!(msg.contains("not found"));
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn tool_parameter_creation() {
        let param = ToolParameter::new(
            "name".to_string(),
            "string".to_string(),
            "A name parameter".to_string(),
            true,
        );
        assert_eq!(param.name, "name");
        assert_eq!(param.param_type, "string");
        assert!(param.required);
    }
}
