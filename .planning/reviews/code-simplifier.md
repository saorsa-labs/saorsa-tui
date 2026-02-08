# Code Simplifier Review

## Findings

### [MEDIUM] Redundant Display::fmt pattern in ThinkingLevel

- File: crates/saorsa-agent/src/config/settings.rs:18-28
- Current: Implements Display with intermediate variable `let s = match self {...}; f.write_str(s)`
- Simpler: Direct write without intermediate binding:
  ```rust
  impl fmt::Display for ThinkingLevel {
      fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
          f.write_str(match self {
              Self::Off => "off",
              Self::Low => "low",
              Self::Medium => "medium",
              Self::High => "high",
          })
      }
  }
  ```

### [LOW] Over-engineered command dispatch with nested helper functions

- File: crates/saorsa/src/commands/mod.rs:65-107
- Current: `dispatch_bookmark()` and `dispatch_model()` are separate helper functions only called from one place
- Simpler: Inline these helpers directly into the main `dispatch()` match arms. Since they're single-use, the abstraction adds indirection without clarity benefit. Alternatively, if the pattern recognition logic is the concern, extract just that logic, not the full dispatch.

### [LOW] AppState dirty flag could use std::cell::Cell

- File: crates/saorsa/src/app.rs:146-178
- Current: Manual dirty flag management with `take_dirty()` and `mark_dirty()` methods
- Simpler: Consider using `Cell<bool>` for interior mutability, which makes the "check and clear" pattern more idiomatic:
  ```rust
  dirty: Cell<bool>,

  pub fn mark_dirty(&self) {
      self.dirty.set(true);
  }

  pub fn take_dirty(&self) -> bool {
      self.dirty.replace(false)
  }
  ```
  This allows marking dirty without `&mut self`, reducing the need to thread mutability through the entire call chain. (Note: This is more of a "consider" than a hard recommendation, as the current approach is perfectly valid.)

### [LOW] Nested tokio::select! with pending future could be simplified

- File: crates/saorsa/src/main.rs:322-378
- Current: Agent event handling uses `std::future::pending()` when no agent is active to prevent the branch from completing
- Simpler: Use `Option<EventReceiver>` and `if let Some(rx) = agent_rx.as_mut()` guard outside the select:
  ```rust
  loop {
      tokio::select! {
          _ = tick_interval.tick() => { /* ... */ }

          event = async {
              match &mut agent_rx {
                  Some(rx) => rx.recv().await,
                  None => std::future::pending().await,
              }
          } => { /* ... */ }

          // ...
      }
  }
  ```
  The current code works but the pending() pattern is somewhat obscure. However, the existing approach is explicit about intent, so this is a minor style preference rather than a clarity issue.

### [MEDIUM] Duplicate string formatting in main.rs model cycling

- File: crates/saorsa/src/main.rs:461-502
- Current: Model cycling (forward and backward) duplicate the "No other models" system message and logic
- Simpler: Extract a helper function for model cycling notification:
  ```rust
  fn notify_model_cycle(state: &mut AppState, new_model: Option<&str>) {
      if let Some(model) = new_model {
          state.add_system_message(format!("Switched to: {model}"));
      } else {
          state.add_system_message(
              "No other models configured. Add models to ~/.saorsa/settings.json"
          );
      }
  }
  ```
  Then use `notify_model_cycle(&mut state, state.cycle_model_forward())` etc.

### [LOW] InputAction::Redraw returned multiple times for input editing

- File: crates/saorsa/src/input.rs:107-138
- Current: Every input editing action (insert, delete, cursor movement) returns `InputAction::Redraw`, but modern code already marks state dirty through AppState methods
- Simpler: Since AppState methods (`insert_char`, `delete_char_before`, `cursor_left`, etc.) all call `state.mark_dirty()` internally, the `InputAction::Redraw` return is redundant. The tick handler will flush and render based on the dirty flag. The explicit Redraw action could be renamed to `InputAction::None` for these cases, simplifying the caller logic.
- Caveat: Looking at main.rs:395-560, only Redraw and a few special actions trigger immediate renders. The dirty flag is for deferred rendering via tick. So the current approach is intentional — input editing triggers immediate visual feedback while async events batch via dirty flag. This is actually **good design**, not over-engineering.

### [MINOR] Common prefix calculation could use itertools

- File: crates/saorsa/src/main.rs:745-761
- Current: Manual common prefix calculation with byte-by-byte comparison
- Simpler: This is a one-off utility with clear logic. Adding itertools as a dependency for one function is **not** simpler. The current implementation is fine.

### [LOW] Command test boilerplate could use test helpers

- File: Multiple command test modules
- Current: Every command test contains:
  ```rust
  #[test]
  fn test_name() {
      let mut state = AppState::new("test");
      let result = execute(...);
      match result { Ok(text) => assert!(...), Err(_) => panic!() }
  }
  ```
- Simpler: Create test helper macros or functions:
  ```rust
  fn expect_ok(result: anyhow::Result<String>) -> String {
      result.expect("command should succeed")
  }
  ```
  Then tests become:
  ```rust
  let text = expect_ok(execute("", &mut state));
  assert!(text.contains("expected"));
  ```
  The current tests already use `#[allow(clippy::unwrap_used, clippy::expect_used)]` and `.expect("should succeed")`, which is reasonable. Further abstraction may reduce clarity for test readers.

### [INFO] RenderThrottle is well-designed

- File: crates/saorsa/src/render_throttle.rs:1-170
- Current: Clean, single-responsibility module with clear API
- Assessment: **No simplification needed.** This is exemplary code — clear purpose, minimal surface area, comprehensive tests.

### [INFO] Command dispatch pattern is appropriate

- File: crates/saorsa/src/commands/mod.rs:1-443
- Current: Central dispatch function with match on command name and alias support
- Assessment: The current design (single dispatch point with helpers for complex subcommands) is the right balance for a CLI command system. Further simplification (like a macro-based command registration system) would be over-engineering for ~15 commands.

## Summary

**5 simplification opportunities found:**
- 1 Medium: ThinkingLevel Display implementation (easy fix, removes intermediate variable)
- 1 Medium: Duplicate model cycling notification logic (extract helper)
- 1 Low: Command dispatch helpers could be inlined (minor)
- 1 Low: AppState dirty flag pattern (optional Cell refactor)
- 1 Low: Nested tokio::select with pending() (style preference)

**Overall Assessment:**

The code is generally **well-structured and appropriately engineered** for a TUI application with async agent interaction. Most patterns (dirty flag tracking, event-driven rendering, command dispatch, render throttling) are correct solutions to real problems, not over-engineering.

The main opportunities are:
1. Small formatting/duplication improvements (ThinkingLevel Display, model cycling)
2. Optional refactors that are more about style than complexity reduction

**No critical over-engineering detected.** The codebase favors explicitness over cleverness, which is the right trade-off for maintainability.

---

**Recommendations:**

1. **Apply:** ThinkingLevel Display simplification (one-line change)
2. **Consider:** Extract model cycling notification helper (reduces duplication)
3. **Leave as-is:** RenderThrottle, command dispatch, dirty flag pattern, InputAction::Redraw semantics
