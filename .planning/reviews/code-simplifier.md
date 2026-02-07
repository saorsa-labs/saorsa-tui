# Code Simplification Review

**Date**: 2026-02-07
**Scope**: Files changed in last 5 commits + broader tool crate scan
**Mode**: Analysis only (no changes)

## Files Reviewed

### Primary (changed in last 5 commits)
- `crates/saorsa-agent/src/session/path.rs`
- `crates/saorsa-agent/src/tools/bash.rs`
- `crates/saorsa-agent/tests/tool_integration.rs`

### Broader scan
- `crates/saorsa-agent/src/tools/read.rs`
- `crates/saorsa-agent/src/tools/write.rs`
- `crates/saorsa-agent/src/tools/edit.rs`
- `crates/saorsa-agent/src/tools/grep.rs`
- `crates/saorsa-agent/src/tools/find.rs`
- `crates/saorsa-agent/src/tools/ls.rs`
- `crates/saorsa-agent/src/tools/mod.rs`
- `crates/saorsa-agent/src/tool.rs`
- `crates/saorsa-agent/src/agent.rs`

---

## Finding 1: Duplicated `resolve_path` method across 5 tool structs

**Severity**: Medium (redundancy / DRY violation)
**Files**: `read.rs`, `write.rs`, `edit.rs`, `grep.rs`, `find.rs`, `ls.rs`

The exact same path resolution logic is copy-pasted into 5 of the 7 tool structs (ReadTool, WriteTool, EditTool, GrepTool have the identical `fn resolve_path(&self, path: &str) -> PathBuf` signature; FindTool and LsTool have a slightly different `fn resolve_path(&self, path: Option<&str>) -> PathBuf` variant). The body is identical:

```rust
fn resolve_path(&self, path: &str) -> PathBuf {
    let path = Path::new(path);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        self.working_dir.join(path)
    }
}
```

**Recommendation**: Extract a standalone function `fn resolve_path(working_dir: &Path, path: &str) -> PathBuf` in the `tools/mod.rs` module (or a shared utility), and have all tools call it. The `Option<&str>` variant used by FindTool and LsTool can be a trivial wrapper. This eliminates 6 copies of the same logic.

---

## Finding 2: Duplicated `generate_diff` method in WriteTool and EditTool

**Severity**: Medium (redundancy)
**Files**: `write.rs` (line 46), `edit.rs` (line 51)

Both WriteTool and EditTool contain nearly identical `generate_diff` methods. The only difference is the header label: WriteTool uses `"(new)"` while EditTool uses `"(edited)"`.

```rust
// write.rs
output.push_str(&format!("+++ {} (new)\n", file_path.display()));

// edit.rs
output.push_str(&format!("+++ {} (edited)\n", file_path.display()));
```

**Recommendation**: Extract a shared `fn generate_diff(old: &str, new: &str, path: &Path, label: &str) -> String` into `tools/mod.rs`. Both tools call it with their specific label.

---

## Finding 3: Redundant test patterns in `path.rs` -- `assert!(x.is_ok()) + match { Ok => ..., Err => unreachable!() }`

**Severity**: Low (unnecessary verbosity in tests)
**File**: `crates/saorsa-agent/src/session/path.rs`, lines 78-116

Every test in path.rs follows this verbose pattern:

```rust
let dir = sessions_dir();
assert!(dir.is_ok());
match dir {
    Ok(path) => {
        assert!(path.to_string_lossy().contains("xdg_test"));
    }
    Err(_) => unreachable!(),
}
```

Since the test already asserts `is_ok()`, the `Err(_) => unreachable!()` branch is dead code. This is test code, so `.unwrap()` is permitted by project standards. The simpler form would be:

```rust
let path = sessions_dir().unwrap();
assert!(path.to_string_lossy().contains("xdg_test"));
```

This applies to all 6 tests in this file.

---

## Finding 4: `ensure_dir` has a TOCTOU race condition

**Severity**: Low (correctness)
**File**: `crates/saorsa-agent/src/session/path.rs`, lines 60-67

```rust
pub fn ensure_dir(path: &Path) -> Result<(), SaorsaAgentError> {
    if !path.exists() {
        std::fs::create_dir_all(path).map_err(|e| { ... })?;
    }
    Ok(())
}
```

The existence check before `create_dir_all` is unnecessary. `create_dir_all` is already idempotent -- it succeeds silently if the directory already exists. The check-then-act introduces a TOCTOU race (between `exists()` and `create_dir_all()`, another process could create or remove the directory). The entire function can be simplified to just the `create_dir_all` call.

**Recommendation**:
```rust
pub fn ensure_dir(path: &Path) -> Result<(), SaorsaAgentError> {
    std::fs::create_dir_all(path).map_err(|e| {
        SaorsaAgentError::Session(format!("Failed to create directory {:?}: {}", path, e))
    })
}
```

---

## Finding 5: Duplicated `working_dir: PathBuf` field across all tool structs

**Severity**: Low (structural redundancy)
**Files**: All 7 tool files

Every tool struct has the same field:
```rust
pub struct XxxTool {
    working_dir: PathBuf,
}
```

And the same constructor pattern:
```rust
pub fn new(working_dir: impl Into<PathBuf>) -> Self {
    Self { working_dir: working_dir.into() }
}
```

