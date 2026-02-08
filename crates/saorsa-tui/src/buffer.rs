//! Screen buffer — a 2D grid of terminal cells.

use crate::cell::Cell;
use crate::geometry::Size;

/// A 2D grid of terminal cells representing one frame of terminal content.
#[derive(Clone, Debug)]
pub struct ScreenBuffer {
    cells: Vec<Cell>,
    width: u16,
    height: u16,
}

impl ScreenBuffer {
    /// Create a new screen buffer filled with blank cells.
    pub fn new(size: Size) -> Self {
        let len = usize::from(size.width) * usize::from(size.height);
        Self {
            cells: vec![Cell::blank(); len],
            width: size.width,
            height: size.height,
        }
    }

    /// Get the buffer dimensions.
    pub fn size(&self) -> Size {
        Size::new(self.width, self.height)
    }

    /// Get the buffer width.
    pub fn width(&self) -> u16 {
        self.width
    }

    /// Get the buffer height.
    pub fn height(&self) -> u16 {
        self.height
    }

    /// Clear the buffer, resetting all cells to blank.
    pub fn clear(&mut self) {
        for cell in &mut self.cells {
            *cell = Cell::blank();
        }
    }

    /// Resize the buffer. Contents are lost (filled with blanks).
    pub fn resize(&mut self, size: Size) {
        self.width = size.width;
        self.height = size.height;
        let len = usize::from(size.width) * usize::from(size.height);
        self.cells.clear();
        self.cells.resize(len, Cell::blank());
    }

    /// Get a reference to the cell at (x, y), or `None` if out of bounds.
    pub fn get(&self, x: u16, y: u16) -> Option<&Cell> {
        if x < self.width && y < self.height {
            let idx = self.index(x, y);
            self.cells.get(idx)
        } else {
            None
        }
    }

    /// Get a mutable reference to the cell at (x, y), or `None` if out of bounds.
    pub fn get_mut(&mut self, x: u16, y: u16) -> Option<&mut Cell> {
        if x < self.width && y < self.height {
            let idx = self.index(x, y);
            self.cells.get_mut(idx)
        } else {
            None
        }
    }

    /// Set a cell at (x, y). If the cell is wide (width > 1), the next
    /// cell is automatically set to a continuation cell. No-op if out of bounds.
    ///
    /// This method handles wide character edge cases:
    /// - If writing over a continuation cell, the preceding wide character's
    ///   primary cell is blanked to avoid leaving a half-rendered glyph.
    /// - If writing over a wide character's primary cell, the old continuation
    ///   cell at x+1 is blanked.
    /// - If a wide character would place its continuation cell beyond the last
    ///   column, the wide character is replaced with a single space instead.
    pub fn set(&mut self, x: u16, y: u16, cell: Cell) {
        if x >= self.width || y >= self.height {
            return;
        }

        let is_wide = cell.is_wide();

        // If this is a wide character at the last column (continuation would be out of bounds),
        // replace with a space instead of placing a half-visible wide character.
        if is_wide && x + 1 >= self.width {
            let idx = self.index(x, y);
            if let Some(c) = self.cells.get_mut(idx) {
                *c = Cell::blank();
            }
            return;
        }

        // If the cell we are about to overwrite is a continuation cell (width == 0),
        // blank the preceding cell that was the primary half of the wide character.
        let idx = self.index(x, y);
        if let Some(existing) = self.cells.get(idx)
            && existing.is_continuation()
            && x > 0
        {
            let prev_idx = self.index(x - 1, y);
            if let Some(prev) = self.cells.get_mut(prev_idx) {
                *prev = Cell::blank();
            }
        }

        // If the cell we are about to overwrite is a wide character (width > 1),
        // blank the old continuation cell at x+1.
        if let Some(existing) = self.cells.get(idx)
            && existing.is_wide()
        {
            let next_x = x + 1;
            if next_x < self.width {
                let next_idx = self.index(next_x, y);
                if let Some(cont) = self.cells.get_mut(next_idx) {
                    *cont = Cell::blank();
                }
            }
        }

        // Write the new cell
        if let Some(c) = self.cells.get_mut(idx) {
            *c = cell;
        }

        // Set continuation cell for wide characters
        if is_wide {
            let next_x = x + 1;
            if next_x < self.width {
                // If the continuation target is itself a wide character's primary cell,
                // blank that wide character's continuation cell too.
                let next_idx = self.index(next_x, y);
                if let Some(next_cell) = self.cells.get(next_idx)
                    && next_cell.is_wide()
                {
                    let after_next = next_x + 1;
                    if after_next < self.width {
                        let after_idx = self.index(after_next, y);
                        if let Some(after_cell) = self.cells.get_mut(after_idx) {
                            *after_cell = Cell::blank();
                        }
                    }
                }
                if let Some(c) = self.cells.get_mut(next_idx) {
                    *c = Cell::continuation();
                }
            }
        }
    }

