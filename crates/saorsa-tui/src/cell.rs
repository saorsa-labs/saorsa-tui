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

    /// Returns true if this is a continuation cell (width == 0).
    ///
    /// Continuation cells occupy the second column of a wide character.
    pub fn is_continuation(&self) -> bool {
        self.width == 0
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

    // --- Task 6: Unicode cell tests ---

    #[test]
    fn cell_from_emoji_width_two() {
        let c = Cell::new("\u{1f389}", Style::default()); // 🎉
        assert_eq!(c.width, 2);
        assert!(c.is_wide());
    }

    #[test]
    fn cell_from_combining_mark_width_zero() {
        // U+0301 combining acute accent alone
        let c = Cell::new("\u{0301}", Style::default());
        assert_eq!(c.width, 0);
    }

    #[test]
    fn cell_from_cjk_width_two() {
        let c = Cell::new("\u{6f22}", Style::default()); // 漢
        assert_eq!(c.width, 2);
        assert!(c.is_wide());
    }

    #[test]
    fn cell_from_ascii_width_one() {
        let c = Cell::new("A", Style::default());
        assert_eq!(c.width, 1);
        assert!(!c.is_wide());
    }

    #[test]
    fn cell_equality_same_grapheme_and_style() {
        let style = Style::new().fg(Color::Named(NamedColor::Green));
        let c1 = Cell::new("X", style.clone());
        let c2 = Cell::new("X", style);
        assert_eq!(c1, c2);
    }

    #[test]
    fn cell_inequality_different_width() {
        // ASCII "A" (width 1) vs CJK "世" (width 2)
        let c1 = Cell::new("A", Style::default());
        let c2 = Cell::new("\u{4e16}", Style::default());
        assert_ne!(c1, c2);
        assert_ne!(c1.width, c2.width);
    }

    // --- Multi-codepoint emoji cell tests ---

    #[test]
    fn cell_from_zwj_emoji_width_two() {
        // ZWJ family emoji: man + ZWJ + woman + ZWJ + girl
        let c = Cell::new(
            "\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}",
            Style::default(),
        );
        assert_eq!(c.width, 2);
        assert!(c.is_wide());
    }

    #[test]
    fn cell_from_flag_emoji_width_two() {
        // US flag: regional indicator U + regional indicator S
        let c = Cell::new("\u{1F1FA}\u{1F1F8}", Style::default());
        assert_eq!(c.width, 2);
        assert!(c.is_wide());
    }

    #[test]
    fn cell_from_skin_tone_emoji_width_two() {
        // Thumbs up + medium skin tone
        let c = Cell::new("\u{1F44D}\u{1F3FD}", Style::default());
        assert_eq!(c.width, 2);
        assert!(c.is_wide());
    }

    #[test]
    fn cell_continuation_after_emoji() {
        // Continuation cell should be width 0 regardless of what preceded it
        let cont = Cell::continuation();
        assert!(cont.is_continuation());
        assert_eq!(cont.width, 0);
        assert!(cont.grapheme.is_empty());
    }
}
