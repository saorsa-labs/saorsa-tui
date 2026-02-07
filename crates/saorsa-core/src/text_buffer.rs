//! Text buffer with rope-based storage for efficient text editing.
//!
//! Wraps [`ropey::Rope`] with a clean API for line-oriented text editing
//! operations used by the [`crate::widget::TextArea`] widget.

use ropey::Rope;
use std::fmt;

/// A text buffer backed by a rope data structure for efficient editing.
///
/// Provides line-oriented access and editing operations suitable for
/// building a text editor widget. All positions are expressed as
/// `(line, col)` pairs using zero-based indexing.
#[derive(Clone, Debug)]
pub struct TextBuffer {
    rope: Rope,
}

impl TextBuffer {
    /// Create a new empty text buffer.
    pub fn new() -> Self {
        Self { rope: Rope::new() }
    }

    /// Create a text buffer from a string.
    pub fn from_text(text: &str) -> Self {
        Self {
            rope: Rope::from_str(text),
        }
    }

    /// Return the number of lines in the buffer.
    ///
    /// An empty buffer has 1 line. A buffer ending with a newline has
    /// an extra empty line at the end.
    pub fn line_count(&self) -> usize {
        self.rope.len_lines()
    }

    /// Get the content of a line by index (without trailing newline).
    ///
    /// Returns `None` if the index is out of bounds.
    pub fn line(&self, idx: usize) -> Option<String> {
        if idx >= self.rope.len_lines() {
            return None;
        }
        let line = self.rope.line(idx);
        let text = line.to_string();
        // Strip trailing newline characters
        let trimmed = text.trim_end_matches('\n').trim_end_matches('\r');
        Some(trimmed.to_string())
    }

    /// Get the character count of a line (excluding trailing newline).
    ///
    /// Returns `None` if the index is out of bounds.
    pub fn line_len(&self, idx: usize) -> Option<usize> {
        self.line(idx).map(|l| l.chars().count())
    }

    /// Return the total number of characters in the buffer.
    pub fn total_chars(&self) -> usize {
        self.rope.len_chars()
    }

    /// Insert a character at the given `(line, col)` position.
    ///
    /// If the position is beyond the end of a line, the character is
    /// appended at the end of that line.
    pub fn insert_char(&mut self, line: usize, col: usize, ch: char) {
        if let Some(char_idx) = self.line_col_to_char(line, col) {
            self.rope.insert_char(char_idx, ch);
        }
    }

    /// Insert a string at the given `(line, col)` position.
    ///
    /// The string may contain newlines, which will split the line.
    pub fn insert_str(&mut self, line: usize, col: usize, text: &str) {
        if let Some(char_idx) = self.line_col_to_char(line, col) {
            self.rope.insert(char_idx, text);
        }
    }

    /// Delete a single character at the given `(line, col)` position.
    ///
    /// If the position is at the end of a line, the trailing newline
    /// is removed, joining this line with the next.
    pub fn delete_char(&mut self, line: usize, col: usize) {
        if let Some(char_idx) = self.line_col_to_char(line, col)
            && char_idx < self.rope.len_chars()
        {
            self.rope.remove(char_idx..char_idx + 1);
        }
    }

    /// Delete a range of text between two `(line, col)` positions.
    ///
    /// The range is from `(start_line, start_col)` inclusive to
    /// `(end_line, end_col)` exclusive. If start equals end, nothing
    /// is deleted.
    pub fn delete_range(
        &mut self,
        start_line: usize,
        start_col: usize,
        end_line: usize,
        end_col: usize,
    ) {
        let start = self.line_col_to_char(start_line, start_col);
        let end = self.line_col_to_char(end_line, end_col);
        if let (Some(s), Some(e)) = (start, end)
            && s < e
            && e <= self.rope.len_chars()
        {
            self.rope.remove(s..e);
        }
    }

    /// Get a range of lines as strings (without trailing newlines).
    ///
    /// The range is `[start, end)` (end-exclusive). Returns an empty
    /// `Vec` if the range is invalid.
    pub fn lines_range(&self, start: usize, end: usize) -> Vec<String> {
        let total = self.rope.len_lines();
        let start = start.min(total);
        let end = end.min(total);
        (start..end).filter_map(|i| self.line(i)).collect()
    }

    /// Convert a `(line, col)` position to a character index into the rope.
    ///
    /// Returns `None` if the line is out of bounds. If the column is
    /// beyond the line length, it is clamped to the end of the line.
    fn line_col_to_char(&self, line: usize, col: usize) -> Option<usize> {
        if line >= self.rope.len_lines() {
            return None;
        }
        let line_start = self.rope.line_to_char(line);
        let line_rope = self.rope.line(line);
        let line_char_len = line_rope.len_chars();
        // Clamp col: for a line with trailing newline, allow up to
        // the content length (before newline) so inserts happen in the
        // right place. For the last line (no trailing newline) allow up to
        // line_char_len.
        let clamped_col = col.min(line_char_len);
        Some(line_start + clamped_col)
    }
}

impl Default for TextBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for TextBuffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for chunk in self.rope.chunks() {
            f.write_str(chunk)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Construction ---

    #[test]
    fn empty_buffer() {
        let buf = TextBuffer::new();
        assert!(buf.line_count() == 1);
        assert!(buf.total_chars() == 0);
        assert!(buf.to_string().is_empty());
    }

