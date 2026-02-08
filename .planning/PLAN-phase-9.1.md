# Phase 9.1: Render Throttling

## Overview

This phase implements frame rate control and render batching to improve input responsiveness and reduce unnecessary rendering during AI streaming. Currently, `render_ui()` is called on every `TextDelta` event, which can cause input lag during high-frequency streaming. We'll introduce a 30fps frame cap, batch text deltas, and skip no-op renders.

## Goals

1. **30fps Frame Cap**: Limit renders to 30 frames per second using `Instant` tracking
2. **TextDelta Batching**: Accumulate streaming text during a frame interval, render once per frame
3. **Dirty Tracking**: Skip renders when nothing has changed
4. **No Behavior Changes**: Maintain current UX, only improve performance

## Tasks

### Task 1: Create RenderThrottle Module

**Files:**
- `crates/saorsa/src/render_throttle.rs` (new)
- `crates/saorsa/src/lib.rs` (add module declaration)

**Description:**
Create a `RenderThrottle` struct that tracks the last render time and enforces a configurable frame rate limit. Default to 30fps (33.33ms between frames).

**API:**
```rust
pub struct RenderThrottle {
    frame_duration: Duration,
    last_render: Instant,
}

impl RenderThrottle {
    /// Create a new throttle with the given FPS limit.
    pub fn new(fps: u32) -> Self;
    
    /// Create a 30fps throttle (default).
    pub fn default_fps() -> Self;
    
    /// Check if enough time has passed to allow a render.
    /// Returns true if rendering should proceed.
    pub fn should_render(&self) -> bool;
    
    /// Mark that a render has occurred (update last_render to now).
    pub fn mark_rendered(&mut self);
    
    /// Get time until next allowed render.
    pub fn time_until_next_frame(&self) -> Duration;
}
```

**Tests:**
- `new()` creates throttle with correct frame duration
- `should_render()` returns true immediately after creation
- `should_render()` returns false immediately after `mark_rendered()`
- `should_render()` returns true after waiting one frame duration
- `default_fps()` creates 30fps throttle (33.33ms frame time)
- `time_until_next_frame()` returns correct remaining duration

**Acceptance:**
- All tests pass
- Zero warnings from clippy
- Fully documented with examples

---

### Task 2: Add Dirty Flag to AppState

**Files:**
- `crates/saorsa/src/app.rs`

**Description:**
Add a `dirty: bool` field to `AppState` that tracks whether the UI needs to be re-rendered. Default to `true` (initial render needed). Add helper methods to mark dirty and check/clear the flag.

**API:**
```rust
impl AppState {
    /// Mark the state as dirty (needs re-render).
    pub fn mark_dirty(&mut self);
    
    /// Check if the state is dirty and clear the flag.
    /// Returns true if a render is needed.
    pub fn take_dirty(&mut self) -> bool;
}
```

**Changes:**
- Add `dirty: bool` field to `AppState` struct
- Initialize to `true` in `new()`
- Call `mark_dirty()` in all mutation methods: `add_user_message()`, `add_assistant_message()`, `add_tool_message()`, `add_system_message()`, `insert_char()`, `delete_char_before()`, `cursor_left()`, `cursor_right()`

**Tests:**
- `new()` state is dirty initially
- `take_dirty()` returns true first time, false second time
- `mark_dirty()` sets dirty flag
- All message/input methods mark state dirty

**Acceptance:**
- All existing tests pass
- New tests for dirty tracking pass
- Zero clippy warnings

---

### Task 3: Add TextDelta Batching to AppState

**Files:**
- `crates/saorsa/src/app.rs`

**Description:**
Add a `pending_stream_text: String` field that accumulates `TextDelta` events between renders. Add methods to accumulate text and flush it to `streaming_text`.

**API:**
```rust
impl AppState {
    /// Accumulate a text delta without marking dirty (batching).
    pub fn accumulate_stream_text(&mut self, text: &str);
    
    /// Flush pending stream text to streaming_text and mark dirty.
    /// Returns true if any text was flushed.
    pub fn flush_stream_text(&mut self) -> bool;
}
```

**Changes:**
- Add `pending_stream_text: String` field to `AppState`
- Initialize to empty in `new()`
- `accumulate_stream_text()` appends to `pending_stream_text` without marking dirty
- `flush_stream_text()` moves `pending_stream_text` to `streaming_text`, marks dirty, returns `!pending_stream_text.is_empty()`

**Tests:**
- `accumulate_stream_text()` appends to pending buffer
- `flush_stream_text()` moves pending to streaming_text
- `flush_stream_text()` marks state dirty
- Multiple accumulates followed by single flush works correctly

**Acceptance:**
- All tests pass
- No behavior changes yet (not integrated into main loop)
- Zero clippy warnings

---

### Task 4: Integrate RenderThrottle into Main Loop

**Files:**
- `crates/saorsa/src/main.rs`