**Recommendation**: This is a borderline finding. A shared `ToolBase` struct or a common trait could reduce this, but the current approach is clear and explicit. If more fields are added to tools in the future (e.g., config, permissions), this duplication would compound. For now, this is acceptable but worth noting.

---

## Finding 6: Redundant `assert!(result.is_ok())` before `result.unwrap()` in integration tests

**Severity**: Low (noise)
**File**: `crates/saorsa-agent/tests/tool_integration.rs`

Many tests have the pattern:
```rust
assert!(read_result.is_ok());
let content = read_result.unwrap();
```

The `assert!` is redundant since `unwrap()` on the next line would panic with the error displayed. Either use just `unwrap()` or use `expect("descriptive message")` for better error messages. The `assert!` + `unwrap()` pattern provides no additional value.

**Recommendation**: Use just `.unwrap()` or `.expect("context")` in tests. The `.unwrap()` already panics with the Err value when it fails, which is more informative than `assertion failed: result.is_ok()`.

---

## Finding 7: Double JSON parsing in `agent.rs::execute_tool_calls`

**Severity**: Low (unnecessary allocation)
**File**: `crates/saorsa-agent/src/agent.rs`, lines 155 and 231

In the `run` method, tool call JSON is parsed once at line 155 to emit the event, then the raw JSON string is stored in the assistant content block. Later in `execute_tool_calls`, the same JSON string is parsed again at line 231. The input is parsed twice for each tool call.

**Recommendation**: Parse the JSON once and pass the parsed `serde_json::Value` to `execute_tool_calls` instead of having it re-parse the string. This requires changing `ToolCallInfo` to store a `serde_json::Value` instead of (or alongside) the raw string.

---

## Finding 8: `unwrap_or` with `serde_json::Value::Object(serde_json::Map::new())` in agent.rs

**Severity**: Low (readability)
**File**: `crates/saorsa-agent/src/agent.rs`, lines 155-156 and 231-232

```rust
let input: serde_json::Value = serde_json::from_str(&tc.input_json)
    .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
```

This appears twice. The intent is "default to empty object on parse failure", but the production code standard prohibits `unwrap_or` variants silently swallowing errors. A malformed tool call JSON being silently replaced with `{}` could cause confusing downstream errors.

**Recommendation**: At minimum, log a warning when the JSON fails to parse. Ideally, surface it as an error to the LLM via the tool result so it can correct its output.

---

## Finding 9: `BashTool::truncate_output` may split on a multi-byte character boundary

**Severity**: Low (correctness edge case)
**File**: `crates/saorsa-agent/src/tools/bash.rs`, line 44

```rust
let truncated = &output[..MAX_OUTPUT_BYTES];
```

If the output contains multi-byte UTF-8 characters, slicing at an arbitrary byte offset could panic. This is safe in practice because `String::from_utf8_lossy` was used to construct the output (which guarantees valid UTF-8), but truncating at a byte boundary within a multi-byte sequence will panic.

**Recommendation**: Use `output.char_indices()` to find the last valid character boundary at or before `MAX_OUTPUT_BYTES`, or use `output.get(..MAX_OUTPUT_BYTES).unwrap_or(output)` to handle it gracefully. Alternatively, use `floor_char_boundary` (nightly) or a manual scan.

---

## Finding 10: `LsTool::entry_type` calls `fs::metadata` redundantly

**Severity**: Low (unnecessary I/O)
**File**: `crates/saorsa-agent/src/tools/ls.rs`, lines 70-83

In both the recursive and non-recursive listing branches, the code already fetches metadata for size calculation. Then `entry_type` is called, which fetches metadata again via `fs::metadata(path)`. This doubles the syscalls per entry.

**Recommendation**: Pass the already-fetched `Metadata` to `entry_type` instead of having it re-read from the filesystem.

---

## Summary

| # | Finding | Severity | Category |
|---|---------|----------|----------|
| 1 | Duplicated `resolve_path` across 6 tool files | Medium | DRY violation |
| 2 | Duplicated `generate_diff` in WriteTool and EditTool | Medium | DRY violation |
| 3 | Verbose test patterns in path.rs (assert + match + unreachable) | Low | Readability |
| 4 | TOCTOU race in `ensure_dir` (unnecessary existence check) | Low | Correctness |
| 5 | Duplicated `working_dir` field and constructor across all tools | Low | Structural |
| 6 | Redundant `assert!(is_ok())` before `unwrap()` in integration tests | Low | Readability |
| 7 | Double JSON parsing of tool call input in agent.rs | Low | Performance |
| 8 | Silent error swallowing on malformed tool call JSON | Low | Error handling |
| 9 | Byte-boundary truncation may panic on multi-byte UTF-8 | Low | Correctness |
| 10 | Redundant `fs::metadata` call in `LsTool::entry_type` | Low | Performance |

## Grade: B+

The code is well-structured and readable overall. The most impactful improvements would be extracting the duplicated `resolve_path` and `generate_diff` helpers (findings 1 and 2), which would eliminate roughly 60 lines of duplicated logic. The remaining findings are minor cleanups that improve correctness and clarity without changing behavior.
