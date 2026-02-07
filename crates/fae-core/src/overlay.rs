//! Overlay management for modal dialogs, toasts, and tooltips.
//!
//! Provides [`ScreenStack`] to manage a stack of overlay layers with
//! automatic z-indexing, position resolution, and optional background dimming.

use crate::compositor::Layer;
use crate::geometry::{Position, Rect, Size};
use crate::segment::Segment;
use crate::style::Style;

/// Unique overlay identifier.
pub type OverlayId = u64;

/// Placement of an overlay relative to an anchor element.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Placement {
    /// Above the anchor.
    Above,
    /// Below the anchor.
    Below,
    /// Left of the anchor.
    Left,
    /// Right of the anchor.
    Right,
}

/// Position strategy for an overlay.
#[derive(Debug, Clone, PartialEq)]
pub enum OverlayPosition {
    /// Centered on the screen.
    Center,
    /// At a fixed position.
    At(Position),
    /// Anchored relative to a rectangle.
    Anchored {
        /// The anchor rectangle.
        anchor: Rect,
        /// Placement relative to the anchor.
        placement: Placement,
    },
}

/// Configuration for an overlay.
#[derive(Debug, Clone)]
pub struct OverlayConfig {
    /// How the overlay should be positioned.
    pub position: OverlayPosition,
    /// Size of the overlay content.
    pub size: Size,
    /// Z-index offset from the stack's base z-index.
    pub z_offset: i32,
    /// Whether to insert a dim layer behind this overlay.
    pub dim_background: bool,
}

struct OverlayEntry {
    id: OverlayId,
    config: OverlayConfig,
    lines: Vec<Vec<Segment>>,
}

/// Manages a stack of overlay layers with auto z-indexing.
///
/// Overlays are rendered in insertion order. Each overlay receives a unique
/// z-index spaced 10 apart from the base. Dim layers are inserted one
/// z-level below overlays that request background dimming.
pub struct ScreenStack {
    overlays: Vec<OverlayEntry>,
    next_id: OverlayId,
    base_z: i32,
}

impl ScreenStack {
    /// Creates an empty screen stack with base z-index 1000.
    pub fn new() -> Self {
        Self {
            overlays: Vec::new(),
            next_id: 1,
            base_z: 1000,
        }
    }

    /// Pushes an overlay onto the stack.
    ///
    /// Returns a unique [`OverlayId`] for later removal.
    pub fn push(&mut self, config: OverlayConfig, lines: Vec<Vec<Segment>>) -> OverlayId {
        let id = self.next_id;
        self.next_id += 1;
        self.overlays.push(OverlayEntry { id, config, lines });
        id
    }

    /// Removes the topmost overlay from the stack.
    pub fn pop(&mut self) -> Option<OverlayId> {
        self.overlays.pop().map(|e| e.id)
    }

    /// Removes a specific overlay by ID.
    ///
    /// Returns `true` if the overlay was found and removed.
    pub fn remove(&mut self, id: OverlayId) -> bool {
        let before = self.overlays.len();
        self.overlays.retain(|e| e.id != id);
        self.overlays.len() < before
    }

    /// Removes all overlays.
    pub fn clear(&mut self) {
        self.overlays.clear();
    }

    /// Returns the number of overlays in the stack.
    pub fn len(&self) -> usize {
        self.overlays.len()
    }

    /// Returns `true` if the stack has no overlays.
    pub fn is_empty(&self) -> bool {
        self.overlays.is_empty()
    }

