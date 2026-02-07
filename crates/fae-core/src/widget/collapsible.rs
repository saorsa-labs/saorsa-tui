//! Collapsible section widget with expandable/collapsible content.
//!
//! Displays a title line with an expand/collapse indicator and optional
//! content that is shown or hidden based on the expanded state.

use crate::buffer::ScreenBuffer;
use crate::cell::Cell;
use crate::event::{Event, KeyCode, KeyEvent};
use crate::geometry::Rect;
use crate::segment::Segment;
use crate::style::Style;
use crate::text::truncate_to_display_width;
use unicode_width::UnicodeWidthStr;

use super::{BorderStyle, EventResult, InteractiveWidget, Widget};

/// A collapsible section widget.
///
/// Shows a title with an expand/collapse indicator. When expanded,
/// content lines are rendered below the title.
pub struct Collapsible {
    /// Section title.
    title: String,
    /// Content lines (only visible when expanded).
    content: Vec<Vec<Segment>>,
    /// Whether the section is expanded.
    expanded: bool,
    /// Style for the title line.
    title_style: Style,
    /// Style for content lines.
    content_style: Style,
    /// Border style.
    border: BorderStyle,
    /// Indicator characters: (collapsed, expanded).
    indicators: (&'static str, &'static str),
}

impl Collapsible {
    /// Create a new collapsible section (collapsed by default).
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            content: Vec::new(),
            expanded: false,
            title_style: Style::default(),
            content_style: Style::default(),
            border: BorderStyle::None,
            indicators: ("\u{25b6}", "\u{25bc}"), // ▶ collapsed, ▼ expanded
        }
    }

    /// Set the content lines.
    #[must_use]
    pub fn with_content(mut self, content: Vec<Vec<Segment>>) -> Self {
        self.content = content;
        self
    }

    /// Set the initial expanded state.
    #[must_use]
    pub fn with_expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }

    /// Set the title style.
    #[must_use]
    pub fn with_title_style(mut self, style: Style) -> Self {
        self.title_style = style;
        self
    }

    /// Set the content style.
    #[must_use]
    pub fn with_content_style(mut self, style: Style) -> Self {
        self.content_style = style;
        self
    }

    /// Set the border style.
    #[must_use]
    pub fn with_border(mut self, border: BorderStyle) -> Self {
        self.border = border;
        self
    }

    /// Set custom indicator characters.
    #[must_use]
    pub fn with_indicators(mut self, collapsed: &'static str, expanded: &'static str) -> Self {
        self.indicators = (collapsed, expanded);
        self
    }

    /// Toggle the expanded/collapsed state.
    pub fn toggle(&mut self) {
        self.expanded = !self.expanded;
    }

    /// Set the expanded state.
    pub fn set_expanded(&mut self, expanded: bool) {
        self.expanded = expanded;
    }

    /// Check if the section is expanded.
    pub fn is_expanded(&self) -> bool {
        self.expanded
    }

    /// Render a row of segments into the buffer at the given position.
    fn render_segments(segments: &[Segment], x0: u16, y: u16, w: usize, buf: &mut ScreenBuffer) {
        let mut col: u16 = 0;
        for segment in segments {
            if col as usize >= w {
                break;
            }
            let remaining = w.saturating_sub(col as usize);
            let truncated = truncate_to_display_width(&segment.text, remaining);
            for ch in truncated.chars() {
                if col as usize >= w {
                    break;
                }
                let char_w = UnicodeWidthStr::width(ch.encode_utf8(&mut [0; 4]) as &str);
                if col as usize + char_w > w {
                    break;
                }
                buf.set(
                    x0 + col,
                    y,
                    Cell::new(ch.to_string(), segment.style.clone()),
                );
                col += char_w as u16;
            }
        }
    }
}

impl Widget for Collapsible {
    fn render(&self, area: Rect, buf: &mut ScreenBuffer) {
        if area.size.width == 0 || area.size.height == 0 {
            return;
        }

        super::border::render_border(area, self.border, self.title_style.clone(), buf);
        let inner = super::border::inner_area(area, self.border);
        if inner.size.width == 0 || inner.size.height == 0 {
            return;
        }

        let w = inner.size.width as usize;
        let x0 = inner.position.x;
        let mut y = inner.position.y;

        // Render title line: indicator + space + title
        let indicator = if self.expanded {
            self.indicators.1
        } else {
            self.indicators.0
        };
        let title_line = format!("{indicator} {}", self.title);
        let title_segments = vec![Segment::styled(&title_line, self.title_style.clone())];
        Self::render_segments(&title_segments, x0, y, w, buf);
        y += 1;

        // Render content if expanded
        if self.expanded {
            for line in &self.content {
                if y >= inner.position.y + inner.size.height {
                    break;
                }
                Self::render_segments(line, x0, y, w, buf);
                y += 1;
            }
        }
    }
}

