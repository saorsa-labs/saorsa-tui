# Phase 1.5: Agent Loop (saorsa-agent)

## Goal
Core agent loop with tool execution, bash tool, tool registry, and agent events for UI integration.

## Tasks

### Task 1: Tool Trait & Registry
- `tool.rs`: Tool trait with name, description, input_schema, execute
- ToolRegistry for registering and looking up tools by name
- Convert tool definitions to saorsa-ai ToolDefinition for API calls

### Task 2: Bash Tool
- `tools/bash.rs`: Execute shell commands via tokio::process::Command
- Capture stdout/stderr, enforce timeout
- Working directory support
- Return output as tool result string

### Task 3: Agent Events
- `event.rs`: AgentEvent enum for UI integration
- Events: TurnStart, ToolCall, ToolResult, TextDelta, TurnEnd, Error
- Sender channel for pushing events to UI

### Task 4: Agent Loop
- `agent.rs`: AgentLoop with run() method
- Send messages to provider, handle streaming responses
- Detect tool_use stop reason, execute tools, loop
- Collect text deltas and tool calls from stream events
- Maximum turn limit for safety

### Task 5: Agent Configuration
- `config.rs`: AgentConfig (model, system prompt, max_turns, timeout)
- Builder pattern for configuration

### Task 6: Wire Up lib.rs
- Declare all modules, re-export key types
- Add async-trait dependency if needed
