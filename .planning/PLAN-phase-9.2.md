# Phase 9.2: Scrollback

## Overview

Add scroll offset to AppState so users can scroll through chat history with PageUp/PageDown and mouse wheel. Auto-scroll to bottom on new messages, but preserve scroll position when reading back.

## Tasks

### Task 1: Add scroll_offset to AppState

**Files:** `crates/saorsa/src/app.rs`

Add `scroll_offset: usize` field (number of messages scrolled up from the bottom). Add helper methods:
- `scroll_up(lines: usize)` — increase offset, clamped to max
- `scroll_down(lines: usize)` — decrease offset, clamped to 0
- `scroll_to_bottom()` — set offset to 0
- `is_scrolled_up(&self) -> bool` — offset > 0
- Auto-scroll: `add_*_message()` methods call `scroll_to_bottom()` when at bottom

### Task 2: Add scroll InputAction variants

**Files:** `crates/saorsa/src/input.rs`

Add `InputAction::ScrollUp(usize)` and `InputAction::ScrollDown(usize)` variants.
Handle:
- PageUp → `ScrollUp(half_page)` (use a reasonable default like 10)
- PageDown → `ScrollDown(half_page)`
- Mouse ScrollUp → `ScrollUp(3)`
- Mouse ScrollDown → `ScrollDown(3)`

### Task 3: Update render_messages() for scroll_offset

**Files:** `crates/saorsa/src/ui.rs`

Modify `render_messages()` to use `state.scroll_offset` when calculating which messages to display. Show a scroll indicator when scrolled up.

### Task 4: Handle scroll actions in main loop

**Files:** `crates/saorsa/src/main.rs`

Handle `ScrollUp` and `ScrollDown` in the event loop match.

### Task 5: Tests

Unit tests for scroll_offset clamping, auto-scroll behavior, and render with scroll offset.

## Acceptance

- PageUp/PageDown scroll through history
- Mouse wheel scrolls
- New messages auto-scroll to bottom (unless user has scrolled up)
- Scroll indicator visible when scrolled up
- All tests pass, zero clippy warnings
