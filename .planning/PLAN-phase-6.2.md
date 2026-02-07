# Phase 6.2: Full Tool Suite

## Overview
Implement the complete tool suite for file operations: Read, Write, Edit, Grep, Find, and Ls tools. These tools enable the agent to inspect and modify files in the workspace.

## Tasks

### Task 1: Read Tool Implementation
**Objective**: Implement the Read tool for file reading with optional line ranges.

**Description**:
Create `crates/saorsa-agent/src/tools/read.rs` implementing the Read tool. The tool should:
- Accept a `file_path` (required) and optional `line_range` (e.g., "10-20")
- Read file contents, optionally limiting to specified line range
- Return file contents as a string
- Handle errors gracefully (file not found, permission denied, binary files)
- Include JSON schema for input validation

**Files**:
- `crates/saorsa-agent/src/tools/read.rs` (new)
- `crates/saorsa-agent/src/tools/mod.rs` (export ReadTool)

**Acceptance Criteria**:
- Read tool can read entire files
- Line range filtering works (e.g., "10-20", "5-")
- Error handling for missing files
- Unit tests cover all cases
- Zero clippy warnings

### Task 2: Write Tool Implementation
**Objective**: Implement the Write tool for file writing with diff display.

**Description**:
Create `crates/saorsa-agent/src/tools/write.rs` implementing the Write tool. The tool should:
- Accept `file_path` (required) and `content` (required)
- Write content to file, creating parent directories if needed
- For existing files, generate a diff showing changes
- Return success message with diff if applicable
- Handle permission errors gracefully

**Files**:
- `crates/saorsa-agent/src/tools/write.rs` (new)
- `crates/saorsa-agent/src/tools/mod.rs` (export WriteTool)

**Dependencies**:
- Add `similar` crate for diff generation (already in workspace)

**Acceptance Criteria**:
- Can write new files
- Can overwrite existing files
- Diff output shows changes for existing files
- Creates parent directories automatically
- Unit tests with tempfile
- Zero clippy warnings

### Task 3: Edit Tool Implementation
**Objective**: Implement the Edit tool for surgical file editing with ambiguity detection.

**Description**:
Create `crates/saorsa-agent/src/tools/edit.rs` implementing the Edit tool. The tool should:
- Accept `file_path`, `old_text`, `new_text`, optional `replace_all` flag
- Search for exact match of `old_text` in file
- Detect ambiguity if multiple matches found and `replace_all` is false
- Replace text and return diff showing the change
- Return error if `old_text` not found or ambiguous

**Files**:
- `crates/saorsa-agent/src/tools/edit.rs` (new)
- `crates/saorsa-agent/src/tools/mod.rs` (export EditTool)

**Acceptance Criteria**:
- Single replacement works correctly
- Ambiguity detection when multiple matches
- `replace_all` replaces all occurrences
- Error if `old_text` not found
- Diff output shows changes
- Unit tests cover all scenarios
- Zero clippy warnings

### Task 4: Grep Tool Implementation
**Objective**: Implement the Grep tool for searching file contents with regex.

**Description**:
Create `crates/saorsa-agent/src/tools/grep.rs` implementing the Grep tool. The tool should:
- Accept `pattern` (regex), `path` (file or directory), optional `case_insensitive`
- Search recursively if path is directory
- Return matches with file:line:content format
- Support basic regex patterns
- Limit results to prevent overwhelming output (e.g., max 100 matches)

**Files**:
- `crates/saorsa-agent/src/tools/grep.rs` (new)
- `crates/saorsa-agent/src/tools/mod.rs` (export GrepTool)

**Dependencies**:
- Add `regex` crate for pattern matching
- Add `walkdir` crate for recursive directory traversal

**Acceptance Criteria**:
- Pattern matching works correctly
- Case-insensitive search when requested
- Recursive directory search
- Result limiting to prevent overload
- Unit tests with sample files
- Zero clippy warnings

### Task 5: Find Tool Implementation
**Objective**: Implement the Find tool for locating files by name pattern.

**Description**:
Create `crates/saorsa-agent/src/tools/find.rs` implementing the Find tool. The tool should:
- Accept `pattern` (glob pattern like "*.rs"), `path` (directory to search)
- Search recursively for files matching pattern
- Return list of matching file paths
- Support glob patterns (*, **, ?, [])
- Limit results to prevent overwhelming output

**Files**:
- `crates/saorsa-agent/src/tools/find.rs` (new)
- `crates/saorsa-agent/src/tools/mod.rs` (export FindTool)

**Dependencies**:
- Add `glob` crate or `globset` for pattern matching
- Use `walkdir` for traversal (already added for Grep)

**Acceptance Criteria**:
- Glob pattern matching works
- Recursive search
- Result limiting
- Unit tests with temp directory structure
- Zero clippy warnings

### Task 6: Ls Tool Implementation
**Objective**: Implement the Ls tool for listing directory contents.

**Description**:
Create `crates/saorsa-agent/src/tools/ls.rs` implementing the Ls tool. The tool should:
- Accept `path` (directory to list), optional `recursive` flag
- List files and directories with metadata (size, type, permissions)
- Format output in readable table format
- Support recursive listing if requested
- Handle symlinks gracefully

**Files**:
- `crates/saorsa-agent/src/tools/ls.rs` (new)
- `crates/saorsa-agent/src/tools/mod.rs` (export LsTool)

**Acceptance Criteria**:
- Directory listing with metadata
- Recursive mode works
- Readable output format
- Handles symlinks
- Unit tests with tempfile
- Zero clippy warnings

### Task 7: Tool Registry Integration
**Objective**: Register all new tools in the agent's default tool registry.

**Description**:
Update the agent initialization to register all new tools by default. Create a helper function to build the default tool registry with all available tools.

**Files**:
- `crates/saorsa-agent/src/agent.rs` (add default_tools() helper)
- `crates/saorsa-agent/src/lib.rs` (re-export all tools)

**Acceptance Criteria**:
- `Agent::new()` or similar includes all tools by default
- All tools accessible via registry
- Unit test verifies all tools registered
- Documentation updated
- Zero clippy warnings

### Task 8: Integration Tests & Documentation
**Objective**: Add integration tests and comprehensive documentation for the tool suite.

**Description**:
Create integration tests that exercise all tools in realistic scenarios. Update crate documentation to describe available tools and usage patterns.

**Files**:
- `crates/saorsa-agent/tests/tool_integration.rs` (new)
- `crates/saorsa-agent/src/tools/mod.rs` (module documentation)
- `crates/saorsa-agent/README.md` (tool suite documentation)

**Test Scenarios**:
- Read, edit, and write workflow
- Find files, grep for pattern
- List directory contents
- Error handling across all tools

**Acceptance Criteria**:
- Integration tests pass
- All tools documented with examples
- README explains tool suite
- Zero clippy warnings
- All tests pass

## Dependencies to Add

```toml
[dependencies]
regex = "1.11"
walkdir = "2.5"
globset = "0.4"

[dev-dependencies]
tempfile = "3.14" # Already in workspace
```

## Success Criteria
- All 8 tasks complete
- All tools working and tested
- Zero compilation warnings
- All tests passing
- Documentation complete
