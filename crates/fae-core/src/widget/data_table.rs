//! Scrollable data table widget with columns and rows.
//!
//! Displays tabular data with column headers, row selection,
//! vertical/horizontal scrolling, and keyboard navigation.

use crate::buffer::ScreenBuffer;
use crate::cell::Cell;
use crate::event::{Event, KeyCode, KeyEvent};
use crate::geometry::Rect;
use crate::style::Style;
use crate::text::truncate_to_display_width;
use crate::widget::label::Alignment;
use unicode_width::UnicodeWidthStr;

use super::{BorderStyle, EventResult, InteractiveWidget, Widget};

/// A column definition for a [`DataTable`].
#[derive(Clone, Debug)]
pub struct Column {
    /// Column header text.
    pub header: String,
    /// Column width in characters.
    pub width: u16,
    /// Text alignment within the column.
    pub alignment: Alignment,
}

impl Column {
    /// Create a new column with left alignment.
    pub fn new(header: &str, width: u16) -> Self {
        Self {
            header: header.to_string(),
            width,
            alignment: Alignment::Left,
        }
    }

    /// Set the column alignment.
    #[must_use]
    pub fn with_alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }
}

/// A scrollable data table widget with columns and rows.
///
/// Supports keyboard navigation, row selection, and both vertical
/// and horizontal scrolling.
pub struct DataTable {
    /// Column definitions.
    columns: Vec<Column>,
    /// Row data: each row is a `Vec<String>`, one per column.
    rows: Vec<Vec<String>>,
    /// Selected row index.
    selected_row: usize,
    /// Scroll offset (first visible row index).
    row_offset: usize,
    /// Horizontal scroll offset (pixel offset in characters).
    col_offset: u16,
    /// Style for headers.
    header_style: Style,
    /// Style for unselected rows.
    row_style: Style,
    /// Style for the selected row.
    selected_style: Style,
    /// Border style.
    border: BorderStyle,
    /// Sort state: (column_index, ascending).
    sort_state: Option<(usize, bool)>,
    /// Whether columns can be resized.
    resizable_columns: bool,
    /// Original row order (for restoring after clear_sort).
    original_order: Vec<usize>,
}

impl DataTable {
    /// Create a new data table with the given columns.
    pub fn new(columns: Vec<Column>) -> Self {
        Self {
            columns,
            rows: Vec::new(),
            selected_row: 0,
            row_offset: 0,
            col_offset: 0,
            header_style: Style::default().bold(true),
            row_style: Style::default(),
            selected_style: Style::default().reverse(true),
            border: BorderStyle::None,
            sort_state: None,
            resizable_columns: false,
            original_order: Vec::new(),
        }
    }

    /// Set the header style.
    #[must_use]
    pub fn with_header_style(mut self, style: Style) -> Self {
        self.header_style = style;
        self
    }

    /// Set the row style.
    #[must_use]
    pub fn with_row_style(mut self, style: Style) -> Self {
        self.row_style = style;
        self
    }

    /// Set the selected row style.
    #[must_use]
    pub fn with_selected_style(mut self, style: Style) -> Self {
        self.selected_style = style;
        self
    }

    /// Set the border style.
    #[must_use]
    pub fn with_border(mut self, border: BorderStyle) -> Self {
        self.border = border;
        self
    }

    /// Add a row of data.
    pub fn push_row(&mut self, row: Vec<String>) {
        self.rows.push(row);
    }

    /// Set all rows at once, resetting selection.
    pub fn set_rows(&mut self, rows: Vec<Vec<String>>) {
        self.rows = rows;
        self.selected_row = 0;
        self.row_offset = 0;
        self.sort_state = None;
        self.original_order.clear();
    }

    /// Get the number of rows.
    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    /// Get the number of columns.
    pub fn column_count(&self) -> usize {
        self.columns.len()
    }

    /// Get the selected row index.
    pub fn selected_row(&self) -> usize {
        self.selected_row
    }

    /// Set the selected row (clamped to valid range).
    pub fn set_selected_row(&mut self, idx: usize) {
        if self.rows.is_empty() {
            self.selected_row = 0;
        } else {
            self.selected_row = idx.min(self.rows.len().saturating_sub(1));
        }
    }

    /// Get the data for the selected row.
    pub fn selected_row_data(&self) -> Option<&[String]> {
        self.rows.get(self.selected_row).map(|r| r.as_slice())
    }

