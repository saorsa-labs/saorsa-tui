//! Ls tool for listing directory contents with metadata.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::error::{FaeAgentError, Result};
use crate::tool::Tool;

/// Tool for listing directory contents.
pub struct LsTool {
    /// Base directory for resolving relative paths.
    working_dir: PathBuf,
}

/// Input parameters for the Ls tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct LsInput {
    /// Directory to list (default: current working directory).
    #[serde(default)]
    path: Option<String>,
    /// Recursive listing (default: false).
    #[serde(default)]
    recursive: bool,
}

impl LsTool {
    /// Create a new Ls tool with the given working directory.
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

    /// Format file size in human-readable format.
    fn format_size(size: u64) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;

        if size >= GB {
            format!("{:.1}G", size as f64 / GB as f64)
        } else if size >= MB {
            format!("{:.1}M", size as f64 / MB as f64)
        } else if size >= KB {
            format!("{:.1}K", size as f64 / KB as f64)
        } else {
            format!("{}B", size)
        }
    }

    /// Get entry type as a string.
    fn entry_type(path: &Path) -> &'static str {
        let metadata = match fs::metadata(path) {
            Ok(m) => m,
            Err(_) => return "?",
        };

        if metadata.is_dir() {
            "DIR"
        } else if metadata.is_symlink() {
            "LNK"
        } else {
            "FILE"
        }
    }
}

#[async_trait::async_trait]
impl Tool for LsTool {
    fn name(&self) -> &str {
        "ls"
    }

