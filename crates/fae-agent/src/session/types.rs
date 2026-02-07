use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::str::FromStr;
use uuid::Uuid;

/// Unique identifier for a session
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(Uuid);

impl SessionId {
    /// Create a new random session ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Get the full UUID as a string
    pub fn as_str(&self) -> String {
        self.0.to_string()
    }

    /// Get a short 8-character prefix for display
    pub fn prefix(&self) -> String {
        self.0.to_string()[..8].to_string()
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.prefix())
    }
}

impl FromStr for SessionId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

/// Metadata associated with a session
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SessionMetadata {
    /// When the session was created
    pub created: DateTime<Utc>,
    /// When the session was last modified
    pub modified: DateTime<Utc>,
    /// User-provided title for the session
    pub title: Option<String>,
    /// Optional description
    pub description: Option<String>,
    /// Tags for organization and filtering
    pub tags: HashSet<String>,
}

impl SessionMetadata {
    /// Create new metadata with current timestamp
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            created: now,
            modified: now,
            title: None,
            description: None,
            tags: HashSet::new(),
        }
    }

    /// Update the modified timestamp to now
    pub fn touch(&mut self) {
        self.modified = Utc::now();
    }

    /// Add a tag
    pub fn add_tag(&mut self, tag: String) {
        self.tags.insert(tag);
    }

    /// Remove a tag
    pub fn remove_tag(&mut self, tag: &str) -> bool {
        self.tags.remove(tag)
    }
}

impl Default for SessionMetadata {
    fn default() -> Self {
        Self::new()
    }
}

/// Node in the session tree (represents parent-child relationships)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SessionNode {
    /// The session ID this node represents
    pub id: SessionId,
    /// Parent session ID (None for root sessions)
    pub parent_id: Option<SessionId>,
    /// Child session IDs (forked/branched sessions)
    pub child_ids: Vec<SessionId>,
}

impl SessionNode {
    /// Create a new root node (no parent)
    pub fn new_root(id: SessionId) -> Self {
        Self {
            id,
            parent_id: None,
            child_ids: Vec::new(),
        }
    }

    /// Create a new child node
    pub fn new_child(id: SessionId, parent_id: SessionId) -> Self {
        Self {
            id,
            parent_id: Some(parent_id),
            child_ids: Vec::new(),
        }
    }

    /// Add a child to this node
    pub fn add_child(&mut self, child_id: SessionId) {
        if !self.child_ids.contains(&child_id) {
            self.child_ids.push(child_id);
        }
    }

    /// Remove a child from this node
    pub fn remove_child(&mut self, child_id: &SessionId) -> bool {
        if let Some(pos) = self.child_ids.iter().position(|id| id == child_id) {
            self.child_ids.remove(pos);
            true
        } else {
            false
        }
    }

    /// Check if this is a root node
    pub fn is_root(&self) -> bool {
        self.parent_id.is_none()
    }
}

/// A message in the conversation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Message {
    /// Message from the user
    User {
        /// The message content
        content: String,
        /// When the message was sent
        timestamp: DateTime<Utc>,
    },
    /// Message from the assistant
    Assistant {
        /// The message content
        content: String,
        /// When the message was sent
        timestamp: DateTime<Utc>,
    },
    /// Tool call by the assistant
    ToolCall {
        /// Name of the tool being called
        tool_name: String,
        /// Input arguments for the tool
        tool_input: serde_json::Value,
        /// When the tool was called
        timestamp: DateTime<Utc>,
    },
    /// Result from a tool execution
    ToolResult {
        /// Name of the tool that was executed
        tool_name: String,
        /// Result returned by the tool
        result: serde_json::Value,
        /// When the result was received
        timestamp: DateTime<Utc>,
    },
}

impl Message {
    /// Get the timestamp of this message
    pub fn timestamp(&self) -> &DateTime<Utc> {
        match self {
            Message::User { timestamp, .. }
            | Message::Assistant { timestamp, .. }
            | Message::ToolCall { timestamp, .. }
            | Message::ToolResult { timestamp, .. } => timestamp,
        }
    }

    /// Create a new user message with current timestamp
    pub fn user(content: String) -> Self {
        Message::User {
            content,
            timestamp: Utc::now(),
        }
    }

    /// Create a new assistant message with current timestamp
    pub fn assistant(content: String) -> Self {
        Message::Assistant {
            content,
            timestamp: Utc::now(),
        }
    }

    /// Create a new tool call message with current timestamp
    pub fn tool_call(tool_name: String, tool_input: serde_json::Value) -> Self {
        Message::ToolCall {
            tool_name,
            tool_input,
            timestamp: Utc::now(),
        }
    }

