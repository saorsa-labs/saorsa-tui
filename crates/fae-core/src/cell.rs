//! Cell type — a single terminal cell.

use crate::style::Style;
use unicode_width::UnicodeWidthStr;

/// A single cell in the terminal screen buffer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cell {
    /// The grapheme cluster displayed in this cell.
    pub grapheme: String,
    /// The style of this cell.
    pub style: Style,
    /// Display width (1 for most chars, 2 for CJK/emoji, 0 for continuation).
    pub width: u8,
}

impl Cell {
    /// Create a new cell, auto-detecting width from the grapheme.
    pub fn new(grapheme: impl Into<String>, style: Style) -> Self {
        let grapheme = grapheme.into();
        let width = UnicodeWidthStr::width(grapheme.as_str()) as u8;
        Self {
            grapheme,
            style,
            width,
        }
    }

    /// Create a blank cell (space, default style, width 1).
    pub fn blank() -> Self {
        Self {
            grapheme: " ".into(),
            style: Style::default(),
            width: 1,
        }
    }

    /// Returns true if this is a blank cell (space with default style).
    pub fn is_blank(&self) -> bool {
        self.grapheme == " " && self.style.is_empty() && self.width == 1
    }

    /// Returns true if this is a wide character (width > 1).
    pub fn is_wide(&self) -> bool {
        self.width > 1
    }

    /// Create a continuation cell (placeholder for the second cell of a wide character).
    pub fn continuation() -> Self {
        Self {
            grapheme: String::new(),
            style: Style::default(),
            width: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::{Color, NamedColor};

    #[test]
    fn blank_cell() {
        let c = Cell::blank();
        assert!(c.is_blank());
        assert_eq!(c.width, 1);
    }

    #[test]
    fn ascii_cell() {
        let c = Cell::new("A", Style::default());
        assert_eq!(c.width, 1);
        assert!(!c.is_wide());
    }

    #[test]
    fn cjk_cell() {
        let c = Cell::new("\u{4e16}", Style::default()); // 世
        assert_eq!(c.width, 2);
        assert!(c.is_wide());
    }

    #[test]
    fn continuation_cell() {
        let c = Cell::continuation();
        assert_eq!(c.width, 0);
        assert!(c.grapheme.is_empty());
    }

    #[test]
    fn styled_not_blank() {
        let c = Cell::new(" ", Style::new().fg(Color::Named(NamedColor::Red)));
        assert!(!c.is_blank());
    }

    #[test]
    fn space_default_is_blank() {
        let c = Cell::new(" ", Style::default());
        assert!(c.is_blank());
    }
}
