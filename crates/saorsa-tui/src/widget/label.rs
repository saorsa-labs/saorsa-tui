//! Label widget — a single line of styled text.

use crate::buffer::ScreenBuffer;
use crate::cell::Cell;
use crate::geometry::Rect;
use crate::style::Style;
use unicode_width::UnicodeWidthStr;

use super::Widget;

/// Text alignment.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Alignment {
    /// Align to the left (default).
    #[default]
    Left,
    /// Center the text.
    Center,
    /// Align to the right.
    Right,
}

/// A single-line text label widget.
#[derive(Clone, Debug)]
pub struct Label {
    /// The text to display.
    text: String,
    /// The style for the text.
    style: Style,
    /// Text alignment within the available area.
    alignment: Alignment,
}

impl Label {
    /// Create a new label with the given text.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            style: Style::default(),
            alignment: Alignment::Left,
        }
    }

    /// Set the style.
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Set the alignment.
    #[must_use]
    pub fn alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    /// Get the text.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Set new text content.
    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
    }

    /// Get the current style.
    pub fn style_ref(&self) -> &Style {
        &self.style
    }

    /// Set the style.
    pub fn set_style(&mut self, style: Style) {
        self.style = style;
    }

    /// Get the current alignment.
    pub fn alignment_value(&self) -> Alignment {
        self.alignment
    }

    /// Set the alignment.
    pub fn set_alignment(&mut self, alignment: Alignment) {
        self.alignment = alignment;
    }
}

impl Widget for Label {
    fn render(&self, area: Rect, buf: &mut ScreenBuffer) {
        if area.size.width == 0 || area.size.height == 0 {
            return;
        }

        let width = usize::from(area.size.width);
        let text_width = UnicodeWidthStr::width(self.text.as_str());

        // If a background is set, fill the entire row. This makes
        // `background` styling behave like a block element in the terminal.
        if self.style.bg.is_some() {
            for x in 0..area.size.width {
                buf.set(
                    area.position.x + x,
                    area.position.y,
                    Cell::new(" ", self.style.clone()),
                );
            }
        }

        // Truncate with ellipsis if needed
        let display_text = if text_width > width {
            truncate_with_ellipsis(&self.text, width)
        } else {
            self.text.clone()
        };

        let display_width = UnicodeWidthStr::width(display_text.as_str());

        // Calculate horizontal offset based on alignment
        let offset = match self.alignment {
            Alignment::Left => 0,
            Alignment::Center => (width.saturating_sub(display_width)) / 2,
            Alignment::Right => width.saturating_sub(display_width),
        };

        // Write characters to buffer
        let mut col = 0usize;
        for ch in display_text.chars() {
            let ch_width = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
            let x = area.position.x + (offset + col) as u16;
            if x >= area.position.x + area.size.width {
                break;
            }
            buf.set(
                x,
                area.position.y,
                Cell::new(ch.to_string(), self.style.clone()),
            );
            col += ch_width;
        }
    }
}

/// Truncate a string to fit within `max_width` columns, adding an ellipsis.
fn truncate_with_ellipsis(text: &str, max_width: usize) -> String {
    if max_width == 0 {
        return String::new();
    }
    if max_width == 1 {
        return "\u{2026}".to_string(); // …
    }

    let target = max_width - 1; // leave room for ellipsis
    let mut result = String::new();
    let mut current_width = 0usize;

    for ch in text.chars() {
        let ch_width = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
        if current_width + ch_width > target {
            break;
        }
        result.push(ch);
        current_width += ch_width;
    }
    result.push('\u{2026}'); // …
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::{Color, NamedColor};
    use crate::geometry::Size;

    #[test]
    fn label_renders_left_aligned() {
        let label = Label::new("Hello");
        let mut buf = ScreenBuffer::new(Size::new(10, 1));
        label.render(Rect::new(0, 0, 10, 1), &mut buf);
        assert_eq!(buf.get(0, 0).map(|c| c.grapheme.as_str()), Some("H"));
        assert_eq!(buf.get(4, 0).map(|c| c.grapheme.as_str()), Some("o"));
    }

    #[test]
    fn label_renders_center_aligned() {
        let label = Label::new("Hi").alignment(Alignment::Center);
        let mut buf = ScreenBuffer::new(Size::new(10, 1));
        label.render(Rect::new(0, 0, 10, 1), &mut buf);
        // "Hi" is 2 wide, centered in 10 → offset 4
        assert_eq!(buf.get(4, 0).map(|c| c.grapheme.as_str()), Some("H"));
        assert_eq!(buf.get(5, 0).map(|c| c.grapheme.as_str()), Some("i"));
    }

    #[test]
    fn label_renders_right_aligned() {
        let label = Label::new("Hi").alignment(Alignment::Right);
        let mut buf = ScreenBuffer::new(Size::new(10, 1));
        label.render(Rect::new(0, 0, 10, 1), &mut buf);
        // "Hi" is 2 wide, right-aligned in 10 → offset 8
        assert_eq!(buf.get(8, 0).map(|c| c.grapheme.as_str()), Some("H"));
        assert_eq!(buf.get(9, 0).map(|c| c.grapheme.as_str()), Some("i"));
    }

    #[test]
    fn label_truncates_with_ellipsis() {
        let label = Label::new("Hello, World!");
        let mut buf = ScreenBuffer::new(Size::new(8, 1));
        label.render(Rect::new(0, 0, 8, 1), &mut buf);
        // Should truncate to "Hello, \u{2026}" (7 chars + ellipsis)
        assert_eq!(buf.get(0, 0).map(|c| c.grapheme.as_str()), Some("H"));
        assert_eq!(buf.get(7, 0).map(|c| c.grapheme.as_str()), Some("\u{2026}"));
    }

    #[test]
    fn label_with_style() {
        let style = Style::new().fg(Color::Named(NamedColor::Red));
        let label = Label::new("X").style(style.clone());
        let mut buf = ScreenBuffer::new(Size::new(5, 1));
        label.render(Rect::new(0, 0, 5, 1), &mut buf);
        assert_eq!(buf.get(0, 0).map(|c| &c.style), Some(&style));
    }

    #[test]
    fn label_empty_area() {
        let label = Label::new("test");
        let mut buf = ScreenBuffer::new(Size::new(10, 1));
        // Should not crash on zero-width area
        label.render(Rect::new(0, 0, 0, 1), &mut buf);
    }

    #[test]
    fn label_set_text() {
        let mut label = Label::new("before");
        assert_eq!(label.text(), "before");
        label.set_text("after");
        assert_eq!(label.text(), "after");
    }
}
