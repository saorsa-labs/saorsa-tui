//! Session continuation and resumption functionality.

use crate::SaorsaAgentError;
use crate::session::{Message, SessionId, SessionMetadata, SessionStorage};
use std::fs;
use std::str::FromStr;

/// Find the most recently active session.
pub fn find_last_active_session(
    storage: &SessionStorage,
) -> Result<Option<SessionId>, SaorsaAgentError> {
    let sessions = list_all_sessions(storage)?;

    if sessions.is_empty() {
        return Ok(None);
    }

    // Find session with most recent last_active timestamp
    let mut most_recent: Option<(SessionId, SessionMetadata)> = None;

    for (id, metadata) in sessions {
        if let Some((_, ref current_meta)) = most_recent {
            if metadata.last_active > current_meta.last_active {
                most_recent = Some((id, metadata));
            }
        } else {
            most_recent = Some((id, metadata));
        }
    }

    Ok(most_recent.map(|(id, _)| id))
}

/// Resume a session by ID prefix (shortest unique match).
///
/// Returns an error if:
/// - No matching sessions found
/// - Multiple sessions match (ambiguous prefix)
pub fn find_session_by_prefix(
    storage: &SessionStorage,
    prefix: &str,
) -> Result<SessionId, SaorsaAgentError> {
    let sessions = list_all_sessions(storage)?;

    let matches: Vec<SessionId> = sessions
        .into_iter()
        .filter(|(id, _)| id.as_str().starts_with(prefix) || id.prefix().starts_with(prefix))
        .map(|(id, _)| id)
        .collect();

    match matches.len() {
        0 => Err(SaorsaAgentError::Session(format!(
            "No session found matching prefix '{}'",
            prefix
        ))),
        1 => Ok(matches[0]),
        _ => Err(SaorsaAgentError::Session(format!(
            "Ambiguous prefix '{}': matches {} sessions",
            prefix,
            matches.len()
        ))),
    }
}

/// List all sessions in storage.
fn list_all_sessions(
    storage: &SessionStorage,
) -> Result<Vec<(SessionId, SessionMetadata)>, SaorsaAgentError> {
    let base_path = storage.base_path();

    if !base_path.exists() {
        return Ok(Vec::new());
    }

    let entries = fs::read_dir(base_path).map_err(|e| {
        SaorsaAgentError::Session(format!("Failed to read sessions directory: {}", e))
    })?;

    let mut sessions = Vec::new();

    for entry in entries {
        let entry = entry.map_err(|e| {
            SaorsaAgentError::Session(format!("Failed to read directory entry: {}", e))
        })?;

        let path = entry.path();
        if path.is_dir()
            && let Some(dir_name) = path.file_name().and_then(|s| s.to_str())
            && let Ok(session_id) = SessionId::from_str(dir_name)
        {
            // Try to load manifest for this session
            if let Ok(metadata) = storage.load_manifest(&session_id) {
                sessions.push((session_id, metadata));
            }
        }
    }

    Ok(sessions)
}

