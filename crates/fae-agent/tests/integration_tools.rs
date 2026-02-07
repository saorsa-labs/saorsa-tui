//! Integration tests for individual fae-agent tools.
//!
//! Tests the Read and Grep tools in detail with mock data.

#![allow(clippy::unwrap_used)]

use std::fs;
use std::io::Write;
use tempfile::{NamedTempFile, TempDir};

use fae_agent::Tool;
use fae_agent::tools::{GrepTool, ReadTool};

#[tokio::test]
async fn read_tool_full_file() {
    let mut temp = NamedTempFile::new().unwrap();
    writeln!(temp, "Line 1").unwrap();
    writeln!(temp, "Line 2").unwrap();
    writeln!(temp, "Line 3").unwrap();
    temp.flush().unwrap();

    let tool = ReadTool::new(std::env::current_dir().unwrap());
    let input = serde_json::json!({
        "file_path": temp.path().to_str().unwrap()
    });

    let result = tool.execute(input).await;
    assert!(result.is_ok());

    let content = result.unwrap();
    assert!(content.contains("Line 1"));
    assert!(content.contains("Line 2"));
    assert!(content.contains("Line 3"));
}

#[tokio::test]
async fn read_tool_line_range_full() {
    let mut temp = NamedTempFile::new().unwrap();
    for i in 1..=10 {
        writeln!(temp, "Line {i}").unwrap();
    }
    temp.flush().unwrap();

    let tool = ReadTool::new(std::env::current_dir().unwrap());
    let input = serde_json::json!({
        "file_path": temp.path().to_str().unwrap(),
        "line_range": "3-7"
    });

    let result = tool.execute(input).await;
    assert!(result.is_ok());

    let content = result.unwrap();
    assert!(!content.contains("Line 1"));
    assert!(!content.contains("Line 2"));
    assert!(content.contains("Line 3"));
    assert!(content.contains("Line 7"));
    assert!(!content.contains("Line 8"));
}

#[tokio::test]
async fn read_tool_line_range_from() {
    let mut temp = NamedTempFile::new().unwrap();
    for i in 1..=10 {
        writeln!(temp, "Line {i}").unwrap();
    }
    temp.flush().unwrap();

    let tool = ReadTool::new(std::env::current_dir().unwrap());
    let input = serde_json::json!({
        "file_path": temp.path().to_str().unwrap(),
        "line_range": "8-"
    });

    let result = tool.execute(input).await;
    assert!(result.is_ok());

    let content = result.unwrap();
    assert!(!content.contains("Line 7"));
    assert!(content.contains("Line 8"));
    assert!(content.contains("Line 9"));
    assert!(content.contains("Line 10"));
}

#[tokio::test]
async fn read_tool_line_range_to() {
    let mut temp = NamedTempFile::new().unwrap();
    for i in 1..=10 {
        writeln!(temp, "Line {i}").unwrap();
    }
    temp.flush().unwrap();

    let tool = ReadTool::new(std::env::current_dir().unwrap());
    let input = serde_json::json!({
        "file_path": temp.path().to_str().unwrap(),
        "line_range": "-3"
    });

    let result = tool.execute(input).await;
    assert!(result.is_ok());

    let content = result.unwrap();
    assert!(content.contains("Line 1"));
    assert!(content.contains("Line 2"));
    assert!(content.contains("Line 3"));
    assert!(!content.contains("Line 4"));
}