impl InteractiveWidget for Collapsible {
    fn handle_event(&mut self, event: &Event) -> EventResult {
        let Event::Key(KeyEvent { code, .. }) = event else {
            return EventResult::Ignored;
        };

        match code {
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.toggle();
                EventResult::Consumed
            }
            KeyCode::Left => {
                self.set_expanded(false);
                EventResult::Consumed
            }
            KeyCode::Right => {
                self.set_expanded(true);
                EventResult::Consumed
            }
            _ => EventResult::Ignored,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::geometry::Size;

    fn content_lines(texts: &[&str]) -> Vec<Vec<Segment>> {
        texts.iter().map(|t| vec![Segment::new(*t)]).collect()
    }

    #[test]
    fn create_collapsed() {
        let c = Collapsible::new("Section");
        assert!(!c.is_expanded());
    }

    #[test]
    fn create_expanded() {
        let c = Collapsible::new("Section").with_expanded(true);
        assert!(c.is_expanded());
    }

    #[test]
    fn render_collapsed_title_only() {
        let c = Collapsible::new("Hello").with_content(content_lines(&["line1", "line2"]));
        let mut buf = ScreenBuffer::new(Size::new(30, 5));
        c.render(Rect::new(0, 0, 30, 5), &mut buf);

        // Title row should contain indicator and title
        let row0: String = (0..30)
            .map(|x| buf.get(x, 0).map(|c| c.grapheme.as_str()).unwrap_or(" "))
            .collect();
        assert!(row0.contains("Hello"));
        assert!(row0.contains("\u{25b6}")); // ▶

        // Content rows should be empty (default spaces)
        let row1: String = (0..30)
            .map(|x| buf.get(x, 1).map(|c| c.grapheme.as_str()).unwrap_or(" "))
            .collect();
        assert!(!row1.contains("line1"));
    }

    #[test]
    fn render_expanded_title_and_content() {
        let c = Collapsible::new("Hello")
            .with_content(content_lines(&["line1", "line2"]))
            .with_expanded(true);
        let mut buf = ScreenBuffer::new(Size::new(30, 5));
        c.render(Rect::new(0, 0, 30, 5), &mut buf);

        let row0: String = (0..30)
            .map(|x| buf.get(x, 0).map(|c| c.grapheme.as_str()).unwrap_or(" "))
            .collect();
        assert!(row0.contains("\u{25bc}")); // ▼

        let row1: String = (0..30)
            .map(|x| buf.get(x, 1).map(|c| c.grapheme.as_str()).unwrap_or(" "))
            .collect();
        assert!(row1.contains("line1"));
    }

    #[test]
    fn toggle_changes_state() {
        let mut c = Collapsible::new("T");
        assert!(!c.is_expanded());
        c.toggle();
        assert!(c.is_expanded());
        c.toggle();
        assert!(!c.is_expanded());
    }

    #[test]
    fn set_expanded_explicitly() {
        let mut c = Collapsible::new("T");
        c.set_expanded(true);
        assert!(c.is_expanded());
        c.set_expanded(false);
        assert!(!c.is_expanded());
    }

    #[test]
    fn custom_indicators() {
        let c = Collapsible::new("Test")
            .with_indicators("+", "-")
            .with_expanded(false);
        let mut buf = ScreenBuffer::new(Size::new(20, 3));
        c.render(Rect::new(0, 0, 20, 3), &mut buf);

        let row0: String = (0..20)
            .map(|x| buf.get(x, 0).map(|c| c.grapheme.as_str()).unwrap_or(" "))
            .collect();
        assert!(row0.contains("+"));
    }

    #[test]
    fn enter_toggles() {
        let mut c = Collapsible::new("T");
        let result = c.handle_event(&Event::Key(KeyEvent::plain(KeyCode::Enter)));
        assert_eq!(result, EventResult::Consumed);
        assert!(c.is_expanded());
    }

    #[test]
    fn left_collapses_right_expands() {
        let mut c = Collapsible::new("T").with_expanded(true);

        c.handle_event(&Event::Key(KeyEvent::plain(KeyCode::Left)));
        assert!(!c.is_expanded());

        c.handle_event(&Event::Key(KeyEvent::plain(KeyCode::Right)));
        assert!(c.is_expanded());
    }

    #[test]
    fn empty_content_when_expanded() {
        let c = Collapsible::new("Empty").with_expanded(true);
        let mut buf = ScreenBuffer::new(Size::new(20, 5));
        c.render(Rect::new(0, 0, 20, 5), &mut buf);
        // Should render title only, no panic
        let row0: String = (0..20)
            .map(|x| buf.get(x, 0).map(|c| c.grapheme.as_str()).unwrap_or(" "))
            .collect();
        assert!(row0.contains("Empty"));
    }

    #[test]
    fn border_rendering() {
        let c = Collapsible::new("B").with_border(BorderStyle::Single);
        let mut buf = ScreenBuffer::new(Size::new(20, 5));
        c.render(Rect::new(0, 0, 20, 5), &mut buf);

        assert_eq!(buf.get(0, 0).unwrap().grapheme, "┌");
    }
}
