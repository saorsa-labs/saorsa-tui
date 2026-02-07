//! Integration tests for the full tool suite.
//!
//! These tests exercise real-world scenarios using all available tools.

use std::fs;
use tempfile::tempdir;

use fae_agent::{tool::ToolRegistry, tools::{BashTool, ReadTool}, Tool};

fn default_bash_tool() -> BashTool {
    BashTool::new(std::env::current_dir().unwrap())
}

fn default_read_tool() -> ReadTool {
    ReadTool::new(std::env::current_dir().unwrap())
}

#[tokio::test]
async fn test_read_write_bash_workflow() {
    // Create a temporary directory for our test
    let dir = tempdir().expect("Failed to create temp dir");
    let file_path = dir.path().join("test_file.txt");
    
    // Write content using BashTool (echo command)
    let bash = default_bash_tool();
    let write_result: Result<String, _> = bash.execute(
        serde_json::json!({
            "command": format!("echo 'Hello, World!' > {}", file_path.to_str().unwrap())
        }),
    ).await;
    assert!(write_result.is_ok(), "Failed to write file: {:?}", write_result);
    
    // Read the file content
    let read = default_read_tool();
    let read_result: Result<String, _> = read.execute(
        serde_json::json!({
            "file_path": file_path.to_str().unwrap()
        }),
    ).await;
    assert!(read_result.is_ok(), "Failed to read file: {:?}", read_result);
    
    let content = read_result.unwrap();
    assert_eq!(content, "Hello, World!\n");
    
    // Append more content
    let append_result: Result<String, _> = bash.execute(
        serde_json::json!({
            "command": format!("echo 'Second line' >> {}", file_path.to_str().unwrap())
        }),
    ).await;
    assert!(append_result.is_ok(), "Failed to append to file: {:?}", append_result);
    
    // Read again to verify append
    let read_result: Result<String, _> = read.execute(
        serde_json::json!({
            "file_path": file_path.to_str().unwrap()
        }),
    ).await;
    assert!(read_result.is_ok(), "Failed to read file: {:?}", read_result);
    
    let content = read_result.unwrap();
    assert!(content.contains("Hello, World!"));
    assert!(content.contains("Second line"));
}

#[tokio::test]
async fn test_read_line_range() {
    let dir = tempdir().expect("Failed to create temp dir");
    let file_path = dir.path().join("multiline.txt");
    
    // Create a multiline file
    let bash = default_bash_tool();
    bash.execute(
        serde_json::json!({
            "command": format!("cat > {} << 'EOF'
Line 1
Line 2
Line 3
Line 4
Line 5
EOF", file_path.to_str().unwrap())
        }),
    ).await.expect("Failed to create multiline file");
    
    let read = default_read_tool();
    
    // Read specific lines (2-4)
    let result: Result<String, _> = read.execute(
        serde_json::json!({
            "file_path": file_path.to_str().unwrap(),
            "line_range": "2-4"
        }),
    ).await;
    assert!(result.is_ok(), "Failed to read line range: {:?}", result);
    
    let content = result.unwrap();
    assert!(content.contains("Line 2"));
    assert!(content.contains("Line 3"));
    assert!(content.contains("Line 4"));
    assert!(!content.contains("Line 1"));
    assert!(!content.contains("Line 5"));
    
    // Read from line 3 onwards
    let result: Result<String, _> = read.execute(
        serde_json::json!({
            "file_path": file_path.to_str().unwrap(),
            "line_range": "3-"
        }),
    ).await;
    assert!(result.is_ok(), "Failed to read from line 3: {:?}", result);
    
    let content = result.unwrap();
    assert!(content.contains("Line 3"));
    assert!(content.contains("Line 4"));
    assert!(content.contains("Line 5"));
    assert!(!content.contains("Line 1"));
    assert!(!content.contains("Line 2"));
}

#[tokio::test]
async fn test_read_error_handling() {
    let read = default_read_tool();
    
    // Try to read non-existent file
    let result: Result<String, _> = read.execute(
        serde_json::json!({
            "file_path": "/nonexistent/file.txt"
        }),
    ).await;
    assert!(result.is_err(), "Expected error for non-existent file");
    
    // Try to read directory as file
    let dir = tempdir().expect("Failed to create temp dir");
    let result: Result<String, _> = read.execute(
        serde_json::json!({
            "file_path": dir.path().to_str().unwrap()
        }),
    ).await;
    assert!(result.is_err(), "Expected error when reading directory");
    
    // Invalid line range format
    let dir = tempdir().expect("Failed to create temp dir");
    let file_path = dir.path().join("test.txt");
    fs::write(&file_path, "test content").unwrap();
    
    let result: Result<String, _> = read.execute(
        serde_json::json!({
            "file_path": file_path.to_str().unwrap(),
            "line_range": "invalid"
        }),
    ).await;
    assert!(result.is_err(), "Expected error for invalid line range");
}

#[tokio::test]
async fn test_bash_working_directory() {
    let dir = tempdir().expect("Failed to create temp dir");
    let file_path = dir.path().join("test.txt");
    
    let bash = BashTool::new(dir.path());
    
    // Execute command in different working directory
    let result: Result<String, _> = bash.execute(
        serde_json::json!({
            "command": format!("echo 'test content' > {}", file_path.to_str().unwrap())
        }),
    ).await;
    assert!(result.is_ok(), "Failed to execute in working directory: {:?}", result);
    
    // Verify file was created in the correct location
    assert!(fs::metadata(&file_path).is_ok(), "File not created in working directory");
    
    // Read the file to verify content
    let read = default_read_tool();
    let read_result: Result<String, _> = read.execute(
        serde_json::json!({
            "file_path": file_path.to_str().unwrap()
        }),
    ).await;
    assert!(read_result.is_ok(), "Failed to read created file: {:?}", read_result);
    assert_eq!(read_result.unwrap(), "test content\n");
}

#[tokio::test]
async fn test_bash_timeout() {
    let bash = BashTool::new(std::env::current_dir().unwrap()).timeout(std::time::Duration::from_millis(1000));
    
    // Execute command with timeout
    let result: Result<String, _> = bash.execute(
        serde_json::json!({
            "command": "sleep 0.1",
            "timeout_ms": 1000
        }),
    ).await;
    assert!(result.is_ok(), "Sleep command should succeed: {:?}", result);
}

#[tokio::test]
async fn test_bash_error_propagation() {
    let bash = default_bash_tool();
    
    // Execute command that should fail
    let result: Result<String, _> = bash.execute(
        serde_json::json!({
            "command": "exit 1"
        }),
    ).await;
    assert!(result.is_err(), "Exit 1 command should fail");
    
    // Check if error contains expected message
    if let Err(e) = result {
        let error_str: String = e.to_string();
        assert!(error_str.contains("exit status: 1") || error_str.contains("command failed"));
    }
}

// Test to verify all tools are accessible through the registry
#[test]
fn test_tool_registry_access() {
    let mut registry = ToolRegistry::new();
    
    // Manually register tools for testing
    registry.register(Box::new(default_bash_tool()));
    registry.register(Box::new(default_read_tool()));
    
    // Verify bash tool is registered
    assert!(registry.get("bash").is_some());
    assert!(registry.get("read").is_some());
    
    // Verify tool types are correct
    if let Some(tool) = registry.get("bash") {
        assert_eq!(tool.name(), "bash");
    }
    
    if let Some(tool) = registry.get("read") {
        assert_eq!(tool.name(), "read");
    }
}
