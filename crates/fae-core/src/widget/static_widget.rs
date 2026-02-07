//! Static widget — displays pre-rendered content.

use crate::buffer::ScreenBuffer;
use crate::cell::Cell;
use crate::geometry::Rect;
use crate::segment::Segment;

use super::Widget;

/// A widget that displays pre-rendered segments without interaction.
///
/// Each segment becomes one "line" of content, rendered on successive rows.
#[derive(Clone, Debug)]
pub struct StaticWidget {
    lines: Vec<Segment>,
}

impl StaticWidget {
    /// Create a new static widget from a list of segments (one per line).
    pub fn new(lines: Vec<Segment>) -> Self {
        Self { lines }
    }

    /// Create a static widget from a single segment.
    pub fn from_segment(segment: Segment) -> Self {
        Self {
            lines: vec![segment],
        }
    }

    /// Get the lines.
    pub fn lines(&self) -> &[Segment] {
        &self.lines
    }
}

impl Widget for StaticWidget {
    fn render(&self, area: Rect, buf: &mut ScreenBuffer) {
        if area.size.width == 0 || area.size.height == 0 {
            return;
        }

        for (row, segment) in self.lines.iter().enumerate() {
            let y = area.position.y + row as u16;
            if y >= area.position.y + area.size.height {
                break;
            }

            let mut col: u16 = 0;
            for ch in segment.text.chars() {
                let x = area.position.x + col;
                if x >= area.position.x + area.size.width {
                    break;
                }
                let ch_width = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
                buf.set(x, y, Cell::new(ch.to_string(), segment.style.clone()));
                col += ch_width as u16;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::{Color, NamedColor};
    use crate::geometry::Size;
    use crate::style::Style;

    #[test]
    fn single_line() {
        let seg = Segment::new("hello");
        let w = StaticWidget::from_segment(seg);
        let mut buf = ScreenBuffer::new(Size::new(10, 3));
        w.render(Rect::new(0, 0, 10, 3), &mut buf);
        assert_eq!(buf.get(0, 0).map(|c| c.grapheme.as_str()), Some("h"));
        assert_eq!(buf.get(4, 0).map(|c| c.grapheme.as_str()), Some("o"));
    }

    #[test]
    fn multi_line() {
        let lines = vec![Segment::new("line1"), Segment::new("line2")];
        let w = StaticWidget::new(lines);
        let mut buf = ScreenBuffer::new(Size::new(10, 5));
        w.render(Rect::new(0, 0, 10, 5), &mut buf);
        assert_eq!(buf.get(0, 0).map(|c| c.grapheme.as_str()), Some("l"));
        assert_eq!(buf.get(0, 1).map(|c| c.grapheme.as_str()), Some("l"));
        assert_eq!(buf.get(4, 1).map(|c| c.grapheme.as_str()), Some("2"));
    }

    #[test]
    fn styled_segment() {
        let style = Style::new().fg(Color::Named(NamedColor::Green));
        let seg = Segment::styled("X", style.clone());
        let w = StaticWidget::from_segment(seg);
        let mut buf = ScreenBuffer::new(Size::new(5, 1));
        w.render(Rect::new(0, 0, 5, 1), &mut buf);
        assert_eq!(buf.get(0, 0).map(|c| &c.style), Some(&style));
    }

    #[test]
    fn truncates_to_area_width() {
        let seg = Segment::new("very long text here");
        let w = StaticWidget::from_segment(seg);
        let mut buf = ScreenBuffer::new(Size::new(5, 1));
        w.render(Rect::new(0, 0, 5, 1), &mut buf);
        // Only first 5 chars should be written
        assert_eq!(buf.get(4, 0).map(|c| c.grapheme.as_str()), Some(" "));
    }

    #[test]
    fn truncates_to_area_height() {
        let lines = vec![Segment::new("a"), Segment::new("b"), Segment::new("c")];
        let w = StaticWidget::new(lines);
        let mut buf = ScreenBuffer::new(Size::new(5, 2));
        w.render(Rect::new(0, 0, 5, 2), &mut buf);
        assert_eq!(buf.get(0, 0).map(|c| c.grapheme.as_str()), Some("a"));
        assert_eq!(buf.get(0, 1).map(|c| c.grapheme.as_str()), Some("b"));
        // Row 2 should not be written (area height is 2)
    }

    #[test]
    fn empty_area() {
        let w = StaticWidget::from_segment(Segment::new("x"));
        let mut buf = ScreenBuffer::new(Size::new(10, 10));
        w.render(Rect::new(0, 0, 0, 0), &mut buf);
        // No crash
    }
}
