//! Edit tool for surgical file editing with ambiguity detection.

use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::{generate_diff, resolve_path};
use crate::error::{Result, SaorsaAgentError};
use crate::tool::Tool;

/// Tool for surgical file editing.
pub struct EditTool {
    /// Base directory for resolving relative paths.
    working_dir: PathBuf,
}

/// Input parameters for the Edit tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct EditInput {
    /// Path to the file to edit.
    file_path: String,
    /// Text to search for and replace.
    old_text: String,
    /// Replacement text.
    new_text: String,
    /// Replace all occurrences (default: false).
    #[serde(default)]
    replace_all: bool,
}

impl EditTool {
    /// Create a new Edit tool with the given working directory.
    pub fn new(working_dir: impl Into<PathBuf>) -> Self {
        Self {
            working_dir: working_dir.into(),
        }
    }
}

#[async_trait::async_trait]
impl Tool for EditTool {
    fn name(&self) -> &str {
        "edit"
    }

    fn description(&self) -> &str {
        "Edit a file by replacing exact text matches, with ambiguity detection"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "Path to the file to edit (absolute or relative to working directory)"
                },
                "old_text": {
                    "type": "string",
                    "description": "Exact text to search for and replace"
                },
                "new_text": {
                    "type": "string",
                    "description": "Replacement text"
                },
                "replace_all": {
                    "type": "boolean",
                    "description": "Replace all occurrences (default: false, errors if multiple matches found)",
                    "default": false
                }
            },
            "required": ["file_path", "old_text", "new_text"]
        })
    }

    async fn execute(&self, input: serde_json::Value) -> Result<String> {
        let input: EditInput = serde_json::from_value(input)
            .map_err(|e| SaorsaAgentError::Tool(format!("Invalid input: {e}")))?;

        let path = resolve_path(&self.working_dir, &input.file_path);

        // Check if file exists
        if !path.exists() {
            return Err(SaorsaAgentError::Tool(format!(
                "File not found: {}",
                path.display()
            )));
        }

        // Check if path is a file
        if !path.is_file() {
            return Err(SaorsaAgentError::Tool(format!(
                "Path is not a file: {}",
                path.display()
            )));
        }

        // Read file contents
        let content = fs::read_to_string(&path)
            .map_err(|e| SaorsaAgentError::Tool(format!("Failed to read file: {e}")))?;

        // Count occurrences of old_text
        let match_count = content.matches(&input.old_text).count();

        if match_count == 0 {
            return Err(SaorsaAgentError::Tool(format!(
                "Text not found in file: '{}'",
                input.old_text
            )));
        }

        // Check for ambiguity
        if match_count > 1 && !input.replace_all {
            return Err(SaorsaAgentError::Tool(format!(
                "Ambiguous: found {} matches for '{}'. Use replace_all: true to replace all occurrences, or provide more context to make the match unique.",
                match_count, input.old_text
            )));
        }

        // Perform replacement
        let new_content = if input.replace_all {
            content.replace(&input.old_text, &input.new_text)
        } else {
            content.replacen(&input.old_text, &input.new_text, 1)
        };

        // Write the updated content
        fs::write(&path, &new_content)
            .map_err(|e| SaorsaAgentError::Tool(format!("Failed to write file: {e}")))?;

        // Build response
        let mut response = if input.replace_all {
            format!(
                "Replaced {} occurrence(s) of text in: {}\n\n",
                match_count,
                path.display()
            )
        } else {
            format!("Replaced text in: {}\n\n", path.display())
        };

        // Add diff
        response.push_str("Diff:\n");
        response.push_str(&generate_diff(&content, &new_content, &path, "edited"));

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
    async fn edit_single_replacement() {
        let mut temp = NamedTempFile::new().unwrap();
        writeln!(temp, "Line 1").unwrap();
        writeln!(temp, "Line 2").unwrap();
        writeln!(temp, "Line 3").unwrap();
        temp.flush().unwrap();

        let tool = EditTool::new(std::env::current_dir().unwrap());
        let input = serde_json::json!({
            "file_path": temp.path().to_str().unwrap(),
            "old_text": "Line 2",
            "new_text": "Modified Line 2"
        });

        let result = tool.execute(input).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.contains("Replaced text"));
        assert!(response.contains("Diff:"));
        assert!(response.contains("-Line 2"));
        assert!(response.contains("+Modified Line 2"));

        // Verify file was edited
        let content = fs::read_to_string(temp.path()).unwrap();
        assert!(content.contains("Modified Line 2"));
        // Verify expected final content
        assert_eq!(content, "Line 1\nModified Line 2\nLine 3\n");
    }

    #[tokio::test]
    async fn edit_ambiguous_without_replace_all() {
        let mut temp = NamedTempFile::new().unwrap();
        writeln!(temp, "foo bar").unwrap();
        writeln!(temp, "foo baz").unwrap();
        writeln!(temp, "foo qux").unwrap();
        temp.flush().unwrap();

        let tool = EditTool::new(std::env::current_dir().unwrap());
        let input = serde_json::json!({
            "file_path": temp.path().to_str().unwrap(),
            "old_text": "foo",
            "new_text": "FOO"
        });

        let result = tool.execute(input).await;
        assert!(result.is_err());

        match result {
            Err(SaorsaAgentError::Tool(msg)) => {
                assert!(msg.contains("Ambiguous"));
                assert!(msg.contains("3 matches"));
                assert!(msg.contains("replace_all"));
            }
            _ => panic!("Expected Tool error"),
        }
    }

    #[tokio::test]
    async fn edit_replace_all() {
        let mut temp = NamedTempFile::new().unwrap();
        writeln!(temp, "foo bar").unwrap();
        writeln!(temp, "foo baz").unwrap();
        writeln!(temp, "foo qux").unwrap();
        temp.flush().unwrap();

        let tool = EditTool::new(std::env::current_dir().unwrap());
        let input = serde_json::json!({
            "file_path": temp.path().to_str().unwrap(),
            "old_text": "foo",
            "new_text": "FOO",
            "replace_all": true
        });

        let result = tool.execute(input).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.contains("Replaced 3 occurrence(s)"));
        assert!(response.contains("Diff:"));

        // Verify all occurrences were replaced
        let content = fs::read_to_string(temp.path()).unwrap();
        assert_eq!(content.matches("FOO").count(), 3);
        assert_eq!(content.matches("foo").count(), 0);
    }

    #[tokio::test]
    async fn edit_text_not_found() {
        let mut temp = NamedTempFile::new().unwrap();
        writeln!(temp, "Some content").unwrap();
        temp.flush().unwrap();

        let tool = EditTool::new(std::env::current_dir().unwrap());
        let input = serde_json::json!({
            "file_path": temp.path().to_str().unwrap(),
            "old_text": "Nonexistent text",
            "new_text": "Replacement"
        });

        let result = tool.execute(input).await;
        assert!(result.is_err());

        match result {
            Err(SaorsaAgentError::Tool(msg)) => {
                assert!(msg.contains("Text not found"));
            }
            _ => panic!("Expected Tool error"),
        }
    }

    #[tokio::test]
    async fn edit_file_not_found() {
        let tool = EditTool::new(std::env::current_dir().unwrap());
        let input = serde_json::json!({
            "file_path": "/nonexistent/file.txt",
            "old_text": "old",
            "new_text": "new"
        });

        let result = tool.execute(input).await;
        assert!(result.is_err());

        match result {
            Err(SaorsaAgentError::Tool(msg)) => {
                assert!(msg.contains("File not found"));
            }
            _ => panic!("Expected Tool error"),
        }
    }

    #[tokio::test]
    async fn edit_multiline_text() {
        let mut temp = NamedTempFile::new().unwrap();
        writeln!(temp, "Line 1").unwrap();
        writeln!(temp, "Line 2").unwrap();
        writeln!(temp, "Line 3").unwrap();
        writeln!(temp, "Line 4").unwrap();
        temp.flush().unwrap();

        let tool = EditTool::new(std::env::current_dir().unwrap());
        let input = serde_json::json!({
            "file_path": temp.path().to_str().unwrap(),
            "old_text": "Line 2\nLine 3",
            "new_text": "Modified Lines 2-3"
        });

        let result = tool.execute(input).await;
        assert!(result.is_ok());

        let content = fs::read_to_string(temp.path()).unwrap();
        assert!(content.contains("Modified Lines 2-3"));
        assert!(!content.contains("Line 2\nLine 3"));
    }

    #[tokio::test]
    async fn edit_preserve_other_content() {
        let mut temp = NamedTempFile::new().unwrap();
        writeln!(temp, "Before").unwrap();
        writeln!(temp, "Target").unwrap();
        writeln!(temp, "After").unwrap();
        temp.flush().unwrap();

        let tool = EditTool::new(std::env::current_dir().unwrap());
        let input = serde_json::json!({
            "file_path": temp.path().to_str().unwrap(),
            "old_text": "Target",
            "new_text": "Modified"
        });

        let result = tool.execute(input).await;
        assert!(result.is_ok());

        let content = fs::read_to_string(temp.path()).unwrap();
        assert!(content.contains("Before"));
        assert!(content.contains("Modified"));
        assert!(content.contains("After"));
        assert!(!content.contains("Target"));
    }
}
