//! Grep tool for searching file contents with regex patterns.

use std::fs;
use std::path::{Path, PathBuf};

use regex::Regex;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::error::{FaeAgentError, Result};
use crate::tool::Tool;

/// Maximum number of matches to return (prevents overwhelming output).
const MAX_MATCHES: usize = 100;

/// Tool for searching file contents with regex.
pub struct GrepTool {
    /// Base directory for resolving relative paths.
    working_dir: PathBuf,
}

/// Input parameters for the Grep tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct GrepInput {
    /// Regex pattern to search for.
    pattern: String,
    /// Path to file or directory to search.
    path: String,
    /// Case-insensitive search (default: false).
    #[serde(default)]
    case_insensitive: bool,
}

impl GrepTool {
    /// Create a new Grep tool with the given working directory.
    pub fn new(working_dir: impl Into<PathBuf>) -> Self {
        Self {
            working_dir: working_dir.into(),
        }
    }

    /// Resolve a file path relative to the working directory.
    fn resolve_path(&self, path: &str) -> PathBuf {
        let path = Path::new(path);
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.working_dir.join(path)
        }
    }

    /// Search a single file for pattern matches.
    fn search_file(
        file_path: &Path,
        regex: &Regex,
        matches: &mut Vec<String>,
    ) -> Result<()> {
        // Only search text files (skip binary files)
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => return Ok(()), // Skip files we can't read as text
        };

        for (line_num, line) in content.lines().enumerate() {
            if matches.len() >= MAX_MATCHES {
                break;
            }

            if regex.is_match(line) {
                matches.push(format!(
                    "{}:{}:{}",
                    file_path.display(),
                    line_num + 1,
                    line
                ));
            }
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl Tool for GrepTool {
    fn name(&self) -> &str {
        "grep"
    }

    fn description(&self) -> &str {
        "Search file contents using regex patterns, with recursive directory search"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "Regex pattern to search for"
                },
                "path": {
                    "type": "string",
                    "description": "Path to file or directory to search (absolute or relative to working directory)"
                },
                "case_insensitive": {
                    "type": "boolean",
                    "description": "Case-insensitive search (default: false)",
                    "default": false
                }
            },
            "required": ["pattern", "path"]
        })
    }

    async fn execute(&self, input: serde_json::Value) -> Result<String> {
        let input: GrepInput = serde_json::from_value(input)
            .map_err(|e| FaeAgentError::Tool(format!("Invalid input: {e}")))?;

        let path = self.resolve_path(&input.path);

        // Check if path exists
        if !path.exists() {
            return Err(FaeAgentError::Tool(format!(
                "Path not found: {}",
                path.display()
            )));
        }

        // Compile regex
        let regex = if input.case_insensitive {
            Regex::new(&format!("(?i){}", input.pattern))
        } else {
            Regex::new(&input.pattern)
        }
        .map_err(|e| FaeAgentError::Tool(format!("Invalid regex pattern: {e}")))?;

        let mut matches = Vec::new();

        // Search file or directory
        if path.is_file() {
            Self::search_file(&path, &regex, &mut matches)?;
        } else if path.is_dir() {
            // Recursive directory search
            for entry in WalkDir::new(&path)
                .follow_links(false)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if matches.len() >= MAX_MATCHES {
                    break;
                }

                if entry.file_type().is_file() {
                    Self::search_file(entry.path(), &regex, &mut matches)?;
                }
            }
        }

        // Build response
        if matches.is_empty() {
            Ok(format!("No matches found for pattern: '{}'", input.pattern))
        } else {
            let truncated = if matches.len() >= MAX_MATCHES {
                format!("\n\n(Results limited to {} matches)", MAX_MATCHES)
            } else {
                String::new()
            };

            Ok(format!(
                "Found {} match(es):\n\n{}{}",
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
    use std::io::Write;
    use tempfile::{NamedTempFile, TempDir};

    #[tokio::test]
    async fn grep_single_file_match() {
        let mut temp = NamedTempFile::new().unwrap();
        writeln!(temp, "Hello World").unwrap();
        writeln!(temp, "Goodbye World").unwrap();
        temp.flush().unwrap();

        let tool = GrepTool::new(std::env::current_dir().unwrap());
        let input = serde_json::json!({
            "pattern": "Hello",
            "path": temp.path().to_str().unwrap()
        });

        let result = tool.execute(input).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.contains("Found 1 match"));
        assert!(response.contains("Hello World"));
        assert!(!response.contains("Goodbye"));
    }

    #[tokio::test]
    async fn grep_case_insensitive() {
        let mut temp = NamedTempFile::new().unwrap();
        writeln!(temp, "Hello World").unwrap();
        writeln!(temp, "hello world").unwrap();
        writeln!(temp, "HELLO WORLD").unwrap();
        temp.flush().unwrap();

        let tool = GrepTool::new(std::env::current_dir().unwrap());
        let input = serde_json::json!({
            "pattern": "hello",
            "path": temp.path().to_str().unwrap(),
            "case_insensitive": true
        });

        let result = tool.execute(input).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.contains("Found 3 match"));
    }

    #[tokio::test]
    async fn grep_regex_pattern() {
        let mut temp = NamedTempFile::new().unwrap();
        writeln!(temp, "foo123").unwrap();
        writeln!(temp, "bar456").unwrap();
        writeln!(temp, "baz789").unwrap();
        temp.flush().unwrap();

        let tool = GrepTool::new(std::env::current_dir().unwrap());
        let input = serde_json::json!({
            "pattern": r"ba[rz]\d+",
            "path": temp.path().to_str().unwrap()
        });

        let result = tool.execute(input).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.contains("Found 2 match"));
        assert!(response.contains("bar456"));
        assert!(response.contains("baz789"));
        assert!(!response.contains("foo123"));
    }

    #[tokio::test]
    async fn grep_no_matches() {
        let mut temp = NamedTempFile::new().unwrap();
        writeln!(temp, "Some content").unwrap();
        temp.flush().unwrap();

        let tool = GrepTool::new(std::env::current_dir().unwrap());
        let input = serde_json::json!({
            "pattern": "nonexistent",
            "path": temp.path().to_str().unwrap()
        });

        let result = tool.execute(input).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.contains("No matches found"));
    }

    #[tokio::test]
    async fn grep_directory_recursive() {
        let temp_dir = TempDir::new().unwrap();
        let file1 = temp_dir.path().join("file1.txt");
        let subdir = temp_dir.path().join("subdir");
        fs::create_dir(&subdir).unwrap();
        let file2 = subdir.join("file2.txt");

        fs::write(&file1, "match in file1\n").unwrap();
        fs::write(&file2, "match in file2\n").unwrap();

        let tool = GrepTool::new(std::env::current_dir().unwrap());
        let input = serde_json::json!({
            "pattern": "match",
            "path": temp_dir.path().to_str().unwrap()
        });

        let result = tool.execute(input).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.contains("Found 2 match"));
        assert!(response.contains("file1.txt"));
        assert!(response.contains("file2.txt"));
    }

    #[tokio::test]
    async fn grep_invalid_regex() {
        let mut temp = NamedTempFile::new().unwrap();
        writeln!(temp, "content").unwrap();
        temp.flush().unwrap();

        let tool = GrepTool::new(std::env::current_dir().unwrap());
        let input = serde_json::json!({
            "pattern": "[invalid",
            "path": temp.path().to_str().unwrap()
        });

        let result = tool.execute(input).await;
        assert!(result.is_err());

        match result {
            Err(FaeAgentError::Tool(msg)) => {
                assert!(msg.contains("Invalid regex"));
            }
            _ => panic!("Expected Tool error"),
        }
    }

    #[tokio::test]
    async fn grep_path_not_found() {
        let tool = GrepTool::new(std::env::current_dir().unwrap());
        let input = serde_json::json!({
            "pattern": "test",
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