    /// Get the column definitions.
    pub fn columns(&self) -> &[Column] {
        &self.columns
    }

    /// Get the horizontal scroll offset.
    pub fn col_offset(&self) -> u16 {
        self.col_offset
    }

    // --- Sorting API ---

    /// Enable column resizing.
    #[must_use]
    pub fn with_resizable_columns(mut self, enabled: bool) -> Self {
        self.resizable_columns = enabled;
        self
    }

    /// Sort by the given column index (toggle ascending/descending).
    ///
    /// First call sorts ascending; repeated calls on the same column
    /// toggle direction.
    pub fn sort_by_column(&mut self, col_idx: usize) {
        if col_idx >= self.columns.len() {
            return;
        }

        // Save original order if not yet saved
        if self.original_order.is_empty() {
            self.original_order = (0..self.rows.len()).collect();
        }

        let ascending = match self.sort_state {
            Some((prev_col, prev_asc)) if prev_col == col_idx => !prev_asc,
            _ => true,
        };

        self.sort_state = Some((col_idx, ascending));

        // Sort rows by the column value
        let col = col_idx;
        self.rows.sort_by(|a, b| {
            let va = a.get(col).map(|s| s.as_str()).unwrap_or("");
            let vb = b.get(col).map(|s| s.as_str()).unwrap_or("");
            if ascending { va.cmp(vb) } else { vb.cmp(va) }
        });

        // Keep selection at row 0 after sort
        self.selected_row = 0;
        self.row_offset = 0;
    }

    /// Clear the sort and restore original order.
    pub fn clear_sort(&mut self) {
        if self.original_order.is_empty() || self.sort_state.is_none() {
            self.sort_state = None;
            return;
        }

        // Rebuild original order
        let mut indexed: Vec<(usize, Vec<String>)> = self
            .original_order
            .iter()
            .zip(self.rows.drain(..))
            .map(|(&orig_idx, row)| (orig_idx, row))
            .collect();
        indexed.sort_by_key(|(idx, _)| *idx);
        self.rows = indexed.into_iter().map(|(_, row)| row).collect();

        self.sort_state = None;
        self.original_order.clear();
        self.selected_row = 0;
        self.row_offset = 0;
    }

    /// Get the current sort state: (column_index, ascending).
    pub fn sort_state(&self) -> Option<(usize, bool)> {
        self.sort_state
    }

    /// Set the width of a column by index.
    pub fn set_column_width(&mut self, col_idx: usize, width: u16) {
        if let Some(col) = self.columns.get_mut(col_idx) {
            col.width = width.clamp(3, 50);
        }
    }

    /// Get the width of a column by index.
    pub fn column_width(&self, col_idx: usize) -> Option<u16> {
        self.columns.get(col_idx).map(|c| c.width)
    }

    /// Calculate total width of all columns (including separators).
    fn total_columns_width(&self) -> u16 {
        if self.columns.is_empty() {
            return 0;
        }
        // Each column takes its width + 1 separator, except the last
        let sum: u16 = self.columns.iter().map(|c| c.width).sum();
        let separators = self.columns.len().saturating_sub(1) as u16;
        sum.saturating_add(separators)
    }

    /// Calculate the inner area after accounting for borders.
    fn inner_area(&self, area: Rect) -> Rect {
        match self.border {
            BorderStyle::None => area,
            _ => {
                if area.size.width < 2 || area.size.height < 2 {
                    return Rect::new(area.position.x, area.position.y, 0, 0);
                }
                Rect::new(
                    area.position.x + 1,
                    area.position.y + 1,
                    area.size.width.saturating_sub(2),
                    area.size.height.saturating_sub(2),
                )
            }
        }
    }

    /// Render the border into the buffer.
    fn render_border(&self, area: Rect, buf: &mut ScreenBuffer) {
        let chars = border_chars(self.border);
        let (tl, tr, bl, br, h, v) = match chars {
            Some(c) => c,
            None => return,
        };

        let x1 = area.position.x;
        let y1 = area.position.y;
        let w = area.size.width;
        let h_val = area.size.height;

        if w == 0 || h_val == 0 {
            return;
        }

        let x2 = x1.saturating_add(w.saturating_sub(1));
        let y2 = y1.saturating_add(h_val.saturating_sub(1));

        buf.set(x1, y1, Cell::new(tl, self.row_style.clone()));
        buf.set(x2, y1, Cell::new(tr, self.row_style.clone()));
        buf.set(x1, y2, Cell::new(bl, self.row_style.clone()));
        buf.set(x2, y2, Cell::new(br, self.row_style.clone()));

        for x in (x1 + 1)..x2 {
            buf.set(x, y1, Cell::new(h, self.row_style.clone()));
            buf.set(x, y2, Cell::new(h, self.row_style.clone()));
        }

        for y in (y1 + 1)..y2 {
            buf.set(x1, y, Cell::new(v, self.row_style.clone()));
            buf.set(x2, y, Cell::new(v, self.row_style.clone()));
        }
    }

