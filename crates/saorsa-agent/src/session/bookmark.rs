//! Session bookmarking for quick access.

use crate::SaorsaAgentError;
use crate::session::SessionId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// A bookmark mapping a name to a session ID.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    /// Bookmark name
    pub name: String,
    /// Session ID
    pub session_id: SessionId,
    /// When the bookmark was created
    pub created: chrono::DateTime<chrono::Utc>,
}

/// Bookmark manager for persisting bookmarks.
#[derive(Debug, Clone)]
pub struct BookmarkManager {
    bookmarks_path: PathBuf,
}

impl BookmarkManager {
    /// Create a new bookmark manager with the default path.
    pub fn new() -> Result<Self, SaorsaAgentError> {
        let base = crate::session::path::sessions_dir()?;
        let bookmarks_path = base
            .parent()
            .ok_or_else(|| SaorsaAgentError::Session("Invalid sessions directory".to_string()))?
            .join("bookmarks.json");
        Ok(Self { bookmarks_path })
    }

    /// Create a bookmark manager with a custom path (for testing).
    pub fn with_path(path: PathBuf) -> Self {
        Self {
            bookmarks_path: path,
        }
    }

    /// Load all bookmarks from disk.
    fn load_bookmarks(&self) -> Result<HashMap<String, Bookmark>, SaorsaAgentError> {
        if !self.bookmarks_path.exists() {
            return Ok(HashMap::new());
        }

        let json = fs::read_to_string(&self.bookmarks_path).map_err(|e| {
            SaorsaAgentError::Session(format!("Failed to read bookmarks file: {}", e))
        })?;

        serde_json::from_str(&json)
            .map_err(|e| SaorsaAgentError::Session(format!("Failed to parse bookmarks: {}", e)))
    }

    /// Save bookmarks to disk.
    fn save_bookmarks(
        &self,
        bookmarks: &HashMap<String, Bookmark>,
    ) -> Result<(), SaorsaAgentError> {
        // Ensure parent directory exists
        if let Some(parent) = self.bookmarks_path.parent() {
            crate::session::path::ensure_dir(parent)?;
        }

        let json = serde_json::to_string_pretty(bookmarks).map_err(|e| {
            SaorsaAgentError::Session(format!("Failed to serialize bookmarks: {}", e))
        })?;

        fs::write(&self.bookmarks_path, json).map_err(|e| {
            SaorsaAgentError::Session(format!("Failed to write bookmarks file: {}", e))
        })?;

        Ok(())
    }

    /// Add or update a bookmark.
    pub fn add_bookmark(
        &self,
        name: String,
        session_id: SessionId,
    ) -> Result<(), SaorsaAgentError> {
        let mut bookmarks = self.load_bookmarks()?;

        bookmarks.insert(
            name.clone(),
            Bookmark {
                name,
                session_id,
                created: chrono::Utc::now(),
            },
        );

        self.save_bookmarks(&bookmarks)?;
        Ok(())
    }

    /// Remove a bookmark.
    pub fn remove_bookmark(&self, name: &str) -> Result<bool, SaorsaAgentError> {
        let mut bookmarks = self.load_bookmarks()?;

        let removed = bookmarks.remove(name).is_some();
        if removed {
            self.save_bookmarks(&bookmarks)?;
        }

        Ok(removed)
    }

    /// Rename a bookmark.
    pub fn rename_bookmark(
        &self,
        old_name: &str,
        new_name: String,
    ) -> Result<(), SaorsaAgentError> {
        let mut bookmarks = self.load_bookmarks()?;

        let bookmark = bookmarks.remove(old_name).ok_or_else(|| {
            SaorsaAgentError::Session(format!("Bookmark '{}' not found", old_name))
        })?;

        bookmarks.insert(
            new_name.clone(),
            Bookmark {
                name: new_name,
                ..bookmark
            },
        );

        self.save_bookmarks(&bookmarks)?;
        Ok(())
    }

    /// Get a bookmark by name.
    pub fn get_bookmark(&self, name: &str) -> Result<Option<Bookmark>, SaorsaAgentError> {
        let bookmarks = self.load_bookmarks()?;
        Ok(bookmarks.get(name).cloned())
    }

    /// List all bookmarks, sorted by name.
    pub fn list_bookmarks(&self) -> Result<Vec<Bookmark>, SaorsaAgentError> {
        let bookmarks = self.load_bookmarks()?;
        let mut list: Vec<Bookmark> = bookmarks.into_values().collect();
        list.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(list)
    }