    fn description(&self) -> &str {
        "List directory contents with metadata (size, type)"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Directory to list (default: current working directory)"
                },
                "recursive": {
                    "type": "boolean",
                    "description": "Recursive listing (default: false)",
                    "default": false
                }
            }
        })
    }

    async fn execute(&self, input: serde_json::Value) -> Result<String> {
        let input: LsInput = serde_json::from_value(input)
            .map_err(|e| FaeAgentError::Tool(format!("Invalid input: {e}")))?;

        let list_path = self.resolve_path(input.path.as_deref());

        // Check if path exists
        if !list_path.exists() {
            return Err(FaeAgentError::Tool(format!(
                "Path not found: {}",
                list_path.display()
            )));
        }

        // Check if path is a directory
        if !list_path.is_dir() {
            return Err(FaeAgentError::Tool(format!(
                "Path is not a directory: {}",
                list_path.display()
            )));
        }

        let mut entries = Vec::new();

        if input.recursive {
            // Recursive listing
            for entry in WalkDir::new(&list_path)
                .follow_links(false)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let path = entry.path();
                if path == list_path {
                    continue; // Skip the root directory itself
                }

                let metadata = match fs::metadata(path) {
                    Ok(m) => m,
                    Err(_) => continue,
                };

                let size = if metadata.is_file() {
                    Self::format_size(metadata.len())
                } else {
                    "-".to_string()
                };

                let rel_path = path
                    .strip_prefix(&list_path)
                    .unwrap_or(path)
                    .display()
                    .to_string();

                entries.push(format!(
                    "{:>8}  {:4}  {}",
                    size,
                    Self::entry_type(path),
                    rel_path
                ));
            }
        } else {
            // Non-recursive listing
            let mut dir_entries: Vec<_> = fs::read_dir(&list_path)
                .map_err(|e| FaeAgentError::Tool(format!("Failed to read directory: {e}")))?
                .filter_map(|e| e.ok())
                .collect();

            // Sort by name
            dir_entries.sort_by_key(|e| e.file_name());

            for entry in dir_entries {
                let path = entry.path();
                let metadata = match entry.metadata() {
                    Ok(m) => m,
                    Err(_) => continue,
                };

                let size = if metadata.is_file() {
                    Self::format_size(metadata.len())
                } else {
                    "-".to_string()
                };

                let name = entry.file_name().to_string_lossy().to_string();

                entries.push(format!(
                    "{:>8}  {:4}  {}",
                    size,
                    Self::entry_type(&path),
                    name
                ));
            }
        }

        // Build response
        if entries.is_empty() {
            Ok("(empty directory)".to_string())
        } else {
            let header = format!("{:>8}  {:4}  {}", "SIZE", "TYPE", "NAME");
            Ok(format!("{}\n{}", header, entries.join("\n")))
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
    async fn ls_empty_directory() {
        let temp_dir = TempDir::new().unwrap();

        let tool = LsTool::new(std::env::current_dir().unwrap());
        let input = serde_json::json!({
            "path": temp_dir.path().to_str().unwrap()
        });

        let result = tool.execute(input).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.contains("empty directory"));
    }

    #[tokio::test]
    async fn ls_with_files() {
        let temp_dir = TempDir::new().unwrap();
        let file1 = temp_dir.path().join("file1.txt");
        let file2 = temp_dir.path().join("file2.txt");

        fs::write(&file1, "small").unwrap();
        fs::write(&file2, "a bit larger content").unwrap();

        let tool = LsTool::new(std::env::current_dir().unwrap());
        let input = serde_json::json!({
            "path": temp_dir.path().to_str().unwrap()
        });

        let result = tool.execute(input).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.contains("FILE"));
        assert!(response.contains("file1.txt"));
        assert!(response.contains("file2.txt"));
        assert!(response.contains("SIZE"));
        assert!(response.contains("TYPE"));
    }

    #[tokio::test]
    async fn ls_with_subdirectories() {
        let temp_dir = TempDir::new().unwrap();
        let subdir = temp_dir.path().join("subdir");
        fs::create_dir(&subdir).unwrap();

        let file = temp_dir.path().join("file.txt");
        fs::write(&file, "content").unwrap();

        let tool = LsTool::new(std::env::current_dir().unwrap());
        let input = serde_json::json!({
            "path": temp_dir.path().to_str().unwrap()
        });

        let result = tool.execute(input).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.contains("FILE"));
        assert!(response.contains("DIR"));
        assert!(response.contains("file.txt"));
        assert!(response.contains("subdir"));
    }

    #[tokio::test]
    async fn ls_recursive() {
        let temp_dir = TempDir::new().unwrap();
        let subdir = temp_dir.path().join("subdir");
        fs::create_dir(&subdir).unwrap();

        let file1 = temp_dir.path().join("file1.txt");
        let file2 = subdir.join("file2.txt");

        fs::write(&file1, "content").unwrap();
        fs::write(&file2, "content").unwrap();

        let tool = LsTool::new(std::env::current_dir().unwrap());
        let input = serde_json::json!({
            "path": temp_dir.path().to_str().unwrap(),
            "recursive": true
        });

        let result = tool.execute(input).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.contains("file1.txt"));
        assert!(response.contains("subdir"));
        assert!(response.contains("file2.txt") || response.contains("subdir/file2.txt"));
    }

    #[tokio::test]
    async fn ls_path_not_found() {
        let tool = LsTool::new(std::env::current_dir().unwrap());
        let input = serde_json::json!({
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

    #[tokio::test]
    async fn ls_not_a_directory() {
        let temp_dir = TempDir::new().unwrap();
        let file = temp_dir.path().join("file.txt");
        fs::write(&file, "content").unwrap();

        let tool = LsTool::new(std::env::current_dir().unwrap());
        let input = serde_json::json!({
            "path": file.to_str().unwrap()
        });

        let result = tool.execute(input).await;
        assert!(result.is_err());

        match result {
            Err(FaeAgentError::Tool(msg)) => {
                assert!(msg.contains("not a directory"));
            }
            _ => panic!("Expected Tool error"),
        }
    }

    #[test]
    fn format_size_bytes() {
        assert_eq!(LsTool::format_size(0), "0B");
        assert_eq!(LsTool::format_size(512), "512B");
    }

    #[test]
    fn format_size_kilobytes() {
        assert_eq!(LsTool::format_size(1024), "1.0K");
        assert_eq!(LsTool::format_size(5120), "5.0K");
    }

    #[test]
    fn format_size_megabytes() {
        assert_eq!(LsTool::format_size(1048576), "1.0M");
        assert_eq!(LsTool::format_size(5242880), "5.0M");
    }

    #[test]
    fn format_size_gigabytes() {
        assert_eq!(LsTool::format_size(1073741824), "1.0G");
        assert_eq!(LsTool::format_size(5368709120), "5.0G");
    }
}
