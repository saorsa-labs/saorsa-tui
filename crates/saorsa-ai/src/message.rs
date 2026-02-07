//! Message and content types for LLM conversations.

use serde::{Deserialize, Serialize};

/// The role of a message participant.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// User message.
    User,
    /// Assistant (model) message.
    Assistant,
}

/// A conversation message.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    /// The role of the message sender.
    pub role: Role,
    /// The content blocks.
    pub content: Vec<ContentBlock>,
}

impl Message {
    /// Create a user message with text content.
    pub fn user(text: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: vec![ContentBlock::Text { text: text.into() }],
        }
    }

    /// Create an assistant message with text content.
    pub fn assistant(text: impl Into<String>) -> Self {
        Self {
            role: Role::Assistant,
            content: vec![ContentBlock::Text { text: text.into() }],
        }
    }

    /// Create a message with tool result content.
    pub fn tool_result(tool_use_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: vec![ContentBlock::ToolResult {
                tool_use_id: tool_use_id.into(),
                content: content.into(),
            }],
        }
    }
}

/// A block of content within a message.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    /// Plain text content.
    Text {
        /// The text.
        text: String,
    },
    /// A tool use request from the assistant.
    ToolUse {
        /// Unique ID for this tool use.
        id: String,
        /// Tool name.
        name: String,
        /// Tool input (JSON object).
        input: serde_json::Value,
    },
    /// A tool result from the user.
    ToolResult {
        /// The ID of the tool_use this is responding to.
        tool_use_id: String,
        /// The result content.
        content: String,
    },
}

/// Definition of a tool the model can use.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// The tool name.
    pub name: String,
    /// Description of what the tool does.
    pub description: String,
    /// JSON Schema for the tool's input parameters.
    pub input_schema: serde_json::Value,
}

impl ToolDefinition {
    /// Create a new tool definition.
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        input_schema: serde_json::Value,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            input_schema,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_message_construction() {
        let msg = Message::user("Hello");
        assert_eq!(msg.role, Role::User);
        assert_eq!(msg.content.len(), 1);
        match &msg.content[0] {
            ContentBlock::Text { text } => {
                assert_eq!(text, "Hello");
            }
            _ => panic!("Expected Text content block"),
        }
    }

    #[test]
    fn assistant_message_construction() {
        let msg = Message::assistant("Hi there");
        assert_eq!(msg.role, Role::Assistant);
    }

    #[test]
    fn message_serialization_roundtrip() {
        let msg = Message::user("test");
        let json = serde_json::to_string(&msg);
        assert!(json.is_ok());
        let json = json.as_deref().unwrap_or("");
        let parsed: Result<Message, _> = serde_json::from_str(json);
        assert!(parsed.is_ok());
    }

    #[test]
    fn tool_use_serialization() {
        let block = ContentBlock::ToolUse {
            id: "tool_1".into(),
            name: "bash".into(),
            input: serde_json::json!({"command": "ls"}),
        };
        let json = serde_json::to_string(&block);
        assert!(json.is_ok());
        let json_str = json.as_deref().unwrap_or("");
        assert!(json_str.contains("tool_use"));
        assert!(json_str.contains("bash"));
    }

    #[test]
    fn tool_result_message() {
        let msg = Message::tool_result("tool_1", "file.txt");
        assert_eq!(msg.role, Role::User);
        match &msg.content[0] {
            ContentBlock::ToolResult {
                tool_use_id,
                content,
            } => {
                assert_eq!(tool_use_id, "tool_1");
                assert_eq!(content, "file.txt");
            }
            _ => panic!("Expected ToolResult content block"),
        }
    }

    #[test]
    fn tool_definition_creation() {
        let tool = ToolDefinition::new(
            "read_file",
            "Read a file from disk",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string"}
                },
                "required": ["path"]
            }),
        );
        assert_eq!(tool.name, "read_file");
    }
}