/// Restore a session by loading all its messages.
pub fn restore_session(
    storage: &SessionStorage,
    session_id: &SessionId,
) -> Result<(SessionMetadata, Vec<Message>), SaorsaAgentError> {
    let metadata = storage.load_manifest(session_id)?;
    let messages = storage.load_messages(session_id)?;
    Ok((metadata, messages))
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

    fn create_test_session(storage: &SessionStorage) -> SessionId {
        let id = SessionId::new();
        let metadata = SessionMetadata::new();
        assert!(storage.save_manifest(&id, &metadata).is_ok());
        id
    }

    #[test]
    fn test_find_last_active_empty() {
        let (_temp, storage) = test_storage();
        let result = find_last_active_session(&storage);
        assert!(result.is_ok());
        match result {
            Ok(None) => {}
            Ok(Some(_)) => panic!("Expected None for empty storage"),
            Err(_) => unreachable!(),
        }
    }

    #[test]
    fn test_find_last_active_single_session() {
        let (_temp, storage) = test_storage();
        let id = create_test_session(&storage);

        let result = find_last_active_session(&storage);
        assert!(result.is_ok());
        match result {
            Ok(Some(found_id)) => assert!(found_id == id),
            Ok(None) => panic!("Expected to find session"),
            Err(_) => unreachable!(),
        }
    }

    #[test]
    fn test_find_last_active_multiple_sessions() {
        let (_temp, storage) = test_storage();

        let id1 = SessionId::new();
        let meta1 = SessionMetadata::new();
        assert!(storage.save_manifest(&id1, &meta1).is_ok());

        // Sleep briefly to ensure different timestamps
        std::thread::sleep(std::time::Duration::from_millis(10));

        let id2 = SessionId::new();
        let mut meta2 = SessionMetadata::new();
        meta2.touch(); // Update modified timestamp
        assert!(storage.save_manifest(&id2, &meta2).is_ok());

        let result = find_last_active_session(&storage);
        assert!(result.is_ok());
        match result {
            Ok(Some(found_id)) => assert!(found_id == id2, "Expected most recent session"),
            Ok(None) => panic!("Expected to find session"),
            Err(_) => unreachable!(),
        }
    }

    #[test]
    fn test_find_session_by_full_id() {
        let (_temp, storage) = test_storage();
        let id = create_test_session(&storage);

        let result = find_session_by_prefix(&storage, &id.as_str());
        assert!(result.is_ok());
        match result {
            Ok(found_id) => assert!(found_id == id),
            Err(_) => unreachable!(),
        }
    }

    #[test]
    fn test_find_session_by_short_prefix() {
        let (_temp, storage) = test_storage();
        let id = create_test_session(&storage);
        let prefix = &id.prefix()[..4]; // Use 4-char prefix

        let result = find_session_by_prefix(&storage, prefix);
        assert!(result.is_ok());
        match result {
            Ok(found_id) => assert!(found_id == id),
            Err(_) => unreachable!(),
        }
    }

    #[test]
    fn test_find_session_by_prefix_not_found() {
        let (_temp, storage) = test_storage();
        let _id = create_test_session(&storage);

        let result = find_session_by_prefix(&storage, "zzzzzzzz");
        assert!(result.is_err());
    }

    #[test]
    fn test_find_session_by_prefix_ambiguous() {
        let (_temp, storage) = test_storage();

        // Create multiple sessions - some may share prefixes
        let id1 = create_test_session(&storage);
        let id2 = create_test_session(&storage);

        // Use empty prefix to match all sessions
        let result = find_session_by_prefix(&storage, "");
        // This should be ambiguous if both sessions exist
        if id1 != id2 {
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_restore_session() {
        let (_temp, storage) = test_storage();
        let id = SessionId::new();

        let mut metadata = SessionMetadata::new();
        metadata.title = Some("Test Session".to_string());
        assert!(storage.save_manifest(&id, &metadata).is_ok());

        let msg1 = Message::user("Hello".to_string());
        let msg2 = Message::assistant("Hi there".to_string());
        assert!(storage.save_message(&id, 0, &msg1).is_ok());
        assert!(storage.save_message(&id, 1, &msg2).is_ok());

        let result = restore_session(&storage, &id);
        assert!(result.is_ok());
        match result {
            Ok((restored_meta, restored_messages)) => {
                assert!(restored_meta.title == Some("Test Session".to_string()));
                assert!(restored_messages.len() == 2);
            }
            Err(_) => unreachable!(),
        }
    }

    #[test]
    fn test_restore_session_empty_messages() {
        let (_temp, storage) = test_storage();
        let id = SessionId::new();
        let metadata = SessionMetadata::new();
        assert!(storage.save_manifest(&id, &metadata).is_ok());

        let result = restore_session(&storage, &id);
        assert!(result.is_ok());
        match result {
            Ok((_, messages)) => assert!(messages.is_empty()),
            Err(_) => unreachable!(),
        }
    }
}
