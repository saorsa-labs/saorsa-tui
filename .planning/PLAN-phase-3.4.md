# Phase 3.4: Modal & Overlay Rendering

**Milestone**: 3 — Compositor & Advanced Rendering
**Phase**: 3.4 — Modal & Overlay Rendering
**Status**: Planned
**Estimated Tests**: 60+

## Overview

Build overlay rendering primitives: modal dialogs, toast notifications, tooltips,
and a screen stack manager. The compositor already handles z-ordering and overlapping
layers; this phase adds high-level overlay types on top of that foundation.

### Current Foundation (end of Phase 3.3)
- Compositor with z-ordering and layer composition (809 tests)
- Style with `dim: bool` field (for background dimming)
- Viewport with scrolling and clipping
- Widget trait system (Label, Container, StaticWidget)
- RenderContext with compositor integration

### What This Phase Adds
1. **Overlay trait** — common interface for modal/toast/tooltip
2. **ScreenStack** — push/pop overlay manager with auto z-index
3. **Modal widget** — centered overlay with dim background effect
4. **Toast widget** — corner-positioned notification with lifetime
5. **Tooltip widget** — smart-positioned text near an anchor
6. **Dim background rendering** — compositor applies dim style to covered layers
7. **Integration tests** — full overlay pipeline
8. **Documentation and exports**

## Dependencies

- Phase 3.3 complete (809 tests)
- Existing compositor, viewport, style, widget infrastructure

---

## Task 1: Overlay Trait and ScreenStack

**Files to create:**
- `crates/saorsa-core/src/overlay.rs` (new module)

**Description**: Define the Overlay trait and ScreenStack manager for overlay lifecycle.

**Types:**
```rust
/// Position hint for overlay placement.
pub enum OverlayPosition {
    /// Centered on screen.
    Center,
    /// Specific position.
    At(Position),
    /// Relative to anchor rect.
    Anchored { anchor: Rect, placement: Placement },
}

/// Placement relative to an anchor.
pub enum Placement {
    Above,
    Below,
    Left,
    Right,
}

/// Common overlay properties.
pub struct OverlayConfig {
    pub position: OverlayPosition,
    pub size: Size,
    pub z_offset: i32,
    pub dim_background: bool,
}

/// Unique overlay identifier.
pub type OverlayId = u64;

/// Manages a stack of overlay layers with auto z-indexing.
pub struct ScreenStack {
    overlays: Vec<OverlayEntry>,
    next_id: OverlayId,
    base_z: i32,
}

struct OverlayEntry {
    id: OverlayId,
    config: OverlayConfig,
    lines: Vec<Vec<Segment>>,
}
```

**Methods on ScreenStack:**
- `new() -> Self`
- `push(&mut self, config: OverlayConfig, lines: Vec<Vec<Segment>>) -> OverlayId`
- `pop(&mut self) -> Option<OverlayId>` — remove topmost
- `remove(&mut self, id: OverlayId) -> bool` — remove specific
- `clear(&mut self)` — remove all
- `len(&self) -> usize` / `is_empty(&self) -> bool`
- `apply_to_compositor(&self, compositor: &mut Compositor, screen: Size)` — add all overlays as layers
- `resolve_position(config: &OverlayConfig, screen: Size) -> Position` — compute concrete position

**Don't forget:** Add `pub mod overlay;` and exports to `lib.rs`.

**Tests (10+):**
1. New ScreenStack is empty
2. Push overlay → len() == 1
3. Pop overlay → empty again
4. Push two, pop → first remains
5. Remove by ID → correct one removed
6. Clear removes all
7. Center position resolves to correct coords
8. Anchored/Above position resolves correctly
9. Anchored/Below with screen edge clamping
10. apply_to_compositor adds correct layers

---

## Task 2: Dim Background Effect

**Files to modify:**
- `crates/saorsa-core/src/overlay.rs` — add dim rendering helper
- `crates/saorsa-core/src/compositor/mod.rs` — optional: dim support in compose

**Description**: Implement background dimming when a modal overlay is active.
When `dim_background: true`, the ScreenStack inserts a full-screen dim layer
behind the overlay that applies the `dim` style attribute.

**Implementation:**
- `create_dim_layer(screen: Size, z_index: i32) -> Layer` — creates a layer
  covering the full screen filled with dim-styled space characters
- Dim layer inserted at z_index just below the overlay
- Uses `Style::new().dim(true).bg(Color::Named(NamedColor::Black))` for the dim effect

**Tests (8+):**
1. create_dim_layer produces correct size
2. Dim layer cells have dim style set
3. Dim layer z_index is one below overlay
4. ScreenStack with dim_background inserts dim layer
5. ScreenStack without dim_background skips dim layer
6. Multiple overlays → only topmost dim overlay adds dim layer
7. Compositing with dim layer → dim style visible in buffer
8. Removing dimmed overlay removes dim layer too

---

## Task 3: Modal Widget

**Files to create:**
- `crates/saorsa-core/src/widget/modal.rs` (new)

**Description**: A modal dialog widget with title, body, border, and optional
close button indicator. Uses Container internally for border rendering.

**Type:**
```rust
pub struct Modal {
    title: String,
    body_lines: Vec<Vec<Segment>>,
    style: Style,
    border_style: BorderStyle,
    width: u16,
    height: u16,
}
```

**Methods:**
- `new(title: impl Into<String>, width: u16, height: u16) -> Self`
- `with_body(lines: Vec<Vec<Segment>>) -> Self`
- `with_style(style: Style) -> Self`
- `with_border(border: BorderStyle) -> Self`
- `render_to_lines(&self) -> Vec<Vec<Segment>>` — produces the rendered lines
  (border + title + body) ready for compositor
