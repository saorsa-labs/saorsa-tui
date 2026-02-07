//! Soft-wrap logic for splitting logical lines into visual lines.
//!
//! Provides functions for wrapping text to a given display width,
//! accounting for double-width characters (CJK, emoji). Used by
//! [`crate::widget::TextArea`] to render wrapped text with line numbers.

use crate::text_buffer::TextBuffer;
use unicode_width::UnicodeWidthChar;

/// A single visual line resulting from soft-wrapping a logical line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WrapLine {
    /// The text content of this visual line.
    pub text: String,
    /// The index of the logical line this visual line belongs to.
    pub logical_line: usize,
    /// The character offset within the logical line where this visual
    /// line starts.
    pub start_col: usize,
}

/// The result of wrapping an entire buffer.
#[derive(Clone, Debug)]
pub struct WrapResult {
    /// All visual lines in display order.
    pub lines: Vec<WrapLine>,
    /// The width in characters needed for line number display.
    pub line_number_width: u16,
}

/// Wrap a single line of text to the given display width.
///
/// Returns a list of `(text, start_col)` pairs. The text for each
/// visual line and the character offset within the original line.
///
/// The algorithm:
/// 1. Break at word boundaries (whitespace) when possible.
/// 2. Fall back to character boundary for words longer than `width`.
/// 3. Respects display width (CJK = 2, emoji = 2).
/// 4. Never splits multi-byte characters.
pub fn wrap_line(text: &str, width: usize) -> Vec<(String, usize)> {
    if width == 0 {
        return vec![(text.to_string(), 0)];
    }

    if text.is_empty() {
        return vec![(String::new(), 0)];
    }

    let mut result = Vec::new();
    let mut current_line = String::new();
    let mut current_width: usize = 0;
    let mut line_start_col: usize = 0;

    for (char_col, ch) in text.chars().enumerate() {
        let ch_width = ch.width().unwrap_or(0);

        if current_width + ch_width > width && !current_line.is_empty() {
            // Need to wrap — try to find a word boundary to break at
            if let Some(space_byte_idx) = find_last_space(&current_line) {
                // Break at the last space
                let before: String = current_line[..space_byte_idx].to_string();
                let after: String = current_line[space_byte_idx..].trim_start().to_string();
                let before_char_count = before.chars().count();
                result.push((before, line_start_col));
                current_width = display_width_of(&after);
                line_start_col +=
                    before_char_count + count_trimmed_spaces(&current_line[space_byte_idx..]);
                current_line = after;
            } else {
                // No space found — break at character boundary
                result.push((current_line.clone(), line_start_col));
                line_start_col = char_col;
                current_line = String::new();
                current_width = 0;
            }
        }

        current_line.push(ch);
        current_width += ch_width;
    }

    if !current_line.is_empty() || result.is_empty() {
        result.push((current_line, line_start_col));
    }

    result
}

/// Wrap all lines of a text buffer to the given display width.
pub fn wrap_lines(buffer: &TextBuffer, width: usize) -> WrapResult {
    let total_lines = buffer.line_count();
    let mut lines = Vec::new();

    for line_idx in 0..total_lines {
        if let Some(line_text) = buffer.line(line_idx) {
            let wrapped = wrap_line(&line_text, width);
            for (text, start_col) in wrapped {
                lines.push(WrapLine {
                    text,
                    logical_line: line_idx,
                    start_col,
                });
            }
        }
    }

    let lnw = line_number_width(total_lines);
    WrapResult {
        lines,
        line_number_width: lnw,
    }
}

/// Calculate the width in characters needed for line number display.
///
/// Returns the number of digits needed to display the largest line
/// number (1-based).
pub fn line_number_width(line_count: usize) -> u16 {
    if line_count == 0 {
        return 1;
    }
    let digits = (line_count as f64).log10().floor() as u16 + 1;
    digits.max(1)
}

/// Calculate the display width of a string.
fn display_width_of(text: &str) -> usize {
    text.chars().map(|c| c.width().unwrap_or(0)).sum()
}

/// Find the byte index of the last space character in a string.
fn find_last_space(text: &str) -> Option<usize> {
    text.rfind(' ')
}

/// Count spaces at the start of a string (for trimming calculation).
fn count_trimmed_spaces(text: &str) -> usize {
    text.chars().take_while(|c| *c == ' ').count()
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- wrap_line ---

    #[test]
    fn short_line_no_wrap() {
        let result = wrap_line("hello", 20);
        assert!(result.len() == 1);
        assert!(result[0].0 == "hello");
        assert!(result[0].1 == 0);
    }

    #[test]
    fn exact_width_no_wrap() {
        let result = wrap_line("12345", 5);
        assert!(result.len() == 1);
        assert!(result[0].0 == "12345");
    }

    #[test]
    fn overflow_by_one_char() {
        let result = wrap_line("123456", 5);
        assert!(result.len() == 2);
    }

    #[test]
    fn word_wrap() {
        let result = wrap_line("hello world foo", 12);
        assert!(result.len() == 2);
        assert!(result[0].0 == "hello world");
        assert!(result[1].0 == "foo");
    }

    #[test]
    fn long_word_break() {
        let result = wrap_line("abcdefghij", 5);
        assert!(result.len() == 2);
        assert!(result[0].0 == "abcde");
        assert!(result[1].0 == "fghij");
    }

    #[test]
    fn cjk_characters_width_2() {
        // Each CJK char is 2 cells wide
        let result = wrap_line("日本語テスト", 6);
        // 日(2)+本(2)+語(2) = 6, テ(2)+ス(2)+ト(2) = 6
        assert!(result.len() == 2);
        assert!(result[0].0 == "日本語");
        assert!(result[1].0 == "テスト");
    }

    #[test]
    fn mixed_content() {
        let result = wrap_line("abc日本", 5);
        // a(1)+b(1)+c(1)+日(2) = 5
        assert!(result.len() == 2);
        assert!(result[0].0 == "abc日");
        // 本(2) fits in 5
        assert!(result[1].0 == "本");
    }

    #[test]
    fn empty_line() {
        let result = wrap_line("", 10);
        assert!(result.len() == 1);
        assert!(result[0].0.is_empty());
    }

    #[test]
    fn single_char_line() {
        let result = wrap_line("x", 10);
        assert!(result.len() == 1);
        assert!(result[0].0 == "x");
    }

    // --- line_number_width ---

    #[test]
    fn line_number_width_small() {
        assert!(line_number_width(1) == 1);
        assert!(line_number_width(9) == 1);
    }

    #[test]
    fn line_number_width_medium() {
        assert!(line_number_width(10) == 2);
        assert!(line_number_width(99) == 2);
        assert!(line_number_width(100) == 3);
    }

    #[test]
    fn line_number_width_zero() {
        assert!(line_number_width(0) == 1);
    }

    // --- wrap_lines (buffer) ---

    #[test]
    fn wrap_buffer_multiline() {
        let buf = TextBuffer::from_text("short\nthis is a longer line");
        let result = wrap_lines(&buf, 10);
        // "short" → 1 visual line
        // "this is a longer line" → wraps
        assert!(result.lines.len() >= 3);
        assert!(result.lines[0].logical_line == 0);
        assert!(result.lines[1].logical_line == 1);
    }

    #[test]
    fn wrap_result_line_number_width() {
        let buf = TextBuffer::from_text("a\nb\nc\nd\ne\nf\ng\nh\ni\nj");
        let result = wrap_lines(&buf, 80);
        assert!(result.line_number_width == 2); // 10 lines → 2 digits
    }
}
