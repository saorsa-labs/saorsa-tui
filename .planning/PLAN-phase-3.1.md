# Phase 3.1: Compositor Core

## Overview

Build the compositing layer that sits between the widget tree (which produces styled segments) and the screen buffer (which is a flat cell grid). The compositor enables overlapping widgets by collecting per-widget render output, tracking z-order, finding column boundaries ("cuts") where widget edges meet, and selecting the topmost visible widget for each region.

### Current Architecture

```
Widget.render(area, &mut ScreenBuffer)   ← direct write, no overlap support
     │
     ▼
ScreenBuffer (flat Cell grid)
     │
     ▼
RenderContext.end_frame() → diff → Renderer → ANSI escape sequences
```

### Target Architecture

```
Widget.render_segments(area) → Vec<Vec<Segment>>  ← per-line segment output
     │
     ▼
Compositor                                        ← NEW
  ├── collect layers (widget_id, region, z_index, segments)
  ├── find cuts (x-offsets at widget edges per line)
  ├── chop segments at cut boundaries
  ├── select topmost widget per region
  └── concatenate final segment list per line
     │
     ▼
ScreenBuffer ← compositor writes resolved cells
     │
     ▼
RenderContext.end_frame() → diff → Renderer → ANSI
```

## Tasks

### Task 1: Layer and CompositorRegion Types

**Files**: `crates/fae-core/src/compositor/mod.rs` (new), `crates/fae-core/src/compositor/layer.rs` (new)

**Description**: Define the core data structures for the compositor.

Types to create:
- `Layer` — represents one widget's render output:
  - `widget_id: WidgetId` (u64)
  - `region: Rect` — the bounding box on screen
  - `z_index: i32` — stacking order (higher = on top)
  - `lines: Vec<Vec<Segment>>` — per-line segment output (line 0 = top of region)
- `CompositorRegion` — a sub-region within a single row after cut-finding:
  - `x: u16` — start column
  - `width: u16` — width in columns
  - `source_layer_idx: usize` — which layer owns this region
- `CompositorError` — error type for compositor operations

Module structure:
- `compositor/mod.rs` — re-exports, `Compositor` struct shell
- `compositor/layer.rs` — `Layer`, `CompositorRegion`, `CompositorError`

Tests:
- Layer construction and field access
- Layer with empty lines
- CompositorRegion construction
- CompositorError Display impl

---

### Task 2: Compositor Struct and Layer Collection

**Files**: `crates/fae-core/src/compositor/mod.rs`

**Description**: Implement the `Compositor` struct that collects layers and manages the rendering pipeline.

```rust
pub struct Compositor {
    layers: Vec<Layer>,
    screen_width: u16,
    screen_height: u16,
}
```

Methods:
- `new(width, height) -> Self`
- `clear() -> &mut Self` — remove all layers
- `add_layer(layer: Layer) -> &mut Self` — add a layer
- `add_widget(widget_id, region, z_index, lines) -> &mut Self` — convenience
- `layer_count() -> usize`
- `screen_size() -> Size`

Tests:
- Create compositor, add layers, verify count
- Clear removes all layers
- Adding multiple layers preserves order
- Screen dimensions accessible

---

### Task 3: Cut-Finding Algorithm

**Files**: `crates/fae-core/src/compositor/cuts.rs` (new)

**Description**: For a given row, find all x-offsets where widget edges create boundaries. These "cuts" define the columns where we need to check which widget is on top.

Algorithm:
1. For each layer that intersects this row, collect `region.x` (left edge) and `region.x + region.width` (right edge)
2. Add `0` (screen left) and `screen_width` (screen right)
3. Deduplicate and sort the offsets
4. These define intervals: `[cut[0]..cut[1]], [cut[1]..cut[2]], ...`

```rust
pub fn find_cuts(layers: &[Layer], row: u16, screen_width: u16) -> Vec<u16>
```

Tests:
- No layers → cuts at [0, screen_width]
- Single layer centered → cuts at [0, layer.x, layer.x+w, screen_width]
- Two non-overlapping layers → all edges present
- Two overlapping layers → merged edges
- Layer at screen edge → no duplicate edge points
- Empty-width layer → handled gracefully

---

### Task 4: Z-Order Selection

**Files**: `crates/fae-core/src/compositor/zorder.rs` (new)

**Description**: For a given cut interval on a given row, determine which layer is the topmost visible one.

```rust
pub fn select_topmost(
    layers: &[Layer],
    row: u16,
    x_start: u16,
    x_end: u16,
) -> Option<usize>
```