**Description:**
Integrate `RenderThrottle` into the `run_interactive()` event loop. Replace immediate `render_ui()` calls with throttled renders that respect the frame rate limit and dirty flag.

**Changes:**
1. Create `RenderThrottle::default_fps()` before event loop
2. Wrap `render_ui()` in a helper that checks `throttle.should_render()` and `state.take_dirty()`
3. Add a periodic "tick" to the event loop using `tokio::time::interval()` at 30fps
4. On each tick, flush stream text and render if dirty
5. For `InputAction::Redraw`, mark dirty but don't force immediate render
6. For `InputAction::Submit`, force immediate render after marking `Thinking`

**Implementation Pattern:**
```rust
// Helper function
fn maybe_render_ui(
    state: &mut AppState,
    ctx: &mut RenderContext,
    backend: &mut CrosstermBackend,
    throttle: &mut RenderThrottle,
) {
    if throttle.should_render() && state.take_dirty() {
        render_ui(state, ctx, backend);
        throttle.mark_rendered();
    }
}

// In run_interactive(), add tick stream
let mut tick_interval = tokio::time::interval(Duration::from_millis(33)); // ~30fps
let mut throttle = RenderThrottle::default_fps();

loop {
    tokio::select! {
        _ = tick_interval.tick() => {
            state.flush_stream_text();
            maybe_render_ui(&mut state, &mut ctx, &mut backend, &mut throttle);
        }
        maybe_event = event_stream.next() => {
            // ... handle events, mark dirty, maybe_render_ui() instead of render_ui()
        }
    }
}
```

**Acceptance:**
- App still renders correctly
- Renders are limited to ~30fps
- No visual glitches
- Input feels responsive

---

### Task 5: Update run_agent_interaction() for Batching

**Files:**
- `crates/saorsa/src/main.rs`

**Description:**
Modify `run_agent_interaction()` to use `accumulate_stream_text()` instead of directly appending to `streaming_text` and calling `render_ui()`. Let the main loop's tick handle rendering.

**Changes:**
1. Replace `state.streaming_text.push_str(&text); render_ui(...)` with `state.accumulate_stream_text(&text)`
2. Keep immediate `render_ui()` calls for non-streaming events (ToolCall, ToolResult, TextComplete)
3. Mark state dirty after events that change visible state

**Rationale:**
TextDelta events are high-frequency and benefit from batching. Other events (tool start/finish) are low-frequency and should render immediately for responsiveness.

**Acceptance:**
- Streaming responses still appear smooth
- Tool status changes appear immediately
- No text is lost or delayed excessively
- Reduced render frequency during streaming (observable via tracing logs)

---

### Task 6: Add Tracing for Render Performance

**Files:**
- `crates/saorsa/src/main.rs`

**Description:**
Add tracing instrumentation to `render_ui()` and the throttle helper to enable performance monitoring.

**Changes:**
1. Add `tracing::debug!` in `maybe_render_ui()` when skipping a render (throttled or not dirty)
2. Add `tracing::debug!` in `render_ui()` with frame number or timestamp
3. Add `tracing::trace!` for TextDelta accumulation count

**Acceptance:**
- `RUST_LOG=saorsa=debug cargo run` shows render skip/execute decisions
- Easy to verify frame rate is ~30fps
- Easy to see batching effectiveness (N deltas per render)

---

### Task 7: Integration Testing and Validation

**Files:**
- `crates/saorsa/tests/render_throttle_integration.rs` (new)

**Description:**
Write integration tests that verify throttling and batching behavior without requiring a live LLM.

**Tests:**
1. Mock rapid state changes (100 dirty marks in 1 second) - verify <50 renders occur
2. Mock streaming text (1000 TextDelta events) - verify text eventually appears in full
3. Verify dirty flag prevents renders when nothing changed
4. Verify initial render always happens (dirty=true on startup)

**Acceptance:**
- All integration tests pass
- Coverage for throttle, batching, and dirty tracking
- No flakiness (time-based tests use generous tolerances)

---

## Testing Strategy

- **Unit tests** for `RenderThrottle` time calculations
- **Unit tests** for `AppState` dirty flag and batching logic
- **Integration tests** for end-to-end throttle behavior
- **Manual testing** with `cargo run` and high-frequency streaming

## Success Criteria

1. ✅ Render rate capped at ~30fps during streaming
2. ✅ Input remains responsive (no lag)
3. ✅ No visual glitches or missing text
4. ✅ Zero clippy warnings, all tests pass
5. ✅ Tracing logs confirm batching effectiveness

## Rollback Plan

If performance degrades or visual bugs appear:
1. Remove tick interval from main loop
2. Restore direct `render_ui()` calls in event handlers
3. Keep `RenderThrottle` module for future use

## Future Enhancements (Not in This Phase)

- Adaptive FPS based on terminal update cost
- Per-widget dirty tracking (render only changed regions)
- Frame timing histogram metrics
- Configurable FPS via CLI/settings
