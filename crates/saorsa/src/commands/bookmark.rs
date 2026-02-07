//! Bookmark commands for session quick access.

use saorsa_agent::{Bookmark, BookmarkManager, SaorsaAgentError, SessionId};

/// Bookmark command handlers.
pub struct BookmarkCommand;

impl BookmarkCommand {
    /// Add a bookmark.
    pub fn add(session_id: SessionId, name: Option<String>) -> Result<String, SaorsaAgentError> {
        let manager = BookmarkManager::new()?;
        let bookmark_name = if let Some(n) = name {
            n
        } else {
            manager.generate_auto_name()?
        };

        manager.add_bookmark(bookmark_name.clone(), session_id)?;
        Ok(bookmark_name)
    }

    /// Remove a bookmark.
    pub fn remove(name: &str) -> Result<bool, SaorsaAgentError> {
        let manager = BookmarkManager::new()?;
        manager.remove_bookmark(name)
    }

    /// Rename a bookmark.
    pub fn rename(old_name: &str, new_name: String) -> Result<(), SaorsaAgentError> {
        let manager = BookmarkManager::new()?;
        manager.rename_bookmark(old_name, new_name)
    }

    /// List all bookmarks.
    pub fn list() -> Result<Vec<Bookmark>, SaorsaAgentError> {
        let manager = BookmarkManager::new()?;
        manager.list_bookmarks()
    }

    /// Jump to a bookmarked session.
    pub fn jump(name: &str) -> Result<Option<SessionId>, SaorsaAgentError> {
        let manager = BookmarkManager::new()?;
        Ok(manager.get_bookmark(name)?.map(|b| b.session_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bookmark_command_exists() {
        let _ = BookmarkCommand;
    }
}
