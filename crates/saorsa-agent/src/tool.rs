//! Tool trait and registry for agent tool execution.

use std::collections::HashMap;

use saorsa_ai::ToolDefinition;

use crate::error::Result;

/// A tool that the agent can execute.
#[async_trait::async_trait]
pub trait Tool: Send + Sync {
    /// The unique name of the tool.
    fn name(&self) -> &str;

    /// A human-readable description of what the tool does.
    fn description(&self) -> &str;

    /// JSON Schema describing the tool's input parameters.
    fn input_schema(&self) -> serde_json::Value;

    /// Execute the tool with the given JSON input and return the result as a string.
    async fn execute(&self, input: serde_json::Value) -> Result<String>;

    /// Convert this tool to a `ToolDefinition` for the LLM API.
    fn to_definition(&self) -> ToolDefinition {
        ToolDefinition::new(self.name(), self.description(), self.input_schema())
    }
}

/// Registry for managing available tools.
pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
}

impl ToolRegistry {
    /// Create a new empty tool registry.
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// Register a tool. Replaces any existing tool with the same name.
    pub fn register(&mut self, tool: Box<dyn Tool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    /// Look up a tool by name.
    pub fn get(&self, name: &str) -> Option<&dyn Tool> {
        self.tools.get(name).map(AsRef::as_ref)
    }

    /// Get all tool definitions for the LLM API.
    pub fn definitions(&self) -> Vec<ToolDefinition> {
        self.tools.values().map(|t| t.to_definition()).collect()
    }

    /// Get the names of all registered tools.
    pub fn names(&self) -> Vec<&str> {
        self.tools.keys().map(String::as_str).collect()
    }

    /// Return the number of registered tools.
    pub fn len(&self) -> usize {
        self.tools.len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
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

    struct EchoTool;

    #[async_trait::async_trait]
    impl Tool for EchoTool {
        fn name(&self) -> &str {
            "echo"
        }
        fn description(&self) -> &str {
            "Echoes input back"
        }
        fn input_schema(&self) -> serde_json::Value {
            serde_json::json!({
                "type": "object",
                "properties": {
                    "text": {"type": "string"}
                },
                "required": ["text"]
            })
        }
        async fn execute(&self, input: serde_json::Value) -> Result<String> {
            let text = input
                .get("text")
                .and_then(|v| v.as_str())
                .unwrap_or("(empty)");
            Ok(text.to_string())
        }
    }

    #[test]
    fn registry_register_and_get() {
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(EchoTool));
        assert_eq!(registry.len(), 1);
        assert!(!registry.is_empty());
        assert!(registry.get("echo").is_some());
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn registry_definitions() {
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(EchoTool));
        let defs = registry.definitions();
        assert_eq!(defs.len(), 1);
        assert_eq!(defs[0].name, "echo");
    }

    #[test]
    fn registry_names() {
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(EchoTool));
        let names = registry.names();
        assert!(names.contains(&"echo"));
    }

    #[test]
    fn tool_to_definition() {
        let tool = EchoTool;
        let def = tool.to_definition();
        assert_eq!(def.name, "echo");
        assert_eq!(def.description, "Echoes input back");
    }

    #[test]
    fn registry_default() {
        let registry = ToolRegistry::default();
        assert!(registry.is_empty());
    }

    #[tokio::test]
    async fn tool_execute() {
        let tool = EchoTool;
        let result = tool.execute(serde_json::json!({"text": "hello"})).await;
        assert!(result.is_ok());
        if let Ok(output) = result {
            assert_eq!(output, "hello");
        }
    }
}
