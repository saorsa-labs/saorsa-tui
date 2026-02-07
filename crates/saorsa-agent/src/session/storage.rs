//! Session storage and serialization.

use crate::SaorsaAgentError;
use crate::session::{Message, SessionId, SessionMetadata, SessionNode};
use std::fs;
use std::path::PathBuf;

/// Manages filesystem storage for sessions.
#[derive(Clone)]
pub struct SessionStorage {
    base_path: PathBuf,
}

impl SessionStorage {
    /// Create a new storage manager with the default base path.
    pub fn new() -> Result<Self, SaorsaAgentError> {
        let base_path = crate::session::path::sessions_dir()?;
        Ok(Self { base_path })
    }

    /// Create a storage manager with a custom base path (for testing).
    pub fn with_base_path(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    /// Get the base path for all sessions.
    pub fn base_path(&self) -> &std::path::Path {
        &self.base_path
    }

    /// Get the directory for a specific session.
    fn session_dir(&self, session_id: &SessionId) -> PathBuf {
        self.base_path.join(session_id.as_str())
    }

    /// Ensure the session directory exists.
    fn ensure_session_dir(&self, session_id: &SessionId) -> Result<(), SaorsaAgentError> {
        let dir = self.session_dir(session_id);
        crate::session::path::ensure_dir(&dir)?;
        crate::session::path::ensure_dir(&dir.join("messages"))?;
        Ok(())
    }

    /// Write data to a file atomically (write to temp, then rename).
    fn write_atomic(&self, path: &std::path::Path, data: &[u8]) -> Result<(), SaorsaAgentError> {
        let temp_path = path.with_extension("tmp");
        fs::write(&temp_path, data)
            .map_err(|e| SaorsaAgentError::Session(format!("Failed to write temp file: {}", e)))?;
        fs::rename(&temp_path, path)
            .map_err(|e| SaorsaAgentError::Session(format!("Failed to rename temp file: {}", e)))?;
        Ok(())
    }

    /// Save session metadata to manifest.json.
    pub fn save_manifest(
        &self,
        session_id: &SessionId,
        metadata: &SessionMetadata,
    ) -> Result<(), SaorsaAgentError> {
        self.ensure_session_dir(session_id)?;
        let path = self.session_dir(session_id).join("manifest.json");
        let json = serde_json::to_string_pretty(metadata).map_err(|e| {
            SaorsaAgentError::Session(format!("Failed to serialize manifest: {}", e))
        })?;
        self.write_atomic(&path, json.as_bytes())?;
        Ok(())
    }

    /// Load session metadata from manifest.json.
    pub fn load_manifest(
        &self,
        session_id: &SessionId,
    ) -> Result<SessionMetadata, SaorsaAgentError> {
        let path = self.session_dir(session_id).join("manifest.json");
        let json = fs::read_to_string(&path)
            .map_err(|e| SaorsaAgentError::Session(format!("Failed to read manifest: {}", e)))?;
        serde_json::from_str(&json).map_err(|e| {
            SaorsaAgentError::Session(format!("Failed to deserialize manifest: {}", e))
        })
    }

    /// Save session tree structure to tree.json.
    pub fn save_tree(
        &self,
        session_id: &SessionId,
        node: &SessionNode,
    ) -> Result<(), SaorsaAgentError> {
        self.ensure_session_dir(session_id)?;
        let path = self.session_dir(session_id).join("tree.json");
        let json = serde_json::to_string_pretty(node)
            .map_err(|e| SaorsaAgentError::Session(format!("Failed to serialize tree: {}", e)))?;
        self.write_atomic(&path, json.as_bytes())?;
        Ok(())
    }

    /// Load session tree structure from tree.json.
    pub fn load_tree(&self, session_id: &SessionId) -> Result<SessionNode, SaorsaAgentError> {
        let path = self.session_dir(session_id).join("tree.json");
        let json = fs::read_to_string(&path)
            .map_err(|e| SaorsaAgentError::Session(format!("Failed to read tree: {}", e)))?;
        serde_json::from_str(&json)
            .map_err(|e| SaorsaAgentError::Session(format!("Failed to deserialize tree: {}", e)))
    }

    /// Save a message to messages/{index}-{type}.json.
    pub fn save_message(
        &self,
        session_id: &SessionId,
        index: usize,
        message: &Message,
    ) -> Result<(), SaorsaAgentError> {
        self.ensure_session_dir(session_id)?;

        let message_type = match message {
            Message::User { .. } => "user",
            Message::Assistant { .. } => "assistant",
            Message::ToolCall { .. } => "tool_call",
            Message::ToolResult { .. } => "tool_result",
        };

        let path = self
            .session_dir(session_id)
            .join("messages")
            .join(format!("{}-{}.json", index, message_type));

        let json = serde_json::to_string_pretty(message).map_err(|e| {
            SaorsaAgentError::Session(format!("Failed to serialize message: {}", e))
        })?;

        self.write_atomic(&path, json.as_bytes())?;
        Ok(())
    }

    /// Load all messages for a session, in order.
    pub fn load_messages(&self, session_id: &SessionId) -> Result<Vec<Message>, SaorsaAgentError> {
        let messages_dir = self.session_dir(session_id).join("messages");

        if !messages_dir.exists() {
            return Ok(Vec::new());
        }

        let mut entries: Vec<_> = fs::read_dir(&messages_dir)
            .map_err(|e| {
                SaorsaAgentError::Session(format!("Failed to read messages directory: {}", e))
            })?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| {
                SaorsaAgentError::Session(format!("Failed to read directory entry: {}", e))
            })?;

        // Sort by index (extracted from filename)
        entries.sort_by_key(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .split('-')
                .next()
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(usize::MAX)
        });