    /// Resolves an overlay position to absolute screen coordinates.
    pub fn resolve_position(position: &OverlayPosition, size: Size, screen: Size) -> Position {
        match position {
            OverlayPosition::Center => {
                let x = screen.width.saturating_sub(size.width) / 2;
                let y = screen.height.saturating_sub(size.height) / 2;
                Position::new(x, y)
            }
            OverlayPosition::At(pos) => *pos,
            OverlayPosition::Anchored { anchor, placement } => match placement {
                Placement::Above => {
                    let x = anchor
                        .position
                        .x
                        .saturating_add(anchor.size.width / 2)
                        .saturating_sub(size.width / 2);
                    let y = anchor.position.y.saturating_sub(size.height);
                    Position::new(x, y)
                }
                Placement::Below => {
                    let x = anchor
                        .position
                        .x
                        .saturating_add(anchor.size.width / 2)
                        .saturating_sub(size.width / 2);
                    let y = anchor.position.y.saturating_add(anchor.size.height);
                    Position::new(x, y)
                }
                Placement::Left => {
                    let x = anchor.position.x.saturating_sub(size.width);
                    let y = anchor
                        .position
                        .y
                        .saturating_add(anchor.size.height / 2)
                        .saturating_sub(size.height / 2);
                    Position::new(x, y)
                }
                Placement::Right => {
                    let x = anchor.position.x.saturating_add(anchor.size.width);
                    let y = anchor
                        .position
                        .y
                        .saturating_add(anchor.size.height / 2)
                        .saturating_sub(size.height / 2);
                    Position::new(x, y)
                }
            },
        }
    }

    /// Applies all overlays as layers to a compositor.
    ///
    /// Each overlay is added with its resolved position and z-index.
    /// Overlays with `dim_background` get a full-screen dim layer inserted
    /// one z-level below them.
    pub fn apply_to_compositor(
        &self,
        compositor: &mut crate::compositor::Compositor,
        screen: Size,
    ) {
        for (i, entry) in self.overlays.iter().enumerate() {
            let z = self.base_z + (i as i32) * 10 + entry.config.z_offset;

            if entry.config.dim_background {
                compositor.add_layer(create_dim_layer(screen, z - 1));
            }

            let pos = Self::resolve_position(&entry.config.position, entry.config.size, screen);
            let region = Rect::new(
                pos.x,
                pos.y,
                entry.config.size.width,
                entry.config.size.height,
            );
            compositor.add_layer(Layer::new(entry.id, region, z, entry.lines.clone()));
        }
    }
}

impl Default for ScreenStack {
    fn default() -> Self {
        Self::new()
    }
}

