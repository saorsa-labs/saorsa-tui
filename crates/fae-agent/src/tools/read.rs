//! Read tool for reading file contents with optional line ranges.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{FaeAgentError, Result};
use crate::tool::Tool;

/// Maximum file size in bytes (10 MB).
const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024;

/// Tool for reading file contents.
pub struct ReadTool {
    /// Base directory for resolving relative paths.
    working_dir: PathBuf,
}

/// Input parameters for the Read tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ReadInput {
    /// Path to the file to read.
    file_path: String,
    /// Optional line range (e.g., "10-20", "5-", "-10").
    #[serde(default)]
    line_range: Option<String>,
}

impl ReadTool {
    /// Create a new Read tool with the given working directory.
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

    /// Parse a line range string (e.g., "10-20", "5-", "-10").
    fn parse_line_range(range: &str) -> Result<(Option<usize>, Option<usize>)> {
        let parts: Vec<&str> = range.split('-').collect();

        match parts.as_slice() {
            // "N-M" - lines N through M (inclusive, 1-indexed)
            [start, end] if !start.is_empty() && !end.is_empty() => {
                let start = start.parse::<usize>().map_err(|_| {
                    FaeAgentError::Tool(format!("Invalid start line number: {start}"))
                })?;
                let end = end
                    .parse::<usize>()
                    .map_err(|_| FaeAgentError::Tool(format!("Invalid end line number: {end}")))?;

                if start == 0 || end == 0 {
                    return Err(FaeAgentError::Tool("Line numbers must be >= 1".to_string()));
                }
                if start > end {
                    return Err(FaeAgentError::Tool(format!(
                        "Start line ({start}) must be <= end line ({end})"
                    )));
                }

                Ok((Some(start), Some(end)))
            }
            // "N-" - from line N to end
            [start, ""] if !start.is_empty() => {
                let start = start.parse::<usize>().map_err(|_| {
                    FaeAgentError::Tool(format!("Invalid start line number: {start}"))
                })?;

                if start == 0 {
                    return Err(FaeAgentError::Tool("Line numbers must be >= 1".to_string()));
                }

                Ok((Some(start), None))
            }
            // "-M" - from start to line M
            ["", end] if !end.is_empty() => {
                let end = end
                    .parse::<usize>()
                    .map_err(|_| FaeAgentError::Tool(format!("Invalid end line number: {end}")))?;

                if end == 0 {
                    return Err(FaeAgentError::Tool("Line numbers must be >= 1".to_string()));
                }

                Ok((None, Some(end)))
            }
            _ => Err(FaeAgentError::Tool(format!(
                "Invalid line range format: {range}"
            ))),
        }
    }

    /// Filter lines based on the line range.
    fn filter_lines(content: &str, range: Option<&str>) -> Result<String> {
        let Some(range_str) = range else {
            return Ok(content.to_string());
        };

        let (start, end) = Self::parse_line_range(range_str)?;
        let lines: Vec<&str> = content.lines().collect();
        let total_lines = lines.len();

        // Convert 1-indexed to 0-indexed
        let start_idx = start.map(|n| n.saturating_sub(1)).unwrap_or(0);
        let end_idx = end.map(|n| n.min(total_lines)).unwrap_or(total_lines);

        if start_idx >= total_lines {
            return Err(FaeAgentError::Tool(format!(
                "Start line {} exceeds file length ({} lines)",
                start.unwrap_or(1),
                total_lines
            )));
        }

        let selected = &lines[start_idx..end_idx];
        Ok(selected.join("\n"))
    }
}

#[async_trait::async_trait]
impl Tool for ReadTool {
    fn name(&self) -> &str {
        "read"
    }

