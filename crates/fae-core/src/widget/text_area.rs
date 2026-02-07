//! Multi-line text editing widget with cursor, selection, soft wrap,
//! line numbers, and syntax highlighting.

use crate::buffer::ScreenBuffer;
use crate::cell::Cell;
use crate::cursor::{CursorPosition, CursorState, Selection};
use crate::event::{Event, KeyCode, KeyEvent, Modifiers};
use crate::geometry::Rect;
use crate::highlight::{Highlighter, NoHighlighter};
use crate::style::Style;
use crate::text_buffer::TextBuffer;
use crate::undo::{EditOperation, UndoStack};
use crate::wrap::wrap_line;
use unicode_width::UnicodeWidthChar;

use super::{EventResult, InteractiveWidget, Widget};

/// A multi-line text editing widget.
///
/// Supports cursor movement, text selection, undo/redo, soft wrapping,
/// optional line numbers, and pluggable syntax highlighting.
pub struct TextArea {
    /// The text content.
    pub buffer: TextBuffer,
    /// Cursor and selection state.
    pub cursor: CursorState,
    /// Undo/redo history.
    pub undo_stack: UndoStack,
    highlighter: Box<dyn Highlighter>,
    /// Index of the first visible logical line.
    pub scroll_offset: usize,
    /// Whether to show line numbers in the left gutter.
    pub show_line_numbers: bool,
    /// Base text style.
    pub style: Style,
    /// Style for the cursor cell.
    pub cursor_style: Style,
    /// Style for selected text.
    pub selection_style: Style,
    /// Style for line numbers.
    pub line_number_style: Style,
}

impl TextArea {
    /// Create a new empty text area.
    pub fn new() -> Self {
        Self {
            buffer: TextBuffer::new(),
            cursor: CursorState::new(0, 0),
            undo_stack: UndoStack::new(1000),
            highlighter: Box::new(NoHighlighter),
            scroll_offset: 0,
            show_line_numbers: false,
            style: Style::default(),
            cursor_style: Style::new().reverse(true),
            selection_style: Style::new().reverse(true),
            line_number_style: Style::new().dim(true),
        }
    }

    /// Create a text area pre-filled with text.
    pub fn from_text(text: &str) -> Self {
        let mut ta = Self::new();
        ta.buffer = TextBuffer::from_text(text);
        ta
    }

    /// Set a custom syntax highlighter.
    #[must_use]
    pub fn with_highlighter(mut self, h: Box<dyn Highlighter>) -> Self {
        self.highlighter = h;
        self
    }

    /// Set the base text style.
    #[must_use]
    pub fn with_style(mut self, s: Style) -> Self {
        self.style = s;
        self
    }

    /// Enable or disable line numbers.
    #[must_use]
    pub fn with_line_numbers(mut self, show: bool) -> Self {
        self.show_line_numbers = show;
        self
    }

    /// Set the cursor display style.
    #[must_use]
    pub fn with_cursor_style(mut self, s: Style) -> Self {
        self.cursor_style = s;
        self
    }

    /// Set the selection display style.
    #[must_use]
    pub fn with_selection_style(mut self, s: Style) -> Self {
        self.selection_style = s;
        self
    }

    /// Get the current text content as a string.
    pub fn text(&self) -> String {
        self.buffer.to_string()
    }

    // --- Editing operations ---

    /// Insert a character at the cursor position.
    pub fn insert_char(&mut self, ch: char) {
        self.delete_selection_if_active();
        let pos = self.cursor.position;
        self.buffer.insert_char(pos.line, pos.col, ch);
        self.undo_stack.push(EditOperation::Insert {
            pos,
            text: ch.to_string(),
        });
        self.highlighter.on_edit(pos.line);
        // Advance cursor
        if ch == '\n' {
            self.cursor.position.line += 1;
            self.cursor.position.col = 0;
        } else {
            self.cursor.position.col += 1;
        }
        self.cursor.preferred_col = None;
    }

    /// Insert a string at the cursor position.
    pub fn insert_str(&mut self, text: &str) {
        self.delete_selection_if_active();
        let pos = self.cursor.position;
        self.buffer.insert_str(pos.line, pos.col, text);
        self.undo_stack.push(EditOperation::Insert {
            pos,
            text: text.to_string(),
        });
        self.highlighter.on_edit(pos.line);

        // Advance cursor past inserted text
        for ch in text.chars() {
            if ch == '\n' {
                self.cursor.position.line += 1;
                self.cursor.position.col = 0;
            } else {
                self.cursor.position.col += 1;
            }
        }
        self.cursor.preferred_col = None;
    }