- `to_overlay_config(&self) -> OverlayConfig` — produces an OverlayConfig centered
  with dim_background: true

**Tests (8+):**
1. New modal with defaults
2. Modal render_to_lines produces correct line count
3. Modal title appears in top border line
4. Modal body content appears inside border
5. Empty body → border-only modal
6. Style applied to border and content
7. to_overlay_config returns centered position with dim
8. Modal with custom border style

---

## Task 4: Toast Widget

**Files to create:**
- `crates/saorsa-core/src/widget/toast.rs` (new)

**Description**: A toast notification that appears in a corner of the screen.

**Type:**
```rust
/// Corner position for toast.
pub enum ToastPosition {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

pub struct Toast {
    message: String,
    position: ToastPosition,
    style: Style,
    width: u16,
}
```

**Methods:**
- `new(message: impl Into<String>) -> Self`
- `with_position(pos: ToastPosition) -> Self`
- `with_style(style: Style) -> Self`
- `with_width(width: u16) -> Self`
- `render_to_lines(&self) -> Vec<Vec<Segment>>`
- `to_overlay_config(&self, screen: Size) -> OverlayConfig` — computes position
  based on corner and screen size, dim_background: false

**Tests (8+):**
1. New toast with defaults
2. Toast render_to_lines produces content
3. Toast at TopRight computes correct position
4. Toast at BottomLeft computes correct position
5. Toast at TopLeft computes correct position
6. Toast at BottomRight computes correct position
7. Toast style applied
8. Custom width respected

---

## Task 5: Tooltip Widget

**Files to create:**
- `crates/saorsa-core/src/widget/tooltip.rs` (new)

**Description**: A tooltip that appears near an anchor element with smart positioning.

**Type:**
```rust
pub struct Tooltip {
    text: String,
    anchor: Rect,
    placement: Placement,
    style: Style,
}
```

**Methods:**
- `new(text: impl Into<String>, anchor: Rect) -> Self`
- `with_placement(placement: Placement) -> Self`
- `with_style(style: Style) -> Self`
- `render_to_lines(&self) -> Vec<Vec<Segment>>`
- `compute_position(&self, screen: Size) -> Position` — smart positioning
  that flips placement if tooltip would go off-screen
- `to_overlay_config(&self, screen: Size) -> OverlayConfig`

**Smart positioning rules:**
- Above: place above anchor; if would go above screen, flip to Below
- Below: place below anchor; if would go below screen, flip to Above
- Left: place left of anchor; if would go off-screen, flip to Right
- Right: place right of anchor; if would go off-screen, flip to Left

**Tests (10+):**
1. Tooltip placed above anchor — correct position
2. Tooltip placed below anchor — correct position
3. Tooltip placed left of anchor — correct position
4. Tooltip placed right of anchor — correct position
5. Above placement flips to below when at top edge
6. Below placement flips to above when at bottom edge
7. Left placement flips to right when at left edge
8. Right placement flips to left when at right edge
9. Tooltip text renders correctly
10. Style preserved in output

---

## Task 6: Widget Module Integration

**Files to modify:**
- `crates/saorsa-core/src/widget/mod.rs` — add modal, toast, tooltip modules

**Description**: Wire the new widget types into the widget module and export them.

**Changes:**
- Add `pub mod modal;`, `pub mod toast;`, `pub mod tooltip;`
- Re-export: `pub use modal::Modal;`, `pub use toast::{Toast, ToastPosition};`, `pub use tooltip::Tooltip;`
- Ensure all public items have doc comments

**Tests (6+):**
1. Modal can be created and rendered
2. Toast can be created and rendered
3. Tooltip can be created and rendered
4. Modal pushed to ScreenStack and composed
5. Toast pushed to ScreenStack and composed
6. Multiple overlay types in same ScreenStack

---

## Task 7: Integration Tests — Full Overlay Pipeline

**Files to modify:**
- `crates/saorsa-core/src/overlay.rs` — integration test module

**Description**: End-to-end tests showing overlays flowing through the full
rendering pipeline: Widget → ScreenStack → Compositor → ScreenBuffer.

**Tests (8+):**
1. Modal dialog centered on screen → correct buffer cells
2. Modal with dim background → dim cells visible around modal
3. Toast at TopRight → correct position in buffer
4. Tooltip below anchor → correct position in buffer
5. Two modals stacked → topmost visible in overlap region
6. Modal + toast simultaneously → both visible, correct z-order
7. Remove modal → background no longer dimmed
8. ScreenStack clear → compositor has no overlay layers

---

## Task 8: Documentation and Module Exports

**Files to modify:**
- `crates/saorsa-core/src/lib.rs` — exports
- Add doc comments to all new public items

**Description**: Export all new types and ensure documentation is complete.

**Exports to add:**
```rust
pub mod overlay;
pub use overlay::{OverlayConfig, OverlayId, OverlayPosition, Placement, ScreenStack};
pub use widget::{Modal, Toast, ToastPosition, Tooltip};
```

**Verification:** `cargo doc --workspace --no-deps` produces zero warnings.

---

## Success Criteria

- ✅ ScreenStack manages overlay lifecycle
- ✅ Modal dialog renders with dim background
- ✅ Toast notification renders at corners
- ✅ Tooltip positions smartly near anchors
- ✅ Dim background effect works in compositor
- ✅ 60+ new tests added
- ✅ Zero clippy warnings
- ✅ Zero compilation errors
- ✅ 100% public API documentation
