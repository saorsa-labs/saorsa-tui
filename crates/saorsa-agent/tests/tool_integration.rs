//! Integration tests for saorsa-agent tools.
//!
//! These tests exercise tools in realistic workflow scenarios.

#![allow(clippy::unwrap_used)]

use std::fs;
use tempfile::TempDir;

use saorsa_agent::default_tools;

#[tokio::test]
async fn workflow_read_edit_write() {
    // Create a temp directory with a test file
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    fs::write(&file_path, "Line 1\nLine 2\nLine 3\n").unwrap();

    // Create tools with the temp dir as working dir
    let tools = default_tools(temp_dir.path());

    // 1. Read the file
    let read_tool = tools.get("read").unwrap();
    let content = read_tool
        .execute(serde_json::json!({
            "file_path": file_path.to_str().unwrap()
        }))
        .await
        .unwrap();
    assert!(content.contains("Line 1"));

    // 2. Edit a line
    let edit_tool = tools.get("edit").unwrap();
    let edit_output = edit_tool
        .execute(serde_json::json!({
            "file_path": file_path.to_str().unwrap(),
            "old_text": "Line 2",
            "new_text": "Modified Line 2"
        }))
        .await
        .unwrap();
    assert!(edit_output.contains("Replaced text"));

    // 3. Verify with read
    let content = read_tool
        .execute(serde_json::json!({
            "file_path": file_path.to_str().unwrap()
        }))
        .await
        .unwrap();
    assert!(content.contains("Modified Line 2"));
}

#[tokio::test]
async fn workflow_find_and_grep() {
    let temp_dir = TempDir::new().unwrap();

    // Create multiple test files
    let file1 = temp_dir.path().join("test1.rs");
    let file2 = temp_dir.path().join("test2.rs");
    let file3 = temp_dir.path().join("other.txt");

    fs::write(&file1, "fn main() {\n    println!(\"test\");\n}\n").unwrap();
    fs::write(&file2, "fn helper() {\n    println!(\"helper\");\n}\n").unwrap();
    fs::write(&file3, "Some text file\n").unwrap();

    let tools = default_tools(temp_dir.path());

    // 1. Find all .rs files
    let find_tool = tools.get("find").unwrap();
    let found = find_tool
        .execute(serde_json::json!({
            "pattern": "*.rs",
            "path": temp_dir.path().to_str().unwrap()
        }))
        .await
        .unwrap();
    assert!(found.contains("test1.rs"));
    assert!(found.contains("test2.rs"));
    assert!(!found.contains("other.txt"));

    // 2. Grep for "println" in the directory
    let grep_tool = tools.get("grep").unwrap();
    let matches = grep_tool
        .execute(serde_json::json!({
            "pattern": "println",
            "path": temp_dir.path().to_str().unwrap()
        }))
        .await
        .unwrap();
    assert!(matches.contains("test1.rs"));
    assert!(matches.contains("test2.rs"));
}

#[tokio::test]
async fn workflow_list_directory() {
    let temp_dir = TempDir::new().unwrap();

    // Create files and subdirectory
    let file1 = temp_dir.path().join("file1.txt");
    let subdir = temp_dir.path().join("subdir");
    fs::create_dir(&subdir).unwrap();
    let file2 = subdir.join("file2.txt");

    fs::write(&file1, "content1").unwrap();
    fs::write(&file2, "content2").unwrap();

    let tools = default_tools(temp_dir.path());

    // 1. Non-recursive ls
    let ls_tool = tools.get("ls").unwrap();
    let listing = ls_tool
        .execute(serde_json::json!({
            "path": temp_dir.path().to_str().unwrap()
        }))
        .await
        .unwrap();
    assert!(listing.contains("file1.txt"));
    assert!(listing.contains("subdir"));
    assert!(listing.contains("FILE"));
    assert!(listing.contains("DIR"));

    // 2. Recursive ls
    let listing = ls_tool
        .execute(serde_json::json!({
            "path": temp_dir.path().to_str().unwrap(),
            "recursive": true
        }))
        .await
        .unwrap();
    assert!(listing.contains("file1.txt"));
    assert!(listing.contains("file2.txt") || listing.contains("subdir/file2.txt"));
}

#[cfg(unix)]
#[tokio::test]
async fn workflow_bash_and_read() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("bash_output.txt");

    let tools = default_tools(temp_dir.path());

    // 1. Use bash to create a file
    let bash_tool = tools.get("bash").unwrap();
    bash_tool
        .execute(serde_json::json!({
            "command": format!("echo 'Hello from bash' > {}", file_path.to_str().unwrap())
        }))
        .await
        .unwrap();

    // 2. Read the file with read tool
    let read_tool = tools.get("read").unwrap();
    let content = read_tool
        .execute(serde_json::json!({
            "file_path": file_path.to_str().unwrap()
        }))
        .await
        .unwrap();
    assert!(content.contains("Hello from bash"));
}

#[tokio::test]
async fn error_handling_across_tools() {
    let temp_dir = TempDir::new().unwrap();
    let tools = default_tools(temp_dir.path());

    // Test error handling for each tool

    // Read: file not found
    let read_tool = tools.get("read").unwrap();
    let result = read_tool
        .execute(serde_json::json!({
            "file_path": "/nonexistent/file.txt"
        }))
        .await;
    assert!(result.is_err());

    // Write: invalid path (we can't test permission errors in a portable way)
    // Edit: file not found
    let edit_tool = tools.get("edit").unwrap();
    let result = edit_tool
        .execute(serde_json::json!({
            "file_path": "/nonexistent/file.txt",
            "old_text": "old",
            "new_text": "new"
        }))
        .await;
    assert!(result.is_err());

    // Grep: invalid regex
    let grep_tool = tools.get("grep").unwrap();
    let result = grep_tool
        .execute(serde_json::json!({
            "pattern": "[invalid",
            "path": temp_dir.path().to_str().unwrap()
        }))
        .await;
    assert!(result.is_err());

    // Find: invalid glob
    let find_tool = tools.get("find").unwrap();
    let result = find_tool
        .execute(serde_json::json!({
            "pattern": "[invalid",
            "path": temp_dir.path().to_str().unwrap()
        }))
        .await;
    assert!(result.is_err());

    // Ls: path not found
    let ls_tool = tools.get("ls").unwrap();
    let result = ls_tool
        .execute(serde_json::json!({
            "path": "/nonexistent/path"
        }))
        .await;
    assert!(result.is_err());

    // Bash: command failure (Unix only — bash not available on Windows)
    #[cfg(unix)]
    {
        let bash_tool = tools.get("bash").unwrap();
        let result = bash_tool
            .execute(serde_json::json!({
                "command": "exit 1"
            }))
            .await;
        // Bash tool returns Ok with stderr output, so this won't error
        assert!(result.is_ok());
    }
}
