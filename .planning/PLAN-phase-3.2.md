# Phase 3.2: Renderer Integration & Advanced Rendering

**Milestone**: 3 — Compositor & Advanced Rendering
**Phase**: 3.2 — Renderer Integration & Advanced Rendering
**Status**: Planned
**Estimated Tests**: 80+

## Overview

Integrate the compositor with the existing renderer pipeline, implement delta rendering optimization, advanced text styling (underline, strikethrough, blink), and scrollable regions with clipping. This phase connects the high-level compositor output to the low-level terminal rendering.

### Current Architecture (end of Phase 3.1)

```
Widget.render_segments(area) → Vec<Vec<Segment>>
     │
     ▼
Compositor (resolve overlapping layers)
     │
     ▼
ScreenBuffer ← write cells
     │
     ▼
RenderContext.end_frame() → diff → Renderer → ANSI
```

### Phase 3.2 Goals

1. **Compositor Integration** — Hook Compositor into RenderContext rendering pipeline
2. **Delta Rendering** — Compare old/new buffers, send only changed cells (ANSI optimization)
3. **Advanced Styling** — Underline, strikethrough, blink, inverse, bold, italic combinations
4. **Scrollable Regions** — Implement viewport clipping and scroll offset management
5. **Unicode Edge Cases** — Handle emoji, RTL, combining marks across layer boundaries

## Dependencies

- Phase 3.1 complete (Compositor core: 671 tests)
- Existing renderer infrastructure from Phase 1

---

## Task 1: Renderer Integration with Compositor

**Files to modify/create:**
- `crates/fae-core/src/renderer.rs` — add compositor integration
- `crates/fae-core/src/render_context.rs` — hook into pipeline

**Description**: Modify the rendering pipeline to call the compositor before writing to terminal. The compositor becomes the data transformation layer between the widget tree output and the screen buffer.

**Changes:**
- Add `compositor: Option<Compositor>` field to `RenderContext`
- Add `with_compositor(compositor: Compositor)` builder method
- In `end_frame()`, call `compositor.compose(&mut buffer)` before diff/render
- Handle case where compositor is None (legacy mode)

**Tests (8+):**
1. RenderContext with compositor set
2. RenderContext without compositor (None)
3. end_frame() with compositor composes layers
4. Multiple widgets at different z-levels render correctly
5. Compositor errors propagate properly
6. Screen resize updates compositor dimensions
7. compositor.layers() accessible for debugging
8. Integration: widget renders segments → compositor composes → renderer outputs ANSI

---

## Task 2: Delta Rendering Optimization

**Files to create:**
- `crates/fae-core/src/renderer/delta.rs` (new module)

**Description**: Implement differential rendering to minimize terminal output. Only send ANSI escape sequences for cells that actually changed.

**New types:**
- `DeltaRenderer` — compares old/new buffers, generates minimal ANSI diff
- `CellDelta` — represents a cell change (position, old cell, new cell)

**Functions:**
- `compute_delta(old: &ScreenBuffer, new: &ScreenBuffer) -> Vec<CellDelta>`
- `render_delta(writer, delta: &[CellDelta])` — emit ANSI for changes only
- `should_emit_style_change(old_style, new_style) -> bool` — detect style changes

**Optimizations:**
- Skip cells that haven't changed (compare grapheme + style)
- Batch consecutive cells in same row with identical style
- Use cursor movement optimization (nearby cells)
- Cache last emitted style to minimize "reset + restyle" sequences

**Tests (10+):**
1. No changes → empty delta
2. Single cell change → one delta entry
3. Multiple cells in same row → batched
4. Style change only → cell content same
5. Content and style change → combined
6. Wide character boundary → handled correctly
7. Delta rendering produces fewer ANSI bytes than full render
8. Render delta to writer → correct output
9. All cells changed → same as full render
10. Empty buffer to populated → all deltas present

---

## Task 3: Advanced Text Styling — Attributes

**Files to modify:**
- `crates/fae-core/src/style.rs` — extend Style type

**Description**: Add support for advanced text attributes: underline, strikethrough, blink, inverse, bold, italic. These can be combined.

**New fields on Style:**
- `underline: bool` (SGR 4)
- `strikethrough: bool` (SGR 9)
- `blink: bool` (SGR 5, slow)
- `inverse: bool` (SGR 7)
- Bold and italic already exist

**ANSI Mapping:**
- `Style { bold: true, underline: true }` → `\x1b[1;4m`
- `Style { strikethrough: true, inverse: true }` → `\x1b[9;7m`

**Changes:**
- Add fields to Style struct
- Implement `style_to_ansi()` helper that emits all active SGR codes
- Update `Segment` rendering to use new attributes
- Builder pattern for Style construction

**Tests (12+):**
1. Underline style → SGR 4
2. Strikethrough style → SGR 9
3. Blink style → SGR 5
4. Inverse style → SGR 7
5. Combined bold + underline → SGR 1;4
6. Combined all attributes → all SGR codes
7. Style equality with new fields
8. Style clone preserves attributes
9. style_to_ansi() produces correct ANSI for each attribute
10. Multiple attributes in same sequence
11. Reset after styled text
12. Attributes + color + foreground/background

---

## Task 4: Scrollable Regions with Viewport Clipping

**Files to create:**
- `crates/fae-core/src/viewport.rs` (new module)

**Description**: Implement viewport clipping and scroll offset handling. Widgets can specify scroll position, and only the visible portion renders.

**New types:**
- `Viewport` — represents visible portion of content
  - `offset: Position` — top-left of visible area
  - `size: Size` — size of visible area (matches widget size on screen)
  - `content_size: Size` — total content size (may be larger than viewport)

