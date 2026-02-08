//! Scroll region management.
//!
//! Tracks scroll state for widgets with `overflow: scroll` or `overflow: auto`,
//! computing visible content regions within viewports.

use std::collections::HashMap;

use crate::focus::WidgetId;
use crate::geometry::Rect;
use crate::tcss::cascade::ComputedStyle;
use crate::tcss::property::PropertyName;
use crate::tcss::value::CssValue;

/// Overflow behavior for a single axis.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OverflowBehavior {
    /// Content is not clipped and may overflow.
    #[default]
    Visible,
    /// Content is clipped at the boundary.
    Hidden,
    /// Content is clipped and scrollbars are shown.
    Scroll,
    /// Content is clipped; scrollbars shown only when needed.
    Auto,
}

/// Scroll state for a single widget.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ScrollState {
    /// Horizontal scroll offset in cells.
    pub offset_x: u16,
    /// Vertical scroll offset in cells.
    pub offset_y: u16,
    /// Total content width in cells.
    pub content_width: u16,
    /// Total content height in cells.
    pub content_height: u16,
    /// Viewport width in cells.
    pub viewport_width: u16,
    /// Viewport height in cells.
    pub viewport_height: u16,
}

impl ScrollState {
    /// Create a new scroll state.
    pub const fn new(
        content_width: u16,
        content_height: u16,
        viewport_width: u16,
        viewport_height: u16,
    ) -> Self {
        Self {
            offset_x: 0,
            offset_y: 0,
            content_width,
            content_height,
            viewport_width,
            viewport_height,
        }
    }

    /// Whether horizontal scrolling is possible.
    pub const fn can_scroll_x(&self) -> bool {
        self.content_width > self.viewport_width
    }

    /// Whether vertical scrolling is possible.
    pub const fn can_scroll_y(&self) -> bool {
        self.content_height > self.viewport_height
    }

    /// Maximum horizontal scroll offset.
    pub fn max_offset_x(&self) -> u16 {
        self.content_width.saturating_sub(self.viewport_width)
    }

    /// Maximum vertical scroll offset.
    pub fn max_offset_y(&self) -> u16 {
        self.content_height.saturating_sub(self.viewport_height)
    }

    /// The visible content rectangle (in content coordinates).
    pub fn visible_rect(&self) -> Rect {
        Rect::new(
            self.offset_x,
            self.offset_y,
            self.viewport_width,
            self.viewport_height,
        )
    }
}

/// Manages scroll regions for all scrollable widgets.
pub struct ScrollManager {
    regions: HashMap<WidgetId, ScrollState>,
}

impl ScrollManager {
    /// Create a new scroll manager.
    pub fn new() -> Self {
        Self {
            regions: HashMap::new(),
        }
    }

    /// Register a scrollable region for a widget.
    pub fn register(
        &mut self,
        widget_id: WidgetId,
        content_width: u16,
        content_height: u16,
        viewport_width: u16,
        viewport_height: u16,
    ) {
        self.regions.insert(
            widget_id,
            ScrollState::new(
                content_width,
                content_height,
                viewport_width,
                viewport_height,
            ),
        );
    }

    /// Scroll by a relative offset, clamping to valid range.
    pub fn scroll_by(&mut self, widget_id: WidgetId, dx: i16, dy: i16) {
        if let Some(state) = self.regions.get_mut(&widget_id) {
            let new_x = i32::from(state.offset_x) + i32::from(dx);
            let new_y = i32::from(state.offset_y) + i32::from(dy);
            state.offset_x = clamp_offset(new_x, state.max_offset_x());
            state.offset_y = clamp_offset(new_y, state.max_offset_y());
        }
    }

    /// Scroll to an absolute position, clamping to valid range.
    pub fn scroll_to(&mut self, widget_id: WidgetId, x: u16, y: u16) {
        if let Some(state) = self.regions.get_mut(&widget_id) {
            state.offset_x = x.min(state.max_offset_x());
            state.offset_y = y.min(state.max_offset_y());
        }
    }

    /// Get the scroll state for a widget.
    pub fn get(&self, widget_id: WidgetId) -> Option<&ScrollState> {
        self.regions.get(&widget_id)
    }

    /// Check if a widget can scroll horizontally.
    pub fn can_scroll_x(&self, widget_id: WidgetId) -> bool {
        self.regions
            .get(&widget_id)
            .is_some_and(|s| s.can_scroll_x())
    }

