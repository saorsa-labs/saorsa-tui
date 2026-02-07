# saorsa-agent

Agent runtime for the Saorsa AI framework, providing a complete environment for AI agents to execute tools and interact with the system.

## Overview

The `saorsa-agent` crate implements the core agent loop that enables AI models to:
- Execute tools in a controlled environment
- Manage tool sessions and state
- Handle tool execution errors and recoveries
- Provide a unified interface for multiple LLM providers

## Features

### Agent Core
- **Tool Registry**: Manages available tools and their schemas
- **Session Management**: Maintains context across tool calls
- **Error Recovery**: Handles tool failures gracefully
- **Streaming Support**: Processes streaming responses from LLM providers

### Tool Suite

The agent comes with a comprehensive suite of built-in tools:

#### File Operations Tools

##### Read Tool (`read`)
Read file contents with optional line range filtering.

**Usage:**
```json
{
  "file_path": "/path/to/file.txt"
}
```

**With line range:**
```json
{
  "file_path": "/path/to/file.txt",
  "line_range": "10-20"
}
```

**Examples:**
- Read entire file: `{"file_path": "src/main.rs"}`
- Read lines 5-10: `{"file_path": "src/main.rs", "line_range": "5-10"}`
- Read from line 20 onwards: `{"file_path": "src/main.rs", "line_range": "20-"}`

##### Write Tool (`write`)
Write content to files with automatic directory creation.

**Usage:**
```json
{
  "file_path": "/path/to/file.txt",
  "content": "File content here"
}
```

##### Edit Tool (`edit`)
Perform surgical text replacements in files.

**Usage:**
```json
{
  "file_path": "/path/to/file.txt",
  "old_text": "old text",
  "new_text": "new text",
  "replace_all": false
}
```

##### Grep Tool (`grep`)
Search file contents using regex patterns.

**Usage:**
```json
{
  "pattern": "fn main",
  "path": "/path/to/search",
  "case_insensitive": false
}
```

##### Find Tool (`find`)
Locate files by name patterns.

**Usage:**
```json
{
  "pattern": "*.rs",
  "path": "/path/to/search"
}
```

##### Ls Tool (`ls`)
List directory contents with metadata.

**Usage:**
```json
{
  "path": "/path/to/directory",
  "recursive": false
}
```

#### System Operations Tools

##### Bash Tool (`bash`)
Execute bash commands in a controlled environment.

**Usage:**
```json
{
  "command": "echo 'Hello, World!'"
}
```

**With working directory:**
```json
{
  "command": "ls -la",
  "working_directory": "/tmp"
}
```

**With timeout:**
```json
{
  "command": "sleep 2",
  "timeout_ms": 1000
}
```

## Quick Start

### Basic Agent Usage

```rust
use saorsa_agent::{Agent, ToolRegistry};
use saorsa_ai::AnthropicProvider;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create an agent
    let agent = Agent::new();
    
    // Add your LLM provider
    let anthropic = AnthropicProvider::new()?;
    agent.add_provider("anthropic", Box::new(anthropic));
    
    // Start an agent session
    let mut session = agent.start_session(
        "You are a helpful assistant that can use tools.",
        "anthropic",
    )?;
    
    // Process user messages
    let response = session.process_message(
        "List the files in the current directory",
    ).await?;
    
    println!("Response: {}", response);
    
    Ok(())
}
```

### Using Specific Tools

```rust
use saorsa_agent::tools::{ReadTool, BashTool};

// Read a file
let read_tool = ReadTool;
let result = read_tool.execute(
    &serde_json::json!({
        "file_path": "Cargo.toml"
    }),
    std::env::current_dir()?,
    None,
)?;

println!("File content: {}", result);

// Execute a bash command
let bash_tool = BashTool;
let result = bash_tool.execute(
    "ls -la src/",
    std::env::current_dir()?,
    None,
)?;

println!("Command output: {}", result);
```

## Agent Configuration

### Tool Registry

The agent maintains a registry of available tools:

```rust
use saorsa_agent::Agent;

let agent = Agent::new();
let registry = agent.tool_registry();

// List all available tools
for name in registry.tool_names() {
    println!("Tool: {}", name);
}

// Get a specific tool
if let Some(tool) = registry.get_tool("read") {
    println!("Tool schema: {}", tool.schema());
}
```

### Custom Tools

You can add custom tools to the registry:

```rust
use saorsa_agent::{Tool, ToolRegistry};

struct CustomTool;

impl Tool for CustomTool {
    fn name(&self) -> &'static str {
        "custom"
    }
    
    fn description(&self) -> &'static str {
        "A custom tool implementation"
    }
    
    fn schema(&self) -> &str {
        r#"
        {
          "type": "object",
          "properties": {
            "input": {"type": "string"}
          }
        }
        "#
    }
    
    fn execute(&self, input: &serde_json::Value, _context: ToolContext) -> Result<String, ToolError> {
        // Your tool implementation here
        Ok("Custom tool result".to_string())
    }
}

// Add to registry
let mut registry = ToolRegistry::new();
registry.register_tool(Box::new(CustomTool));
```

## Error Handling

All tool operations return a `Result`:

```rust
use saorsa_agent::ToolError;

match read_tool.execute(input, context, timeout) {
    Ok(output) => println!("Success: {}", output),
    Err(ToolError::FileNotFound(path)) => eprintln!("File not found: {}", path),
    Err(ToolError::PermissionDenied(path)) => eprintln!("Permission denied: {}", path),
    Err(e) => eprintln!("Error: {}", e),
}
```

## Security Considerations

- **File Operations**: Restricted to the working directory tree
- **Command Execution**: Runs in a sandboxed environment with timeout limits
- **Input Validation**: All tool inputs are validated against JSON schema
- **Logging**: All tool executions are logged for auditability

## Development

### Running Tests

```bash
cargo test --all-features
```

### Integration Tests

Run the tool suite integration tests:

```bash
cargo test tool_integration -- --nocapture
```

## License

This project is licensed under either of:

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
  https://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or
  https://opensource.org/licenses/MIT)

at your option.