**Functions:**
- `Viewport::new(size) -> Self` — create viewport with offset (0,0)
- `scroll_by(dx, dy)` — move viewport
- `scroll_to(x, y)` — absolute scroll
- `is_visible(rect: Rect) -> bool` — check if rect is visible
- `clip_to_viewport(rect: Rect) -> Option<Rect>` — get visible portion

**Integration:**
- Layer stores viewport info
- `compose_line` respects viewport bounds
- `write_segments_to_buffer` clips to viewport

**Tests (10+):**
1. Create viewport, no scroll
2. Scroll down → offset changes
3. Scroll past content → clamped
4. is_visible() for on-screen region
5. is_visible() for off-screen region
6. clip_to_viewport() within bounds
7. clip_to_viewport() partial overlap
8. clip_to_viewport() no overlap
9. Layer with viewport clips correctly
10. Multiple layers with different viewports render independently

---

## Task 5: Unicode Edge Cases — Emoji and Combining Marks

**Files to modify:**
- `crates/fae-core/src/cell.rs` — enhance cell handling
- `crates/fae-core/src/segment.rs` — improve segment splitting

**Description**: Handle edge cases in unicode: emoji (wide), combining marks (zero-width), RTL scripts. Ensure correct rendering across layer boundaries.

**Changes:**
- Emoji detection: use `unicode-width` to identify wide characters (width=2)
- Combining marks: width=0, should attach to previous grapheme
- Segment splitting at boundaries: preserve combining marks with base character
- Cell width tracking for correct column calculation

**Implementation:**
- Enhance `Segment::split_at()` to preserve grapheme clusters
- Add grapheme cluster iteration (already using `unicode-segmentation`)
- Update width calculations in `write_segments_to_buffer()`

**Tests (8+):**
1. Emoji (width=2) in segment → renders in 2 columns
2. Emoji at cut boundary → no split, moves to next interval
3. Combining diacritics (width=0) → attached to base character
4. Mixed ASCII + emoji + combining marks
5. Layer boundary in middle of emoji → emoji clipped/moved appropriately
6. Wide character continuation cell handling
7. Emoji in different styling layers
8. Long combining mark sequence (multiple diacritics)

---

## Task 6: Renderer Output Optimization

**Files to modify:**
- `crates/fae-core/src/renderer.rs`

**Description**: Implement performance optimizations for rendering: cursor hiding, batch ANSI emission, minimize color resets.

**Optimizations:**
- Hide cursor during render, show at end
- Batch SGR changes when possible
- Track last color state to avoid redundant changes
- Use `\x1b[m` reset only when necessary
- Combine multiple SGR codes into single escape: `\x1b[1;4;31m`

**Tests (6+):**
1. Rendering with cursor hidden/shown
2. Color state tracking → fewer resets
3. SGR batching → fewer escape sequences
4. Performance: render 100x50 screen in <10ms
5. Large buffer with minimal changes → fast delta render
6. Render to string produces valid ANSI
7. Output byte count is minimal for given content

---

## Task 7: Integration Tests — Complex Rendering Scenarios

**Files to modify:**
- `crates/fae-core/src/tests/` (or integration test directory)

**Description**: Write comprehensive integration tests covering realistic rendering scenarios.

**Test scenarios:**
1. **Chat app**: Header (fixed) + scrollable message list + input box → each layer renders at correct position
2. **Modal dialog**: Modal window (z=100) overlays background → modal clips at edges, background visible around edges
3. **Syntax highlighting**: Code widget with multiple colors and styles → styles preserved through composition
4. **Scrolling**: User scrolls message list → viewport updates, only visible portion renders
5. **Resize event**: Terminal resize from 80x24 to 120x40 → compositor resizes, all layers recompose
6. **Overlapping styled windows**: Two windows overlapping with different colors → topmost color visible in overlap
7. **Emoji in UI**: Button label with emoji + regular text → correct width, renders at expected position
8. **Theme switch**: Dark mode to light mode → styles update, compositor re-renders

**Tests (8):**
- One integration test per scenario
- Each verifies final rendered output (ANSI) is correct
- Each tests performance is acceptable

---

## Task 8: Documentation and Module Exports

**Files to modify:**
- `crates/fae-core/src/lib.rs` — add module exports
- `crates/fae-core/src/renderer.rs` — add doc comments
- `crates/fae-core/src/viewport.rs` — add doc comments
- Create `crates/fae-core/RENDERING.md` — high-level architecture

**Description**: Document the rendering pipeline, compositor integration, and viewport system. Export public APIs.

**Exports:**
```rust
pub mod viewport;
pub mod renderer;
pub use viewport::Viewport;
pub use renderer::DeltaRenderer;
```

**Documentation:**
- Module-level docs explaining rendering pipeline
- Example code showing compositor usage
- Performance tips for rendering optimization
- Unicode handling best practices

**Tests:**
- All doc examples compile and run
- Documentation warnings: zero

---

## Success Criteria

- ✅ Compositor integrated into render pipeline
- ✅ Delta rendering working (produces fewer ANSI bytes)
- ✅ Advanced text styling (underline, strikethrough, etc.) working
- ✅ Viewport clipping and scrolling functional
- ✅ Unicode edge cases handled correctly
- ✅ Performance optimizations in place
- ✅ 80+ new tests added
- ✅ Zero clippy warnings
- ✅ Zero compilation errors
- ✅ 100% public API documentation

---

## Estimated Effort

- 8 tasks × ~2 hours per task = ~16 hours
- Includes testing, documentation, and integration
