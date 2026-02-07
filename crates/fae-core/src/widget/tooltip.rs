//! Tooltip widget — smart-positioned text near an anchor element.

use crate::geometry::{Position, Rect, Size};
use crate::overlay::{OverlayConfig, OverlayPosition, Placement};
use crate::segment::Segment;
use crate::style::Style;

/// A tooltip that appears near an anchor element with smart positioning.
///
/// If the tooltip would go off-screen in the preferred placement direction,
/// it automatically flips to the opposite side. Use [`Tooltip::to_overlay_config`]
/// to create an [`OverlayConfig`] for use with [`crate::overlay::ScreenStack`].
#[derive(Clone, Debug)]
pub struct Tooltip {
    text: String,
    anchor: Rect,
    placement: Placement,
    style: Style,
}

impl Tooltip {
    /// Create a new tooltip with text anchored to a rectangle (defaults to Below).
    pub fn new(text: impl Into<String>, anchor: Rect) -> Self {
        Self {
            text: text.into(),
            anchor,
            placement: Placement::Below,
            style: Style::default(),
        }
    }

    /// Set the preferred placement direction.
    #[must_use]
    pub fn with_placement(mut self, placement: Placement) -> Self {
        self.placement = placement;
        self
    }

    /// Set the tooltip style.
    #[must_use]
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Render the tooltip to lines ready for the compositor.
    pub fn render_to_lines(&self) -> Vec<Vec<Segment>> {
        vec![vec![Segment::styled(&self.text, self.style.clone())]]
    }

    /// Compute the tooltip size based on text length.
    fn size(&self) -> Size {
        let w = self.text.len() as u16;
        Size::new(w.max(1), 1)
    }

    /// Compute the tooltip position with smart flip when off-screen.
    ///
    /// If the tooltip would go off-screen in the preferred direction,
    /// it flips to the opposite side:
    /// - Above flips to Below if at top edge
    /// - Below flips to Above if at bottom edge
    /// - Left flips to Right if at left edge
    /// - Right flips to Left if at right edge
    pub fn compute_position(&self, screen: Size) -> Position {
        let tip_size = self.size();
        let effective_placement = self.flip_if_needed(screen, tip_size);

        // Center the tooltip along the axis perpendicular to the placement
        match effective_placement {
            Placement::Above => {
                let x = self
                    .anchor
                    .position
                    .x
                    .saturating_add(self.anchor.size.width / 2)
                    .saturating_sub(tip_size.width / 2);
                let y = self.anchor.position.y.saturating_sub(tip_size.height);
                Position::new(x, y)
            }
            Placement::Below => {
                let x = self
                    .anchor
                    .position
                    .x
                    .saturating_add(self.anchor.size.width / 2)
                    .saturating_sub(tip_size.width / 2);
                let y = self
                    .anchor
                    .position
                    .y
                    .saturating_add(self.anchor.size.height);
                Position::new(x, y)
            }
            Placement::Left => {
                let x = self.anchor.position.x.saturating_sub(tip_size.width);
                let y = self
                    .anchor
                    .position
                    .y
                    .saturating_add(self.anchor.size.height / 2)
                    .saturating_sub(tip_size.height / 2);
                Position::new(x, y)
            }
            Placement::Right => {
                let x = self
                    .anchor
                    .position
                    .x
                    .saturating_add(self.anchor.size.width);
                let y = self
                    .anchor
                    .position
                    .y
                    .saturating_add(self.anchor.size.height / 2)
                    .saturating_sub(tip_size.height / 2);
                Position::new(x, y)
            }
        }
    }

    /// Create an overlay config for this tooltip (no dim background).
    pub fn to_overlay_config(&self, screen: Size) -> OverlayConfig {
        let pos = self.compute_position(screen);
        OverlayConfig {
            position: OverlayPosition::At(pos),
            size: self.size(),
            z_offset: 0,
            dim_background: false,
        }
    }