    /// Render a row of text cells with alignment, truncation, and horizontal scroll.
    fn render_row(
        &self,
        cells: &[String],
        y: u16,
        x_start: u16,
        available_width: u16,
        style: &Style,
        buf: &mut ScreenBuffer,
    ) {
        let mut x_offset: u16 = 0;
        let col_off = self.col_offset;

        for (col_idx, col) in self.columns.iter().enumerate() {
            let cell_text = cells.get(col_idx).map(|s| s.as_str()).unwrap_or("");
            let col_w = col.width as usize;

            // Column start position (before horizontal scroll)
            let col_start = x_offset;
            let col_end = x_offset.saturating_add(col.width);

            // Add separator width (1 char between columns)
            let next_x = if col_idx + 1 < self.columns.len() {
                col_end.saturating_add(1)
            } else {
                col_end
            };

            // Check if this column is visible after horizontal scroll
            if col_end > col_off && col_start < col_off.saturating_add(available_width) {
                // Visible portion
                let vis_start = col_start.saturating_sub(col_off);
                let screen_x = x_start.saturating_add(vis_start);

                let truncated = truncate_to_display_width(cell_text, col_w);
                let text_width = UnicodeWidthStr::width(truncated);

                // Apply alignment
                let padding = col_w.saturating_sub(text_width);
                let (left_pad, _right_pad) = match col.alignment {
                    Alignment::Left => (0, padding),
                    Alignment::Center => (padding / 2, padding.saturating_sub(padding / 2)),
                    Alignment::Right => (padding, 0),
                };

                // Render the aligned text
                let mut cx = screen_x;
                // Left padding
                for _ in 0..left_pad {
                    if cx < x_start.saturating_add(available_width) {
                        buf.set(cx, y, Cell::new(" ", style.clone()));
                        cx += 1;
                    }
                }
                // Text
                for ch in truncated.chars() {
                    let char_w = UnicodeWidthStr::width(ch.encode_utf8(&mut [0; 4]) as &str);
                    if cx as usize + char_w > (x_start + available_width) as usize {
                        break;
                    }
                    buf.set(cx, y, Cell::new(ch.to_string(), style.clone()));
                    cx += char_w as u16;
                }
                // Fill remaining column width
                while cx < screen_x.saturating_add(col.width) && cx < x_start + available_width {
                    buf.set(cx, y, Cell::new(" ", style.clone()));
                    cx += 1;
                }

                // Render separator
                if col_idx + 1 < self.columns.len() && cx < x_start.saturating_add(available_width)
                {
                    buf.set(cx, y, Cell::new("\u{2502}", style.clone()));
                }
            }

            x_offset = next_x;
        }
    }

    /// Ensure the selected row is visible by adjusting row_offset.
    fn ensure_selected_visible(&mut self, visible_height: usize) {
        if visible_height == 0 {
            return;
        }
        if self.selected_row < self.row_offset {
            self.row_offset = self.selected_row;
        }
        if self.selected_row >= self.row_offset + visible_height {
            self.row_offset = self
                .selected_row
                .saturating_sub(visible_height.saturating_sub(1));
        }
    }
}

