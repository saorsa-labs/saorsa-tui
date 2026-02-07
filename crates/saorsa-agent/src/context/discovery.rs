//! Context file discovery system for AGENTS.md and SYSTEM.md files.
//!
//! Discovers context files across multiple locations with precedence:
//! 1. Current working directory (highest precedence)
//! 2. Parent directories (walking up to root or home)
//! 3. Global location (~/.saorsa-tui/, lowest precedence)

use std::path::{Path, PathBuf};

/// Discovers context files (AGENTS.md, SYSTEM.md) across multiple locations.
#[derive(Debug, Clone)]
pub struct ContextDiscovery {
    /// Current working directory to start search from
    cwd: PathBuf,
}

impl ContextDiscovery {
    /// Create a new context discovery instance from the current working directory.
    pub fn new() -> Result<Self, std::io::Error> {
        let cwd = std::env::current_dir()?;
        Ok(Self { cwd })
    }

    /// Create a context discovery instance from a specific directory.
    pub fn from_dir(cwd: PathBuf) -> Self {
        Self { cwd }
    }

    /// Discover AGENTS.md files across all locations.
    ///
    /// Returns paths ordered by precedence (highest first):
    /// - CWD/AGENTS.md
    /// - Parent directories (walking up)
    /// - ~/.saorsa-tui/AGENTS.md
    pub fn discover_agents_md(&self) -> Vec<PathBuf> {
        self.discover_file("AGENTS.md")
    }

    /// Discover SYSTEM.md files across all locations.
    ///
    /// Returns paths ordered by precedence (highest first):
    /// - CWD/SYSTEM.md
    /// - Parent directories (walking up)
    /// - ~/.saorsa-tui/SYSTEM.md
    pub fn discover_system_md(&self) -> Vec<PathBuf> {
        self.discover_file("SYSTEM.md")
    }

    /// Generic file discovery with precedence ordering.
    fn discover_file(&self, filename: &str) -> Vec<PathBuf> {
        let mut found = Vec::new();

        // 1. Check CWD
        let cwd_file = self.cwd.join(filename);
        if cwd_file.exists() {
            found.push(cwd_file);
        }

        // 2. Walk parent directories up to root or home
        let mut current = self.cwd.as_path();
        let home = dirs::home_dir();

        while let Some(parent) = current.parent() {
            // Stop if we've reached the home directory (we'll check global location separately)
            if let Some(ref home_path) = home
                && parent == home_path
            {
                break;
            }

            // Stop if we've reached the root
            if parent == Path::new("/") {
                break;
            }

            let parent_file = parent.join(filename);
            if parent_file.exists() {
                found.push(parent_file);
            }

            current = parent;
        }

        // 3. Check global location (~/.saorsa-tui/)
        if let Some(home_path) = home {
            let global_file = home_path.join(".saorsa-tui").join(filename);
            if global_file.exists() {
                found.push(global_file);
            }
        }

        found
    }
}

impl Default for ContextDiscovery {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            cwd: PathBuf::from("."),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn make_temp_dir() -> TempDir {
        match TempDir::new() {
            Ok(t) => t,
            Err(e) => unreachable!("Failed to create temp dir: {e}"),
        }
    }

    fn create_file(dir: &Path, name: &str) -> PathBuf {
        let path = dir.join(name);
        assert!(fs::write(&path, "test content").is_ok());
        path
    }

    #[test]
    fn test_discovery_finds_cwd_file() {
        let temp = make_temp_dir();
        let cwd = temp.path().to_path_buf();

        create_file(&cwd, "AGENTS.md");

        let discovery = ContextDiscovery::from_dir(cwd.clone());
        let found = discovery.discover_agents_md();

        assert_eq!(found.len(), 1);
        assert_eq!(found[0], cwd.join("AGENTS.md"));
    }

    #[test]
    fn test_discovery_empty_when_no_files() {
        let temp = make_temp_dir();
        let cwd = temp.path().to_path_buf();

        let discovery = ContextDiscovery::from_dir(cwd);
        let found = discovery.discover_agents_md();

        assert!(found.is_empty());
    }

    #[test]
    fn test_discovery_walks_parent_directories() {
        let temp = make_temp_dir();
        let parent = temp.path().to_path_buf();
        let child = parent.join("child");
        let grandchild = child.join("grandchild");

        assert!(fs::create_dir_all(&grandchild).is_ok());

        create_file(&parent, "AGENTS.md");
        create_file(&child, "AGENTS.md");

        let discovery = ContextDiscovery::from_dir(grandchild.clone());
        let found = discovery.discover_agents_md();

        // Should find child first (higher precedence), then parent
        assert_eq!(found.len(), 2);
        assert_eq!(found[0], child.join("AGENTS.md"));
        assert_eq!(found[1], parent.join("AGENTS.md"));
    }

    #[test]
    fn test_precedence_ordering_cwd_over_parent() {
        let temp = make_temp_dir();
        let parent = temp.path().to_path_buf();
        let child = parent.join("child");

        assert!(fs::create_dir(&child).is_ok());

        create_file(&parent, "AGENTS.md");
        create_file(&child, "AGENTS.md");

        let discovery = ContextDiscovery::from_dir(child.clone());
        let found = discovery.discover_agents_md();

        // CWD should be first (highest precedence)
        assert_eq!(found.len(), 2);
        assert_eq!(found[0], child.join("AGENTS.md"));
        assert_eq!(found[1], parent.join("AGENTS.md"));
    }

    #[test]
    fn test_system_md_discovery_works_same_way() {
        let temp = make_temp_dir();
        let cwd = temp.path().to_path_buf();

        create_file(&cwd, "SYSTEM.md");

        let discovery = ContextDiscovery::from_dir(cwd.clone());
        let found = discovery.discover_system_md();

        assert_eq!(found.len(), 1);
        assert_eq!(found[0], cwd.join("SYSTEM.md"));
    }

    #[test]
    fn test_discovery_filters_nonexistent_paths() {
        let temp = make_temp_dir();
        let parent = temp.path().to_path_buf();
        let child = parent.join("child");

        assert!(fs::create_dir(&child).is_ok());

        // Only create file in parent, not child
        create_file(&parent, "AGENTS.md");

        let discovery = ContextDiscovery::from_dir(child);
        let found = discovery.discover_agents_md();

        // Should only find parent file, not nonexistent child file
        assert_eq!(found.len(), 1);
        assert_eq!(found[0], parent.join("AGENTS.md"));
    }

    #[test]
    fn test_new_uses_current_dir() {
        // This test validates that ContextDiscovery::new() succeeds
        let result = ContextDiscovery::new();
        assert!(result.is_ok());

        match result {
            Ok(discovery) => {
                assert!(discovery.cwd.is_absolute() || discovery.cwd == *".");
            }
            Err(_) => unreachable!("new() should succeed"),
        }
    }

    #[test]
    fn test_default_creates_valid_instance() {
        let discovery = ContextDiscovery::default();
        // Should not panic and should have a valid cwd
        assert!(!discovery.cwd.as_os_str().is_empty());
    }
}
