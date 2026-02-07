//! Find tool for locating files by name pattern.

use std::path::{Path, PathBuf};

use globset::{Glob, GlobMatcher};
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::error::{FaeAgentError, Result};
use crate::tool::Tool;

/// Maximum number of results to return (prevents overwhelming output).
const MAX_RESULTS: usize = 100;

/// Tool for finding files by name pattern.
pub struct FindTool {
    /// Base directory for resolving relative paths.
    working_dir: PathBuf,
}

/// Input parameters for the Find tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct FindInput {
    /// Glob pattern to match file names (e.g., "*.rs", "test_*.txt").
    pattern: String,
    /// Directory to search (default: current working directory).
    #[serde(default)]
    path: Option<String>,
}

impl FindTool {
    /// Create a new Find tool with the given working directory.
    pub fn new(working_dir: impl Into<PathBuf>) -> Self {
        Self {
            working_dir: working_dir.into(),
        }
    }

    /// Resolve a file path relative to the working directory.
    fn resolve_path(&self, path: Option<&str>) -> PathBuf {
        match path {
            Some(p) => {
                let path = Path::new(p);
                if path.is_absolute() {
                    path.to_path_buf()
                } else {
                    self.working_dir.join(path)
                }
            }
            None => self.working_dir.clone(),
        }
    }
}

#[async_trait::async_trait]
impl Tool for FindTool {
    fn name(&self) -> &str {
        "find"
    }

