//! Filesystem path utilities for session storage.

use crate::SaorsaAgentError;
use crate::session::SessionId;
use std::path::{Path, PathBuf};

/// Get the base directory for session storage.
///
/// Uses XDG Base Directory specification:
/// - `$XDG_DATA_HOME/saorsa/sessions` if XDG_DATA_HOME is set
/// - `~/.saorsa/sessions` otherwise
pub fn sessions_dir() -> Result<PathBuf, SaorsaAgentError> {
    let base = if let Ok(xdg_data) = std::env::var("XDG_DATA_HOME") {
        PathBuf::from(xdg_data).join("saorsa")
    } else if let Some(home) = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"))
    {
        PathBuf::from(home).join(".saorsa")
    } else {
        return Err(SaorsaAgentError::Session(
            "Cannot determine home directory".to_string(),
        ));
    };

    Ok(base.join("sessions"))
}

/// Get the directory for a specific session.
pub fn session_dir(session_id: &SessionId) -> Result<PathBuf, SaorsaAgentError> {
    Ok(sessions_dir()?.join(session_id.as_str()))
}

/// Get the path to the manifest file for a session.
pub fn manifest_path(session_id: &SessionId) -> Result<PathBuf, SaorsaAgentError> {
    Ok(session_dir(session_id)?.join("manifest.json"))
}

/// Get the path to the tree file for a session.
pub fn tree_path(session_id: &SessionId) -> Result<PathBuf, SaorsaAgentError> {
    Ok(session_dir(session_id)?.join("tree.json"))
}

/// Get the messages directory for a session.
pub fn messages_dir(session_id: &SessionId) -> Result<PathBuf, SaorsaAgentError> {
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
) -> Result<PathBuf, SaorsaAgentError> {
    Ok(messages_dir(session_id)?.join(format!("{}-{}.json", index, message_type)))
}

/// Ensure a directory exists, creating it if necessary.
pub fn ensure_dir(path: &Path) -> Result<(), SaorsaAgentError> {
    std::fs::create_dir_all(path).map_err(|e| {
        SaorsaAgentError::Session(format!("Failed to create directory {:?}: {}", path, e))
    })?;
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_sessions_dir_uses_xdg() {
        unsafe {
            std::env::set_var("XDG_DATA_HOME", "/tmp/xdg_test");
        }
        let path = sessions_dir().unwrap();
        assert!(path.to_string_lossy().contains("xdg_test"));
        assert!(path.ends_with("saorsa/sessions"));
        unsafe {
            std::env::remove_var("XDG_DATA_HOME");
        }
    }

    #[test]
    fn test_sessions_dir_falls_back_to_home() {
        unsafe {
            std::env::remove_var("XDG_DATA_HOME");
        }
        let path = sessions_dir().unwrap();
        assert!(path.to_string_lossy().contains(".saorsa"));
        assert!(path.ends_with(".saorsa/sessions"));
    }

    #[test]
    fn test_session_dir_includes_id() {
        let id = SessionId::new();
        let path = session_dir(&id).unwrap();
        assert!(path.to_string_lossy().contains(&id.as_str()));
    }

    #[test]
    fn test_manifest_path() {
        let id = SessionId::new();
        let p = manifest_path(&id).unwrap();
        assert!(p.ends_with("manifest.json"));
        assert!(p.to_string_lossy().contains(&id.as_str()));
    }

    #[test]
    fn test_tree_path() {
        let id = SessionId::new();
        let p = tree_path(&id).unwrap();
        assert!(p.ends_with("tree.json"));
    }

    #[test]
    fn test_messages_dir() {
        let id = SessionId::new();
        let p = messages_dir(&id).unwrap();
        assert!(p.ends_with("messages"));
    }

    #[test]
    fn test_message_path_format() {
        let id = SessionId::new();
        let p = message_path(&id, 0, "user").unwrap();
        assert!(p.ends_with(Path::new("messages").join("0-user.json")));

        let p2 = message_path(&id, 42, "assistant").unwrap();
        assert!(p2.ends_with(Path::new("messages").join("42-assistant.json")));
    }
}