/// Creates a full-screen dim layer for background dimming.
pub fn create_dim_layer(screen: Size, z_index: i32) -> Layer {
    let dim_style = Style::new().dim(true);
    let mut lines = Vec::new();
    for _ in 0..screen.height {
        lines.push(vec![Segment::styled(
            " ".repeat(screen.width as usize),
            dim_style.clone(),
        )]);
    }
    Layer::new(
        0,
        Rect::new(0, 0, screen.width, screen.height),
        z_index,
        lines,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_stack() {
        let stack = ScreenStack::new();
        assert!(stack.is_empty());
        assert!(stack.is_empty());
    }

    #[test]
    fn push_increments_len() {
        let mut stack = ScreenStack::new();
        let config = OverlayConfig {
            position: OverlayPosition::Center,
            size: Size::new(10, 5),
            z_offset: 0,
            dim_background: false,
        };
        let id = stack.push(config, vec![vec![Segment::new("hi")]]);
        assert!(id == 1);
        assert!(stack.len() == 1);
    }

    #[test]
    fn pop_returns_topmost() {
        let mut stack = ScreenStack::new();
        let config = OverlayConfig {
            position: OverlayPosition::Center,
            size: Size::new(10, 5),
            z_offset: 0,
            dim_background: false,
        };
        let _id1 = stack.push(config.clone(), vec![]);
        let id2 = stack.push(config, vec![]);
        assert!(stack.pop() == Some(id2));
        assert!(stack.len() == 1);
    }

    #[test]
    fn pop_empty_returns_none() {
        let mut stack = ScreenStack::new();
        assert!(stack.pop().is_none());
    }

    #[test]
    fn remove_by_id() {
        let mut stack = ScreenStack::new();
        let config = OverlayConfig {
            position: OverlayPosition::Center,
            size: Size::new(10, 5),
            z_offset: 0,
            dim_background: false,
        };
        let id1 = stack.push(config.clone(), vec![]);
        let _id2 = stack.push(config, vec![]);
        assert!(stack.remove(id1));
        assert!(stack.len() == 1);
    }

    #[test]
    fn remove_nonexistent_returns_false() {
        let mut stack = ScreenStack::new();
        assert!(!stack.remove(999));
    }

    #[test]
    fn clear_removes_all() {
        let mut stack = ScreenStack::new();
        let config = OverlayConfig {
            position: OverlayPosition::Center,
            size: Size::new(10, 5),
            z_offset: 0,
            dim_background: false,
        };
        stack.push(config.clone(), vec![]);
        stack.push(config, vec![]);
        stack.clear();
        assert!(stack.is_empty());
    }

    #[test]
    fn resolve_center() {
        let pos = ScreenStack::resolve_position(
            &OverlayPosition::Center,
            Size::new(20, 10),
            Size::new(80, 24),
        );
        assert!(pos.x == 30);
        assert!(pos.y == 7);
    }

    #[test]
    fn resolve_at() {
        let pos = ScreenStack::resolve_position(
            &OverlayPosition::At(Position::new(5, 3)),
            Size::new(20, 10),
            Size::new(80, 24),
        );
        assert!(pos.x == 5);
        assert!(pos.y == 3);
    }

    #[test]
    fn resolve_anchored_below() {
        let anchor = Rect::new(30, 5, 10, 2);
        let pos = ScreenStack::resolve_position(
            &OverlayPosition::Anchored {
                anchor,
                placement: Placement::Below,
            },
            Size::new(20, 3),
            Size::new(80, 24),
        );
        // x centered: 30 + 5 - 10 = 25
        assert!(pos.x == 25);
        // y below: 5 + 2 = 7
        assert!(pos.y == 7);
    }

    #[test]
    fn resolve_anchored_above() {
        let anchor = Rect::new(30, 10, 10, 2);
        let pos = ScreenStack::resolve_position(
            &OverlayPosition::Anchored {
                anchor,
                placement: Placement::Above,
            },
            Size::new(20, 3),
            Size::new(80, 24),
        );
        assert!(pos.x == 25);
        assert!(pos.y == 7); // 10 - 3 = 7
    }

    #[test]
    fn resolve_anchored_right() {
        let anchor = Rect::new(10, 10, 5, 4);
        let pos = ScreenStack::resolve_position(
            &OverlayPosition::Anchored {
                anchor,
                placement: Placement::Right,
            },
            Size::new(8, 3),
            Size::new(80, 24),
        );
        assert!(pos.x == 15); // 10 + 5
        assert!(pos.y == 11); // 10 + 2 - 1
    }

    #[test]
    fn dim_layer_covers_screen() {
        let layer = create_dim_layer(Size::new(80, 24), 999);
        assert!(layer.z_index == 999);
        assert!(layer.region.size.width == 80);
        assert!(layer.region.size.height == 24);
        assert!(layer.lines.len() == 24);
    }

    #[test]
    fn dim_layer_style_is_dim() {
        let layer = create_dim_layer(Size::new(10, 2), 500);
        assert!(layer.lines.len() == 2);
        assert!(layer.lines[0][0].style.dim);
    }

    #[test]
    fn apply_to_compositor_adds_layers() {
        let mut stack = ScreenStack::new();
        let config = OverlayConfig {
            position: OverlayPosition::Center,
            size: Size::new(10, 3),
            z_offset: 0,
            dim_background: false,
        };
        stack.push(config, vec![vec![Segment::new("test")]]);

        let mut compositor = crate::compositor::Compositor::new(80, 24);
        stack.apply_to_compositor(&mut compositor, Size::new(80, 24));

        let mut buf = crate::buffer::ScreenBuffer::new(Size::new(80, 24));
        compositor.compose(&mut buf);

        // Check content appears near center (x=35, y=10)
        match buf.get(35, 10) {
            Some(cell) => assert!(cell.grapheme == "t"),
            None => unreachable!(),
        }
    }

    #[test]
    fn apply_with_dim_background() {
        let mut stack = ScreenStack::new();
        let config = OverlayConfig {
            position: OverlayPosition::At(Position::new(5, 5)),
            size: Size::new(10, 3),
            z_offset: 0,
            dim_background: true,
        };
        stack.push(config, vec![vec![Segment::new("modal")]]);

        let mut compositor = crate::compositor::Compositor::new(80, 24);
        stack.apply_to_compositor(&mut compositor, Size::new(80, 24));

        let mut buf = crate::buffer::ScreenBuffer::new(Size::new(80, 24));
        compositor.compose(&mut buf);

        // Corner should have dim style from dim layer
        match buf.get(0, 0) {
            Some(cell) => assert!(cell.style.dim),
            None => unreachable!(),
        }
        // Overlay content should be visible
        match buf.get(5, 5) {
            Some(cell) => assert!(cell.grapheme == "m"),
            None => unreachable!(),
        }
    }

    // --- Integration tests: full overlay pipeline ---

    #[test]
    fn modal_centered_on_screen() {
        use crate::widget::modal::Modal;

        let modal = Modal::new("Test", 20, 5);
        let lines = modal.render_to_lines();
        let config = modal.to_overlay_config();

        let mut stack = ScreenStack::new();
        stack.push(config, lines);

        let screen = Size::new(80, 24);
        let mut compositor = crate::compositor::Compositor::new(80, 24);
        stack.apply_to_compositor(&mut compositor, screen);

        let mut buf = crate::buffer::ScreenBuffer::new(screen);
        compositor.compose(&mut buf);

        // Center x = (80-20)/2 = 30, y = (24-5)/2 = 9
        // Top-left corner should be the border character
        match buf.get(30, 9) {
            Some(cell) => assert!(cell.grapheme == "┌"),
            None => unreachable!(),
        }
    }

    #[test]
    fn modal_with_dim_background_pipeline() {
        use crate::widget::modal::Modal;

        let modal = Modal::new("Dim", 20, 5);
        let lines = modal.render_to_lines();
        let config = modal.to_overlay_config();
        // Confirm dim is set
        assert!(config.dim_background);

        let mut stack = ScreenStack::new();
        stack.push(config, lines);

        let screen = Size::new(80, 24);
        let mut compositor = crate::compositor::Compositor::new(80, 24);
        stack.apply_to_compositor(&mut compositor, screen);

        let mut buf = crate::buffer::ScreenBuffer::new(screen);
        compositor.compose(&mut buf);

        // Corner (outside modal) should have dim style
        match buf.get(0, 0) {
            Some(cell) => assert!(cell.style.dim),
            None => unreachable!(),
        }
    }

    #[test]
    fn toast_at_top_right_pipeline() {
        use crate::widget::toast::Toast;

        let toast = Toast::new("Saved!").with_width(10);
        let lines = toast.render_to_lines();
        let screen = Size::new(80, 24);
        let config = toast.to_overlay_config(screen);

        let mut stack = ScreenStack::new();
        stack.push(config, lines);

        let mut compositor = crate::compositor::Compositor::new(80, 24);
        stack.apply_to_compositor(&mut compositor, screen);

        let mut buf = crate::buffer::ScreenBuffer::new(screen);
        compositor.compose(&mut buf);

        // Toast at top-right: x = 80-10 = 70, y = 0
        match buf.get(70, 0) {
            Some(cell) => assert!(cell.grapheme == "S"),
            None => unreachable!(),
        }
    }

    #[test]
    fn tooltip_below_anchor_pipeline() {
        use crate::overlay::Placement;
        use crate::widget::tooltip::Tooltip;

        let anchor = Rect::new(30, 5, 10, 2);
        let tooltip = Tooltip::new("hint", anchor).with_placement(Placement::Below);
        let lines = tooltip.render_to_lines();
        let screen = Size::new(80, 24);
        let config = tooltip.to_overlay_config(screen);

        let mut stack = ScreenStack::new();
        stack.push(config, lines);

        let mut compositor = crate::compositor::Compositor::new(80, 24);
        stack.apply_to_compositor(&mut compositor, screen);

        let mut buf = crate::buffer::ScreenBuffer::new(screen);
        compositor.compose(&mut buf);

        // Below anchor: y = 5 + 2 = 7, x centered: 30 + 5 - 2 = 33
        match buf.get(33, 7) {
            Some(cell) => assert!(cell.grapheme == "h"),
            None => unreachable!(),
        }
    }

    #[test]
    fn two_modals_stacked() {
        let mut stack = ScreenStack::new();

        // First modal at (10, 5)
        let config1 = OverlayConfig {
            position: OverlayPosition::At(Position::new(10, 5)),
            size: Size::new(10, 3),
            z_offset: 0,
            dim_background: false,
        };
        stack.push(config1, vec![vec![Segment::new("first")]]);

        // Second modal at same position (on top)
        let config2 = OverlayConfig {
            position: OverlayPosition::At(Position::new(10, 5)),
            size: Size::new(10, 3),
            z_offset: 0,
            dim_background: false,
        };
        stack.push(config2, vec![vec![Segment::new("second")]]);

        let screen = Size::new(80, 24);
        let mut compositor = crate::compositor::Compositor::new(80, 24);
        stack.apply_to_compositor(&mut compositor, screen);

        let mut buf = crate::buffer::ScreenBuffer::new(screen);
        compositor.compose(&mut buf);

        // Topmost (second) should be visible
        match buf.get(10, 5) {
            Some(cell) => assert!(cell.grapheme == "s"),
            None => unreachable!(),
        }
    }

    #[test]
    fn modal_plus_toast_z_order() {
        use crate::widget::modal::Modal;
        use crate::widget::toast::Toast;

        let modal = Modal::new("M", 20, 5);
        let modal_lines = modal.render_to_lines();
        let modal_config = modal.to_overlay_config();

        let toast = Toast::new("Toast!").with_width(10);
        let screen = Size::new(80, 24);
        let toast_lines = toast.render_to_lines();
        let toast_config = toast.to_overlay_config(screen);

        let mut stack = ScreenStack::new();
        stack.push(modal_config, modal_lines);
        stack.push(toast_config, toast_lines);

        let mut compositor = crate::compositor::Compositor::new(80, 24);
        stack.apply_to_compositor(&mut compositor, screen);

        let mut buf = crate::buffer::ScreenBuffer::new(screen);
        compositor.compose(&mut buf);

        // Toast at top-right should be visible (higher z since added second)
        match buf.get(70, 0) {
            Some(cell) => assert!(cell.grapheme == "T"),
            None => unreachable!(),
        }
    }

    #[test]
    fn remove_modal_clears_dim() {
        let mut stack = ScreenStack::new();
        let config = OverlayConfig {
            position: OverlayPosition::Center,
            size: Size::new(10, 3),
            z_offset: 0,
            dim_background: true,
        };
        let id = stack.push(config, vec![vec![Segment::new("x")]]);

        // Remove it
        assert!(stack.remove(id));
        assert!(stack.is_empty());

        let screen = Size::new(80, 24);
        let mut compositor = crate::compositor::Compositor::new(80, 24);
        stack.apply_to_compositor(&mut compositor, screen);

        let mut buf = crate::buffer::ScreenBuffer::new(screen);
        compositor.compose(&mut buf);

        // No dim layer should be present — corner should be blank
        match buf.get(0, 0) {
            Some(cell) => assert!(!cell.style.dim),
            None => unreachable!(),
        }
    }

    #[test]
    fn clear_removes_all_overlays() {
        let mut stack = ScreenStack::new();
        let config = OverlayConfig {
            position: OverlayPosition::At(Position::new(0, 0)),
            size: Size::new(5, 1),
            z_offset: 0,
            dim_background: false,
        };
        stack.push(config.clone(), vec![vec![Segment::new("A")]]);
        stack.push(config, vec![vec![Segment::new("B")]]);
        stack.clear();

        let screen = Size::new(80, 24);
        let mut compositor = crate::compositor::Compositor::new(80, 24);
        stack.apply_to_compositor(&mut compositor, screen);

        let mut buf = crate::buffer::ScreenBuffer::new(screen);
        compositor.compose(&mut buf);

        // Should be blank (no overlays)
        match buf.get(0, 0) {
            Some(cell) => assert!(cell.grapheme == " "),
            None => unreachable!(),
        }
    }
}
