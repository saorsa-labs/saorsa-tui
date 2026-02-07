//! Write tool for writing file contents with diff display.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use similar::{ChangeTag, TextDiff};

use crate::error::{Result, SaorsaAgentError};
use crate::tool::Tool;

/// Tool for writing file contents.
pub struct WriteTool {
    /// Base directory for resolving relative paths.
    working_dir: PathBuf,
}

/// Input parameters for the Write tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct WriteInput {
    /// Path to the file to write.
    file_path: String,
    /// Content to write to the file.
    content: String,
}

impl WriteTool {
    /// Create a new Write tool with the given working directory.
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

    /// Generate a unified diff between old and new content.
    fn generate_diff(old_content: &str, new_content: &str, file_path: &Path) -> String {
        let diff = TextDiff::from_lines(old_content, new_content);

        let mut output = String::new();
        output.push_str(&format!("--- {}\n", file_path.display()));
        output.push_str(&format!("+++ {} (new)\n", file_path.display()));

        for change in diff.iter_all_changes() {
            let sign = match change.tag() {
                ChangeTag::Delete => "-",
                ChangeTag::Insert => "+",
                ChangeTag::Equal => " ",
            };
            output.push_str(&format!("{}{}", sign, change));
        }

        output
    }
}

#[async_trait::async_trait]
impl Tool for WriteTool {
    fn name(&self) -> &str {
        "write"
    }

    fn description(&self) -> &str {
        "Write content to a file, creating parent directories if needed, with diff for existing files"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "Path to the file to write (absolute or relative to working directory)"
                },
                "content": {
                    "type": "string",
                    "description": "Content to write to the file"
                }
            },
            "required": ["file_path", "content"]
        })
    }

    async fn execute(&self, input: serde_json::Value) -> Result<String> {
        let input: WriteInput = serde_json::from_value(input)
            .map_err(|e| SaorsaAgentError::Tool(format!("Invalid input: {e}")))?;

        let path = self.resolve_path(&input.file_path);

        // Check if file already exists and generate diff if so
        let (old_content, file_exists) = if path.exists() {
            if path.is_dir() {
                return Err(SaorsaAgentError::Tool(format!(
                    "Path is a directory, cannot write: {}",
                    path.display()
                )));
            }

            let content = fs::read_to_string(&path).map_err(|e| {
                SaorsaAgentError::Tool(format!("Failed to read existing file: {e}"))
            })?;
            (Some(content), true)
        } else {
            (None, false)
        };

        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                SaorsaAgentError::Tool(format!("Failed to create parent directories: {e}"))
            })?;
        }

        // Write the file
        fs::write(&path, &input.content)
            .map_err(|e| SaorsaAgentError::Tool(format!("Failed to write file: {e}")))?;

        // Build response
        let mut response = if file_exists {
            format!("File updated: {}\n\n", path.display())
        } else {
            format!("File created: {}\n\n", path.display())
        };

        // Add diff if file was updated
        if let Some(old) = old_content {
            if old != input.content {
                response.push_str("Diff:\n");
                response.push_str(&Self::generate_diff(&old, &input.content, &path));
            } else {
                response.push_str("(No changes - content identical)");
            }
        } else {
            response.push_str(&format!("Wrote {} bytes", input.content.len()));
        }

        Ok(response)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn write_new_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let tool = WriteTool::new(temp_dir.path());

        let file_path = temp_dir.path().join("new_file.txt");
        let input = serde_json::json!({
            "file_path": file_path.to_str().unwrap(),
            "content": "Hello, World!"
        });

        let result = tool.execute(input).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.contains("File created"));
        assert!(response.contains("13 bytes")); // "Hello, World!" is 13 bytes

        // Verify file was created
        assert!(file_path.exists());
        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "Hello, World!");
    }

    #[tokio::test]
    async fn write_update_existing_file() {
        let mut temp = NamedTempFile::new().unwrap();
        writeln!(temp, "Original content").unwrap();
        temp.flush().unwrap();

        let tool = WriteTool::new(std::env::current_dir().unwrap());
        let input = serde_json::json!({
            "file_path": temp.path().to_str().unwrap(),
            "content": "New content"
        });

        let result = tool.execute(input).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.contains("File updated"));
        assert!(response.contains("Diff:"));
        assert!(response.contains("-Original content"));
        assert!(response.contains("+New content"));

        // Verify file was updated
        let content = fs::read_to_string(temp.path()).unwrap();
        assert_eq!(content, "New content");
    }

    #[tokio::test]
    async fn write_identical_content() {
        let mut temp = NamedTempFile::new().unwrap();
        writeln!(temp, "Same content").unwrap();
        temp.flush().unwrap();

        let tool = WriteTool::new(std::env::current_dir().unwrap());
        let input = serde_json::json!({
            "file_path": temp.path().to_str().unwrap(),
            "content": "Same content\n"
        });

        let result = tool.execute(input).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.contains("File updated"));
        assert!(response.contains("No changes - content identical"));
    }

    #[tokio::test]
    async fn write_create_parent_directories() {
        let temp_dir = tempfile::tempdir().unwrap();
        let tool = WriteTool::new(temp_dir.path());

        let file_path = temp_dir.path().join("subdir/nested/file.txt");
        let input = serde_json::json!({
            "file_path": file_path.to_str().unwrap(),
            "content": "Nested file content"
        });

        let result = tool.execute(input).await;
        assert!(result.is_ok());

        // Verify parent directories were created
        assert!(file_path.parent().unwrap().exists());
        assert!(file_path.exists());

        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "Nested file content");
    }

    #[tokio::test]
    async fn write_to_directory_fails() {
        let temp_dir = tempfile::tempdir().unwrap();
        let tool = WriteTool::new(temp_dir.path());

        let input = serde_json::json!({
            "file_path": temp_dir.path().to_str().unwrap(),
            "content": "This should fail"
        });

        let result = tool.execute(input).await;
        assert!(result.is_err());

        match result {
            Err(SaorsaAgentError::Tool(msg)) => {
                assert!(msg.contains("is a directory"));
            }
            _ => panic!("Expected Tool error"),
        }
    }

    #[test]
    fn diff_generation() {
        let old = "Line 1\nLine 2\nLine 3\n";
        let new = "Line 1\nModified Line 2\nLine 3\n";
        let path = Path::new("test.txt");

        let diff = WriteTool::generate_diff(old, new, path);

        assert!(diff.contains("--- test.txt"));
        assert!(diff.contains("+++ test.txt (new)"));
        assert!(diff.contains("-Line 2"));
        assert!(diff.contains("+Modified Line 2"));
    }

    #[tokio::test]
    async fn write_relative_path() {
        let temp_dir = tempfile::tempdir().unwrap();
        let tool = WriteTool::new(temp_dir.path());

        let input = serde_json::json!({
            "file_path": "relative/path/file.txt",
            "content": "Content in relative path"
        });

        let result = tool.execute(input).await;
        assert!(result.is_ok());

        let file_path = temp_dir.path().join("relative/path/file.txt");
        assert!(file_path.exists());

        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "Content in relative path");
    }
}
