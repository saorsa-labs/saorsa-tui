//! Cursor position and selection types for text editing.
//!
//! Provides [`CursorPosition`], [`Selection`], and [`CursorState`] types
//! used by the [`crate::widget::TextArea`] widget for tracking the editing
//! cursor, text selection, and movement operations.

use crate::text_buffer::TextBuffer;

/// A position within a text buffer, expressed as a line and column.
///
/// Both `line` and `col` are zero-based.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CursorPosition {
    /// Zero-based line index.
    pub line: usize,
    /// Zero-based column index (character offset within the line).
    pub col: usize,
}

impl CursorPosition {
    /// Create a new cursor position.
    pub fn new(line: usize, col: usize) -> Self {
        Self { line, col }
    }

    /// Create a cursor position at the beginning of the buffer (0, 0).
    pub fn beginning() -> Self {
        Self { line: 0, col: 0 }
    }
}

impl PartialOrd for CursorPosition {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CursorPosition {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.line.cmp(&other.line).then(self.col.cmp(&other.col))
    }
}

/// A text selection defined by an anchor and a head position.
///
/// The anchor is where the selection started and the head is where
/// the cursor currently is. The anchor may come before or after the
/// head — use [`Selection::ordered`] to get (start, end).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Selection {
    /// The position where the selection started.
    pub anchor: CursorPosition,
    /// The current cursor position (moving end of the selection).
    pub head: CursorPosition,
}

impl Selection {
    /// Create a new selection.
    pub fn new(anchor: CursorPosition, head: CursorPosition) -> Self {
        Self { anchor, head }
    }

    /// Returns `true` if the selection is empty (anchor equals head).
    pub fn is_empty(&self) -> bool {
        self.anchor == self.head
    }

    /// Return the selection as `(start, end)` in document order.
    pub fn ordered(&self) -> (CursorPosition, CursorPosition) {
        if self.anchor <= self.head {
            (self.anchor, self.head)
        } else {
            (self.head, self.anchor)
        }
    }

    /// Check if a position is contained within the selection.
    pub fn contains(&self, pos: CursorPosition) -> bool {
        let (start, end) = self.ordered();
        pos >= start && pos < end
    }

    /// Return the range of lines spanned by the selection.
    pub fn line_range(&self) -> (usize, usize) {
        let (start, end) = self.ordered();
        (start.line, end.line)
    }
}

/// The full cursor state for a text editing session.
///
/// Tracks the cursor position, optional selection, and the preferred
/// column for vertical movement (so moving up/down through short lines
/// returns to the original column).
#[derive(Clone, Debug)]
pub struct CursorState {
    /// Current cursor position.
    pub position: CursorPosition,
    /// Active selection, if any.
    pub selection: Option<Selection>,
    /// Preferred column for vertical movement.
    pub preferred_col: Option<usize>,
}

impl CursorState {
    /// Create a new cursor state at the given position.
    pub fn new(line: usize, col: usize) -> Self {
        Self {
            position: CursorPosition::new(line, col),
            selection: None,
            preferred_col: None,
        }
    }

    /// Move cursor left by one character, wrapping to the previous line.
    pub fn move_left(&mut self, buffer: &TextBuffer) {
        self.clear_selection();
        if self.position.col > 0 {
            self.position.col -= 1;
        } else if self.position.line > 0 {
            self.position.line -= 1;
            self.position.col = buffer.line_len(self.position.line).unwrap_or(0);
        }
        self.preferred_col = None;
    }

    /// Move cursor right by one character, wrapping to the next line.
    pub fn move_right(&mut self, buffer: &TextBuffer) {
        self.clear_selection();
        let line_len = buffer.line_len(self.position.line).unwrap_or(0);
        if self.position.col < line_len {
            self.position.col += 1;
        } else if self.position.line + 1 < buffer.line_count() {
            self.position.line += 1;
            self.position.col = 0;
        }
        self.preferred_col = None;
    }

    /// Move cursor up by one line, preserving the preferred column.
    pub fn move_up(&mut self, buffer: &TextBuffer) {
        self.clear_selection();
        if self.position.line > 0 {
            let target_col = self.preferred_col.unwrap_or(self.position.col);
            self.preferred_col = Some(target_col);
            self.position.line -= 1;
            let line_len = buffer.line_len(self.position.line).unwrap_or(0);
            self.position.col = target_col.min(line_len);
        }
    }

    /// Move cursor down by one line, preserving the preferred column.
    pub fn move_down(&mut self, buffer: &TextBuffer) {
        self.clear_selection();
        if self.position.line + 1 < buffer.line_count() {
            let target_col = self.preferred_col.unwrap_or(self.position.col);
            self.preferred_col = Some(target_col);
            self.position.line += 1;
            let line_len = buffer.line_len(self.position.line).unwrap_or(0);
            self.position.col = target_col.min(line_len);
        }
    }