    /// Create a new tool result message with current timestamp
    pub fn tool_result(tool_name: String, result: serde_json::Value) -> Self {
        Message::ToolResult {
            tool_name,
            result,
            timestamp: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_id_generation() {
        let id1 = SessionId::new();
        let id2 = SessionId::new();
        assert!(id1 != id2, "Session IDs should be unique");
    }

    #[test]
    fn test_session_id_prefix() {
        let id = SessionId::new();
        let prefix = id.prefix();
        assert!(prefix.len() == 8, "Prefix should be 8 characters");
    }

    #[test]
    fn test_session_id_roundtrip() {
        let id = SessionId::new();
        let s = id.as_str();
        let parsed = SessionId::from_str(&s);
        assert!(parsed.is_ok());
        match parsed {
            Ok(parsed_id) => assert!(id == parsed_id),
            Err(_) => unreachable!(),
        }
    }

    #[test]
    fn test_metadata_new() {
        let meta = SessionMetadata::new();
        assert!(meta.title.is_none());
        assert!(meta.description.is_none());
        assert!(meta.tags.is_empty());
        assert!(meta.created <= meta.modified);
    }

    #[test]
    fn test_metadata_touch() {
        let mut meta = SessionMetadata::new();
        let original_modified = meta.modified;
        std::thread::sleep(std::time::Duration::from_millis(10));
        meta.touch();
        assert!(meta.modified > original_modified);
    }

    #[test]
    fn test_metadata_tags() {
        let mut meta = SessionMetadata::new();
        meta.add_tag("rust".to_string());
        meta.add_tag("ai".to_string());
        assert!(meta.tags.contains("rust"));
        assert!(meta.tags.contains("ai"));

        let removed = meta.remove_tag("rust");
        assert!(removed);
        assert!(!meta.tags.contains("rust"));

        let not_found = meta.remove_tag("nonexistent");
        assert!(!not_found);
    }

    #[test]
    fn test_session_node_root() {
        let id = SessionId::new();
        let node = SessionNode::new_root(id);
        assert!(node.is_root());
        assert!(node.child_ids.is_empty());
    }

    #[test]
    fn test_session_node_child() {
        let parent_id = SessionId::new();
        let child_id = SessionId::new();
        let node = SessionNode::new_child(child_id, parent_id);
        assert!(!node.is_root());
        assert!(node.parent_id == Some(parent_id));
    }

    #[test]
    fn test_session_node_add_remove_child() {
        let id = SessionId::new();
        let child1 = SessionId::new();
        let child2 = SessionId::new();

        let mut node = SessionNode::new_root(id);
        node.add_child(child1);
        node.add_child(child2);
        assert!(node.child_ids.len() == 2);

        // Adding same child again should not duplicate
        node.add_child(child1);
        assert!(node.child_ids.len() == 2);

        let removed = node.remove_child(&child1);
        assert!(removed);
        assert!(node.child_ids.len() == 1);

        let not_found = node.remove_child(&child1);
        assert!(!not_found);
    }

    #[test]
    fn test_message_user() {
        let msg = Message::user("Hello".to_string());
        match msg {
            Message::User { content, .. } => assert!(content == "Hello"),
            _ => panic!("Expected User message"),
        }
    }

    #[test]
    fn test_message_assistant() {
        let msg = Message::assistant("Hi there".to_string());
        match msg {
            Message::Assistant { content, .. } => assert!(content == "Hi there"),
            _ => panic!("Expected Assistant message"),
        }
    }

    #[test]
    fn test_message_tool_call() {
        let input = serde_json::json!({"arg": "value"});
        let msg = Message::tool_call("bash".to_string(), input.clone());
        match msg {
            Message::ToolCall {
                tool_name,
                tool_input,
                ..
            } => {
                assert!(tool_name == "bash");
                assert!(tool_input == input);
            }
            _ => panic!("Expected ToolCall message"),
        }
    }

    #[test]
    fn test_message_tool_result() {
        let result = serde_json::json!({"output": "success"});
        let msg = Message::tool_result("bash".to_string(), result.clone());
        match msg {
            Message::ToolResult {
                tool_name,
                result: res,
                ..
            } => {
                assert!(tool_name == "bash");
                assert!(res == result);
            }
            _ => panic!("Expected ToolResult message"),
        }
    }

    #[test]
    fn test_message_serialization() {
        let msg = Message::user("Test message".to_string());
        let json = serde_json::to_string(&msg);
        assert!(json.is_ok());

        match json {
            Ok(json_str) => {
                let deserialized = serde_json::from_str::<Message>(&json_str);
                assert!(deserialized.is_ok());
                match deserialized {
                    Ok(deser_msg) => assert!(deser_msg == msg),
                    Err(_) => unreachable!(),
                }
            }
            Err(_) => unreachable!(),
        }
    }

    #[test]
    fn test_metadata_clone() {
        let meta = SessionMetadata::new();
        let cloned = meta.clone();
        assert!(meta == cloned);
    }

    #[test]
    fn test_session_node_equality() {
        let id = SessionId::new();
        let node1 = SessionNode::new_root(id);
        let node2 = SessionNode::new_root(id);
        assert!(node1 == node2);
    }
}
