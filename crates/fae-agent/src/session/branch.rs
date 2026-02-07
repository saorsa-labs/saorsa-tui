//! Session branching and forking functionality.

use crate::FaeAgentError;
use crate::session::{Message, SessionId, SessionMetadata, SessionNode, SessionStorage};

/// Fork a session at a specific point, creating a new branch.
///
/// This creates a new session with:
/// - A new unique session ID
/// - A copy of all messages up to the fork point
/// - A parent-child relationship in the tree
/// - Optional custom title
pub fn fork_session(
    storage: &SessionStorage,
    parent_id: &SessionId,
    fork_point: Option<usize>,
    title: Option<String>,
) -> Result<SessionId, FaeAgentError> {
    // Load parent session data
    let parent_metadata = storage.load_manifest(parent_id)?;
    let parent_messages = storage.load_messages(parent_id)?;
    let mut parent_node = storage
        .load_tree(parent_id)
        .unwrap_or_else(|_| SessionNode::new_root(*parent_id));

    // Determine fork point
    let fork_index = fork_point.unwrap_or(parent_messages.len());
    if fork_index > parent_messages.len() {
        return Err(FaeAgentError::Session(format!(
            "Fork point {} exceeds message count {}",
            fork_index,
            parent_messages.len()
        )));
    }

    // Create new session
    let new_id = SessionId::new();
    let mut new_metadata = SessionMetadata::new();
    new_metadata.title = title.or_else(|| {
        Some(format!(
            "Fork of {} at message {}",
            parent_metadata
                .title
                .as_deref()
                .unwrap_or("untitled"),
            fork_index
        ))
    });

    // Copy messages up to fork point
    let forked_messages: Vec<Message> = parent_messages.into_iter().take(fork_index).collect();

    // Create tree node with parent relationship
    let new_node = SessionNode::new_child(new_id, *parent_id);

    // Update parent node to include this child
    parent_node.add_child(new_id);

    // Save everything
    storage.save_manifest(&new_id, &new_metadata)?;
    storage.save_tree(&new_id, &new_node)?;
    for (i, msg) in forked_messages.iter().enumerate() {
        storage.save_message(&new_id, i, msg)?;
    }

    // Update parent tree
    storage.save_tree(parent_id, &parent_node)?;

    Ok(new_id)
}