    /// Move cursor to the start of the current line.
    pub fn move_to_line_start(&mut self) {
        self.clear_selection();
        self.position.col = 0;
        self.preferred_col = None;
    }

    /// Move cursor to the end of the current line.
    pub fn move_to_line_end(&mut self, buffer: &TextBuffer) {
        self.clear_selection();
        self.position.col = buffer.line_len(self.position.line).unwrap_or(0);
        self.preferred_col = None;
    }

    /// Move cursor to the beginning of the buffer.
    pub fn move_to_buffer_start(&mut self) {
        self.clear_selection();
        self.position = CursorPosition::beginning();
        self.preferred_col = None;
    }

    /// Move cursor to the end of the buffer.
    pub fn move_to_buffer_end(&mut self, buffer: &TextBuffer) {
        self.clear_selection();
        let last_line = buffer.line_count().saturating_sub(1);
        self.position.line = last_line;
        self.position.col = buffer.line_len(last_line).unwrap_or(0);
        self.preferred_col = None;
    }

    /// Start a selection at the current cursor position.
    pub fn start_selection(&mut self) {
        self.selection = Some(Selection::new(self.position, self.position));
    }

    /// Extend the selection to the current cursor position.
    ///
    /// If no selection exists, this starts one from the current position.
    pub fn extend_selection(&mut self) {
        match &mut self.selection {
            Some(sel) => sel.head = self.position,
            None => self.start_selection(),
        }
    }

    /// Clear the current selection.
    pub fn clear_selection(&mut self) {
        self.selection = None;
    }

