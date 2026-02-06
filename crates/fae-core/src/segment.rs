//! Segment type — the fundamental rendering unit.

use crate::style::Style;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

/// A piece of styled text, the fundamental rendering unit.
///
/// Every widget's render method produces lines of segments.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Segment {
    /// The text content.
    pub text: String,
    /// The style applied to this segment.
    pub style: Style,
    /// Whether this is a control sequence (not visible text).
    pub is_control: bool,
}

impl Segment {
    /// Create a new segment with default style.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            style: Style::default(),
            is_control: false,
        }
    }

    /// Create a new segment with the given style.
    pub fn styled(text: impl Into<String>, style: Style) -> Self {
        Self {
            text: text.into(),
            style,
            is_control: false,
        }
    }

    /// Create a control segment (not rendered as visible text).
    pub fn control(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            style: Style::default(),
            is_control: true,
        }
    }

    /// Display width in terminal cells.
    pub fn width(&self) -> usize {
        if self.is_control {
            return 0;
        }
        UnicodeWidthStr::width(self.text.as_str())
    }

    /// Returns true if the segment has no text.
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    /// Split this segment at the given display-width offset.
    ///
    /// Returns (left, right) where left has the specified display width.
    /// If the offset falls in the middle of a wide character, the left side
    /// is padded with a space.
    pub fn split_at(&self, offset: usize) -> (Segment, Segment) {
        if offset == 0 {
            return (
                Segment::styled(String::new(), self.style.clone()),
                self.clone(),
            );
        }
        if offset >= self.width() {
            return (
                self.clone(),
                Segment::styled(String::new(), self.style.clone()),
            );
        }

        let mut left = String::new();
        let mut current_width = 0;

        for grapheme in self.text.graphemes(true) {
            let gw = UnicodeWidthStr::width(grapheme);
            if current_width + gw > offset {
                // This grapheme would exceed the offset.
                // If we're exactly at offset, stop here.
                // If the wide char straddles the boundary, pad left with space.
                if current_width < offset {
                    left.push(' ');
                }
                break;
            }
            left.push_str(grapheme);
            current_width += gw;
            if current_width == offset {
                break;
            }
        }

        // Build right side from remaining graphemes
        let mut right = String::new();
        let mut seen_width = 0;
        let mut past_split = false;
        for grapheme in self.text.graphemes(true) {
            let gw = UnicodeWidthStr::width(grapheme);
            if past_split {
                right.push_str(grapheme);
            } else {
                seen_width += gw;
                if seen_width > offset {
                    // This grapheme straddles the boundary — skip it
                    // (it was replaced by space on the left side, and its
                    // right half becomes a space on the right side)
                    if seen_width - gw < offset {
                        right.push(' ');
                    } else {
                        right.push_str(grapheme);
                    }
                    past_split = true;
                } else if seen_width == offset {
                    past_split = true;
                }
            }
        }

        (
            Segment::styled(left, self.style.clone()),
            Segment::styled(right, self.style.clone()),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_width() {
        assert_eq!(Segment::new("hello").width(), 5);
    }

    #[test]
    fn empty_width() {
        assert_eq!(Segment::new("").width(), 0);
    }

    #[test]
    fn control_width_is_zero() {
        assert_eq!(Segment::control("ESC[1m").width(), 0);
    }

    #[test]
    fn cjk_width() {
        // CJK characters are 2 cells wide
        assert_eq!(Segment::new("\u{4e16}\u{754c}").width(), 4); // 世界
    }

    #[test]
    fn split_ascii() {
        let s = Segment::new("hello");
        let (l, r) = s.split_at(3);
        assert_eq!(l.text, "hel");
        assert_eq!(r.text, "lo");
    }

    #[test]
    fn split_at_zero() {
        let s = Segment::new("hello");
        let (l, r) = s.split_at(0);
        assert_eq!(l.text, "");
        assert_eq!(r.text, "hello");
    }

    #[test]
    fn split_at_end() {
        let s = Segment::new("hello");
        let (l, r) = s.split_at(5);
        assert_eq!(l.text, "hello");
        assert_eq!(r.text, "");
    }

    #[test]
    fn split_beyond_end() {
        let s = Segment::new("hi");
        let (l, r) = s.split_at(100);
        assert_eq!(l.text, "hi");
        assert_eq!(r.text, "");
    }

    #[test]
    fn is_empty() {
        assert!(Segment::new("").is_empty());
        assert!(!Segment::new("x").is_empty());
    }

    #[test]
    fn styled_preserves_style_on_split() {
        let s = Segment::styled("hello", Style::new().bold(true));
        let (l, r) = s.split_at(2);
        assert!(l.style.bold);
        assert!(r.style.bold);
    }
}