    fn description(&self) -> &str {
        "Find files by name pattern using glob syntax (*, ?, \\[abc\\], etc.)"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "Glob pattern to match file names (e.g., '*.rs', 'test_*.txt', 'file[0-9].log')"
                },
                "path": {
                    "type": "string",
                    "description": "Directory to search (default: current working directory)"
                }
            },
            "required": ["pattern"]
        })
    }

    async fn execute(&self, input: serde_json::Value) -> Result<String> {
        let input: FindInput = serde_json::from_value(input)
            .map_err(|e| FaeAgentError::Tool(format!("Invalid input: {e}")))?;

        let search_path = self.resolve_path(input.path.as_deref());

        // Check if search path exists
        if !search_path.exists() {
            return Err(FaeAgentError::Tool(format!(
                "Path not found: {}",
                search_path.display()
            )));
        }

        // Check if search path is a directory
        if !search_path.is_dir() {
            return Err(FaeAgentError::Tool(format!(
                "Path is not a directory: {}",
                search_path.display()
            )));
        }

        // Compile glob pattern
        let glob = Glob::new(&input.pattern)
            .map_err(|e| FaeAgentError::Tool(format!("Invalid glob pattern: {e}")))?;
        let matcher: GlobMatcher = glob.compile_matcher();

        let mut matches = Vec::new();

        // Walk directory tree
        for entry in WalkDir::new(&search_path)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if matches.len() >= MAX_RESULTS {
                break;
            }

            if entry.file_type().is_file()
                && let Some(file_name) = entry.file_name().to_str()
                && matcher.is_match(file_name)
            {
                matches.push(entry.path().display().to_string());
            }
        }

        // Build response
        if matches.is_empty() {
            Ok(format!(
                "No files found matching pattern: '{}'",
                input.pattern
            ))
        } else {
            let truncated = if matches.len() >= MAX_RESULTS {
                format!("\n\n(Results limited to {} files)", MAX_RESULTS)
            } else {
                String::new()
            };

            Ok(format!(
                "Found {} file(s):\n\n{}{}",
                matches.len(),
                matches.join("\n"),
                truncated
            ))
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn find_simple_pattern() {
        let temp_dir = TempDir::new().unwrap();
        let file1 = temp_dir.path().join("test.txt");
        let file2 = temp_dir.path().join("test.rs");
        let file3 = temp_dir.path().join("other.txt");

        fs::write(&file1, "content").unwrap();
        fs::write(&file2, "content").unwrap();
        fs::write(&file3, "content").unwrap();

        let tool = FindTool::new(std::env::current_dir().unwrap());
        let input = serde_json::json!({
            "pattern": "*.txt",
            "path": temp_dir.path().to_str().unwrap()
        });

        let result = tool.execute(input).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.contains("Found 2 file(s)"));
        assert!(response.contains("test.txt"));
        assert!(response.contains("other.txt"));
        assert!(!response.contains("test.rs"));
    }

    #[tokio::test]
    async fn find_question_mark_pattern() {
        let temp_dir = TempDir::new().unwrap();
        let file1 = temp_dir.path().join("file1.log");
        let file2 = temp_dir.path().join("file2.log");
        let file3 = temp_dir.path().join("file10.log");

        fs::write(&file1, "content").unwrap();
        fs::write(&file2, "content").unwrap();
        fs::write(&file3, "content").unwrap();

        let tool = FindTool::new(std::env::current_dir().unwrap());
        let input = serde_json::json!({
            "pattern": "file?.log",
            "path": temp_dir.path().to_str().unwrap()
        });

        let result = tool.execute(input).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.contains("Found 2 file(s)"));
        assert!(response.contains("file1.log"));
        assert!(response.contains("file2.log"));
        assert!(!response.contains("file10.log"));
    }

    #[tokio::test]
    async fn find_bracket_pattern() {
        let temp_dir = TempDir::new().unwrap();
        let file1 = temp_dir.path().join("test_a.txt");
        let file2 = temp_dir.path().join("test_b.txt");
        let file3 = temp_dir.path().join("test_c.txt");

        fs::write(&file1, "content").unwrap();
        fs::write(&file2, "content").unwrap();
        fs::write(&file3, "content").unwrap();

        let tool = FindTool::new(std::env::current_dir().unwrap());
        let input = serde_json::json!({
            "pattern": "test_[ab].txt",
            "path": temp_dir.path().to_str().unwrap()
        });

        let result = tool.execute(input).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.contains("Found 2 file(s)"));
        assert!(response.contains("test_a.txt"));
        assert!(response.contains("test_b.txt"));
        assert!(!response.contains("test_c.txt"));
    }

    #[tokio::test]
    async fn find_recursive() {
        let temp_dir = TempDir::new().unwrap();
        let subdir = temp_dir.path().join("subdir");
        fs::create_dir(&subdir).unwrap();

        let file1 = temp_dir.path().join("test.rs");
        let file2 = subdir.join("test.rs");

        fs::write(&file1, "content").unwrap();
        fs::write(&file2, "content").unwrap();

        let tool = FindTool::new(std::env::current_dir().unwrap());
        let input = serde_json::json!({
            "pattern": "*.rs",
            "path": temp_dir.path().to_str().unwrap()
        });

        let result = tool.execute(input).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.contains("Found 2 file(s)"));
    }

    #[tokio::test]
    async fn find_no_matches() {
        let temp_dir = TempDir::new().unwrap();
        let file1 = temp_dir.path().join("test.txt");
        fs::write(&file1, "content").unwrap();

        let tool = FindTool::new(std::env::current_dir().unwrap());
        let input = serde_json::json!({
            "pattern": "*.rs",
            "path": temp_dir.path().to_str().unwrap()
        });

        let result = tool.execute(input).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.contains("No files found"));
    }

    #[tokio::test]
    async fn find_invalid_pattern() {
        let temp_dir = TempDir::new().unwrap();

        let tool = FindTool::new(std::env::current_dir().unwrap());
        let input = serde_json::json!({
            "pattern": "[invalid",
            "path": temp_dir.path().to_str().unwrap()
        });

        let result = tool.execute(input).await;
        assert!(result.is_err());

        match result {
            Err(FaeAgentError::Tool(msg)) => {
                assert!(msg.contains("Invalid glob pattern"));
            }
            _ => panic!("Expected Tool error"),
        }
    }

    #[tokio::test]
    async fn find_path_not_found() {
        let tool = FindTool::new(std::env::current_dir().unwrap());
        let input = serde_json::json!({
            "pattern": "*.txt",
            "path": "/nonexistent/path"
        });

        let result = tool.execute(input).await;
        assert!(result.is_err());

        match result {
            Err(FaeAgentError::Tool(msg)) => {
                assert!(msg.contains("Path not found"));
            }
            _ => panic!("Expected Tool error"),
        }
    }
}
