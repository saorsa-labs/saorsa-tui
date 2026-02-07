# Phase 1.6: Minimal Chat App (saorsa-app)

## Goal
Wire saorsa-core, saorsa-ai, and saorsa-agent into a minimal working chat application.

## Tasks

### Task 1: CLI Argument Parsing
- Add clap dependency
- `cli.rs`: Parse --model, --api-key (or ANTHROPIC_API_KEY env), --system-prompt
- Print mode (single prompt from stdin/args) vs interactive mode

### Task 2: App State
- `app.rs`: AppState holding conversation, config, status
- ChatMessage struct for display (role, content, timestamp)
- AppStatus enum (Idle, Thinking, ToolRunning)

### Task 3: Chat UI Layout
- `ui.rs`: Render function using saorsa-core widgets
- Header (model name, status), message area, input footer
- Use Container, Label, Layout from saorsa-core

### Task 4: Input Handling
- `input.rs`: Single-line input editor
- Key handling: Enter to submit, Ctrl-C to quit, basic editing
- Integration with saorsa-core event system

### Task 5: Main Loop
- `main.rs`: Async main with tokio runtime
- Terminal setup/teardown, event loop
- Connect agent events to UI updates
- Streaming text display

### Task 6: Wire Up saorsa-cli
- saorsa-cli main.rs delegates to saorsa-app
