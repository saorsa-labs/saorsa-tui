//! Animated loading indicator widget.
//!
//! Displays spinning/cycling characters to indicate an ongoing operation.
//! Supports multiple animation styles and an optional message.

use crate::buffer::ScreenBuffer;
use crate::cell::Cell;
use crate::geometry::Rect;
use crate::style::Style;
use crate::text::truncate_to_display_width;
use unicode_width::UnicodeWidthStr;

use super::Widget;

/// Animation style for the loading indicator.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IndicatorStyle {
    /// Braille spinner: ⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏
    Spinner,
    /// Braille dots: ⠁⠂⠄⡀⢀⠠⠐⠈
    Dots,
    /// Line rotation: ─\|/
    Line,
    /// Box rotation: ▖▘▝▗
    Box,
    /// Circle rotation: ◐◓◑◒
    Circle,
}

impl IndicatorStyle {
    /// Get the frame sequence for this style.
    fn frames(self) -> &'static [&'static str] {
        match self {
            IndicatorStyle::Spinner => &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
            IndicatorStyle::Dots => &["⠁", "⠂", "⠄", "⡀", "⢀", "⠠", "⠐", "⠈"],
            IndicatorStyle::Line => &["─", "\\", "|", "/"],
            IndicatorStyle::Box => &["▖", "▘", "▝", "▗"],
            IndicatorStyle::Circle => &["◐", "◓", "◑", "◒"],
        }
    }
}

/// An animated loading indicator widget.
///
/// Renders a cycling character animation to indicate progress.
/// Call [`tick`](Self::tick) to advance the animation frame.
pub struct LoadingIndicator {
    /// Animation style.
    style: IndicatorStyle,
    /// Current animation frame index.
    frame: usize,
    /// Visual style for the indicator character.
    indicator_style: Style,
    /// Optional message displayed after the indicator.
    message: Option<String>,
}

impl LoadingIndicator {
    /// Create a new loading indicator with the default Spinner style.
    pub fn new() -> Self {
        Self {
            style: IndicatorStyle::Spinner,
            frame: 0,
            indicator_style: Style::default(),
            message: None,
        }
    }

    /// Set the animation style.
    #[must_use]
    pub fn with_style(mut self, style: IndicatorStyle) -> Self {
        self.style = style;
        self.frame = 0;
        self
    }

    /// Set the visual style for the indicator character.
    #[must_use]
    pub fn with_indicator_style(mut self, style: Style) -> Self {
        self.indicator_style = style;
        self
    }

    /// Set the message displayed next to the indicator.
    #[must_use]
    pub fn with_message(mut self, message: &str) -> Self {
        self.message = Some(message.to_string());
        self
    }

    /// Advance the animation to the next frame.
    pub fn tick(&mut self) {
        let len = self.style.frames().len();
        if len > 0 {
            self.frame = (self.frame + 1) % len;
        }
    }

    /// Reset the animation to the first frame.
    pub fn reset(&mut self) {
        self.frame = 0;
    }

    /// Get the current frame index.
    pub fn frame(&self) -> usize {
        self.frame
    }

    /// Get the current animation style.
    pub fn animation_style(&self) -> IndicatorStyle {
        self.style
    }
}