#[tokio::test]
async fn read_tool_nonexistent_file() {
    let tool = ReadTool::new(std::env::current_dir().unwrap());
    let input = serde_json::json!({
        "file_path": "/nonexistent/missing.txt"
    });

    let result = tool.execute(input).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn read_tool_metadata() {
    let tool = ReadTool::new(std::env::current_dir().unwrap());

    assert_eq!(tool.name(), "read");
    assert!(tool.description().contains("Read file"));

    let schema = tool.input_schema();
    assert!(schema["properties"]["file_path"]["type"] == "string");
    assert!(
        schema["required"]
            .as_array()
            .unwrap()
            .contains(&"file_path".into())
    );

    let def = tool.to_definition();
    assert_eq!(def.name, "read");
    assert!(def.description.contains("Read file"));
}

#[tokio::test]
async fn grep_tool_simple_match() {
    let mut temp = NamedTempFile::new().unwrap();
    writeln!(temp, "Hello World").unwrap();
    writeln!(temp, "Goodbye World").unwrap();
    writeln!(temp, "Test line").unwrap();
    temp.flush().unwrap();

    let tool = GrepTool::new(std::env::current_dir().unwrap());
    let input = serde_json::json!({
        "pattern": "World",
        "path": temp.path().to_str().unwrap()
    });

    let result = tool.execute(input).await;
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(output.contains("Found 2 match"));
    assert!(output.contains("Hello World"));
    assert!(output.contains("Goodbye World"));
    assert!(!output.contains("Test line"));
}

#[tokio::test]
async fn grep_tool_regex_pattern() {
    let mut temp = NamedTempFile::new().unwrap();
    writeln!(temp, "error: something failed").unwrap();
    writeln!(temp, "warning: deprecated API").unwrap();
    writeln!(temp, "info: starting up").unwrap();
    temp.flush().unwrap();

    let tool = GrepTool::new(std::env::current_dir().unwrap());
    let input = serde_json::json!({
        "pattern": r"^(error|warning):",
        "path": temp.path().to_str().unwrap()
    });

    let result = tool.execute(input).await;
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(output.contains("Found 2 match"));
    assert!(output.contains("error: something failed"));
    assert!(output.contains("warning: deprecated API"));
    assert!(!output.contains("info: starting up"));
}

#[tokio::test]
async fn grep_tool_case_insensitive() {
    let mut temp = NamedTempFile::new().unwrap();
    writeln!(temp, "TODO: fix this").unwrap();
    writeln!(temp, "todo: also this").unwrap();
    writeln!(temp, "ToDo: and this").unwrap();
    temp.flush().unwrap();

    let tool = GrepTool::new(std::env::current_dir().unwrap());
    let input = serde_json::json!({
        "pattern": "todo",
        "path": temp.path().to_str().unwrap(),
        "case_insensitive": true
    });

    let result = tool.execute(input).await;
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(output.contains("Found 3 match"));
}

#[tokio::test]
async fn grep_tool_directory_search() {
    let temp_dir = TempDir::new().unwrap();

    let file1 = temp_dir.path().join("file1.txt");
    let file2 = temp_dir.path().join("file2.txt");

    fs::write(&file1, "test in file1\nother content\n").unwrap();
    fs::write(&file2, "test in file2\n").unwrap();

    let tool = GrepTool::new(std::env::current_dir().unwrap());
    let input = serde_json::json!({
        "pattern": "test",
        "path": temp_dir.path().to_str().unwrap()
    });

    let result = tool.execute(input).await;
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(output.contains("Found 2 match"));
    assert!(output.contains("file1.txt"));
    assert!(output.contains("file2.txt"));
}

#[tokio::test]
async fn grep_tool_no_matches() {
    let mut temp = NamedTempFile::new().unwrap();
    writeln!(temp, "Some content").unwrap();
    writeln!(temp, "More content").unwrap();
    temp.flush().unwrap();

    let tool = GrepTool::new(std::env::current_dir().unwrap());
    let input = serde_json::json!({
        "pattern": "notfound",
        "path": temp.path().to_str().unwrap()
    });

    let result = tool.execute(input).await;
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(output.contains("No matches found"));
}

#[tokio::test]
async fn grep_tool_invalid_regex() {
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
}

#[tokio::test]
async fn grep_tool_metadata() {
    let tool = GrepTool::new(std::env::current_dir().unwrap());

    assert_eq!(tool.name(), "grep");
    assert!(tool.description().contains("Search"));
    assert!(tool.description().contains("regex"));

    let schema = tool.input_schema();
    assert!(schema["properties"]["pattern"]["type"] == "string");
    assert!(schema["properties"]["path"]["type"] == "string");
    assert!(
        schema["required"]
            .as_array()
            .unwrap()
            .contains(&"pattern".into())
    );
    assert!(
        schema["required"]
            .as_array()
            .unwrap()
            .contains(&"path".into())
    );

    let def = tool.to_definition();
    assert_eq!(def.name, "grep");
}
