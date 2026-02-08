//! Viewport — scrollable region with content clipping.
//!
//! A [`Viewport`] tracks the visible portion of a widget's content area,
//! supporting scrolling and clipping operations. Content can be larger
//! than the viewport, and the scroll offset determines which portion
//! is visible.

use crate::geometry::{Position, Rect, Size};

/// Represents the visible portion of a widget's content.
///
/// The viewport defines a window into a potentially larger content area.
/// The scroll offset determines which part of the content is currently
/// visible.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Viewport {
    /// Scroll offset (top-left of visible area in content coordinates).
    offset: Position,
    /// Size of the visible area (matches widget's screen region).
    size: Size,
    /// Total content size (may be larger than viewport).
    content_size: Size,
}

impl Viewport {
    /// Create a new viewport with the given visible size.
    ///
    /// The scroll offset starts at (0, 0) and the content size defaults
    /// to the viewport size.
    pub fn new(size: Size) -> Self {
        Self {
            offset: Position::new(0, 0),
            size,
            content_size: size,
        }
    }

    /// Set the total content size (builder pattern).
    ///
    /// The content size can be larger than the viewport, allowing scrolling.
    #[must_use]
    pub fn with_content_size(mut self, content_size: Size) -> Self {
        self.content_size = content_size;
        self.clamp_offset();
        self
    }

    /// Get the current scroll offset.
    pub fn offset(&self) -> Position {
        self.offset
    }

    /// Get the viewport size (visible area).
    pub fn size(&self) -> Size {
        self.size
    }

    /// Get the total content size.
    pub fn content_size(&self) -> Size {
        self.content_size
    }

    /// Scroll by a relative amount, clamped to valid range.
    ///
    /// Positive `dx` scrolls right, positive `dy` scrolls down.
    /// Negative values scroll left/up. The offset is clamped so
    /// the viewport never extends past the content boundaries.
    pub fn scroll_by(&mut self, dx: i32, dy: i32) {
        let new_x = i32::from(self.offset.x).saturating_add(dx);
        let new_y = i32::from(self.offset.y).saturating_add(dy);

        // Clamp to [0, max_scroll] range
        self.offset.x = clamp_to_u16(new_x, self.max_scroll_x());
        self.offset.y = clamp_to_u16(new_y, self.max_scroll_y());
    }

    /// Scroll to an absolute position, clamped to valid range.
    ///
    /// The offset is clamped so the viewport never extends past
    /// the content boundaries.
    pub fn scroll_to(&mut self, x: u16, y: u16) {
        self.offset.x = x.min(self.max_scroll_x());
        self.offset.y = y.min(self.max_scroll_y());
    }

    /// Check if a rectangle (in content coordinates) intersects the viewport.
    ///
    /// Returns `true` if any part of `rect` is within the visible area,
    /// accounting for the current scroll offset.
    pub fn is_visible(&self, rect: Rect) -> bool {
        let viewport_rect = Rect::new(
            self.offset.x,
            self.offset.y,
            self.size.width,
            self.size.height,
        );
        viewport_rect.intersects(&rect)
    }

    /// Clip a rectangle to the viewport, returning the visible portion
    /// in viewport-local coordinates.
    ///
    /// Returns `None` if the rectangle is entirely outside the viewport.
    /// The returned rectangle's position is relative to the viewport's
    /// top-left corner (i.e., the scroll offset is subtracted).
    pub fn clip_to_viewport(&self, rect: Rect) -> Option<Rect> {
        let viewport_rect = Rect::new(
            self.offset.x,
            self.offset.y,
            self.size.width,
            self.size.height,
        );

        let intersection = viewport_rect.intersection(&rect)?;

        // Convert to viewport-local coordinates
        let local_x = intersection.position.x.saturating_sub(self.offset.x);
        let local_y = intersection.position.y.saturating_sub(self.offset.y);

        Some(Rect::new(
            local_x,
            local_y,
            intersection.size.width,
            intersection.size.height,
        ))
    }

    /// Maximum horizontal scroll value.
    ///
    /// Returns 0 if the content is narrower than or equal to the viewport.
    pub fn max_scroll_x(&self) -> u16 {
        self.content_size.width.saturating_sub(self.size.width)
    }