    /// Check if a widget can scroll vertically.
    pub fn can_scroll_y(&self, widget_id: WidgetId) -> bool {
        self.regions
            .get(&widget_id)
            .is_some_and(|s| s.can_scroll_y())
    }

    /// Get the visible content rectangle for a scrollable widget.
    pub fn visible_rect(&self, widget_id: WidgetId) -> Option<Rect> {
        self.regions.get(&widget_id).map(|s| s.visible_rect())
    }

    /// Remove a scroll region.
    pub fn remove(&mut self, widget_id: WidgetId) {
        self.regions.remove(&widget_id);
    }
}

impl Default for ScrollManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract overflow behavior from a computed style.
///
/// Returns `(overflow_x, overflow_y)`.
pub fn extract_overflow(style: &ComputedStyle) -> (OverflowBehavior, OverflowBehavior) {
    let base = style
        .get(&PropertyName::Overflow)
        .map(keyword_to_overflow)
        .unwrap_or_default();
    let ox = style
        .get(&PropertyName::OverflowX)
        .map(keyword_to_overflow)
        .unwrap_or(base);
    let oy = style
        .get(&PropertyName::OverflowY)
        .map(keyword_to_overflow)
        .unwrap_or(base);
    (ox, oy)
}

/// Convert a CSS keyword to [`OverflowBehavior`].
fn keyword_to_overflow(value: &CssValue) -> OverflowBehavior {
    match value {
        CssValue::Keyword(k) => match k.to_ascii_lowercase().as_str() {
            "visible" => OverflowBehavior::Visible,
            "hidden" => OverflowBehavior::Hidden,
            "scroll" => OverflowBehavior::Scroll,
            "auto" => OverflowBehavior::Auto,
            _ => OverflowBehavior::Visible,
        },
        _ => OverflowBehavior::Visible,
    }
}