    /// Get a row of cells as a slice.
    pub fn get_row(&self, y: u16) -> Option<&[Cell]> {
        if y < self.height {
            let start = self.index(0, y);
            let end = start + usize::from(self.width);
            Some(&self.cells[start..end])
        } else {
            None
        }
    }

    /// Compute the differences between this buffer and a previous buffer.
    /// Returns a list of cell changes needed to update the terminal.
    pub fn diff(&self, previous: &ScreenBuffer) -> Vec<CellChange> {
        // If sizes differ, emit all non-blank cells as changes (full redraw)
        if self.width != previous.width || self.height != previous.height {
            return self.full_diff();
        }

        let mut changes = Vec::new();
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = self.index(x, y);
                let current = &self.cells[idx];
                let prev = &previous.cells[idx];
                if current != prev {
                    changes.push(CellChange {
                        x,
                        y,
                        cell: current.clone(),
                    });
                }
            }
        }
        changes
    }

    /// Generate changes for every cell (used when sizes differ).
    fn full_diff(&self) -> Vec<CellChange> {
        let mut changes = Vec::new();
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = self.index(x, y);
                let cell = &self.cells[idx];
                changes.push(CellChange {
                    x,
                    y,
                    cell: cell.clone(),
                });
            }
        }
        changes
    }

    /// Convert (x, y) to a linear index.
    fn index(&self, x: u16, y: u16) -> usize {
        usize::from(y) * usize::from(self.width) + usize::from(x)
    }
}