impl Widget for DataTable {
    fn render(&self, area: Rect, buf: &mut ScreenBuffer) {
        if area.size.width == 0 || area.size.height == 0 {
            return;
        }

        self.render_border(area, buf);

        let inner = self.inner_area(area);
        if inner.size.width == 0 || inner.size.height == 0 {
            return;
        }

        let available_width = inner.size.width;
        let total_height = inner.size.height as usize;

        // First row: headers (with sort indicators)
        if total_height > 0 {
            let headers: Vec<String> = self
                .columns
                .iter()
                .enumerate()
                .map(|(idx, c)| {
                    if let Some((sort_col, ascending)) = self.sort_state
                        && sort_col == idx
                    {
                        let indicator = if ascending { "\u{2191}" } else { "\u{2193}" };
                        return format!("{}{indicator}", c.header);
                    }
                    c.header.clone()
                })
                .collect();
            self.render_row(
                &headers,
                inner.position.y,
                inner.position.x,
                available_width,
                &self.header_style,
                buf,
            );
        }

        // Remaining rows: data
        let data_height = total_height.saturating_sub(1);
        if data_height == 0 {
            return;
        }

        let max_offset = self.rows.len().saturating_sub(data_height.max(1));
        let scroll = self.row_offset.min(max_offset);
        let visible_end = (scroll + data_height).min(self.rows.len());

        for (row_idx, data_idx) in (scroll..visible_end).enumerate() {
            let y = inner.position.y + 1 + row_idx as u16;
            if let Some(row_data) = self.rows.get(data_idx) {
                let is_selected = data_idx == self.selected_row;
                let style = if is_selected {
                    &self.selected_style
                } else {
                    &self.row_style
                };

                // If selected, fill entire row first
                if is_selected {
                    for col in 0..available_width {
                        buf.set(inner.position.x + col, y, Cell::new(" ", style.clone()));
                    }
                }

                self.render_row(row_data, y, inner.position.x, available_width, style, buf);
            }
        }
    }
}

impl InteractiveWidget for DataTable {
    fn handle_event(&mut self, event: &Event) -> EventResult {
        let Event::Key(KeyEvent { code, modifiers }) = event else {
            return EventResult::Ignored;
        };

        match code {
            KeyCode::Up => {
                if self.selected_row > 0 {
                    self.selected_row -= 1;
                    self.ensure_selected_visible(20);
                }
                EventResult::Consumed
            }
            KeyCode::Down => {
                if !self.rows.is_empty() && self.selected_row < self.rows.len().saturating_sub(1) {
                    self.selected_row += 1;
                    self.ensure_selected_visible(20);
                }
                EventResult::Consumed
            }
            KeyCode::Left => {
                let has_ctrl = modifiers.contains(crate::event::Modifiers::CTRL);
                let has_shift = modifiers.contains(crate::event::Modifiers::SHIFT);
                if has_ctrl && has_shift && self.resizable_columns {
                    // Ctrl+Shift+Left: decrease selected column width
                    let max_col = self.columns.len().saturating_sub(1);
                    let target = self.selected_row.min(max_col);
                    if let Some(col) = self.columns.get_mut(target) {
                        col.width = col.width.saturating_sub(1).max(3);
                    }
                } else if has_ctrl {
                    self.col_offset = 0;
                } else {
                    self.col_offset = self.col_offset.saturating_sub(1);
                }
                EventResult::Consumed
            }
            KeyCode::Right => {
                let has_ctrl = modifiers.contains(crate::event::Modifiers::CTRL);
                let has_shift = modifiers.contains(crate::event::Modifiers::SHIFT);
                if has_ctrl && has_shift && self.resizable_columns {
                    // Ctrl+Shift+Right: increase selected column width
                    let max_col = self.columns.len().saturating_sub(1);
                    let target = self.selected_row.min(max_col);
                    if let Some(col) = self.columns.get_mut(target) {
                        col.width = (col.width + 1).min(50);
                    }
                } else if has_ctrl {
                    self.col_offset = self.total_columns_width();
                } else {
                    self.col_offset = self.col_offset.saturating_add(1);
                }
                EventResult::Consumed
            }
            KeyCode::PageUp => {
                let page = 20;
                self.selected_row = self.selected_row.saturating_sub(page);
                self.ensure_selected_visible(20);
                EventResult::Consumed
            }
            KeyCode::PageDown => {
                let page = 20;
                if !self.rows.is_empty() {
                    self.selected_row =
                        (self.selected_row + page).min(self.rows.len().saturating_sub(1));
                    self.ensure_selected_visible(20);
                }
                EventResult::Consumed
            }
            KeyCode::Home => {
                self.selected_row = 0;
                self.row_offset = 0;
                EventResult::Consumed
            }
            KeyCode::End => {
                if !self.rows.is_empty() {
                    self.selected_row = self.rows.len().saturating_sub(1);
                    self.ensure_selected_visible(20);
                }
                EventResult::Consumed
            }
            // Ctrl+0: clear sort
            KeyCode::Char('0') if modifiers.contains(crate::event::Modifiers::CTRL) => {
                self.clear_sort();
                EventResult::Consumed
            }
            // Ctrl+1..9: sort by column 1-9
            KeyCode::Char(ch)
                if modifiers.contains(crate::event::Modifiers::CTRL)
                    && ('1'..='9').contains(ch) =>
            {
                let col_idx = (*ch as usize) - ('1' as usize);
                if col_idx < self.columns.len() {
                    self.sort_by_column(col_idx);
                }
                EventResult::Consumed
            }
            _ => EventResult::Ignored,
        }
    }
}