    /// Get the text currently selected in the buffer.
    ///
    /// Returns `None` if there is no selection or the selection is empty.
    pub fn selected_text(&self, buffer: &TextBuffer) -> Option<String> {
        let sel = self.selection.as_ref()?;
        if sel.is_empty() {
            return None;
        }
        let (start, end) = sel.ordered();

        let mut result = String::new();
        for line_idx in start.line..=end.line {
            if let Some(line_text) = buffer.line(line_idx) {
                let line_start = if line_idx == start.line { start.col } else { 0 };
                let line_end = if line_idx == end.line {
                    end.col.min(line_text.chars().count())
                } else {
                    line_text.chars().count()
                };

                let chars: String = line_text
                    .chars()
                    .skip(line_start)
                    .take(line_end.saturating_sub(line_start))
                    .collect();
                result.push_str(&chars);

                // Add newline between lines (but not after the last)
                if line_idx < end.line {
                    result.push('\n');
                }
            }
        }

        if result.is_empty() {
            None
        } else {
            Some(result)
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn buf(text: &str) -> TextBuffer {
        TextBuffer::from_text(text)
    }

    // --- CursorPosition ---

    #[test]
    fn cursor_position_new() {
        let p = CursorPosition::new(3, 5);
        assert!(p.line == 3);
        assert!(p.col == 5);
    }

    #[test]
    fn cursor_position_beginning() {
        let p = CursorPosition::beginning();
        assert!(p.line == 0);
        assert!(p.col == 0);
    }

    #[test]
    fn cursor_position_ordering() {
        let a = CursorPosition::new(0, 5);
        let b = CursorPosition::new(1, 0);
        let c = CursorPosition::new(0, 10);
        assert!(a < b);
        assert!(a < c);
        assert!(b > c);
    }

    // --- Selection ---

    #[test]
    fn selection_empty() {
        let p = CursorPosition::new(0, 0);
        let sel = Selection::new(p, p);
        assert!(sel.is_empty());
    }

    #[test]
    fn selection_ordered_forward() {
        let a = CursorPosition::new(0, 0);
        let b = CursorPosition::new(1, 5);
        let sel = Selection::new(a, b);
        let (start, end) = sel.ordered();
        assert!(start == a);
        assert!(end == b);
    }

    #[test]
    fn selection_ordered_backward() {
        let a = CursorPosition::new(1, 5);
        let b = CursorPosition::new(0, 0);
        let sel = Selection::new(a, b);
        let (start, end) = sel.ordered();
        assert!(start == b);
        assert!(end == a);
    }

    #[test]
    fn selection_contains() {
        let sel = Selection::new(CursorPosition::new(0, 2), CursorPosition::new(0, 8));
        assert!(sel.contains(CursorPosition::new(0, 5)));
        assert!(!sel.contains(CursorPosition::new(0, 8))); // end is exclusive
        assert!(!sel.contains(CursorPosition::new(0, 1)));
    }

    #[test]
    fn selection_line_range() {
        let sel = Selection::new(CursorPosition::new(2, 0), CursorPosition::new(5, 3));
        assert!(sel.line_range() == (2, 5));
    }

    // --- CursorState movement ---

    #[test]
    fn move_left_within_line() {
        let b = buf("hello");
        let mut c = CursorState::new(0, 3);
        c.move_left(&b);
        assert!(c.position.col == 2);
    }

    #[test]
    fn move_left_wraps_to_prev_line() {
        let b = buf("hello\nworld");
        let mut c = CursorState::new(1, 0);
        c.move_left(&b);
        assert!(c.position.line == 0);
        assert!(c.position.col == 5);
    }

    #[test]
    fn move_left_at_beginning_stays() {
        let b = buf("hello");
        let mut c = CursorState::new(0, 0);
        c.move_left(&b);
        assert!(c.position == CursorPosition::beginning());
    }

    #[test]
    fn move_right_within_line() {
        let b = buf("hello");
        let mut c = CursorState::new(0, 2);
        c.move_right(&b);
        assert!(c.position.col == 3);
    }

    #[test]
    fn move_right_wraps_to_next_line() {
        let b = buf("hello\nworld");
        let mut c = CursorState::new(0, 5);
        c.move_right(&b);
        assert!(c.position.line == 1);
        assert!(c.position.col == 0);
    }

    #[test]
    fn move_right_at_end_stays() {
        let b = buf("hello");
        let mut c = CursorState::new(0, 5);
        c.move_right(&b);
        assert!(c.position.line == 0);
        assert!(c.position.col == 5);
    }

    #[test]
    fn move_up_preserves_preferred_col() {
        let b = buf("long line here\nhi\nanother long line");
        let mut c = CursorState::new(0, 10);
        c.move_down(&b); // line 1 "hi" → col clamped to 2
        assert!(c.position.col == 2);
        c.move_down(&b); // line 2 "another long line" → col restored to 10
        assert!(c.position.col == 10);
    }

    #[test]
    fn move_up_at_top_stays() {
        let b = buf("hello");
        let mut c = CursorState::new(0, 3);
        c.move_up(&b);
        assert!(c.position.line == 0);
    }

    #[test]
    fn move_down_at_bottom_stays() {
        let b = buf("hello");
        let mut c = CursorState::new(0, 3);
        c.move_down(&b);
        assert!(c.position.line == 0);
    }

    #[test]
    fn move_to_line_start_and_end() {
        let b = buf("hello");
        let mut c = CursorState::new(0, 3);
        c.move_to_line_start();
        assert!(c.position.col == 0);
        c.move_to_line_end(&b);
        assert!(c.position.col == 5);
    }

    #[test]
    fn move_to_buffer_start_and_end() {
        let b = buf("hello\nworld\nfoo");
        let mut c = CursorState::new(1, 3);
        c.move_to_buffer_start();
        assert!(c.position == CursorPosition::beginning());
        c.move_to_buffer_end(&b);
        assert!(c.position.line == 2);
        assert!(c.position.col == 3);
    }

    // --- Selection operations ---

    #[test]
    fn start_and_extend_selection() {
        let mut c = CursorState::new(0, 5);
        c.start_selection();
        assert!(c.selection.is_some());
        c.position.col = 10;
        c.extend_selection();
        match &c.selection {
            Some(sel) => {
                assert!(sel.anchor.col == 5);
                assert!(sel.head.col == 10);
            }
            None => unreachable!("expected selection"),
        }
    }

    #[test]
    fn clear_selection() {
        let mut c = CursorState::new(0, 0);
        c.start_selection();
        assert!(c.selection.is_some());
        c.clear_selection();
        assert!(c.selection.is_none());
    }

    #[test]
    fn selected_text_single_line() {
        let b = buf("hello world");
        let mut c = CursorState::new(0, 0);
        c.selection = Some(Selection::new(
            CursorPosition::new(0, 6),
            CursorPosition::new(0, 11),
        ));
        match c.selected_text(&b) {
            Some(ref s) if s == "world" => {}
            other => unreachable!("expected 'world', got {other:?}"),
        }
    }

    #[test]
    fn selected_text_multi_line() {
        let b = buf("hello\nworld\nfoo");
        let mut c = CursorState::new(0, 0);
        c.selection = Some(Selection::new(
            CursorPosition::new(0, 3),
            CursorPosition::new(1, 3),
        ));
        match c.selected_text(&b) {
            Some(ref s) if s == "lo\nwor" => {}
            other => unreachable!("expected 'lo\\nwor', got {other:?}"),
        }
    }

    #[test]
    fn selected_text_empty_selection_returns_none() {
        let b = buf("hello");
        let mut c = CursorState::new(0, 3);
        c.selection = Some(Selection::new(
            CursorPosition::new(0, 3),
            CursorPosition::new(0, 3),
        ));
        assert!(c.selected_text(&b).is_none());
    }

    #[test]
    fn movement_clears_selection() {
        let b = buf("hello");
        let mut c = CursorState::new(0, 3);
        c.start_selection();
        c.move_right(&b);
        assert!(c.selection.is_none());
    }
}