/// A single cell change: position + new cell value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CellChange {
    /// Column position.
    pub x: u16,
    /// Row position.
    pub y: u16,
    /// New cell value.
    pub cell: Cell,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::{Color, NamedColor};
    use crate::style::Style;

    #[test]
    fn new_buffer_all_blank() {
        let buf = ScreenBuffer::new(Size::new(10, 5));
        assert_eq!(buf.width(), 10);
        assert_eq!(buf.height(), 5);
        for y in 0..5 {
            for x in 0..10 {
                let cell = buf.get(x, y);
                assert!(cell.is_some());
                assert!(cell.is_some_and(|c| c.is_blank()));
            }
        }
    }

    #[test]
    fn set_and_get() {
        let mut buf = ScreenBuffer::new(Size::new(10, 5));
        let style = Style::new().fg(Color::Named(NamedColor::Red));
        let cell = Cell::new("A", style.clone());
        buf.set(3, 2, cell.clone());
        let got = buf.get(3, 2);
        assert_eq!(got, Some(&cell));
    }

    #[test]
    fn wide_char_sets_continuation() {
        let mut buf = ScreenBuffer::new(Size::new(10, 5));
        let wide = Cell::new("\u{4e16}", Style::default()); // 世 (width 2)
        buf.set(3, 1, wide.clone());
        assert_eq!(buf.get(3, 1), Some(&wide));
        // Next cell should be continuation
        let cont = buf.get(4, 1);
        assert!(cont.is_some());
        assert_eq!(cont.map(|c| c.width), Some(0));
    }

    #[test]
    fn wide_char_at_right_edge() {
        let mut buf = ScreenBuffer::new(Size::new(5, 1));
        let wide = Cell::new("\u{4e16}", Style::default());
        // Set at column 4 (last column) — continuation would be at col 5, out of bounds
        // Wide char should be replaced with a blank space
        buf.set(4, 0, wide);
        let cell = buf.get(4, 0);
        assert!(cell.is_some());
        match cell {
            Some(c) => {
                assert!(c.is_blank(), "Wide char at last column should become blank");
            }
            None => unreachable!(),
        }
    }

    #[test]
    fn out_of_bounds_returns_none() {
        let buf = ScreenBuffer::new(Size::new(5, 3));
        assert!(buf.get(5, 0).is_none());
        assert!(buf.get(0, 3).is_none());
        assert!(buf.get(100, 100).is_none());
    }

    #[test]
    fn out_of_bounds_set_is_noop() {
        let mut buf = ScreenBuffer::new(Size::new(5, 3));
        buf.set(10, 10, Cell::new("X", Style::default()));
        // Should not crash
    }

    #[test]
    fn get_row() {
        let buf = ScreenBuffer::new(Size::new(5, 3));
        let row = buf.get_row(0);
        assert!(row.is_some());
        assert_eq!(row.map(|r| r.len()), Some(5));
        assert!(buf.get_row(3).is_none());
    }

    #[test]
    fn clear_resets_all_cells() {
        let mut buf = ScreenBuffer::new(Size::new(5, 3));
        buf.set(2, 1, Cell::new("X", Style::new().bold(true)));
        buf.clear();
        for y in 0..3 {
            for x in 0..5 {
                assert!(buf.get(x, y).is_some_and(|c| c.is_blank()));
            }
        }
    }

    #[test]
    fn resize_fills_with_blank() {
        let mut buf = ScreenBuffer::new(Size::new(5, 3));
        buf.set(2, 1, Cell::new("X", Style::default()));
        buf.resize(Size::new(10, 8));
        assert_eq!(buf.width(), 10);
        assert_eq!(buf.height(), 8);
        for y in 0..8 {
            for x in 0..10 {
                assert!(buf.get(x, y).is_some_and(|c| c.is_blank()));
            }
        }
    }

    #[test]
    fn diff_no_changes() {
        let buf1 = ScreenBuffer::new(Size::new(5, 3));
        let buf2 = ScreenBuffer::new(Size::new(5, 3));
        let changes = buf1.diff(&buf2);
        assert!(changes.is_empty());
    }

    #[test]
    fn diff_single_change() {
        let mut current = ScreenBuffer::new(Size::new(5, 3));
        let previous = ScreenBuffer::new(Size::new(5, 3));
        current.set(2, 1, Cell::new("A", Style::default()));
        let changes = current.diff(&previous);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].x, 2);
        assert_eq!(changes[0].y, 1);
        assert_eq!(changes[0].cell.grapheme, "A");
    }

    #[test]
    fn diff_style_change() {
        let mut current = ScreenBuffer::new(Size::new(5, 3));
        let mut previous = ScreenBuffer::new(Size::new(5, 3));
        previous.set(0, 0, Cell::new("A", Style::default()));
        current.set(0, 0, Cell::new("A", Style::new().bold(true)));
        let changes = current.diff(&previous);
        assert_eq!(changes.len(), 1);
    }

    #[test]
    fn diff_size_mismatch_full_redraw() {
        let current = ScreenBuffer::new(Size::new(5, 3));
        let previous = ScreenBuffer::new(Size::new(10, 8));
        let changes = current.diff(&previous);
        // Full redraw = all cells
        assert_eq!(changes.len(), 15); // 5 * 3
    }

    #[test]
    fn diff_wide_char_change() {
        let mut current = ScreenBuffer::new(Size::new(10, 1));
        let previous = ScreenBuffer::new(Size::new(10, 1));
        current.set(3, 0, Cell::new("\u{4e16}", Style::default())); // 世
        let changes = current.diff(&previous);
        // Should have 2 changes: the wide char and the continuation
        assert_eq!(changes.len(), 2);
    }

    // --- Wide character protection tests ---

    #[test]
    fn overwrite_continuation_blanks_preceding_wide() {
        let mut buf = ScreenBuffer::new(Size::new(10, 1));
        // Place wide char at column 3 (continuation at 4)
        buf.set(3, 0, Cell::new("\u{4e16}", Style::default()));
        // Overwrite the continuation cell at column 4 with a narrow char
        buf.set(4, 0, Cell::new("X", Style::default()));
        // The preceding wide char at column 3 should now be blank
        match buf.get(3, 0) {
            Some(c) => assert!(c.is_blank(), "Preceding wide char should be blanked"),
            None => unreachable!(),
        }
        // Column 4 should have "X"
        match buf.get(4, 0) {
            Some(c) => assert_eq!(c.grapheme, "X"),
            None => unreachable!(),
        }
    }

    #[test]
    fn overwrite_wide_with_narrow_blanks_continuation() {
        let mut buf = ScreenBuffer::new(Size::new(10, 1));
        // Place wide char at column 3 (continuation at 4)
        buf.set(3, 0, Cell::new("\u{4e16}", Style::default()));
        // Overwrite the wide char primary cell with a narrow char
        buf.set(3, 0, Cell::new("A", Style::default()));
        // Column 3 should have "A"
        match buf.get(3, 0) {
            Some(c) => assert_eq!(c.grapheme, "A"),
            None => unreachable!(),
        }
        // Old continuation at column 4 should now be blank
        match buf.get(4, 0) {
            Some(c) => assert!(c.is_blank(), "Old continuation should be blanked"),
            None => unreachable!(),
        }
    }

    #[test]
    fn wide_char_last_column_replaced_with_space() {
        let mut buf = ScreenBuffer::new(Size::new(10, 1));
        // Column 9 is the last column (width=10)
        buf.set(9, 0, Cell::new("\u{4e16}", Style::default()));
        match buf.get(9, 0) {
            Some(c) => {
                assert!(c.is_blank(), "Wide char at last column should become space");
            }
            None => unreachable!(),
        }
    }

    #[test]
    fn wide_char_second_to_last_fits() {
        let mut buf = ScreenBuffer::new(Size::new(10, 1));
        // Column 8, continuation at 9 — fits exactly
        let wide = Cell::new("\u{4e16}", Style::default());
        buf.set(8, 0, wide.clone());
        match buf.get(8, 0) {
            Some(c) => {
                assert_eq!(c.grapheme, "\u{4e16}");
                assert_eq!(c.width, 2);
            }
            None => unreachable!(),
        }
        match buf.get(9, 0) {
            Some(c) => assert!(c.is_continuation()),
            None => unreachable!(),
        }
    }

    #[test]
    fn set_narrow_over_narrow_no_side_effects() {
        let mut buf = ScreenBuffer::new(Size::new(10, 1));
        buf.set(3, 0, Cell::new("A", Style::default()));
        buf.set(3, 0, Cell::new("B", Style::default()));
        match buf.get(3, 0) {
            Some(c) => assert_eq!(c.grapheme, "B"),
            None => unreachable!(),
        }
        // Neighbors should be unaffected (blank)
        match buf.get(2, 0) {
            Some(c) => assert!(c.is_blank()),
            None => unreachable!(),
        }
        match buf.get(4, 0) {
            Some(c) => assert!(c.is_blank()),
            None => unreachable!(),
        }
    }

    #[test]
    fn set_wide_over_wide_old_continuation_cleaned() {
        let mut buf = ScreenBuffer::new(Size::new(10, 1));
        // Place first wide char at column 2 (continuation at 3)
        buf.set(2, 0, Cell::new("\u{4e16}", Style::default()));
        // Place second wide char at column 2 (new continuation at 3)
        buf.set(2, 0, Cell::new("\u{754c}", Style::default()));
        match buf.get(2, 0) {
            Some(c) => {
                assert_eq!(c.grapheme, "\u{754c}");
                assert_eq!(c.width, 2);
            }
            None => unreachable!(),
        }
        match buf.get(3, 0) {
            Some(c) => assert!(c.is_continuation()),
            None => unreachable!(),
        }
    }

    #[test]
    fn multiple_wide_chars_in_sequence() {
        let mut buf = ScreenBuffer::new(Size::new(10, 1));
        // Place three wide chars: 0-1, 2-3, 4-5
        buf.set(0, 0, Cell::new("\u{4e16}", Style::default())); // 世
        buf.set(2, 0, Cell::new("\u{754c}", Style::default())); // 界
        buf.set(4, 0, Cell::new("\u{4eba}", Style::default())); // 人

        for col in [0, 2, 4] {
            match buf.get(col, 0) {
                Some(c) => assert_eq!(c.width, 2),
                None => unreachable!(),
            }
        }
        for col in [1, 3, 5] {
            match buf.get(col, 0) {
                Some(c) => assert!(c.is_continuation()),
                None => unreachable!(),
            }
        }
    }

    #[test]
    fn overwrite_middle_of_adjacent_wide_chars() {
        let mut buf = ScreenBuffer::new(Size::new(10, 1));
        // Place wide chars at 0-1 and 2-3
        buf.set(0, 0, Cell::new("\u{4e16}", Style::default()));
        buf.set(2, 0, Cell::new("\u{754c}", Style::default()));
        // Overwrite column 1 (continuation of first wide) with narrow char
        buf.set(1, 0, Cell::new("X", Style::default()));
        // First wide char at 0 should be blanked
        match buf.get(0, 0) {
            Some(c) => assert!(c.is_blank(), "First wide char should be blanked"),
            None => unreachable!(),
        }
        // Column 1 should have "X"
        match buf.get(1, 0) {
            Some(c) => assert_eq!(c.grapheme, "X"),
            None => unreachable!(),
        }
        // Second wide char at 2 should be unaffected
        match buf.get(2, 0) {
            Some(c) => {
                assert_eq!(c.grapheme, "\u{754c}");
                assert_eq!(c.width, 2);
            }
            None => unreachable!(),
        }
    }

    #[test]
    fn wide_char_at_column_zero() {
        let mut buf = ScreenBuffer::new(Size::new(10, 1));
        buf.set(0, 0, Cell::new("\u{4e16}", Style::default()));
        match buf.get(0, 0) {
            Some(c) => {
                assert_eq!(c.grapheme, "\u{4e16}");
                assert_eq!(c.width, 2);
            }
            None => unreachable!(),
        }
        match buf.get(1, 0) {
            Some(c) => assert!(c.is_continuation()),
            None => unreachable!(),
        }
    }

    #[test]
    fn wide_char_continuation_exactly_at_last_column() {
        // Buffer width 6: wide char at column 4, continuation at column 5 (last column) — fits
        let mut buf = ScreenBuffer::new(Size::new(6, 1));
        buf.set(4, 0, Cell::new("\u{4e16}", Style::default()));
        match buf.get(4, 0) {
            Some(c) => {
                assert_eq!(c.grapheme, "\u{4e16}");
                assert_eq!(c.width, 2);
            }
            None => unreachable!(),
        }
        match buf.get(5, 0) {
            Some(c) => assert!(c.is_continuation()),
            None => unreachable!(),
        }
    }

    // --- Task 6: Unicode buffer reading tests ---

    #[test]
    fn get_row_with_cjk_primary_and_continuation() {
        // Write 3 CJK chars: each width 2 => 6 cells total (3 primary + 3 continuation)
        let mut buf = ScreenBuffer::new(Size::new(10, 1));
        buf.set(0, 0, Cell::new("\u{4e16}", Style::default())); // 世
        buf.set(2, 0, Cell::new("\u{754c}", Style::default())); // 界
        buf.set(4, 0, Cell::new("\u{4eba}", Style::default())); // 人

        let row = buf.get_row(0);
        assert!(row.is_some());
        match row {
            Some(cells) => {
                assert_eq!(cells.len(), 10);
                // Primary cells at 0, 2, 4
                assert_eq!(cells[0].grapheme, "\u{4e16}");
                assert_eq!(cells[0].width, 2);
                assert_eq!(cells[2].grapheme, "\u{754c}");
                assert_eq!(cells[2].width, 2);
                assert_eq!(cells[4].grapheme, "\u{4eba}");
                assert_eq!(cells[4].width, 2);
                // Continuation cells at 1, 3, 5
                assert!(cells[1].is_continuation());
                assert!(cells[3].is_continuation());
                assert!(cells[5].is_continuation());
                // Remaining cells are blank
                assert!(cells[6].is_blank());
                assert!(cells[7].is_blank());
            }
            None => unreachable!(),
        }
    }

    #[test]
    fn diff_with_wide_char_produces_two_change_entries() {
        let mut current = ScreenBuffer::new(Size::new(10, 1));
        let previous = ScreenBuffer::new(Size::new(10, 1));
        // Write two CJK chars at columns 0 and 4
        current.set(0, 0, Cell::new("\u{4e16}", Style::default()));
        current.set(4, 0, Cell::new("\u{754c}", Style::default()));
        let changes = current.diff(&previous);
        // Each wide char produces 2 changes (primary + continuation)
        assert_eq!(changes.len(), 4);
        // First wide char: change at x=0 and x=1
        assert_eq!(changes[0].x, 0);
        assert_eq!(changes[0].cell.width, 2);
        assert_eq!(changes[1].x, 1);
        assert_eq!(changes[1].cell.width, 0); // continuation
        // Second wide char: change at x=4 and x=5
        assert_eq!(changes[2].x, 4);
        assert_eq!(changes[2].cell.width, 2);
        assert_eq!(changes[3].x, 5);
        assert_eq!(changes[3].cell.width, 0); // continuation
    }

    #[test]
    fn clear_after_wide_char_writes_all_blank() {
        let mut buf = ScreenBuffer::new(Size::new(10, 2));
        // Write wide chars
        buf.set(0, 0, Cell::new("\u{4e16}", Style::default()));
        buf.set(2, 0, Cell::new("\u{754c}", Style::default()));
        buf.set(0, 1, Cell::new("\u{1f600}", Style::default())); // emoji
        // Verify something is there
        match buf.get(0, 0) {
            Some(c) => assert!(!c.is_blank()),
            None => unreachable!(),
        }
        // Clear
        buf.clear();
        // All cells should be blank
        for y in 0..2 {
            for x in 0..10 {
                match buf.get(x, y) {
                    Some(c) => assert!(c.is_blank(), "Cell ({x},{y}) should be blank after clear"),
                    None => unreachable!(),
                }
            }
        }
    }
}
