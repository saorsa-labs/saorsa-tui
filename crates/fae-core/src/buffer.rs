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
    pub fn set(&mut self, x: u16, y: u16, cell: Cell) {
        if x >= self.width || y >= self.height {
            return;
        }
        let is_wide = cell.is_wide();
        let idx = self.index(x, y);
        if let Some(c) = self.cells.get_mut(idx) {
            *c = cell;
        }
        // Set continuation cell for wide characters
        if is_wide {
            let next_x = x + 1;
            if next_x < self.width {
                let next_idx = self.index(next_x, y);
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
        buf.set(4, 0, wide.clone());
        assert_eq!(buf.get(4, 0), Some(&wide));
        // No crash, no out-of-bounds
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
}