        let mut messages = Vec::new();
        for entry in entries {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let json = fs::read_to_string(&path).map_err(|e| {
                    SaorsaAgentError::Session(format!("Failed to read message file: {}", e))
                })?;
                let message: Message = serde_json::from_str(&json).map_err(|e| {
                    SaorsaAgentError::Session(format!("Failed to deserialize message: {}", e))
                })?;
                messages.push(message);
            }
        }

        Ok(messages)
    }
}

impl Default for SessionStorage {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| {
            Self::with_base_path(PathBuf::from("/tmp/saorsa-sessions-fallback"))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn test_storage() -> (TempDir, SessionStorage) {
        let temp_dir = match TempDir::new() {
            Ok(dir) => dir,
            Err(_) => panic!("Failed to create temp dir for test"),
        };
        let storage = SessionStorage::with_base_path(temp_dir.path().to_path_buf());
        (temp_dir, storage)
    }

    #[test]
    fn test_ensure_session_dir_creates_directories() {
        let (_temp, storage) = test_storage();
        let id = SessionId::new();

        assert!(storage.ensure_session_dir(&id).is_ok());

        let session_dir = storage.session_dir(&id);
        assert!(session_dir.exists());
        assert!(session_dir.join("messages").exists());
    }

    #[test]
    fn test_manifest_roundtrip() {
        let (_temp, storage) = test_storage();
        let id = SessionId::new();
        let mut metadata = SessionMetadata::new();
        metadata.title = Some("Test Session".to_string());
        metadata.add_tag("rust".to_string());

        assert!(storage.save_manifest(&id, &metadata).is_ok());
        let loaded = storage.load_manifest(&id);
        assert!(loaded.is_ok());
        match loaded {
            Ok(loaded_meta) => {
                assert!(loaded_meta.title == metadata.title);
                assert!(loaded_meta.tags == metadata.tags);
            }
            Err(_) => unreachable!(),
        }
    }

    #[test]
    fn test_tree_roundtrip() {
        let (_temp, storage) = test_storage();
        let id = SessionId::new();
        let parent_id = SessionId::new();
        let mut node = SessionNode::new_child(id, parent_id);
        node.add_child(SessionId::new());

        assert!(storage.save_tree(&id, &node).is_ok());
        let loaded = storage.load_tree(&id);
        assert!(loaded.is_ok());
        match loaded {
            Ok(loaded_node) => {
                assert!(loaded_node.id == node.id);
                assert!(loaded_node.parent_id == node.parent_id);
                assert!(loaded_node.child_ids.len() == node.child_ids.len());
            }
            Err(_) => unreachable!(),
        }
    }

    #[test]
    fn test_message_serialization() {
        let (_temp, storage) = test_storage();
        let id = SessionId::new();

        let msg1 = Message::user("Hello".to_string());
        let msg2 = Message::assistant("Hi there".to_string());
        let msg3 = Message::tool_call("bash".to_string(), serde_json::json!({"cmd": "ls"}));

        assert!(storage.save_message(&id, 0, &msg1).is_ok());
        assert!(storage.save_message(&id, 1, &msg2).is_ok());
        assert!(storage.save_message(&id, 2, &msg3).is_ok());

        let messages_dir = storage.session_dir(&id).join("messages");
        assert!(messages_dir.join("0-user.json").exists());
        assert!(messages_dir.join("1-assistant.json").exists());
        assert!(messages_dir.join("2-tool_call.json").exists());
    }

    #[test]
    fn test_load_messages_in_order() {
        let (_temp, storage) = test_storage();
        let id = SessionId::new();

        let msg1 = Message::user("First".to_string());
        let msg2 = Message::assistant("Second".to_string());
        let msg3 = Message::user("Third".to_string());

        assert!(storage.save_message(&id, 0, &msg1).is_ok());
        assert!(storage.save_message(&id, 1, &msg2).is_ok());
        assert!(storage.save_message(&id, 2, &msg3).is_ok());

        let loaded = storage.load_messages(&id);
        assert!(loaded.is_ok());
        match loaded {
            Ok(messages) => {
                assert!(messages.len() == 3);

                match &messages[0] {
                    Message::User { content, .. } => assert!(content == "First"),
                    _ => panic!("Expected User message"),
                }

                match &messages[1] {
                    Message::Assistant { content, .. } => assert!(content == "Second"),
                    _ => panic!("Expected Assistant message"),
                }

                match &messages[2] {
                    Message::User { content, .. } => assert!(content == "Third"),
                    _ => panic!("Expected User message"),
                }
            }
            Err(_) => unreachable!(),
        }
    }

    #[test]
    fn test_load_messages_empty_session() {
        let (_temp, storage) = test_storage();
        let id = SessionId::new();

        let messages = storage.load_messages(&id);
        assert!(messages.is_ok());
        match messages {
            Ok(msgs) => assert!(msgs.is_empty()),
            Err(_) => unreachable!(),
        }
    }

    #[test]
    fn test_atomic_write_creates_and_renames() {
        let (_temp, storage) = test_storage();
        let id = SessionId::new();

        assert!(storage.ensure_session_dir(&id).is_ok());
        let path = storage.session_dir(&id).join("test.json");

        assert!(storage.write_atomic(&path, b"test data").is_ok());

        assert!(path.exists());
        assert!(!path.with_extension("tmp").exists());

        let content = fs::read_to_string(&path);
        assert!(content.is_ok());
        match content {
            Ok(c) => assert!(c == "test data"),
            Err(_) => unreachable!(),
        }
    }
}