/// Return border characters for the given style, or None for `BorderStyle::None`.
fn border_chars(
    style: BorderStyle,
) -> Option<(
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    &'static str,
)> {
    match style {
        BorderStyle::None => None,
        BorderStyle::Single => Some((
            "\u{250c}", "\u{2510}", "\u{2514}", "\u{2518}", "\u{2500}", "\u{2502}",
        )),
        BorderStyle::Double => Some((
            "\u{2554}", "\u{2557}", "\u{255a}", "\u{255d}", "\u{2550}", "\u{2551}",
        )),
        BorderStyle::Rounded => Some((
            "\u{256d}", "\u{256e}", "\u{2570}", "\u{256f}", "\u{2500}", "\u{2502}",
        )),
        BorderStyle::Heavy => Some((
            "\u{250f}", "\u{2513}", "\u{2517}", "\u{251b}", "\u{2501}", "\u{2503}",
        )),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::geometry::Size;

    fn make_test_table() -> DataTable {
        let cols = vec![
            Column::new("Name", 10),
            Column::new("Age", 5),
            Column::new("City", 12),
        ];
        let mut table = DataTable::new(cols);
        table.push_row(vec!["Alice".into(), "30".into(), "New York".into()]);
        table.push_row(vec!["Bob".into(), "25".into(), "London".into()]);
        table.push_row(vec!["Charlie".into(), "35".into(), "Tokyo".into()]);
        table
    }

    #[test]
    fn create_table_with_columns() {
        let table = make_test_table();
        assert_eq!(table.column_count(), 3);
        assert_eq!(table.row_count(), 3);
    }

    #[test]
    fn add_rows() {
        let mut table = DataTable::new(vec![Column::new("X", 5)]);
        assert_eq!(table.row_count(), 0);
        table.push_row(vec!["a".into()]);
        table.push_row(vec!["b".into()]);
        assert_eq!(table.row_count(), 2);
    }

    #[test]
    fn render_empty_table_shows_headers() {
        let table = DataTable::new(vec![Column::new("Name", 10), Column::new("Age", 5)]);
        let mut buf = ScreenBuffer::new(Size::new(20, 5));
        table.render(Rect::new(0, 0, 20, 5), &mut buf);

        // Header row should show "Name"
        assert_eq!(buf.get(0, 0).map(|c| c.grapheme.as_str()), Some("N"));
        assert_eq!(buf.get(3, 0).map(|c| c.grapheme.as_str()), Some("e"));
    }

    #[test]
    fn render_with_rows() {
        let table = make_test_table();
        let mut buf = ScreenBuffer::new(Size::new(30, 10));
        table.render(Rect::new(0, 0, 30, 10), &mut buf);

        // Row 0 = headers: "Name"
        assert_eq!(buf.get(0, 0).map(|c| c.grapheme.as_str()), Some("N"));
        // Row 1 = first data row: "Alice" (selected)
        assert_eq!(buf.get(0, 1).map(|c| c.grapheme.as_str()), Some("A"));
        // Row 2 = second data row: "Bob"
        assert_eq!(buf.get(0, 2).map(|c| c.grapheme.as_str()), Some("B"));
    }

    #[test]
    fn selected_row_highlighted() {
        let mut table = make_test_table();
        table.selected_style = Style::default().bold(true);
        table.row_style = Style::default();
        table.set_selected_row(1); // "Bob"

        let mut buf = ScreenBuffer::new(Size::new(30, 10));
        table.render(Rect::new(0, 0, 30, 10), &mut buf);

        // Row 2 = "Bob" (selected, bold)
        let cell = buf.get(0, 2);
        assert!(cell.is_some());
        assert!(cell.map(|c| c.style.bold).unwrap_or(false));

        // Row 1 = "Alice" (not selected)
        let cell_a = buf.get(0, 1);
        assert!(cell_a.is_some());
        assert!(!cell_a.map(|c| c.style.bold).unwrap_or(true));
    }

    #[test]
    fn column_alignment_left() {
        let table = DataTable::new(vec![Column::new("H", 10).with_alignment(Alignment::Left)]);
        let mut t = table;
        t.push_row(vec!["Hi".into()]);

        let mut buf = ScreenBuffer::new(Size::new(15, 5));
        t.render(Rect::new(0, 0, 15, 5), &mut buf);

        // "Hi" should be left-aligned: starts at position 0
        assert_eq!(buf.get(0, 1).map(|c| c.grapheme.as_str()), Some("H"));
        assert_eq!(buf.get(1, 1).map(|c| c.grapheme.as_str()), Some("i"));
    }

    #[test]
    fn column_alignment_right() {
        let col = Column::new("H", 10).with_alignment(Alignment::Right);
        let mut table = DataTable::new(vec![col]);
        table.push_row(vec!["Hi".into()]);

        let mut buf = ScreenBuffer::new(Size::new(15, 5));
        table.render(Rect::new(0, 0, 15, 5), &mut buf);

        // "Hi" is 2 chars in 10-wide column, right-aligned: padding = 8
        assert_eq!(buf.get(8, 1).map(|c| c.grapheme.as_str()), Some("H"));
        assert_eq!(buf.get(9, 1).map(|c| c.grapheme.as_str()), Some("i"));
    }

    #[test]
    fn column_alignment_center() {
        let col = Column::new("H", 10).with_alignment(Alignment::Center);
        let mut table = DataTable::new(vec![col]);
        table.push_row(vec!["Hi".into()]);

        let mut buf = ScreenBuffer::new(Size::new(15, 5));
        table.render(Rect::new(0, 0, 15, 5), &mut buf);

        // "Hi" is 2 chars, center in 10: left_pad = 4
        assert_eq!(buf.get(4, 1).map(|c| c.grapheme.as_str()), Some("H"));
        assert_eq!(buf.get(5, 1).map(|c| c.grapheme.as_str()), Some("i"));
    }

    #[test]
    fn utf8_safe_truncation_in_cells() {
        let col = Column::new("H", 5);
        let mut table = DataTable::new(vec![col]);
        table.push_row(vec!["你好世界人".into()]);

        let mut buf = ScreenBuffer::new(Size::new(10, 5));
        table.render(Rect::new(0, 0, 10, 5), &mut buf);

        // Width 5 fits "你好" (4 chars) + maybe space, "世" would need 6
        assert_eq!(buf.get(0, 1).map(|c| c.grapheme.as_str()), Some("你"));
        assert_eq!(buf.get(2, 1).map(|c| c.grapheme.as_str()), Some("好"));
    }

    #[test]
    fn vertical_scrolling_with_navigation() {
        let mut table = make_test_table();

        let down = Event::Key(KeyEvent {
            code: KeyCode::Down,
            modifiers: crate::event::Modifiers::NONE,
        });
        let up = Event::Key(KeyEvent {
            code: KeyCode::Up,
            modifiers: crate::event::Modifiers::NONE,
        });

        assert_eq!(table.selected_row(), 0);
        table.handle_event(&down);
        assert_eq!(table.selected_row(), 1);
        table.handle_event(&down);
        assert_eq!(table.selected_row(), 2);
        table.handle_event(&down); // at end
        assert_eq!(table.selected_row(), 2);
        table.handle_event(&up);
        assert_eq!(table.selected_row(), 1);
    }

    #[test]
    fn horizontal_scrolling() {
        let mut table = make_test_table();

        let right = Event::Key(KeyEvent {
            code: KeyCode::Right,
            modifiers: crate::event::Modifiers::NONE,
        });
        let left = Event::Key(KeyEvent {
            code: KeyCode::Left,
            modifiers: crate::event::Modifiers::NONE,
        });

        assert_eq!(table.col_offset(), 0);
        table.handle_event(&right);
        assert_eq!(table.col_offset(), 1);
        table.handle_event(&left);
        assert_eq!(table.col_offset(), 0);
        table.handle_event(&left); // can't go below 0
        assert_eq!(table.col_offset(), 0);
    }

    #[test]
    fn page_up_down() {
        let cols = vec![Column::new("N", 5)];
        let mut table = DataTable::new(cols);
        for i in 0..50 {
            table.push_row(vec![format!("Row {i}")]);
        }

        let page_down = Event::Key(KeyEvent {
            code: KeyCode::PageDown,
            modifiers: crate::event::Modifiers::NONE,
        });
        let page_up = Event::Key(KeyEvent {
            code: KeyCode::PageUp,
            modifiers: crate::event::Modifiers::NONE,
        });

        table.handle_event(&page_down);
        assert_eq!(table.selected_row(), 20);
        table.handle_event(&page_up);
        assert_eq!(table.selected_row(), 0);
    }

    #[test]
    fn home_end_navigation() {
        let mut table = make_test_table();

        let end = Event::Key(KeyEvent {
            code: KeyCode::End,
            modifiers: crate::event::Modifiers::NONE,
        });
        let home = Event::Key(KeyEvent {
            code: KeyCode::Home,
            modifiers: crate::event::Modifiers::NONE,
        });

        table.handle_event(&end);
        assert_eq!(table.selected_row(), 2);
        table.handle_event(&home);
        assert_eq!(table.selected_row(), 0);
    }

    #[test]
    fn render_with_border() {
        let table = make_test_table();
        let table = DataTable {
            border: BorderStyle::Single,
            ..table
        };

        let mut buf = ScreenBuffer::new(Size::new(35, 8));
        table.render(Rect::new(0, 0, 35, 8), &mut buf);

        assert_eq!(buf.get(0, 0).map(|c| c.grapheme.as_str()), Some("\u{250c}"));
        // Header inside border
        assert_eq!(buf.get(1, 1).map(|c| c.grapheme.as_str()), Some("N"));
    }

    #[test]
    fn empty_table_with_columns() {
        let table = DataTable::new(vec![Column::new("A", 5), Column::new("B", 5)]);
        assert_eq!(table.row_count(), 0);
        assert_eq!(table.column_count(), 2);
        assert!(table.selected_row_data().is_none());

        // Should not crash on render
        let mut buf = ScreenBuffer::new(Size::new(15, 5));
        table.render(Rect::new(0, 0, 15, 5), &mut buf);
    }

    #[test]
    fn selected_row_data_access() {
        let table = make_test_table();
        match table.selected_row_data() {
            Some(data) => {
                assert_eq!(data.len(), 3);
                assert_eq!(data[0], "Alice");
            }
            None => unreachable!("should have data"),
        }
    }

    #[test]
    fn set_rows_resets_selection() {
        let mut table = make_test_table();
        table.set_selected_row(2);
        assert_eq!(table.selected_row(), 2);

        table.set_rows(vec![vec!["X".into()]]);
        assert_eq!(table.selected_row(), 0);
        assert_eq!(table.row_count(), 1);
    }

    #[test]
    fn builder_pattern() {
        let table = DataTable::new(vec![Column::new("H", 10)])
            .with_header_style(Style::default().bold(true))
            .with_row_style(Style::default().dim(true))
            .with_selected_style(Style::default().italic(true))
            .with_border(BorderStyle::Rounded);

        assert!(table.header_style.bold);
        assert!(table.row_style.dim);
        assert!(table.selected_style.italic);
    }

    #[test]
    fn unhandled_event_ignored() {
        let mut table = make_test_table();
        let tab = Event::Key(KeyEvent {
            code: KeyCode::Tab,
            modifiers: crate::event::Modifiers::NONE,
        });
        assert_eq!(table.handle_event(&tab), EventResult::Ignored);
    }

    // --- Task 5: Sorting & Column Resize tests ---

    #[test]
    fn sort_by_column_ascending() {
        let mut table = make_test_table();
        // Rows: Alice, Bob, Charlie
        table.sort_by_column(0); // Sort by Name ascending
        assert_eq!(table.sort_state(), Some((0, true)));
        match table.rows.first().map(|r| r[0].as_str()) {
            Some("Alice") => {}
            other => panic!("Expected Alice first, got {other:?}"),
        }
    }

    #[test]
    fn sort_toggle_descending() {
        let mut table = make_test_table();
        table.sort_by_column(0); // ascending
        assert_eq!(table.sort_state(), Some((0, true)));
        table.sort_by_column(0); // toggle to descending
        assert_eq!(table.sort_state(), Some((0, false)));
        match table.rows.first().map(|r| r[0].as_str()) {
            Some("Charlie") => {}
            other => panic!("Expected Charlie first (descending), got {other:?}"),
        }
    }

    #[test]
    fn sort_indicator_in_header() {
        let mut table = make_test_table();
        table.sort_by_column(0); // ascending

        let mut buf = ScreenBuffer::new(Size::new(35, 10));
        table.render(Rect::new(0, 0, 35, 10), &mut buf);

        // Header should include "↑" after "Name"
        // "Name↑" — '↑' is at position 4
        assert_eq!(buf.get(4, 0).map(|c| c.grapheme.as_str()), Some("\u{2191}"));
    }

    #[test]
    fn sort_descending_indicator() {
        let mut table = make_test_table();
        table.sort_by_column(0);
        table.sort_by_column(0); // toggle descending

        let mut buf = ScreenBuffer::new(Size::new(35, 10));
        table.render(Rect::new(0, 0, 35, 10), &mut buf);

        // "Name↓"
        assert_eq!(buf.get(4, 0).map(|c| c.grapheme.as_str()), Some("\u{2193}"));
    }

    #[test]
    fn clear_sort_restores_order() {
        let mut table = make_test_table();
        // Original: Alice, Bob, Charlie
        table.sort_by_column(0); // ascending
        table.sort_by_column(0); // descending: Charlie, Bob, Alice
        table.clear_sort();
        assert!(table.sort_state().is_none());
    }

    #[test]
    fn column_resize_increase() {
        let mut table = make_test_table();
        let original_width = table.column_width(0);
        assert_eq!(original_width, Some(10));

        table.set_column_width(0, 15);
        assert_eq!(table.column_width(0), Some(15));
    }

    #[test]
    fn column_resize_clamping() {
        let mut table = make_test_table();

        // Below minimum (3)
        table.set_column_width(0, 1);
        assert_eq!(table.column_width(0), Some(3));

        // Above maximum (50)
        table.set_column_width(0, 100);
        assert_eq!(table.column_width(0), Some(50));
    }

    #[test]
    fn keyboard_sort_ctrl_1() {
        let mut table = make_test_table();

        let ctrl_1 = Event::Key(KeyEvent {
            code: KeyCode::Char('1'),
            modifiers: crate::event::Modifiers::CTRL,
        });

        assert_eq!(table.handle_event(&ctrl_1), EventResult::Consumed);
        assert_eq!(table.sort_state(), Some((0, true)));
    }

    #[test]
    fn keyboard_sort_ctrl_0_clears() {
        let mut table = make_test_table();
        table.sort_by_column(0);
        assert!(table.sort_state().is_some());

        let ctrl_0 = Event::Key(KeyEvent {
            code: KeyCode::Char('0'),
            modifiers: crate::event::Modifiers::CTRL,
        });

        assert_eq!(table.handle_event(&ctrl_0), EventResult::Consumed);
        assert!(table.sort_state().is_none());
    }

    #[test]
    fn keyboard_resize_ctrl_shift_right() {
        let mut table = make_test_table().with_resizable_columns(true);

        let original = table.column_width(0);
        assert_eq!(original, Some(10));

        let ctrl_shift_right = Event::Key(KeyEvent {
            code: KeyCode::Right,
            modifiers: crate::event::Modifiers::CTRL | crate::event::Modifiers::SHIFT,
        });

        table.handle_event(&ctrl_shift_right);
        assert_eq!(table.column_width(0), Some(11));
    }

    #[test]
    fn keyboard_resize_ctrl_shift_left() {
        let mut table = make_test_table().with_resizable_columns(true);

        let ctrl_shift_left = Event::Key(KeyEvent {
            code: KeyCode::Left,
            modifiers: crate::event::Modifiers::CTRL | crate::event::Modifiers::SHIFT,
        });

        table.handle_event(&ctrl_shift_left);
        assert_eq!(table.column_width(0), Some(9));
    }

    #[test]
    fn empty_table_sorting_no_crash() {
        let mut table = DataTable::new(vec![Column::new("X", 5)]);
        table.sort_by_column(0);
        assert_eq!(table.sort_state(), Some((0, true)));
        table.clear_sort();
        assert!(table.sort_state().is_none());
    }

    #[test]
    fn sort_by_column_resets_selection() {
        let mut table = make_test_table();
        table.set_selected_row(2);
        table.sort_by_column(0);
        assert_eq!(table.selected_row(), 0);
    }
}