    #[test]
    fn from_str_single_line() {
        let buf = TextBuffer::from_text("hello");
        assert!(buf.line_count() == 1);
        assert!(buf.total_chars() == 5);
        match buf.line(0) {
            Some(ref s) if s == "hello" => {}
            _ => unreachable!("expected 'hello'"),
        }
    }

    #[test]
    fn from_str_multi_line() {
        let buf = TextBuffer::from_text("one\ntwo\nthree");
        assert!(buf.line_count() == 3);
        match buf.line(0) {
            Some(ref s) if s == "one" => {}
            _ => unreachable!("expected 'one'"),
        }
        match buf.line(1) {
            Some(ref s) if s == "two" => {}
            _ => unreachable!("expected 'two'"),
        }
        match buf.line(2) {
            Some(ref s) if s == "three" => {}
            _ => unreachable!("expected 'three'"),
        }
    }

    // --- Line access ---

    #[test]
    fn line_out_of_bounds() {
        let buf = TextBuffer::from_text("abc");
        assert!(buf.line(1).is_none());
        assert!(buf.line(100).is_none());
    }

    #[test]
    fn line_len_returns_char_count() {
        let buf = TextBuffer::from_text("hello\nhi");
        match buf.line_len(0) {
            Some(5) => {}
            other => unreachable!("expected Some(5), got {other:?}"),
        }
        match buf.line_len(1) {
            Some(2) => {}
            other => unreachable!("expected Some(2), got {other:?}"),
        }
        assert!(buf.line_len(2).is_none());
    }

    #[test]
    fn lines_range_subset() {
        let buf = TextBuffer::from_text("a\nb\nc\nd");
        let range = buf.lines_range(1, 3);
        assert!(range.len() == 2);
        assert!(range[0] == "b");
        assert!(range[1] == "c");
    }

    #[test]
    fn lines_range_out_of_bounds_clamped() {
        let buf = TextBuffer::from_text("x\ny");
        let range = buf.lines_range(0, 100);
        assert!(range.len() == 2);
    }

    // --- Insert ---

    #[test]
    fn insert_char_middle() {
        let mut buf = TextBuffer::from_text("ac");
        buf.insert_char(0, 1, 'b');
        assert!(buf.to_string() == "abc");
    }

    #[test]
    fn insert_newline_splits_line() {
        let mut buf = TextBuffer::from_text("hello world");
        buf.insert_char(0, 5, '\n');
        assert!(buf.line_count() == 2);
        match buf.line(0) {
            Some(ref s) if s == "hello" => {}
            other => unreachable!("expected 'hello', got {other:?}"),
        }
        match buf.line(1) {
            Some(ref s) if s == " world" => {}
            other => unreachable!("expected ' world', got {other:?}"),
        }
    }

    #[test]
    fn insert_str_with_newlines() {
        let mut buf = TextBuffer::from_text("ac");
        buf.insert_str(0, 1, "b\nd\ne");
        // Result: "ab\nd\nec"
        assert!(buf.line_count() == 3);
        match buf.line(0) {
            Some(ref s) if s == "ab" => {}
            other => unreachable!("expected 'ab', got {other:?}"),
        }
    }

    // --- Delete ---

    #[test]
    fn delete_char_middle() {
        let mut buf = TextBuffer::from_text("abc");
        buf.delete_char(0, 1);
        assert!(buf.to_string() == "ac");
    }

    #[test]
    fn delete_char_joins_lines() {
        let mut buf = TextBuffer::from_text("ab\ncd");
        // Delete the newline at end of first line (position line 0, col 2)
        buf.delete_char(0, 2);
        assert!(buf.line_count() == 1);
        assert!(buf.to_string() == "abcd");
    }

    #[test]
    fn delete_range_within_line() {
        let mut buf = TextBuffer::from_text("abcdef");
        buf.delete_range(0, 1, 0, 4);
        assert!(buf.to_string() == "aef");
    }

    #[test]
    fn delete_range_across_lines() {
        let mut buf = TextBuffer::from_text("hello\nworld\nfoo");
        // Delete from (0,3) to (1,3) → "hel" + "ld\nfoo" = "helld\nfoo"
        buf.delete_range(0, 3, 1, 3);
        assert!(buf.to_string() == "helld\nfoo");
    }

    // --- Edge cases ---

    #[test]
    fn empty_lines() {
        let buf = TextBuffer::from_text("\n\n\n");
        assert!(buf.line_count() == 4);
        match buf.line(0) {
            Some(ref s) if s.is_empty() => {}
            other => unreachable!("expected empty string, got {other:?}"),
        }
    }

    #[test]
    fn unicode_content() {
        let buf = TextBuffer::from_text("日本語\némoji 🎉");
        assert!(buf.line_count() == 2);
        match buf.line(0) {
            Some(ref s) if s == "日本語" => {}
            other => unreachable!("expected '日本語', got {other:?}"),
        }
        match buf.line_len(1) {
            Some(7) => {}
            other => unreachable!("expected Some(7), got {other:?}"),
        }
    }

    #[test]
    fn display_trait() {
        let buf = TextBuffer::from_text("hello\nworld");
        assert!(buf.to_string() == "hello\nworld");
    }

    #[test]
    fn default_is_empty() {
        let buf = TextBuffer::default();
        assert!(buf.line_count() == 1);
        assert!(buf.total_chars() == 0);
    }
}