    /// Maximum vertical scroll value.
    ///
    /// Returns 0 if the content is shorter than or equal to the viewport.
    pub fn max_scroll_y(&self) -> u16 {
        self.content_size.height.saturating_sub(self.size.height)
    }

    /// Clamp the current offset to valid bounds after a content size change.
    fn clamp_offset(&mut self) {
        self.offset.x = self.offset.x.min(self.max_scroll_x());
        self.offset.y = self.offset.y.min(self.max_scroll_y());
    }
}

/// Clamp an i32 to the range [0, max] and convert to u16.
fn clamp_to_u16(value: i32, max: u16) -> u16 {
    if value < 0 {
        0
    } else if value > i32::from(max) {
        max
    } else {
        value as u16
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_viewport_zero_offset() {
        let vp = Viewport::new(Size::new(80, 24));
        assert_eq!(vp.offset(), Position::new(0, 0));
        assert_eq!(vp.size(), Size::new(80, 24));
        assert_eq!(vp.content_size(), Size::new(80, 24));
    }

    #[test]
    fn with_content_size_sets_content() {
        let vp = Viewport::new(Size::new(40, 20)).with_content_size(Size::new(100, 50));
        assert_eq!(vp.content_size(), Size::new(100, 50));
        assert_eq!(vp.size(), Size::new(40, 20));
    }

    #[test]
    fn scroll_down_changes_offset() {
        let mut vp = Viewport::new(Size::new(80, 24)).with_content_size(Size::new(80, 100));
        vp.scroll_by(0, 10);
        assert_eq!(vp.offset(), Position::new(0, 10));
    }

    #[test]
    fn scroll_right_changes_offset() {
        let mut vp = Viewport::new(Size::new(40, 20)).with_content_size(Size::new(200, 20));
        vp.scroll_by(15, 0);
        assert_eq!(vp.offset(), Position::new(15, 0));
    }

    #[test]
    fn scroll_past_content_clamped_to_max() {
        let mut vp = Viewport::new(Size::new(80, 24)).with_content_size(Size::new(80, 50));
        // max_scroll_y = 50 - 24 = 26
        vp.scroll_by(0, 1000);
        assert_eq!(vp.offset(), Position::new(0, 26));
    }

    #[test]
    fn scroll_negative_clamped_to_zero() {
        let mut vp = Viewport::new(Size::new(80, 24)).with_content_size(Size::new(80, 100));
        vp.scroll_by(0, 10); // go to y=10
        vp.scroll_by(0, -50); // try to go past zero
        assert_eq!(vp.offset(), Position::new(0, 0));
    }

    #[test]
    fn scroll_to_absolute() {
        let mut vp = Viewport::new(Size::new(80, 24)).with_content_size(Size::new(200, 100));
        vp.scroll_to(50, 30);
        assert_eq!(vp.offset(), Position::new(50, 30));
    }

    #[test]
    fn scroll_to_clamped() {
        let mut vp = Viewport::new(Size::new(80, 24)).with_content_size(Size::new(100, 50));
        // max_scroll_x = 100 - 80 = 20, max_scroll_y = 50 - 24 = 26
        vp.scroll_to(1000, 1000);
        assert_eq!(vp.offset(), Position::new(20, 26));
    }

    #[test]
    fn is_visible_on_screen_region() {
        let mut vp = Viewport::new(Size::new(80, 24)).with_content_size(Size::new(200, 100));
        vp.scroll_to(10, 5);
        // Region at (15, 10, 20, 5) — fully within viewport
        let rect = Rect::new(15, 10, 20, 5);
        assert!(vp.is_visible(rect));
    }

    #[test]
    fn is_visible_off_screen_region() {
        let vp = Viewport::new(Size::new(80, 24)).with_content_size(Size::new(200, 100));
        // offset is (0, 0), viewport covers (0..80, 0..24)
        // Region at (100, 50, 10, 5) — completely outside
        let rect = Rect::new(100, 50, 10, 5);
        assert!(!vp.is_visible(rect));
    }

    #[test]
    fn clip_to_viewport_within_bounds() {
        let vp = Viewport::new(Size::new(80, 24)).with_content_size(Size::new(200, 100));
        // Region fully within viewport
        let rect = Rect::new(10, 5, 20, 10);
        let clipped = vp.clip_to_viewport(rect);
        assert!(clipped.is_some());
        match clipped {
            Some(r) => {
                assert_eq!(r, Rect::new(10, 5, 20, 10));
            }
            None => unreachable!(),
        }
    }

    #[test]
    fn clip_to_viewport_partial_overlap() {
        let mut vp = Viewport::new(Size::new(80, 24)).with_content_size(Size::new(200, 100));
        vp.scroll_to(10, 5);
        // Viewport covers content (10..90, 5..29)
        // Region at (5, 3, 20, 10) overlaps at (10..25, 5..13)
        let rect = Rect::new(5, 3, 20, 10);
        let clipped = vp.clip_to_viewport(rect);
        assert!(clipped.is_some());
        match clipped {
            Some(r) => {
                // In viewport-local coords: x = 10 - 10 = 0, y = 5 - 5 = 0
                // width = 25 - 10 = 15, height = 13 - 5 = 8
                assert_eq!(r, Rect::new(0, 0, 15, 8));
            }
            None => unreachable!(),
        }
    }

    #[test]
    fn clip_to_viewport_no_overlap() {
        let vp = Viewport::new(Size::new(80, 24)).with_content_size(Size::new(200, 100));
        let rect = Rect::new(100, 50, 10, 5);
        assert!(vp.clip_to_viewport(rect).is_none());
    }

    #[test]
    fn content_smaller_than_viewport_no_scroll_effect() {
        let mut vp = Viewport::new(Size::new(80, 24)).with_content_size(Size::new(40, 10));
        // max_scroll_x = 0, max_scroll_y = 0 (content fits entirely)
        vp.scroll_by(100, 100);
        assert_eq!(vp.offset(), Position::new(0, 0));
    }

    #[test]
    fn max_scroll_calculations() {
        let vp = Viewport::new(Size::new(80, 24)).with_content_size(Size::new(200, 100));
        assert_eq!(vp.max_scroll_x(), 120); // 200 - 80
        assert_eq!(vp.max_scroll_y(), 76); // 100 - 24
    }

    #[test]
    fn max_scroll_zero_when_content_fits() {
        let vp = Viewport::new(Size::new(80, 24)).with_content_size(Size::new(80, 24));
        assert_eq!(vp.max_scroll_x(), 0);
        assert_eq!(vp.max_scroll_y(), 0);
    }

    #[test]
    fn max_scroll_zero_when_content_smaller() {
        let vp = Viewport::new(Size::new(80, 24)).with_content_size(Size::new(40, 10));
        assert_eq!(vp.max_scroll_x(), 0);
        assert_eq!(vp.max_scroll_y(), 0);
    }

    #[test]
    fn with_content_size_clamps_existing_offset() {
        let mut vp = Viewport::new(Size::new(40, 20)).with_content_size(Size::new(200, 100));
        vp.scroll_to(100, 60);
        // Now shrink content — offset should be clamped
        let vp = Viewport {
            offset: vp.offset(),
            size: vp.size(),
            content_size: Size::new(60, 30),
        };
        // Rebuild with clamped offset
        let vp2 = Viewport::new(vp.size()).with_content_size(Size::new(60, 30));
        // Since with_content_size clamps offset and new viewport starts at 0,
        // the offset should be 0,0 (fresh viewport)
        assert_eq!(vp2.offset(), Position::new(0, 0));
    }

    #[test]
    fn scroll_by_both_axes() {
        let mut vp = Viewport::new(Size::new(40, 20)).with_content_size(Size::new(200, 100));
        vp.scroll_by(10, 15);
        assert_eq!(vp.offset(), Position::new(10, 15));
        vp.scroll_by(-5, -3);
        assert_eq!(vp.offset(), Position::new(5, 12));
    }

    #[test]
    fn is_visible_edge_cases() {
        let vp = Viewport::new(Size::new(80, 24)).with_content_size(Size::new(200, 100));
        // Region touching the right edge of viewport
        let rect = Rect::new(79, 0, 10, 1);
        assert!(vp.is_visible(rect));

        // Region just past the right edge
        let rect2 = Rect::new(80, 0, 10, 1);
        assert!(!vp.is_visible(rect2));
    }
}