    /// Determine the effective placement, flipping if the tooltip would go off-screen.
    fn flip_if_needed(&self, screen: Size, tip_size: Size) -> Placement {
        match self.placement {
            Placement::Above => {
                if self.anchor.position.y < tip_size.height {
                    Placement::Below
                } else {
                    Placement::Above
                }
            }
            Placement::Below => {
                let bottom = self
                    .anchor
                    .position
                    .y
                    .saturating_add(self.anchor.size.height)
                    .saturating_add(tip_size.height);
                if bottom > screen.height {
                    Placement::Above
                } else {
                    Placement::Below
                }
            }
            Placement::Left => {
                if self.anchor.position.x < tip_size.width {
                    Placement::Right
                } else {
                    Placement::Left
                }
            }
            Placement::Right => {
                let right_edge = self
                    .anchor
                    .position
                    .x
                    .saturating_add(self.anchor.size.width)
                    .saturating_add(tip_size.width);
                if right_edge > screen.width {
                    Placement::Left
                } else {
                    Placement::Right
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn screen() -> Size {
        Size::new(80, 24)
    }

    #[test]
    fn tooltip_above_anchor() {
        let anchor = Rect::new(30, 10, 10, 2);
        let t = Tooltip::new("tip", anchor).with_placement(Placement::Above);
        let pos = t.compute_position(screen());
        // x centered: 30 + 5 - 1 = 34
        assert!(pos.x == 34);
        // y above: 10 - 1 = 9
        assert!(pos.y == 9);
    }

    #[test]
    fn tooltip_below_anchor() {
        let anchor = Rect::new(30, 10, 10, 2);
        let t = Tooltip::new("tip", anchor).with_placement(Placement::Below);
        let pos = t.compute_position(screen());
        assert!(pos.x == 34);
        // y below: 10 + 2 = 12
        assert!(pos.y == 12);
    }

    #[test]
    fn tooltip_left_of_anchor() {
        let anchor = Rect::new(30, 10, 10, 2);
        let t = Tooltip::new("tip", anchor).with_placement(Placement::Left);
        let pos = t.compute_position(screen());
        // x left: 30 - 3 = 27
        assert!(pos.x == 27);
        // y centered: 10 + 1 - 0 = 11
        assert!(pos.y == 11);
    }

    #[test]
    fn tooltip_right_of_anchor() {
        let anchor = Rect::new(30, 10, 10, 2);
        let t = Tooltip::new("tip", anchor).with_placement(Placement::Right);
        let pos = t.compute_position(screen());
        // x right: 30 + 10 = 40
        assert!(pos.x == 40);
        assert!(pos.y == 11);
    }

    #[test]
    fn above_flips_to_below_at_top_edge() {
        let anchor = Rect::new(30, 0, 10, 1);
        let t = Tooltip::new("tip", anchor).with_placement(Placement::Above);
        let pos = t.compute_position(screen());
        // Should flip to below: y = 0 + 1 = 1
        assert!(pos.y == 1);
    }

    #[test]
    fn below_flips_to_above_at_bottom_edge() {
        let anchor = Rect::new(30, 23, 10, 1);
        let t = Tooltip::new("tip", anchor).with_placement(Placement::Below);
        let pos = t.compute_position(screen());
        // Should flip to above: y = 23 - 1 = 22
        assert!(pos.y == 22);
    }

    #[test]
    fn left_flips_to_right_at_left_edge() {
        let anchor = Rect::new(1, 10, 5, 2);
        let t = Tooltip::new("tip", anchor).with_placement(Placement::Left);
        let pos = t.compute_position(screen());
        // Should flip to right: x = 1 + 5 = 6
        assert!(pos.x == 6);
    }

    #[test]
    fn right_flips_to_left_at_right_edge() {
        let anchor = Rect::new(75, 10, 5, 2);
        let t = Tooltip::new("tip", anchor).with_placement(Placement::Right);
        let pos = t.compute_position(screen());
        // Should flip to left: x = 75 - 3 = 72
        assert!(pos.x == 72);
    }

    #[test]
    fn tooltip_text_renders() {
        let anchor = Rect::new(10, 10, 5, 2);
        let t = Tooltip::new("Hello World", anchor);
        let lines = t.render_to_lines();
        assert!(lines.len() == 1);
        assert!(lines[0][0].text == "Hello World");
    }

    #[test]
    fn style_preserved() {
        let anchor = Rect::new(10, 10, 5, 2);
        let style = Style::new().italic(true);
        let t = Tooltip::new("tip", anchor).with_style(style);
        let lines = t.render_to_lines();
        assert!(lines[0][0].style.italic);
    }

    #[test]
    fn overlay_config_no_dim() {
        let anchor = Rect::new(10, 10, 5, 2);
        let t = Tooltip::new("tip", anchor);
        let config = t.to_overlay_config(screen());
        assert!(!config.dim_background);
    }

    #[test]
    fn default_placement_is_below() {
        let anchor = Rect::new(30, 10, 10, 2);
        let t = Tooltip::new("tip", anchor);
        let pos = t.compute_position(screen());
        // Below: y = 10 + 2 = 12
        assert!(pos.y == 12);
    }
}
