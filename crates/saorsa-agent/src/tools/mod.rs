//! Built-in tools for the AI coding agent.
//!
//! This module provides a comprehensive suite of file manipulation and system execution tools
//! that enable the AI agent to interact with the filesystem and execute commands.
//!
//! # Available Tools
//!
//! ## File Reading & Writing
//!
//! - **ReadTool** (`read`) - Read file contents with optional line range filtering
//! - **WriteTool** (`write`) - Write content to files with diff display for existing files
//! - **EditTool** (`edit`) - Surgical file editing with exact text replacement and ambiguity detection
//!
//! ## File Discovery
//!
//! - **FindTool** (`find`) - Find files by name pattern using glob syntax (*, ?, \\[abc\\])
//! - **GrepTool** (`grep`) - Search file contents using regex patterns
//! - **LsTool** (`ls`) - List directory contents with metadata (size, type)
//!
//! ## System Operations
//!
//! - **BashTool** (`bash`) - Execute shell commands in a controlled environment
//!
//! # Usage
//!
//! All tools follow the same pattern:
//! 1. Create a tool instance with a working directory
//! 2. Execute with JSON input (validated via schema)
//! 3. Receive result as a string or error
//!
//! ## Using Individual Tools
//!
//! ```rust
//! use saorsa_agent::{ReadTool, Tool};
//! use std::env;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let read_tool = ReadTool::new(env::current_dir()?);
//!
//! let result = read_tool.execute(serde_json::json!({
//!     "file_path": "Cargo.toml",
//!     "line_range": "1-10"
//! })).await?;
//!
//! println!("First 10 lines:\n{}", result);
//! # Ok(())
//! # }
//! ```
//!
//! ## Using the Tool Registry
//!
//! The recommended approach is to use the `default_tools()` function which creates
//! a ToolRegistry with all built-in tools:
//!
//! ```rust
//! use saorsa_agent::{default_tools, Tool};
//! use std::env;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let tools = default_tools(env::current_dir()?);
//!
//! // Access tools by name
//! let read_tool = tools.get("read").unwrap();
//! let result = read_tool.execute(serde_json::json!({
//!     "file_path": "README.md"
//! })).await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Tool Details
//!
//! ## ReadTool
//!
//! Read file contents, optionally limiting to specific line ranges.
//!
//! **Input:**
//! - `file_path` (required): Path to file
//! - `line_range` (optional): Line range like "10-20", "5-", or "-10"
//!
//! **Example:**
//! ```json
//! {
//!   "file_path": "/path/to/file.txt",
//!   "line_range": "10-20"
//! }
//! ```
//!
//! ## WriteTool
//!
//! Write content to files, creating parent directories if needed. Shows diff for existing files.
//!
//! **Input:**
//! - `file_path` (required): Path to file
//! - `content` (required): Content to write
//!
//! **Example:**
//! ```json
//! {
//!   "file_path": "/path/to/file.txt",
//!   "content": "New file content"
//! }
//! ```
//!
//! ## EditTool
//!
//! Surgically edit files by replacing exact text matches. Detects ambiguity if multiple
//! matches found without `replace_all` flag.
//!
//! **Input:**
//! - `file_path` (required): Path to file
//! - `old_text` (required): Exact text to replace
//! - `new_text` (required): Replacement text
//! - `replace_all` (optional): Replace all occurrences (default: false)
//!
//! **Example:**
//! ```json
//! {
//!   "file_path": "/path/to/file.txt",
//!   "old_text": "old content",
//!   "new_text": "new content"
//! }
//! ```
//!
//! ## GrepTool
//!
//! Search file contents using regex patterns. Searches recursively if path is a directory.
//!
//! **Input:**
//! - `pattern` (required): Regex pattern
//! - `path` (required): File or directory to search
//! - `case_insensitive` (optional): Case-insensitive search (default: false)
//!
//! **Example:**
//! ```json
//! {
//!   "pattern": "TODO|FIXME",
//!   "path": "/path/to/project",
//!   "case_insensitive": true
//! }
//! ```
//!
//! ## FindTool
//!
//! Find files by name pattern using glob syntax. Searches recursively.
//!
//! **Input:**
//! - `pattern` (required): Glob pattern (*, ?, \\[abc\\])
//! - `path` (optional): Directory to search (default: working directory)
//!
//! **Example:**
//! ```json
//! {
//!   "pattern": "*.rs",
//!   "path": "/path/to/project"
//! }
//! ```
//!
//! ## LsTool
//!
//! List directory contents with metadata (size, type).
//!
//! **Input:**
//! - `path` (optional): Directory to list (default: working directory)
//! - `recursive` (optional): Recursive listing (default: false)
//!
//! **Example:**
//! ```json
//! {
//!   "path": "/path/to/directory",
//!   "recursive": true
//! }
//! ```
//!
//! ## BashTool
//!
//! Execute shell commands with timeout and output capture.
//!
//! **Input:**
//! - `command` (required): Shell command to execute
//!
//! **Example:**
//! ```json
//! {
//!   "command": "ls -la"
//! }
//! ```
//!
//! # Error Handling
//!
//! All tools return `Result<String, SaorsaAgentError>`. Common errors include:
//! - **File not found** - Path doesn't exist
//! - **Permission denied** - Insufficient permissions
//! - **Invalid input** - JSON schema validation failed
//! - **Pattern error** - Invalid regex or glob pattern
//! - **Ambiguity** - Multiple matches in EditTool without `replace_all`
//!
//! # Security
//!
//! - File operations use the working directory as the security boundary
//! - Commands execute in a sandboxed environment with timeout limits
//! - Output is limited to prevent memory exhaustion
//! - All tool executions are logged for auditability

pub mod bash;
pub mod edit;
pub mod find;
pub mod grep;
pub mod ls;
pub mod read;
pub mod write;

pub use bash::BashTool;
pub use edit::EditTool;
pub use find::FindTool;
pub use grep::GrepTool;
pub use ls::LsTool;
pub use read::ReadTool;
pub use write::WriteTool;

use std::path::{Path, PathBuf};

use similar::{ChangeTag, TextDiff};

/// Resolve a file path relative to a working directory.
///
/// Returns the path as-is if absolute, otherwise joins it with the working directory.
pub(crate) fn resolve_path(working_dir: &Path, path: &str) -> PathBuf {
    let path = Path::new(path);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        working_dir.join(path)
    }
}

/// Generate a unified diff between old and new content.
///
/// The `label` parameter is appended to the `+++` header (e.g., "new", "edited").
pub(crate) fn generate_diff(
    old_content: &str,
    new_content: &str,
    file_path: &Path,
    label: &str,
) -> String {
    let diff = TextDiff::from_lines(old_content, new_content);

    let mut output = String::new();
    output.push_str(&format!("--- {}\n", file_path.display()));
    output.push_str(&format!("+++ {} ({})\n", file_path.display(), label));

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