/// Clamp a signed offset to `[0, max]`.
fn clamp_offset(value: i32, max: u16) -> u16 {
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

    fn wid(n: u64) -> WidgetId {
        n
    }

    #[test]
    fn scroll_state_creation() {
        let state = ScrollState::new(100, 200, 80, 24);
        assert_eq!(state.content_width, 100);
        assert_eq!(state.content_height, 200);
        assert_eq!(state.viewport_width, 80);
        assert_eq!(state.viewport_height, 24);
        assert_eq!(state.offset_x, 0);
        assert_eq!(state.offset_y, 0);
    }

    #[test]
    fn scroll_state_can_scroll() {
        let state = ScrollState::new(100, 200, 80, 24);
        assert!(state.can_scroll_x());
        assert!(state.can_scroll_y());

        let no_scroll = ScrollState::new(40, 10, 80, 24);
        assert!(!no_scroll.can_scroll_x());
        assert!(!no_scroll.can_scroll_y());
    }

    #[test]
    fn scroll_state_max_offsets() {
        let state = ScrollState::new(100, 200, 80, 24);
        assert_eq!(state.max_offset_x(), 20);
        assert_eq!(state.max_offset_y(), 176);
    }

    #[test]
    fn scroll_state_visible_rect() {
        let mut state = ScrollState::new(100, 200, 80, 24);
        assert_eq!(state.visible_rect(), Rect::new(0, 0, 80, 24));
        state.offset_x = 5;
        state.offset_y = 10;
        assert_eq!(state.visible_rect(), Rect::new(5, 10, 80, 24));
    }

    #[test]
    fn manager_register_and_get() {
        let mut mgr = ScrollManager::new();
        mgr.register(wid(1), 100, 200, 80, 24);
        let state = mgr.get(wid(1));
        assert!(state.is_some());
        let state = match state {
            Some(s) => s,
            None => unreachable!(),
        };
        assert_eq!(state.content_width, 100);
    }

    #[test]
    fn manager_scroll_by_clamps() {
        let mut mgr = ScrollManager::new();
        mgr.register(wid(1), 100, 200, 80, 24);

        mgr.scroll_by(wid(1), 10, 20);
        let state = mgr.get(wid(1));
        assert!(state.is_some());
        let state = match state {
            Some(s) => s,
            None => unreachable!(),
        };
        assert_eq!(state.offset_x, 10);
        assert_eq!(state.offset_y, 20);

        // Scroll beyond max
        mgr.scroll_by(wid(1), 100, 1000);
        let state = match mgr.get(wid(1)) {
            Some(s) => s,
            None => unreachable!(),
        };
        assert_eq!(state.offset_x, 20); // max is 20
        assert_eq!(state.offset_y, 176); // max is 176

        // Scroll negative past zero
        mgr.scroll_by(wid(1), -100, -1000);
        let state = match mgr.get(wid(1)) {
            Some(s) => s,
            None => unreachable!(),
        };
        assert_eq!(state.offset_x, 0);
        assert_eq!(state.offset_y, 0);
    }

    #[test]
    fn manager_scroll_to() {
        let mut mgr = ScrollManager::new();
        mgr.register(wid(1), 100, 200, 80, 24);

        mgr.scroll_to(wid(1), 15, 100);
        let state = match mgr.get(wid(1)) {
            Some(s) => s,
            None => unreachable!(),
        };
        assert_eq!(state.offset_x, 15);
        assert_eq!(state.offset_y, 100);

        // Clamp to max
        mgr.scroll_to(wid(1), 500, 500);
        let state = match mgr.get(wid(1)) {
            Some(s) => s,
            None => unreachable!(),
        };
        assert_eq!(state.offset_x, 20);
        assert_eq!(state.offset_y, 176);
    }

    #[test]
    fn manager_can_scroll() {
        let mut mgr = ScrollManager::new();
        mgr.register(wid(1), 100, 200, 80, 24);
        assert!(mgr.can_scroll_x(wid(1)));
        assert!(mgr.can_scroll_y(wid(1)));
        assert!(!mgr.can_scroll_x(wid(999)));
        assert!(!mgr.can_scroll_y(wid(999)));
    }

    #[test]
    fn manager_visible_rect() {
        let mut mgr = ScrollManager::new();
        mgr.register(wid(1), 100, 200, 80, 24);
        let rect = mgr.visible_rect(wid(1));
        assert_eq!(rect, Some(Rect::new(0, 0, 80, 24)));
        assert_eq!(mgr.visible_rect(wid(999)), None);
    }

    #[test]
    fn manager_remove() {
        let mut mgr = ScrollManager::new();
        mgr.register(wid(1), 100, 200, 80, 24);
        mgr.remove(wid(1));
        assert!(mgr.get(wid(1)).is_none());
    }

    #[test]
    fn extract_overflow_default() {
        let style = ComputedStyle::new();
        let (ox, oy) = extract_overflow(&style);
        assert_eq!(ox, OverflowBehavior::Visible);
        assert_eq!(oy, OverflowBehavior::Visible);
    }

    #[test]
    fn extract_overflow_shorthand() {
        let mut style = ComputedStyle::new();
        style.set(PropertyName::Overflow, CssValue::Keyword("hidden".into()));
        let (ox, oy) = extract_overflow(&style);
        assert_eq!(ox, OverflowBehavior::Hidden);
        assert_eq!(oy, OverflowBehavior::Hidden);
    }

    #[test]
    fn extract_overflow_xy_separate() {
        let mut style = ComputedStyle::new();
        style.set(PropertyName::OverflowX, CssValue::Keyword("scroll".into()));
        style.set(PropertyName::OverflowY, CssValue::Keyword("hidden".into()));
        let (ox, oy) = extract_overflow(&style);
        assert_eq!(ox, OverflowBehavior::Scroll);
        assert_eq!(oy, OverflowBehavior::Hidden);
    }

    #[test]
    fn extract_overflow_auto() {
        let mut style = ComputedStyle::new();
        style.set(PropertyName::Overflow, CssValue::Keyword("auto".into()));
        let (ox, oy) = extract_overflow(&style);
        assert_eq!(ox, OverflowBehavior::Auto);
        assert_eq!(oy, OverflowBehavior::Auto);
    }

    #[test]
    fn overflow_behavior_default() {
        assert_eq!(OverflowBehavior::default(), OverflowBehavior::Visible);
    }

    #[test]
    fn scroll_state_no_scroll_max_offset_zero() {
        let state = ScrollState::new(40, 10, 80, 24);
        assert_eq!(state.max_offset_x(), 0);
        assert_eq!(state.max_offset_y(), 0);
    }
}