/// Auto-fork on message edit.
///
/// When a user edits a message in the middle of a conversation,
/// create a fork at that point to preserve the original branch.
pub fn auto_fork_on_edit(
    storage: &SessionStorage,
    session_id: &SessionId,
    edit_index: usize,
) -> Result<SessionId, FaeAgentError> {
    fork_session(
        storage,
        session_id,
        Some(edit_index),
        Some("Auto-fork on edit".to_string()),
    )
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
    fn test_fork_creates_new_session() {
        let (_temp, storage) = test_storage();

        // Create parent session
        let parent_id = SessionId::new();
        let parent_meta = SessionMetadata::new();
        let parent_node = SessionNode::new_root(parent_id);

        assert!(storage.save_manifest(&parent_id, &parent_meta).is_ok());
        assert!(storage.save_tree(&parent_id, &parent_node).is_ok());

        // Add some messages
        assert!(storage
            .save_message(&parent_id, 0, &Message::user("Message 1".to_string()))
            .is_ok());
        assert!(storage
            .save_message(&parent_id, 1, &Message::user("Message 2".to_string()))
            .is_ok());

        // Fork at message 1
        let result = fork_session(&storage, &parent_id, Some(1), Some("Forked".to_string()));
        assert!(result.is_ok());

        match result {
            Ok(new_id) => {
                assert!(new_id != parent_id);

                // Verify new session exists
                let new_meta = storage.load_manifest(&new_id);
                assert!(new_meta.is_ok());

                match new_meta {
                    Ok(meta) => {
                        assert!(meta.title == Some("Forked".to_string()));
                    }
                    Err(_) => unreachable!(),
                }

                // Verify messages copied
                let messages = storage.load_messages(&new_id);
                assert!(messages.is_ok());
                match messages {
                    Ok(msgs) => {
                        assert!(msgs.len() == 1); // Only first message
                    }
                    Err(_) => unreachable!(),
                }
            }
            Err(_) => unreachable!(),
        }
    }

    #[test]
    fn test_fork_parent_child_relationship() {
        let (_temp, storage) = test_storage();

        let parent_id = SessionId::new();
        let parent_meta = SessionMetadata::new();
        let parent_node = SessionNode::new_root(parent_id);

        assert!(storage.save_manifest(&parent_id, &parent_meta).is_ok());
        assert!(storage.save_tree(&parent_id, &parent_node).is_ok());

        assert!(storage
            .save_message(&parent_id, 0, &Message::user("Test".to_string()))
            .is_ok());

        let result = fork_session(&storage, &parent_id, None, None);
        assert!(result.is_ok());

        match result {
            Ok(child_id) => {
                // Check child node has parent
                let child_node = storage.load_tree(&child_id);
                assert!(child_node.is_ok());
                match child_node {
                    Ok(node) => {
                        assert!(node.parent_id == Some(parent_id));
                    }
                    Err(_) => unreachable!(),
                }

                // Check parent node has child
                let updated_parent = storage.load_tree(&parent_id);
                assert!(updated_parent.is_ok());
                match updated_parent {
                    Ok(node) => {
                        assert!(node.child_ids.contains(&child_id));
                    }
                    Err(_) => unreachable!(),
                }
            }
            Err(_) => unreachable!(),
        }
    }

    #[test]
    fn test_multiple_forks_from_same_parent() {
        let (_temp, storage) = test_storage();

        let parent_id = SessionId::new();
        let parent_meta = SessionMetadata::new();
        let parent_node = SessionNode::new_root(parent_id);

        assert!(storage.save_manifest(&parent_id, &parent_meta).is_ok());
        assert!(storage.save_tree(&parent_id, &parent_node).is_ok());
        assert!(storage
            .save_message(&parent_id, 0, &Message::user("Test".to_string()))
            .is_ok());

        // Create two forks
        let fork1 = fork_session(&storage, &parent_id, None, Some("Fork 1".to_string()));
        let fork2 = fork_session(&storage, &parent_id, None, Some("Fork 2".to_string()));

        assert!(fork1.is_ok());
        assert!(fork2.is_ok());

        match (fork1, fork2) {
            (Ok(f1), Ok(f2)) => {
                assert!(f1 != f2);

                // Parent should have both children
                let parent_node = storage.load_tree(&parent_id);
                assert!(parent_node.is_ok());
                match parent_node {
                    Ok(node) => {
                        assert!(node.child_ids.len() == 2);
                        assert!(node.child_ids.contains(&f1));
                        assert!(node.child_ids.contains(&f2));
                    }
                    Err(_) => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn test_auto_fork_on_edit() {
        let (_temp, storage) = test_storage();

        let session_id = SessionId::new();
        let metadata = SessionMetadata::new();
        let node = SessionNode::new_root(session_id);

        assert!(storage.save_manifest(&session_id, &metadata).is_ok());
        assert!(storage.save_tree(&session_id, &node).is_ok());
        assert!(storage
            .save_message(&session_id, 0, &Message::user("Msg 1".to_string()))
            .is_ok());
        assert!(storage
            .save_message(&session_id, 1, &Message::user("Msg 2".to_string()))
            .is_ok());
        assert!(storage
            .save_message(&session_id, 2, &Message::user("Msg 3".to_string()))
            .is_ok());

        // Auto-fork at message 1 (editing second message)
        let result = auto_fork_on_edit(&storage, &session_id, 1);
        assert!(result.is_ok());

        match result {
            Ok(new_id) => {
                let messages = storage.load_messages(&new_id);
                assert!(messages.is_ok());
                match messages {
                    Ok(msgs) => {
                        assert!(msgs.len() == 1); // Only first message before edit point
                    }
                    Err(_) => unreachable!(),
                }
            }
            Err(_) => unreachable!(),
        }
    }

    #[test]
    fn test_fork_point_out_of_bounds() {
        let (_temp, storage) = test_storage();

        let parent_id = SessionId::new();
        let parent_meta = SessionMetadata::new();
        let parent_node = SessionNode::new_root(parent_id);

        assert!(storage.save_manifest(&parent_id, &parent_meta).is_ok());
        assert!(storage.save_tree(&parent_id, &parent_node).is_ok());
        assert!(storage
            .save_message(&parent_id, 0, &Message::user("Test".to_string()))
            .is_ok());

        // Try to fork at index 10 (out of bounds)
        let result = fork_session(&storage, &parent_id, Some(10), None);
        assert!(result.is_err());
    }
}
