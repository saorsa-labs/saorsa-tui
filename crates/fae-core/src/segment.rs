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

    /// Create a blank segment (spaces) of the given width.
    pub fn blank(width: u16) -> Self {
        Self {
            text: " ".repeat(width as usize),
            style: Style::default(),
            is_control: false,
        }
    }

    /// Display width in terminal cells.
    pub fn width(&self) -> usize {
        if self.is_control {
            return 0;
        }
        UnicodeWidthStr::width(self.text.as_str())
    }

    /// Display width in terminal cells (alias for width()).
    pub fn display_width(&self) -> usize {
        self.width()
    }

    /// Returns true if the segment has no text.
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    /// Returns each grapheme cluster in this segment together with its display width.
    ///
    /// Combining marks (zero-width) are grouped with their base character into
    /// a single grapheme cluster by the Unicode segmentation algorithm.
    pub fn grapheme_widths(&self) -> Vec<(String, usize)> {
        if self.is_control {
            return Vec::new();
        }
        self.text
            .graphemes(true)
            .map(|g| (g.to_string(), UnicodeWidthStr::width(g)))
            .collect()
    }

    /// Returns the number of grapheme clusters in this segment.
    ///
    /// This counts user-perceived characters, so a base character followed by
    /// combining diacritics counts as one.
    pub fn char_count(&self) -> usize {
        if self.is_control {
            return 0;
        }
        self.text.graphemes(true).count()
    }

    /// Truncate this segment to at most `max_width` display columns.
    ///
    /// If the segment is already within `max_width`, returns an identical segment.
    /// If a wide character straddles the boundary, it is excluded (the result may
    /// be slightly shorter than `max_width`).
    pub fn truncate_to_width(&self, max_width: usize) -> Segment {
        self.split_at(max_width).0
    }

    /// Pad this segment with trailing spaces to reach `target_width` display columns.
    ///
    /// If the segment is already at or wider than `target_width`, returns unchanged.
    pub fn pad_to_width(&self, target_width: usize) -> Segment {
        let current = self.width();
        if current >= target_width {
            return self.clone();
        }
        let padding = target_width - current;
        let mut text = self.text.clone();
        for _ in 0..padding {
            text.push(' ');
        }
        Segment::styled(text, self.style.clone())
    }

    /// Split this segment at the given display-width offset.
    ///
    /// Returns (left, right) where left has the specified display width.
    /// If the offset falls in the middle of a wide character, the left side
    /// is padded with a space and the right side gets a leading space.
    ///
    /// Combining marks (zero-width diacritics) are kept attached to their
    /// base character: if the split point falls between a base character and
    /// its combining marks, the combining marks travel with the base.
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

        // Collect graphemes with their widths
        let graphemes: Vec<(&str, usize)> = self
            .text
            .graphemes(true)
            .map(|g| (g, UnicodeWidthStr::width(g)))
            .collect();

        let mut left = String::new();
        let mut current_width = 0;
        let mut split_idx = 0; // index of first grapheme that goes to right side
        let mut need_left_pad = false;

        for (i, &(grapheme, gw)) in graphemes.iter().enumerate() {
            if current_width + gw > offset {
                // This grapheme would exceed the offset.
                if current_width < offset && gw > 1 {
                    // Wide char straddles the boundary — pad left with space
                    left.push(' ');
                    need_left_pad = true;
                }
                split_idx = i;
                break;
            }
            left.push_str(grapheme);
            current_width += gw;
            if current_width == offset {
                // Check if the next grapheme(s) are zero-width combining marks
                // that should stay with the current base character
                let mut j = i + 1;
                while j < graphemes.len() && graphemes[j].1 == 0 {
                    left.push_str(graphemes[j].0);
                    j += 1;
                }
                split_idx = j;
                break;
            }
        }

        // Build right side from remaining graphemes
        let mut right = String::new();
        if need_left_pad {
            // The wide char was split; put a space on the right as placeholder
            right.push(' ');
            // Skip the straddled grapheme
            for &(grapheme, _) in &graphemes[split_idx + 1..] {
                right.push_str(grapheme);
            }
        } else {
            for &(grapheme, _) in &graphemes[split_idx..] {
                right.push_str(grapheme);
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

    // --- Task 5: Unicode edge case tests ---

    #[test]
    fn emoji_width_is_two() {
        // Most emoji are 2 columns wide
        let s = Segment::new("\u{1f600}"); // grinning face
        assert_eq!(s.width(), 2);
    }

    #[test]
    fn emoji_at_split_boundary() {
        // "A" (1) + emoji (2) + "B" (1) = width 4
        let s = Segment::new("A\u{1f600}B");
        assert_eq!(s.width(), 4);

        // Split at offset 1 — before the emoji
        let (l, r) = s.split_at(1);
        assert_eq!(l.text, "A");
        assert_eq!(r.text, "\u{1f600}B");

        // Split at offset 2 — in the middle of the emoji
        // The emoji is width 2 and starts at offset 1, so offset 2 is mid-emoji
        let (l2, r2) = s.split_at(2);
        // left should get "A" + space (padding for straddled emoji)
        assert_eq!(l2.text, "A ");
        assert_eq!(l2.width(), 2);
        // right should get space (placeholder) + "B"
        assert_eq!(r2.text, " B");
    }

    #[test]
    fn combining_diacritics_width() {
        // 'e' followed by combining acute accent (U+0301) = single grapheme cluster "e\u{0301}"
        let s = Segment::new("e\u{0301}"); // é as decomposed
        // Should be width 1 (single character with combining mark)
        assert_eq!(s.width(), 1);
        assert_eq!(s.char_count(), 1);
    }

    #[test]
    fn mixed_ascii_emoji_cjk() {
        // "Hi" (2) + emoji (2) + CJK 世 (2) = width 6
        let s = Segment::new("Hi\u{1f600}\u{4e16}");
        assert_eq!(s.width(), 6);
        assert_eq!(s.char_count(), 4); // H, i, emoji, CJK
    }

    #[test]
    fn grapheme_widths_returns_correct_values() {
        let s = Segment::new("A\u{4e16}B");
        let widths = s.grapheme_widths();
        assert_eq!(widths.len(), 3);
        assert_eq!(widths[0], ("A".to_string(), 1));
        assert_eq!(widths[1], ("\u{4e16}".to_string(), 2));
        assert_eq!(widths[2], ("B".to_string(), 1));
    }

    #[test]
    fn char_count_returns_grapheme_cluster_count() {
        // "Hello" = 5 grapheme clusters
        assert_eq!(Segment::new("Hello").char_count(), 5);
        // Empty = 0
        assert_eq!(Segment::new("").char_count(), 0);
        // CJK characters
        assert_eq!(Segment::new("\u{4e16}\u{754c}").char_count(), 2);
        // Control segments return 0
        assert_eq!(Segment::control("ESC").char_count(), 0);
    }

    #[test]
    fn split_preserves_combining_marks() {
        // "ae\u{0301}b" = "a" + "e\u{0301}" + "b" (3 graphemes, width 3)
        let s = Segment::new("ae\u{0301}b");
        assert_eq!(s.width(), 3);
        assert_eq!(s.char_count(), 3);

        // Split at offset 1 — between "a" and "e\u{0301}"
        let (l, r) = s.split_at(1);
        assert_eq!(l.text, "a");
        // The combining mark should stay attached to "e"
        assert_eq!(r.text, "e\u{0301}b");

        // Split at offset 2 — between "e\u{0301}" and "b"
        let (l2, r2) = s.split_at(2);
        assert_eq!(l2.text, "ae\u{0301}");
        assert_eq!(r2.text, "b");
    }

    #[test]
    fn empty_segment_grapheme_operations() {
        let s = Segment::new("");
        assert_eq!(s.grapheme_widths().len(), 0);
        assert_eq!(s.char_count(), 0);
        let (l, r) = s.split_at(0);
        assert_eq!(l.text, "");
        assert_eq!(r.text, "");
    }

    #[test]
    fn grapheme_widths_empty_for_control() {
        let s = Segment::control("\x1b[1m");
        assert!(s.grapheme_widths().is_empty());
    }

    // --- Task 5: truncate_to_width and pad_to_width tests ---

    #[test]
    fn truncate_to_width_ascii_exact_fit() {
        let s = Segment::new("hello");
        let truncated = s.truncate_to_width(5);
        assert_eq!(truncated.text, "hello");
        assert_eq!(truncated.width(), 5);
    }

    #[test]
    fn truncate_to_width_cuts_before_wide_char_at_boundary() {
        // "A" (1) + "世" (2) + "B" (1) = width 4
        let s = Segment::new("A\u{4e16}B");
        assert_eq!(s.width(), 4);
        // Truncate to width 2 — the wide char starts at offset 1 and spans 1..3,
        // so at max_width=2 it straddles the boundary. split_at pads left with space.
        let truncated = s.truncate_to_width(2);
        assert_eq!(truncated.width(), 2);
        assert_eq!(truncated.text, "A ");
    }

    #[test]
    fn truncate_to_width_zero_gives_empty() {
        let s = Segment::new("hello");
        let truncated = s.truncate_to_width(0);
        assert_eq!(truncated.text, "");
        assert_eq!(truncated.width(), 0);
    }

    #[test]
    fn truncate_to_width_beyond_length_unchanged() {
        let s = Segment::new("hi");
        let truncated = s.truncate_to_width(100);
        assert_eq!(truncated.text, "hi");
        assert_eq!(truncated.width(), 2);
    }

    #[test]
    fn pad_to_width_adds_trailing_spaces() {
        let s = Segment::new("AB");
        let padded = s.pad_to_width(5);
        assert_eq!(padded.text, "AB   ");
        assert_eq!(padded.width(), 5);
    }

    #[test]
    fn pad_to_width_already_at_target_unchanged() {
        let s = Segment::new("hello");
        let padded = s.pad_to_width(5);
        assert_eq!(padded.text, "hello");
    }

    #[test]
    fn pad_to_width_already_wider_unchanged() {
        let s = Segment::new("hello world");
        let padded = s.pad_to_width(5);
        assert_eq!(padded.text, "hello world");
    }

    #[test]
    fn style_preserved_through_truncation_and_padding() {
        let style = Style::new().bold(true);
        let s = Segment::styled("hello world", style.clone());

        let truncated = s.truncate_to_width(5);
        assert!(truncated.style.bold);
        assert_eq!(truncated.style, style);

        let padded = s.pad_to_width(20);
        assert!(padded.style.bold);
        assert_eq!(padded.style, style);
    }

    // --- Multi-codepoint emoji tests ---

    #[test]
    fn zwj_family_emoji_width() {
        // ZWJ family emoji: man + ZWJ + woman + ZWJ + girl
        let s = Segment::new("\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}");
        // Should be width 2 (rendered as a single 2-column-wide grapheme)
        assert_eq!(s.width(), 2);
    }

    #[test]
    fn zwj_family_emoji_grapheme_widths() {
        let s = Segment::new("\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}");
        let widths = s.grapheme_widths();
        // Single grapheme cluster
        assert_eq!(widths.len(), 1);
        // Width should be 2
        assert_eq!(widths[0].1, 2);
    }

    #[test]
    fn flag_emoji_width() {
        // US flag: regional indicator U + regional indicator S
        let s = Segment::new("\u{1F1FA}\u{1F1F8}");
        assert_eq!(s.width(), 2);
    }

    #[test]
    fn skin_tone_emoji_width() {
        // Thumbs up + medium skin tone modifier
        let s = Segment::new("\u{1F44D}\u{1F3FD}");
        assert_eq!(s.width(), 2);
    }

    #[test]
    fn split_segment_at_zwj_emoji_boundary() {
        // "A" (width 1) + ZWJ family emoji (width 2) + "B" (width 1) = width 4
        let s = Segment::new("A\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}B");
        assert_eq!(s.width(), 4);

        // Split at offset 1 — just after "A", before emoji
        let (l, r) = s.split_at(1);
        assert_eq!(l.text, "A");
        assert_eq!(l.width(), 1);
        // Right should start with the family emoji
        assert_eq!(r.width(), 3); // emoji(2) + B(1)
    }

    #[test]
    fn char_count_with_complex_emoji() {
        // ZWJ family is one grapheme cluster
        let s = Segment::new("\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}");
        assert_eq!(s.char_count(), 1);
    }

    #[test]
    fn mixed_ascii_zwj_emoji_cjk() {
        // "Hi" (2) + family emoji (2) + CJK 世 (2) + "!" (1) = 7
        let s = Segment::new("Hi\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}\u{4e16}!");
        assert_eq!(s.width(), 7);
        assert_eq!(s.char_count(), 5); // H, i, family, 世, !
    }

    #[test]
    fn keycap_sequence_handling() {
        // Keycap "#": # + VS16 + combining enclosing keycap
        let s = Segment::new("#\u{FE0F}\u{20E3}");
        // This is a single grapheme cluster
        assert_eq!(s.char_count(), 1);
        // Width depends on unicode-width crate version, but should be reasonable
        let w = s.width();
        assert!((1..=2).contains(&w));
    }
}
