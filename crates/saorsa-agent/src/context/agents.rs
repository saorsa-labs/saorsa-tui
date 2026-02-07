//! AGENTS.md loading and merging.

use crate::error::Result;
use std::path::{Path, PathBuf};

use super::types::MergeStrategy;

/// Loaded and merged AGENTS.md context.
#[derive(Debug, Clone, Default)]
pub struct AgentsContext {
    /// Merged content ready for LLM context.
    pub content: String,
}

impl AgentsContext {
    /// Load and merge AGENTS.md files from discovered paths.
    ///
    /// Files are processed in precedence order (highest first).
    /// Merge strategy is determined by front matter or defaults to Append.
    pub fn load_and_merge(paths: &[PathBuf]) -> Result<Self> {
        if paths.is_empty() {
            return Ok(Self {
                content: String::new(),
            });
        }

        // Determine merge strategy from first file's front matter
        let strategy = parse_merge_strategy(&paths[0]).unwrap_or_default();

        let content = match strategy {
            MergeStrategy::Replace => {
                // Use only the highest precedence file
                load_file(&paths[0])?
            }
            MergeStrategy::Append => {
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
}

/// Load a single file, stripping front matter.
fn load_file(path: &Path) -> Result<String> {
    let content = std::fs::read_to_string(path)?;
    Ok(strip_front_matter(&content))
}

/// Parse merge strategy from front matter.
///
/// Front matter format:
/// ```text
/// ---
/// merge: replace|append
/// ---
/// ```
fn parse_merge_strategy(path: &Path) -> Option<MergeStrategy> {
    let content = std::fs::read_to_string(path).ok()?;
    let front_matter = extract_front_matter(&content)?;

    for line in front_matter.lines() {
        let line = line.trim();
        if let Some(value) = line.strip_prefix("merge:") {
            let value = value.trim();
            return match value {
                "replace" => Some(MergeStrategy::Replace),
                "append" => Some(MergeStrategy::Append),
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
        let path = create_file(temp.path(), "AGENTS.md", "Test content");

        let result = AgentsContext::load_and_merge(&[path]);
        assert!(result.is_ok());

        let ctx = match result {
            Ok(c) => c,
            Err(_) => unreachable!("load_and_merge should succeed"),
        };
        assert_eq!(ctx.content, "Test content");
    }

    #[test]
    fn test_append_strategy_merges_files() {
        let temp = make_temp_dir();
        let path1 = create_file(temp.path(), "AGENTS1.md", "First file");
        let path2 = create_file(temp.path(), "AGENTS2.md", "Second file");

        let result = AgentsContext::load_and_merge(&[path1, path2]);
        assert!(result.is_ok());

        let ctx = match result {
            Ok(c) => c,
            Err(_) => unreachable!("load_and_merge should succeed"),
        };
        assert_eq!(ctx.content, "First file\n\n---\n\nSecond file");
    }

    #[test]
    fn test_replace_strategy_uses_first_only() {
        let temp = make_temp_dir();
        let content1 = "---\nmerge: replace\n---\nFirst file";
        let content2 = "Second file";

        let path1 = create_file(temp.path(), "AGENTS1.md", content1);
        let path2 = create_file(temp.path(), "AGENTS2.md", content2);

        let result = AgentsContext::load_and_merge(&[path1, path2]);
        assert!(result.is_ok());

        let ctx = match result {
            Ok(c) => c,
            Err(_) => unreachable!("load_and_merge should succeed"),
        };
        assert_eq!(ctx.content, "First file");
    }

    #[test]
    fn test_front_matter_parsing() {
        let temp = make_temp_dir();
        let content = "---\nmerge: replace\n---\nBody content";
        let path = create_file(temp.path(), "AGENTS.md", content);

        let strategy = parse_merge_strategy(&path);
        assert_eq!(strategy, Some(MergeStrategy::Replace));
    }

    #[test]
    fn test_front_matter_stripping() {
        let content = "---\nmerge: append\n---\nBody content";
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
        let result = AgentsContext::load_and_merge(&[]);
        assert!(result.is_ok());

        let ctx = match result {
            Ok(c) => c,
            Err(_) => unreachable!("load_and_merge should succeed"),
        };
        assert!(ctx.content.is_empty());
    }

    #[test]
    fn test_file_read_error_propagated() {
        let nonexistent = PathBuf::from("/nonexistent/AGENTS.md");
        let result = AgentsContext::load_and_merge(&[nonexistent]);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_front_matter_valid() {
        let content = "---\nmerge: append\nother: value\n---\nBody";
        let fm = extract_front_matter(content);
        assert!(fm.is_some());
        match fm {
            Some(f) => assert!(f.contains("merge: append")),
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
    fn test_merge_strategy_default() {
        assert_eq!(MergeStrategy::default(), MergeStrategy::Append);
    }
}
