//! SYSTEM.md loading and integration.

use crate::error::Result;
use std::path::{Path, PathBuf};

use super::types::SystemMode;

/// Loaded and merged SYSTEM.md context.
#[derive(Debug, Clone, Default)]
pub struct SystemContext {
    /// Merged content ready for system prompt.
    pub content: String,
}

impl SystemContext {
    /// Load and merge SYSTEM.md files from discovered paths.
    ///
    /// Files are processed in precedence order (highest first).
    /// Mode is determined by front matter or defaults to Append.
    pub fn load_and_merge(paths: &[PathBuf]) -> Result<Self> {
        if paths.is_empty() {
            return Ok(Self {
                content: String::new(),
            });
        }

        // Determine mode from first file's front matter
        let mode = parse_system_mode(&paths[0]).unwrap_or_default();

        let content = match mode {
            SystemMode::Replace => {
                // Use only the highest precedence file
                load_file(&paths[0])?
            }
            SystemMode::Append => {
                // Merge all files with separators
                let mut merged = String::new();
                for (i, path) in paths.iter().enumerate() {
                    let file_content = load_file(path)?;
                    merged.push_str(&file_content);
                    // Add separator between files (but not after the last one)
                    if i < paths.len() - 1 {
                        merged.push_str("\n\n---\n\n");
                    }
                }
                merged
            }
        };

        Ok(Self { content })
    }

    /// Combine with default system prompt according to mode.
    ///
    /// If mode is Replace, returns only custom content.
    /// If mode is Append, returns default + custom.
    pub fn apply_to_default(&self, default: &str, mode: SystemMode) -> String {
        match mode {
            SystemMode::Replace => {
                if self.content.is_empty() {
                    default.to_string()
                } else {
                    self.content.clone()
                }
            }
            SystemMode::Append => {
                if self.content.is_empty() {
                    default.to_string()
                } else {
                    format!("{}\n\n{}", default, self.content)
                }
            }
        }
    }
}

/// Load a single file, stripping front matter.
fn load_file(path: &Path) -> Result<String> {
    let content = std::fs::read_to_string(path)?;
    Ok(strip_front_matter(&content))
}

/// Parse system mode from front matter.
///
/// Front matter format:
/// ```
/// ---
/// mode: replace|append
/// ---
/// ```
fn parse_system_mode(path: &Path) -> Option<SystemMode> {
    let content = std::fs::read_to_string(path).ok()?;
    let front_matter = extract_front_matter(&content)?;

    for line in front_matter.lines() {
        let line = line.trim();
        if let Some(value) = line.strip_prefix("mode:") {
            let value = value.trim();
            return match value {
                "replace" => Some(SystemMode::Replace),
                "append" => Some(SystemMode::Append),
                _ => None,
            };
        }
    }
    None
}

/// Extract front matter from content (text between --- delimiters).
fn extract_front_matter(content: &str) -> Option<String> {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return None;
    }

    let after_first = &trimmed[3..];
    after_first
        .find("---")
        .map(|end_pos| after_first[..end_pos].to_string())
}