    fn description(&self) -> &str {
        "Read file contents with optional line range filtering"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "Path to the file to read (absolute or relative to working directory)"
                },
                "line_range": {
                    "type": "string",
                    "description": "Optional line range (e.g., '10-20' for lines 10 through 20, '5-' from line 5 to end, '-10' first 10 lines)"
                }
            },
            "required": ["file_path"]
        })
    }

    async fn execute(&self, input: serde_json::Value) -> Result<String> {
        let input: ReadInput = serde_json::from_value(input)
            .map_err(|e| FaeAgentError::Tool(format!("Invalid input: {e}")))?;

        let path = self.resolve_path(&input.file_path);

        // Check if file exists
        if !path.exists() {
            return Err(FaeAgentError::Tool(format!(
                "File not found: {}",
                path.display()
            )));
        }

        // Check if path is a file
        if !path.is_file() {
            return Err(FaeAgentError::Tool(format!(
                "Path is not a file: {}",
                path.display()
            )));
        }

        // Check file size
        let metadata = fs::metadata(&path)
            .map_err(|e| FaeAgentError::Tool(format!("Failed to read file metadata: {e}")))?;

        if metadata.len() > MAX_FILE_SIZE {
            return Err(FaeAgentError::Tool(format!(
                "File too large: {} bytes (max {} bytes)",
                metadata.len(),
                MAX_FILE_SIZE
            )));
        }

        // Read file contents
        let content = fs::read_to_string(&path)
            .map_err(|e| FaeAgentError::Tool(format!("Failed to read file: {e}")))?;

        // Filter by line range if specified
        let filtered = Self::filter_lines(&content, input.line_range.as_deref())?;

        Ok(filtered)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn parse_line_range_full() {
        let result = ReadTool::parse_line_range("10-20");
        assert!(result.is_ok());
        let (start, end) = result.unwrap();
        assert_eq!(start, Some(10));
        assert_eq!(end, Some(20));
    }

    #[test]
    fn parse_line_range_from() {
        let result = ReadTool::parse_line_range("5-");
        assert!(result.is_ok());
        let (start, end) = result.unwrap();
        assert_eq!(start, Some(5));
        assert_eq!(end, None);
    }

    #[test]
    fn parse_line_range_to() {
        let result = ReadTool::parse_line_range("-10");
        assert!(result.is_ok());
        let (start, end) = result.unwrap();
        assert_eq!(start, None);
        assert_eq!(end, Some(10));
    }

    #[test]
    fn parse_line_range_invalid() {
        assert!(ReadTool::parse_line_range("invalid").is_err());
        assert!(ReadTool::parse_line_range("10-5").is_err());
        assert!(ReadTool::parse_line_range("0-10").is_err());
        assert!(ReadTool::parse_line_range("10-0").is_err());
    }

    #[test]
    fn filter_lines_no_range() {
        let content = "line1\nline2\nline3";
        let result = ReadTool::filter_lines(content, None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), content);
    }

    #[test]
    fn filter_lines_full_range() {
        let content = "line1\nline2\nline3\nline4\nline5";
        let result = ReadTool::filter_lines(content, Some("2-4"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "line2\nline3\nline4");
    }

    #[test]
    fn filter_lines_from_range() {
        let content = "line1\nline2\nline3\nline4\nline5";
        let result = ReadTool::filter_lines(content, Some("3-"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "line3\nline4\nline5");
    }

    #[test]
    fn filter_lines_to_range() {
        let content = "line1\nline2\nline3\nline4\nline5";
        let result = ReadTool::filter_lines(content, Some("-3"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "line1\nline2\nline3");
    }

    #[test]
    fn filter_lines_exceeds_length() {
        let content = "line1\nline2\nline3";
        let result = ReadTool::filter_lines(content, Some("10-20"));
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn read_full_file() {
        let mut temp = NamedTempFile::new().unwrap();
        writeln!(temp, "line1").unwrap();
        writeln!(temp, "line2").unwrap();
        writeln!(temp, "line3").unwrap();
        temp.flush().unwrap();

        let tool = ReadTool::new(std::env::current_dir().unwrap());
        let input = serde_json::json!({
            "file_path": temp.path().to_str().unwrap()
        });

        let result = tool.execute(input).await;
        assert!(result.is_ok());
        let content = result.unwrap();
        assert!(content.contains("line1"));
        assert!(content.contains("line2"));
        assert!(content.contains("line3"));
    }

    #[tokio::test]
    async fn read_with_range() {
        let mut temp = NamedTempFile::new().unwrap();
        writeln!(temp, "line1").unwrap();
        writeln!(temp, "line2").unwrap();
        writeln!(temp, "line3").unwrap();
        temp.flush().unwrap();

        let tool = ReadTool::new(std::env::current_dir().unwrap());
        let input = serde_json::json!({
            "file_path": temp.path().to_str().unwrap(),
            "line_range": "2-3"
        });

        let result = tool.execute(input).await;
        assert!(result.is_ok());
        let content = result.unwrap();
        assert!(!content.contains("line1"));
        assert!(content.contains("line2"));
        assert!(content.contains("line3"));
    }

    #[tokio::test]
    async fn read_nonexistent_file() {
        let tool = ReadTool::new(std::env::current_dir().unwrap());
        let input = serde_json::json!({
            "file_path": "/nonexistent/file.txt"
        });

        let result = tool.execute(input).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn read_directory() {
        let tool = ReadTool::new(std::env::current_dir().unwrap());
        let input = serde_json::json!({
            "file_path": std::env::current_dir().unwrap().to_str().unwrap()
        });

        let result = tool.execute(input).await;
        assert!(result.is_err());
    }
}