Logic:
1. Filter layers whose region contains this row and overlaps `[x_start, x_end)`
2. Among those, return the index of the layer with the highest `z_index`
3. Ties broken by insertion order (later layer wins)
4. Return `None` if no layer covers this region (background shows through)

Tests:
- No layers at position → None
- Single layer covers region → returns its index
- Two overlapping layers → higher z_index wins
- Same z_index → later insertion wins
- Layer partially overlapping → still selected if it covers the interval
- Layer on different row → not selected

---

### Task 5: Segment Chopping

**Files**: `crates/fae-core/src/compositor/chop.rs` (new)

**Description**: Extract a sub-range of segments from a layer's line, corresponding to a cut interval. This uses `Segment::split_at` to handle cases where a cut falls in the middle of a segment.

```rust
pub fn chop_segments(
    segments: &[Segment],
    layer_x: u16,
    cut_start: u16,
    cut_width: u16,
) -> Vec<Segment>
```

Logic:
1. The segments represent the layer's content starting at `layer_x`
2. Walk through segments, tracking current x offset
3. Skip segments entirely before `cut_start`
4. Split the first segment that straddles `cut_start`
5. Collect segments within the cut range
6. Split the last segment that straddles `cut_start + cut_width`
7. Return the collected sub-segments

Tests:
- Full segment within cut range → returned as-is
- Segment split at left boundary
- Segment split at right boundary
- Segment split at both boundaries
- Empty segments skipped
- Cut range beyond segment end → padded with blanks
- CJK characters split at boundary → space padding (uses Segment::split_at)

---

### Task 6: Line Composition

**Files**: `crates/fae-core/src/compositor/compose.rs` (new)

**Description**: Compose a single row by combining cut-finding, z-order selection, and segment chopping into a final list of segments for that row.

```rust
pub fn compose_line(
    layers: &[Layer],
    row: u16,
    screen_width: u16,
) -> Vec<Segment>
```

Logic:
1. Call `find_cuts(layers, row, screen_width)` to get cut points
2. For each interval `[cuts[i], cuts[i+1])`:
   a. Call `select_topmost(layers, row, cuts[i], cuts[i+1])`
   b. If Some(layer_idx):
      - Get the layer's line for this row (offset from layer's region.y)
      - Call `chop_segments` to extract the interval
      - Append to result
   c. If None:
      - Append a blank segment with width = interval width
3. Return the composed segments

Tests:
- Single layer, full width → segments pass through
- Two layers side by side → segments concatenated
- Overlapping layers → topmost segments used
- Gap between layers → blank segment inserted
- Layer extends beyond screen → clipped at screen edge
- Empty row (no layers) → single blank segment

---

### Task 7: Full-Frame Composition and Buffer Write

**Files**: `crates/fae-core/src/compositor/mod.rs`

**Description**: Implement the full `compose` method that processes all rows and writes the result to a `ScreenBuffer`.

```rust
impl Compositor {
    pub fn compose(&self, buf: &mut ScreenBuffer) {
        for row in 0..self.screen_height {
            let segments = compose_line(&self.layers, row, self.screen_width);
            self.write_segments_to_buffer(buf, row, &segments);
        }
    }

    fn write_segments_to_buffer(
        &self,
        buf: &mut ScreenBuffer,
        row: u16,
        segments: &[Segment],
    ) { ... }
}
```

The `write_segments_to_buffer` method converts segments back to cells and writes them to the buffer, handling:
- Grapheme iteration with unicode-width
- Wide character continuation cells
- Style propagation from segment to cell

Tests:
- Compose single layer → buffer contains layer content
- Compose overlapping layers → topmost visible
- Compose to buffer with correct cell styles
- Wide characters handled correctly across layer boundaries
- Empty compositor → all blank cells

---

### Task 8: Module Integration and lib.rs Exports

**Files**: `crates/fae-core/src/compositor/mod.rs`, `crates/fae-core/src/lib.rs`

**Description**: Wire up the compositor module, add public exports, and write integration tests that exercise the full pipeline: create layers → compose → verify buffer content.

Add to `lib.rs`:
```rust
pub mod compositor;
pub use compositor::{Compositor, CompositorError, CompositorRegion, Layer};
```

Integration tests:
- Chat app layout: header (z=0) + messages (z=0) + input (z=0) + modal overlay (z=10) → modal overlays correctly
- Three overlapping windows at different z-levels → correct stacking
- Layer with styled segments → styles preserved through composition
- Scroll offset integration: layer with content_offset shifts segments
- Resize: create compositor with new dimensions, recompose
- Performance: compose 200x50 screen with 20 layers → completes in reasonable time
