//! Filesystem path utilities for session storage.

use crate::FaeAgentError;
use crate::session::SessionId;
use std::path::{Path, PathBuf};

/// Get the base directory for session storage.
///
/// Uses XDG Base Directory specification:
/// - `$XDG_DATA_HOME/fae/sessions` if XDG_DATA_HOME is set
/// - `~/.fae/sessions` otherwise
pub fn sessions_dir() -> Result<PathBuf, FaeAgentError> {
    let base = if let Ok(xdg_data) = std::env::var("XDG_DATA_HOME") {
        PathBuf::from(xdg_data).join("fae")
    } else if let Some(home) = std::env::var_os("HOME") {
        PathBuf::from(home).join(".fae")
    } else {
        return Err(FaeAgentError::Session(
            "Cannot determine home directory".to_string(),
        ));
    };

    Ok(base.join("sessions"))
}

/// Get the directory for a specific session.
pub fn session_dir(session_id: &SessionId) -> Result<PathBuf, FaeAgentError> {
    Ok(sessions_dir()?.join(session_id.as_str()))
}

/// Get the path to the manifest file for a session.
pub fn manifest_path(session_id: &SessionId) -> Result<PathBuf, FaeAgentError> {
    Ok(session_dir(session_id)?.join("manifest.json"))
}

/// Get the path to the tree file for a session.
pub fn tree_path(session_id: &SessionId) -> Result<PathBuf, FaeAgentError> {
    Ok(session_dir(session_id)?.join("tree.json"))
}

/// Get the messages directory for a session.
pub fn messages_dir(session_id: &SessionId) -> Result<PathBuf, FaeAgentError> {
    Ok(session_dir(session_id)?.join("messages"))
}

/// Get the path for a specific message file.
///
/// Format: `messages/{index}-{type}.json`
/// Example: `messages/0-user.json`, `messages/1-assistant.json`
pub fn message_path(
    session_id: &SessionId,
    index: usize,
    message_type: &str,
) -> Result<PathBuf, FaeAgentError> {
    Ok(messages_dir(session_id)?.join(format!("{}-{}.json", index, message_type)))
}

/// Ensure a directory exists, creating it if necessary.
pub fn ensure_dir(path: &Path) -> Result<(), FaeAgentError> {
    if !path.exists() {
        std::fs::create_dir_all(path).map_err(|e| {
            FaeAgentError::Session(format!("Failed to create directory {:?}: {}", path, e))
        })?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sessions_dir_uses_xdg() {
        unsafe {
            std::env::set_var("XDG_DATA_HOME", "/tmp/xdg_test");
        }
        let dir = sessions_dir();
        assert!(dir.is_ok());
        match dir {
            Ok(path) => {
                assert!(path.to_string_lossy().contains("xdg_test"));
                assert!(path.ends_with("fae/sessions"));
            }
            Err(_) => unreachable!(),
        }
        unsafe {
            std::env::remove_var("XDG_DATA_HOME");
        }
    }

    #[test]
    fn test_sessions_dir_falls_back_to_home() {
        unsafe {
            std::env::remove_var("XDG_DATA_HOME");
        }
        let dir = sessions_dir();
        assert!(dir.is_ok());
        match dir {
            Ok(path) => {
                assert!(path.to_string_lossy().contains(".fae"));
                assert!(path.ends_with(".fae/sessions"));
            }
            Err(_) => unreachable!(),
        }
    }

    #[test]
    fn test_session_dir_includes_id() {
        let id = SessionId::new();
        let dir = session_dir(&id);
        assert!(dir.is_ok());
        match dir {
            Ok(path) => assert!(path.to_string_lossy().contains(&id.as_str())),
            Err(_) => unreachable!(),
        }
    }

    #[test]
    fn test_manifest_path() {
        let id = SessionId::new();
        let path = manifest_path(&id);
        assert!(path.is_ok());
        match path {
            Ok(p) => {
                assert!(p.ends_with("manifest.json"));
                assert!(p.to_string_lossy().contains(&id.as_str()));
            }
            Err(_) => unreachable!(),
        }
    }

    #[test]
    fn test_tree_path() {
        let id = SessionId::new();
        let path = tree_path(&id);
        assert!(path.is_ok());
        match path {
            Ok(p) => assert!(p.ends_with("tree.json")),
            Err(_) => unreachable!(),
        }
    }

    #[test]
    fn test_messages_dir() {
        let id = SessionId::new();
        let path = messages_dir(&id);
        assert!(path.is_ok());
        match path {
            Ok(p) => assert!(p.ends_with("messages")),
            Err(_) => unreachable!(),
        }
    }

    #[test]
    fn test_message_path_format() {
        let id = SessionId::new();
        let path = message_path(&id, 0, "user");
        assert!(path.is_ok());
        match path {
            Ok(p) => assert!(p.to_string_lossy().ends_with("messages/0-user.json")),
            Err(_) => unreachable!(),
        }

        let path2 = message_path(&id, 42, "assistant");
        assert!(path2.is_ok());
        match path2 {
            Ok(p2) => assert!(p2.to_string_lossy().ends_with("messages/42-assistant.json")),
            Err(_) => unreachable!(),
        }
    }
}