/// Strip front matter from content, returning only the body.
fn strip_front_matter(content: &str) -> String {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return content.to_string();
    }

    let after_first = &trimmed[3..];
    if let Some(end_pos) = after_first.find("---") {
        let body_start = end_pos + 3; // skip the closing "---"
        after_first[body_start..].trim_start().to_string()
    } else {
        content.to_string()
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

    fn create_file(dir: &Path, name: &str, content: &str) -> PathBuf {
        let path = dir.join(name);
        assert!(fs::write(&path, content).is_ok());
        path
    }

    #[test]
    fn test_load_single_file() {
        let temp = make_temp_dir();
        let path = create_file(temp.path(), "SYSTEM.md", "Custom system prompt");

        let result = SystemContext::load_and_merge(&[path]);
        assert!(result.is_ok());

        let ctx = match result {
            Ok(c) => c,
            Err(_) => unreachable!("load_and_merge should succeed"),
        };
        assert_eq!(ctx.content, "Custom system prompt");
    }

    #[test]
    fn test_append_mode_merges_files() {
        let temp = make_temp_dir();
        let path1 = create_file(temp.path(), "SYSTEM1.md", "First instruction");
        let path2 = create_file(temp.path(), "SYSTEM2.md", "Second instruction");

        let result = SystemContext::load_and_merge(&[path1, path2]);
        assert!(result.is_ok());

        let ctx = match result {
            Ok(c) => c,
            Err(_) => unreachable!("load_and_merge should succeed"),
        };
        assert_eq!(
            ctx.content,
            "First instruction\n\n---\n\nSecond instruction"
        );
    }

    #[test]
    fn test_replace_mode_uses_first_only() {
        let temp = make_temp_dir();
        let content1 = "---\nmode: replace\n---\nFirst instruction";
        let content2 = "Second instruction";

        let path1 = create_file(temp.path(), "SYSTEM1.md", content1);
        let path2 = create_file(temp.path(), "SYSTEM2.md", content2);

        let result = SystemContext::load_and_merge(&[path1, path2]);
        assert!(result.is_ok());

        let ctx = match result {
            Ok(c) => c,
            Err(_) => unreachable!("load_and_merge should succeed"),
        };
        assert_eq!(ctx.content, "First instruction");
    }

    #[test]
    fn test_apply_to_default_append() {
        let ctx = SystemContext {
            content: "Custom addition".to_string(),
        };
        let result = ctx.apply_to_default("Default prompt", SystemMode::Append);
        assert_eq!(result, "Default prompt\n\nCustom addition");
    }

    #[test]
    fn test_apply_to_default_replace() {
        let ctx = SystemContext {
            content: "Completely custom".to_string(),
        };
        let result = ctx.apply_to_default("Default prompt", SystemMode::Replace);
        assert_eq!(result, "Completely custom");
    }

    #[test]
    fn test_apply_to_default_empty_content_append() {
        let ctx = SystemContext {
            content: String::new(),
        };
        let result = ctx.apply_to_default("Default prompt", SystemMode::Append);
        assert_eq!(result, "Default prompt");
    }

    #[test]
    fn test_apply_to_default_empty_content_replace() {
        let ctx = SystemContext {
            content: String::new(),
        };
        let result = ctx.apply_to_default("Default prompt", SystemMode::Replace);
        assert_eq!(result, "Default prompt");
    }

    #[test]
    fn test_front_matter_parsing() {
        let temp = make_temp_dir();
        let content = "---\nmode: replace\n---\nBody content";
        let path = create_file(temp.path(), "SYSTEM.md", content);

        let mode = parse_system_mode(&path);
        assert_eq!(mode, Some(SystemMode::Replace));
    }

    #[test]
    fn test_front_matter_stripping() {
        let content = "---\nmode: append\n---\nBody content";
        let stripped = strip_front_matter(content);
        assert_eq!(stripped, "Body content");
    }

    #[test]
    fn test_no_front_matter() {
        let content = "Body content only";
        let stripped = strip_front_matter(content);
        assert_eq!(stripped, "Body content only");
    }

    #[test]
    fn test_empty_file_list() {
        let result = SystemContext::load_and_merge(&[]);
        assert!(result.is_ok());

        let ctx = match result {
            Ok(c) => c,
            Err(_) => unreachable!("load_and_merge should succeed"),
        };
        assert!(ctx.content.is_empty());
    }

    #[test]
    fn test_file_read_error_propagated() {
        let nonexistent = PathBuf::from("/nonexistent/SYSTEM.md");
        let result = SystemContext::load_and_merge(&[nonexistent]);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_front_matter_valid() {
        let content = "---\nmode: append\nother: value\n---\nBody";
        let fm = extract_front_matter(content);
        assert!(fm.is_some());
        match fm {
            Some(f) => assert!(f.contains("mode: append")),
            None => unreachable!("Should extract front matter"),
        }
    }

    #[test]
    fn test_extract_front_matter_none_when_missing() {
        let content = "No front matter here";
        let fm = extract_front_matter(content);
        assert!(fm.is_none());
    }

    #[test]
    fn test_system_mode_default() {
        assert_eq!(SystemMode::default(), SystemMode::Append);
    }
}