impl Default for LoadingIndicator {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for LoadingIndicator {
    fn render(&self, area: Rect, buf: &mut ScreenBuffer) {
        if area.size.width == 0 || area.size.height == 0 {
            return;
        }

        let frames = self.style.frames();
        if frames.is_empty() {
            return;
        }

        let frame_idx = self.frame % frames.len();
        let ch = frames[frame_idx];
        let w = area.size.width as usize;
        let x0 = area.position.x;
        let y = area.position.y;

        // Render indicator character
        let char_w = UnicodeWidthStr::width(ch);
        if char_w > w {
            return;
        }

        buf.set(x0, y, Cell::new(ch, self.indicator_style.clone()));
        let mut col = char_w as u16;

        // Render message if present
        if let Some(ref msg) = self.message
            && (col as usize) < w
        {
            // Space between indicator and message
            buf.set(x0 + col, y, Cell::new(" ", self.indicator_style.clone()));
            col += 1;

            if (col as usize) < w {
                let remaining = w.saturating_sub(col as usize);
                let truncated = truncate_to_display_width(msg, remaining);
                for ch in truncated.chars() {
                    if col as usize >= w {
                        break;
                    }
                    let cw = UnicodeWidthStr::width(ch.encode_utf8(&mut [0; 4]) as &str);
                    if col as usize + cw > w {
                        break;
                    }
                    buf.set(
                        x0 + col,
                        y,
                        Cell::new(ch.to_string(), self.indicator_style.clone()),
                    );
                    col += cw as u16;
                }
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::geometry::Size;

    #[test]
    fn create_default() {
        let li = LoadingIndicator::new();
        assert_eq!(li.animation_style(), IndicatorStyle::Spinner);
        assert_eq!(li.frame(), 0);
    }

    #[test]
    fn default_trait() {
        let li: LoadingIndicator = Default::default();
        assert_eq!(li.animation_style(), IndicatorStyle::Spinner);
    }

    #[test]
    fn each_indicator_style() {
        let styles = [
            IndicatorStyle::Spinner,
            IndicatorStyle::Dots,
            IndicatorStyle::Line,
            IndicatorStyle::Box,
            IndicatorStyle::Circle,
        ];
        for style in &styles {
            let li = LoadingIndicator::new().with_style(*style);
            assert_eq!(li.animation_style(), *style);
            assert!(!style.frames().is_empty());
        }
    }

    #[test]
    fn render_at_different_frames() {
        let mut li = LoadingIndicator::new().with_style(IndicatorStyle::Spinner);
        let mut buf = ScreenBuffer::new(Size::new(5, 1));

        li.render(Rect::new(0, 0, 5, 1), &mut buf);
        let first = buf.get(0, 0).unwrap().grapheme.clone();
        assert_eq!(first, "⠋");

        li.tick();
        let mut buf2 = ScreenBuffer::new(Size::new(5, 1));
        li.render(Rect::new(0, 0, 5, 1), &mut buf2);
        let second = buf2.get(0, 0).unwrap().grapheme.clone();
        assert_eq!(second, "⠙");
    }

    #[test]
    fn tick_advances_frame() {
        let mut li = LoadingIndicator::new();
        assert_eq!(li.frame(), 0);
        li.tick();
        assert_eq!(li.frame(), 1);
        li.tick();
        assert_eq!(li.frame(), 2);
    }

    #[test]
    fn frame_wraps_at_end() {
        let mut li = LoadingIndicator::new().with_style(IndicatorStyle::Line);
        // Line has 4 frames
        for _ in 0..4 {
            li.tick();
        }
        assert_eq!(li.frame(), 0); // wrapped
    }

    #[test]
    fn reset_returns_to_zero() {
        let mut li = LoadingIndicator::new();
        li.tick();
        li.tick();
        assert_eq!(li.frame(), 2);
        li.reset();
        assert_eq!(li.frame(), 0);
    }

    #[test]
    fn message_displayed() {
        let li = LoadingIndicator::new()
            .with_style(IndicatorStyle::Spinner)
            .with_message("Loading...");
        let mut buf = ScreenBuffer::new(Size::new(20, 1));
        li.render(Rect::new(0, 0, 20, 1), &mut buf);

        let row: String = (0..20)
            .map(|x| buf.get(x, 0).map(|c| c.grapheme.as_str()).unwrap_or(" "))
            .collect();
        assert!(row.contains("Loading..."));
    }

    #[test]
    fn no_message_indicator_only() {
        let li = LoadingIndicator::new().with_style(IndicatorStyle::Circle);
        let mut buf = ScreenBuffer::new(Size::new(5, 1));
        li.render(Rect::new(0, 0, 5, 1), &mut buf);

        assert_eq!(buf.get(0, 0).unwrap().grapheme, "◐");
        // Rest should be spaces (default)
        assert_eq!(buf.get(1, 0).unwrap().grapheme, " ");
    }

    #[test]
    fn style_applied() {
        let style = Style::default().bold(true);
        let li = LoadingIndicator::new().with_indicator_style(style.clone());
        let mut buf = ScreenBuffer::new(Size::new(5, 1));
        li.render(Rect::new(0, 0, 5, 1), &mut buf);

        assert!(buf.get(0, 0).unwrap().style.bold);
    }

    #[test]
    fn zero_area_no_panic() {
        let li = LoadingIndicator::new();
        let mut buf = ScreenBuffer::new(Size::new(1, 1));
        li.render(Rect::new(0, 0, 0, 0), &mut buf);
        // No panic
    }
}