    /// Delete the character before the cursor (backspace).
    pub fn delete_backward(&mut self) {
        if self.delete_selection_if_active() {
            return;
        }
        let pos = self.cursor.position;
        if pos.col > 0 {
            // Delete within line
            let del_col = pos.col - 1;
            if let Some(line_text) = self.buffer.line(pos.line) {
                let deleted: String = line_text
                    .chars()
                    .nth(del_col)
                    .map(String::from)
                    .unwrap_or_default();
                self.buffer.delete_char(pos.line, del_col);
                self.undo_stack.push(EditOperation::Delete {
                    pos: CursorPosition::new(pos.line, del_col),
                    text: deleted,
                });
                self.highlighter.on_edit(pos.line);
                self.cursor.position.col -= 1;
            }
        } else if pos.line > 0 {
            // Join with previous line
            let prev_line_len = self.buffer.line_len(pos.line - 1).unwrap_or(0);
            self.buffer.delete_char(pos.line - 1, prev_line_len);
            self.undo_stack.push(EditOperation::Delete {
                pos: CursorPosition::new(pos.line - 1, prev_line_len),
                text: "\n".to_string(),
            });
            self.highlighter.on_edit(pos.line - 1);
            self.cursor.position.line -= 1;
            self.cursor.position.col = prev_line_len;
        }
        self.cursor.preferred_col = None;
    }

    /// Delete the character at the cursor position (delete key).
    pub fn delete_forward(&mut self) {
        if self.delete_selection_if_active() {
            return;
        }
        let pos = self.cursor.position;
        let line_len = self.buffer.line_len(pos.line).unwrap_or(0);
        if pos.col < line_len {
            if let Some(line_text) = self.buffer.line(pos.line) {
                let deleted: String = line_text
                    .chars()
                    .nth(pos.col)
                    .map(String::from)
                    .unwrap_or_default();
                self.buffer.delete_char(pos.line, pos.col);
                self.undo_stack
                    .push(EditOperation::Delete { pos, text: deleted });
                self.highlighter.on_edit(pos.line);
            }
        } else if pos.line + 1 < self.buffer.line_count() {
            // Join with next line
            self.buffer.delete_char(pos.line, pos.col);
            self.undo_stack.push(EditOperation::Delete {
                pos,
                text: "\n".to_string(),
            });
            self.highlighter.on_edit(pos.line);
        }
    }

    /// Delete the currently selected text, if any.
    ///
    /// Returns `true` if a selection was deleted.
    pub fn delete_selection(&mut self) -> bool {
        self.delete_selection_if_active()
    }

    /// Insert a newline at the cursor position.
    pub fn new_line(&mut self) {
        self.insert_char('\n');
    }

    /// Undo the last operation.
    pub fn undo(&mut self) {
        if let Some(op) = self.undo_stack.undo() {
            self.apply_operation(&op);
        }
    }

    /// Redo the last undone operation.
    pub fn redo(&mut self) {
        if let Some(op) = self.undo_stack.redo() {
            self.apply_operation(&op);
        }
    }

    /// Ensure the cursor is within the visible area, adjusting scroll.
    pub fn ensure_cursor_visible(&mut self, area_height: u16) {
        let height = area_height as usize;
        if height == 0 {
            return;
        }
        let line = self.cursor.position.line;
        if line < self.scroll_offset {
            self.scroll_offset = line;
        } else if line >= self.scroll_offset + height {
            self.scroll_offset = line.saturating_sub(height - 1);
        }
    }

    // --- Private helpers ---

    /// Delete the active selection text and place cursor at start.
    fn delete_selection_if_active(&mut self) -> bool {
        let sel = match self.cursor.selection.take() {
            Some(s) if !s.is_empty() => s,
            other => {
                self.cursor.selection = other;
                return false;
            }
        };

        let (start, end) = sel.ordered();
        if let Some(selected) = self.selected_text_for(&sel) {
            self.buffer
                .delete_range(start.line, start.col, end.line, end.col);
            self.undo_stack.push(EditOperation::Delete {
                pos: start,
                text: selected,
            });
            self.highlighter.on_edit(start.line);
            self.cursor.position = start;
            self.cursor.preferred_col = None;
        }
        true
    }