    /// Generate a unique auto-bookmark name.
    pub fn generate_auto_name(&self) -> Result<String, SaorsaAgentError> {
        let bookmarks = self.load_bookmarks()?;
        let mut counter = 1;

        loop {
            let name = format!("bookmark-{}", counter);
            if !bookmarks.contains_key(&name) {
                return Ok(name);
            }
            counter += 1;
            if counter > 10000 {
                return Err(SaorsaAgentError::Session(
                    "Could not generate unique bookmark name".to_string(),
                ));
            }
        }
    }
}

impl Default for BookmarkManager {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self::with_path(PathBuf::from("/tmp/saorsa-bookmarks.json")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn test_manager() -> (TempDir, BookmarkManager) {
        let temp_dir = match TempDir::new() {
            Ok(dir) => dir,
            Err(_) => panic!("Failed to create temp dir for test"),
        };
        let path = temp_dir.path().join("bookmarks.json");
        let manager = BookmarkManager::with_path(path);
        (temp_dir, manager)
    }

    #[test]
    fn test_add_and_get_bookmark() {
        let (_temp, manager) = test_manager();
        let session_id = SessionId::new();

        assert!(manager.add_bookmark("test".to_string(), session_id).is_ok());

        let result = manager.get_bookmark("test");
        assert!(result.is_ok());
        match result {
            Ok(Some(bookmark)) => {
                assert!(bookmark.name == "test");
                assert!(bookmark.session_id == session_id);
            }
            Ok(None) => panic!("Expected bookmark to exist"),
            Err(_) => unreachable!(),
        }
    }

    #[test]
    fn test_remove_bookmark() {
        let (_temp, manager) = test_manager();
        let session_id = SessionId::new();

        assert!(
            manager
                .add_bookmark("remove-me".to_string(), session_id)
                .is_ok()
        );

        let removed = manager.remove_bookmark("remove-me");
        assert!(removed.is_ok());
        match removed {
            Ok(true) => {}
            Ok(false) => panic!("Expected bookmark to be removed"),
            Err(_) => unreachable!(),
        }

        let result = manager.get_bookmark("remove-me");
        assert!(result.is_ok());
        match result {
            Ok(None) => {}
            Ok(Some(_)) => panic!("Expected bookmark to not exist"),
            Err(_) => unreachable!(),
        }
    }

    #[test]
    fn test_rename_bookmark() {
        let (_temp, manager) = test_manager();
        let session_id = SessionId::new();

        assert!(
            manager
                .add_bookmark("old-name".to_string(), session_id)
                .is_ok()
        );

        let result = manager.rename_bookmark("old-name", "new-name".to_string());
        assert!(result.is_ok());

        // Old name should not exist
        let old = manager.get_bookmark("old-name");
        assert!(old.is_ok());
        match old {
            Ok(None) => {}
            Ok(Some(_)) => panic!("Old bookmark should not exist"),
            Err(_) => unreachable!(),
        }

        // New name should exist with same session ID
        let new = manager.get_bookmark("new-name");
        assert!(new.is_ok());
        match new {
            Ok(Some(bookmark)) => {
                assert!(bookmark.session_id == session_id);
            }
            Ok(None) => panic!("New bookmark should exist"),
            Err(_) => unreachable!(),
        }
    }

    #[test]
    fn test_list_bookmarks_sorted() {
        let (_temp, manager) = test_manager();

        let id1 = SessionId::new();
        let id2 = SessionId::new();
        let id3 = SessionId::new();

        assert!(manager.add_bookmark("zebra".to_string(), id1).is_ok());
        assert!(manager.add_bookmark("alpha".to_string(), id2).is_ok());
        assert!(manager.add_bookmark("beta".to_string(), id3).is_ok());

        let list = manager.list_bookmarks();
        assert!(list.is_ok());
        match list {
            Ok(bookmarks) => {
                assert!(bookmarks.len() == 3);
                assert!(bookmarks[0].name == "alpha");
                assert!(bookmarks[1].name == "beta");
                assert!(bookmarks[2].name == "zebra");
            }
            Err(_) => unreachable!(),
        }
    }

    #[test]
    fn test_generate_auto_name() {
        let (_temp, manager) = test_manager();

        let name1 = manager.generate_auto_name();
        assert!(name1.is_ok());
        match name1 {
            Ok(name) => {
                assert!(name == "bookmark-1");

                // Add it
                assert!(manager.add_bookmark(name, SessionId::new()).is_ok());

                // Generate next
                let name2 = manager.generate_auto_name();
                assert!(name2.is_ok());
                match name2 {
                    Ok(n) => assert!(n == "bookmark-2"),
                    Err(_) => unreachable!(),
                }
            }
            Err(_) => unreachable!(),
        }
    }
}