    /// Get text for a selection (without clearing it).
    fn selected_text_for(&self, sel: &Selection) -> Option<String> {
        if sel.is_empty() {
            return None;
        }
        let (start, end) = sel.ordered();
        let mut result = String::new();
        for line_idx in start.line..=end.line {
            if let Some(line_text) = self.buffer.line(line_idx) {
                let ls = if line_idx == start.line { start.col } else { 0 };
                let le = if line_idx == end.line {
                    end.col.min(line_text.chars().count())
                } else {
                    line_text.chars().count()
                };
                let chars: String = line_text
                    .chars()
                    .skip(ls)
                    .take(le.saturating_sub(ls))
                    .collect();
                result.push_str(&chars);
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

    /// Apply an edit operation (for undo/redo).
    fn apply_operation(&mut self, op: &EditOperation) {
        match op {
            EditOperation::Insert { pos, text } => {
                self.buffer.insert_str(pos.line, pos.col, text);
                // Move cursor to end of inserted text
                let mut line = pos.line;
                let mut col = pos.col;
                for ch in text.chars() {
                    if ch == '\n' {
                        line += 1;
                        col = 0;
                    } else {
                        col += 1;
                    }
                }
                self.cursor.position = CursorPosition::new(line, col);
            }
            EditOperation::Delete { pos, text } => {
                // Calculate end position
                let mut end_line = pos.line;
                let mut end_col = pos.col;
                for ch in text.chars() {
                    if ch == '\n' {
                        end_line += 1;
                        end_col = 0;
                    } else {
                        end_col += 1;
                    }
                }
                self.buffer
                    .delete_range(pos.line, pos.col, end_line, end_col);
                self.cursor.position = *pos;
            }
            EditOperation::Replace {
                pos,
                old_text,
                new_text,
            } => {
                // Delete old text, insert new
                let mut end_line = pos.line;
                let mut end_col = pos.col;
                for ch in old_text.chars() {
                    if ch == '\n' {
                        end_line += 1;
                        end_col = 0;
                    } else {
                        end_col += 1;
                    }
                }
                self.buffer
                    .delete_range(pos.line, pos.col, end_line, end_col);
                self.buffer.insert_str(pos.line, pos.col, new_text);
                self.cursor.position = *pos;
            }
        }
        self.cursor.preferred_col = None;
    }
}

impl Default for TextArea {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for TextArea {
    fn render(&self, area: Rect, buf: &mut ScreenBuffer) {
        if area.size.width == 0 || area.size.height == 0 {
            return;
        }

        let height = area.size.height as usize;
        let total_width = area.size.width as usize;

        // Calculate gutter width for line numbers
        let gutter_width = if self.show_line_numbers {
            let digits = crate::wrap::line_number_width(self.buffer.line_count()) as usize;
            digits + 1 // extra space after number
        } else {
            0
        };

        let text_width = total_width.saturating_sub(gutter_width);
        if text_width == 0 {
            return;
        }

        // Render visible lines
        let mut row: usize = 0;
        let mut logical_line = self.scroll_offset;

        while row < height && logical_line < self.buffer.line_count() {
            let line_text = self.buffer.line(logical_line).unwrap_or_default();

            // Get highlight spans for this line
            let spans = self.highlighter.highlight_line(logical_line, &line_text);

            // Soft wrap the line
            let wrapped = wrap_line(&line_text, text_width);

            for (wrap_idx, (visual_text, start_col)) in wrapped.iter().enumerate() {
                if row >= height {
                    break;
                }

                let y = area.position.y + row as u16;

                // Render line number (only for first visual line of each logical line)
                if self.show_line_numbers {
                    if wrap_idx == 0 {
                        let num_str = format!("{}", logical_line + 1);
                        let padded = format!("{:>width$} ", num_str, width = gutter_width - 1);
                        for (i, ch) in padded.chars().enumerate() {
                            let x = area.position.x + i as u16;
                            if x < area.position.x + area.size.width {
                                buf.set(
                                    x,
                                    y,
                                    Cell::new(ch.to_string(), self.line_number_style.clone()),
                                );
                            }
                        }
                    } else {
                        // Blank gutter for continuation lines
                        for i in 0..gutter_width {
                            let x = area.position.x + i as u16;
                            if x < area.position.x + area.size.width {
                                buf.set(
                                    x,
                                    y,
                                    Cell::new(" ".to_string(), self.line_number_style.clone()),
                                );
                            }
                        }
                    }
                }

                // Render text content
                let gutter_x = area.position.x + gutter_width as u16;
                let mut col_offset: usize = 0;

                for (char_idx, ch) in visual_text.chars().enumerate() {
                    let buffer_col = start_col + char_idx;
                    let ch_width = UnicodeWidthChar::width(ch).unwrap_or(0);
                    let x = gutter_x + col_offset as u16;

                    if x >= area.position.x + area.size.width {
                        break;
                    }

                    // Determine style: selection > highlight > base
                    let char_style = self.resolve_style(logical_line, buffer_col, &spans);

                    buf.set(x, y, Cell::new(ch.to_string(), char_style));
                    col_offset += ch_width;
                }

                // Render cursor
                if logical_line == self.cursor.position.line {
                    let col = self.cursor.position.col;
                    let end_col = start_col + visual_text.chars().count();
                    let is_last_wrap = wrap_idx == wrapped.len() - 1;
                    let cursor_in_wrap = col >= *start_col && (col < end_col || is_last_wrap);

                    if cursor_in_wrap {
                        let cursor_visual_col = self.cursor.position.col - start_col;
                        // Calculate display offset for cursor
                        let cursor_x_offset: usize = visual_text
                            .chars()
                            .take(cursor_visual_col)
                            .map(|c| UnicodeWidthChar::width(c).unwrap_or(0))
                            .sum();
                        let cursor_x = gutter_x + cursor_x_offset as u16;
                        if cursor_x < area.position.x + area.size.width {
                            let cursor_ch = visual_text
                                .chars()
                                .nth(cursor_visual_col)
                                .map(|c| c.to_string())
                                .unwrap_or_else(|| " ".to_string());
                            buf.set(cursor_x, y, Cell::new(cursor_ch, self.cursor_style.clone()));
                        }
                    }
                }

                row += 1;
            }

            logical_line += 1;
        }
    }
}

impl TextArea {
    /// Resolve the style for a character at (line, col).
    fn resolve_style(
        &self,
        line: usize,
        col: usize,
        spans: &[crate::highlight::HighlightSpan],
    ) -> Style {
        let pos = CursorPosition::new(line, col);

        // Selection takes priority
        if let Some(ref sel) = self.cursor.selection
            && sel.contains(pos)
        {
            return self.selection_style.clone();
        }

        // Check highlight spans
        for span in spans {
            if col >= span.start_col && col < span.end_col {
                return span.style.clone();
            }
        }

        // Base style
        self.style.clone()
    }
}

impl InteractiveWidget for TextArea {
    fn handle_event(&mut self, event: &Event) -> EventResult {
        match event {
            Event::Key(key_event) => self.handle_key(key_event),
            _ => EventResult::Ignored,
        }
    }
}

impl TextArea {
    /// Handle a key event.
    fn handle_key(&mut self, key: &KeyEvent) -> EventResult {
        let shift = key.modifiers.contains(Modifiers::SHIFT);
        let ctrl = key.modifiers.contains(Modifiers::CTRL);

        match key.code {
            KeyCode::Left => {
                if shift {
                    if self.cursor.selection.is_none() {
                        self.cursor.start_selection();
                    }
                    self.cursor.position = self.move_left_pos();
                    self.cursor.extend_selection();
                } else {
                    self.cursor.move_left(&self.buffer);
                }
                EventResult::Consumed
            }
            KeyCode::Right => {
                if shift {
                    if self.cursor.selection.is_none() {
                        self.cursor.start_selection();
                    }
                    self.cursor.position = self.move_right_pos();
                    self.cursor.extend_selection();
                } else {
                    self.cursor.move_right(&self.buffer);
                }
                EventResult::Consumed
            }
            KeyCode::Up => {
                if shift {
                    if self.cursor.selection.is_none() {
                        self.cursor.start_selection();
                    }
                    self.move_up_no_clear();
                    self.cursor.extend_selection();
                } else {
                    self.cursor.move_up(&self.buffer);
                }
                EventResult::Consumed
            }
            KeyCode::Down => {
                if shift {
                    if self.cursor.selection.is_none() {
                        self.cursor.start_selection();
                    }
                    self.move_down_no_clear();
                    self.cursor.extend_selection();
                } else {
                    self.cursor.move_down(&self.buffer);
                }
                EventResult::Consumed
            }
            KeyCode::Home => {
                if ctrl {
                    self.cursor.move_to_buffer_start();
                } else {
                    self.cursor.move_to_line_start();
                }
                EventResult::Consumed
            }
            KeyCode::End => {
                if ctrl {
                    self.cursor.move_to_buffer_end(&self.buffer);
                } else {
                    self.cursor.move_to_line_end(&self.buffer);
                }
                EventResult::Consumed
            }
            KeyCode::Backspace => {
                self.delete_backward();
                EventResult::Consumed
            }
            KeyCode::Delete => {
                self.delete_forward();
                EventResult::Consumed
            }
            KeyCode::Enter => {
                self.new_line();
                EventResult::Consumed
            }
            KeyCode::Char(ch) => {
                if ctrl && ch == 'z' {
                    self.undo();
                } else if ctrl && ch == 'y' {
                    self.redo();
                } else if !ctrl {
                    self.insert_char(ch);
                } else {
                    return EventResult::Ignored;
                }
                EventResult::Consumed
            }
            _ => EventResult::Ignored,
        }
    }

    /// Move cursor left without clearing selection.
    fn move_left_pos(&self) -> CursorPosition {
        let mut pos = self.cursor.position;
        if pos.col > 0 {
            pos.col -= 1;
        } else if pos.line > 0 {
            pos.line -= 1;
            pos.col = self.buffer.line_len(pos.line).unwrap_or(0);
        }
        pos
    }

    /// Move cursor right without clearing selection.
    fn move_right_pos(&self) -> CursorPosition {
        let mut pos = self.cursor.position;
        let line_len = self.buffer.line_len(pos.line).unwrap_or(0);
        if pos.col < line_len {
            pos.col += 1;
        } else if pos.line + 1 < self.buffer.line_count() {
            pos.line += 1;
            pos.col = 0;
        }
        pos
    }

    /// Move cursor up without clearing selection.
    fn move_up_no_clear(&mut self) {
        if self.cursor.position.line > 0 {
            let target_col = self
                .cursor
                .preferred_col
                .unwrap_or(self.cursor.position.col);
            self.cursor.preferred_col = Some(target_col);
            self.cursor.position.line -= 1;
            let line_len = self.buffer.line_len(self.cursor.position.line).unwrap_or(0);
            self.cursor.position.col = target_col.min(line_len);
        }
    }

    /// Move cursor down without clearing selection.
    fn move_down_no_clear(&mut self) {
        if self.cursor.position.line + 1 < self.buffer.line_count() {
            let target_col = self
                .cursor
                .preferred_col
                .unwrap_or(self.cursor.position.col);
            self.cursor.preferred_col = Some(target_col);
            self.cursor.position.line += 1;
            let line_len = self.buffer.line_len(self.cursor.position.line).unwrap_or(0);
            self.cursor.position.col = target_col.min(line_len);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Size;

    // --- Task 6: Rendering ---

    #[test]
    fn empty_textarea_renders() {
        let ta = TextArea::new();
        let mut buf = ScreenBuffer::new(Size::new(20, 5));
        ta.render(Rect::new(0, 0, 20, 5), &mut buf);
        // Should not crash
    }

    #[test]
    fn text_renders_correctly() {
        let ta = TextArea::from_text("hello");
        let mut buf = ScreenBuffer::new(Size::new(20, 5));
        ta.render(Rect::new(0, 0, 20, 5), &mut buf);
        assert!(buf.get(0, 0).map(|c| c.grapheme.as_str()) == Some("h"));
        assert!(buf.get(4, 0).map(|c| c.grapheme.as_str()) == Some("o"));
    }

    #[test]
    fn line_numbers_displayed() {
        let ta = TextArea::from_text("line1\nline2\nline3").with_line_numbers(true);
        let mut buf = ScreenBuffer::new(Size::new(20, 5));
        ta.render(Rect::new(0, 0, 20, 5), &mut buf);
        // Line number "1" should be in the gutter
        assert!(buf.get(0, 0).map(|c| c.grapheme.as_str()) == Some("1"));
    }

    #[test]
    fn soft_wrap_splits_long_line() {
        let ta = TextArea::from_text("abcdefghij");
        let mut buf = ScreenBuffer::new(Size::new(5, 5));
        ta.render(Rect::new(0, 0, 5, 5), &mut buf);
        // First row: "abcde"
        assert!(buf.get(0, 0).map(|c| c.grapheme.as_str()) == Some("a"));
        assert!(buf.get(4, 0).map(|c| c.grapheme.as_str()) == Some("e"));
        // Second row: "fghij"
        assert!(buf.get(0, 1).map(|c| c.grapheme.as_str()) == Some("f"));
    }

    #[test]
    fn cursor_visible_at_position() {
        let ta = TextArea::from_text("hello");
        let mut buf = ScreenBuffer::new(Size::new(20, 5));
        ta.render(Rect::new(0, 0, 20, 5), &mut buf);
        // Cursor at (0,0) should have cursor_style (reverse)
        let cell = buf.get(0, 0);
        assert!(cell.is_some());
        assert!(cell.map(|c| c.style.reverse) == Some(true));
    }

    #[test]
    fn scroll_offset_hides_top_lines() {
        let mut ta = TextArea::from_text("line1\nline2\nline3\nline4");
        ta.scroll_offset = 2;
        let mut buf = ScreenBuffer::new(Size::new(20, 2));
        ta.render(Rect::new(0, 0, 20, 2), &mut buf);
        // Should show line3 and line4
        assert!(buf.get(0, 0).map(|c| c.grapheme.as_str()) == Some("l"));
        assert!(buf.get(4, 0).map(|c| c.grapheme.as_str()) == Some("3"));
    }

    // --- Task 7: Editing ---

    #[test]
    fn insert_char_updates_buffer_and_cursor() {
        let mut ta = TextArea::new();
        ta.insert_char('a');
        assert!(ta.text() == "a");
        assert!(ta.cursor.position.col == 1);
    }

    #[test]
    fn insert_at_middle_of_line() {
        let mut ta = TextArea::from_text("ac");
        ta.cursor.position.col = 1;
        ta.insert_char('b');
        assert!(ta.text() == "abc");
    }

    #[test]
    fn backspace_at_start_joins_lines() {
        let mut ta = TextArea::from_text("ab\ncd");
        ta.cursor.position = CursorPosition::new(1, 0);
        ta.delete_backward();
        assert!(ta.text() == "abcd");
        assert!(ta.cursor.position.line == 0);
        assert!(ta.cursor.position.col == 2);
    }

    #[test]
    fn delete_at_end_joins_lines() {
        let mut ta = TextArea::from_text("ab\ncd");
        ta.cursor.position = CursorPosition::new(0, 2);
        ta.delete_forward();
        assert!(ta.text() == "abcd");
    }

    #[test]
    fn undo_reverses_insert() {
        let mut ta = TextArea::new();
        ta.insert_char('x');
        assert!(ta.text() == "x");
        ta.undo();
        assert!(ta.text().is_empty());
    }

    #[test]
    fn redo_reapplies() {
        let mut ta = TextArea::new();
        ta.insert_char('x');
        ta.undo();
        ta.redo();
        assert!(ta.text() == "x");
    }

    #[test]
    fn selection_delete_removes_text() {
        let mut ta = TextArea::from_text("hello world");
        ta.cursor.selection = Some(Selection::new(
            CursorPosition::new(0, 5),
            CursorPosition::new(0, 11),
        ));
        ta.cursor.position = CursorPosition::new(0, 11);
        let deleted = ta.delete_selection();
        assert!(deleted);
        assert!(ta.text() == "hello");
    }

    #[test]
    fn enter_splits_line() {
        let mut ta = TextArea::from_text("helloworld");
        ta.cursor.position.col = 5;
        ta.new_line();
        assert!(ta.buffer.line_count() == 2);
        match ta.buffer.line(0) {
            Some(ref s) if s == "hello" => {}
            other => unreachable!("expected 'hello', got {other:?}"),
        }
    }

    #[test]
    fn ensure_cursor_visible_scrolls_down() {
        let mut ta = TextArea::from_text("a\nb\nc\nd\ne\nf");
        ta.cursor.position = CursorPosition::new(5, 0);
        ta.ensure_cursor_visible(3);
        assert!(ta.scroll_offset == 3);
    }

    #[test]
    fn ensure_cursor_visible_scrolls_up() {
        let mut ta = TextArea::from_text("a\nb\nc\nd\ne\nf");
        ta.scroll_offset = 4;
        ta.cursor.position = CursorPosition::new(1, 0);
        ta.ensure_cursor_visible(3);
        assert!(ta.scroll_offset == 1);
    }

    #[test]
    fn handle_event_char_input() {
        let mut ta = TextArea::new();
        let event = Event::Key(KeyEvent {
            code: KeyCode::Char('a'),
            modifiers: Modifiers::NONE,
        });
        let result = ta.handle_event(&event);
        assert!(result == EventResult::Consumed);
        assert!(ta.text() == "a");
    }

    #[test]
    fn handle_event_ctrl_z_undoes() {
        let mut ta = TextArea::new();
        ta.insert_char('x');
        let event = Event::Key(KeyEvent {
            code: KeyCode::Char('z'),
            modifiers: Modifiers::CTRL,
        });
        ta.handle_event(&event);
        assert!(ta.text().is_empty());
    }
}
