//! Layer and compositor types for resolving overlapping widgets.

use crate::geometry::Rect;
use crate::segment::Segment;

/// A single widget layer in the compositor stack.
///
/// Contains the widget's rendered output (as lines of segments),
/// its on-screen bounding box, and its z-index for stacking order.
#[derive(Debug, Clone)]
pub struct Layer {
    /// The widget ID that owns this layer.
    pub widget_id: u64,
    /// Bounding box on screen.
    pub region: Rect,
    /// Stacking order (higher = on top).
    pub z_index: i32,
    /// Per-line styled segment output.
    pub lines: Vec<Vec<Segment>>,
}

impl Layer {
    /// Creates a new layer.
    pub fn new(widget_id: u64, region: Rect, z_index: i32, lines: Vec<Vec<Segment>>) -> Self {
        Self {
            widget_id,
            region,
            z_index,
            lines,
        }
    }

    /// Returns true if the given row falls within this layer's region.
    pub fn contains_row(&self, row: u16) -> bool {
        row >= self.region.position.y && row < self.region.position.y + self.region.size.height
    }

    /// Returns the segments for the given screen row, if the row is within this layer's region.
    ///
    /// Maps the screen row to a local line index and returns the corresponding segments.
    pub fn line_for_row(&self, row: u16) -> Option<&Vec<Segment>> {
        if !self.contains_row(row) {
            return None;
        }
        let local_idx = (row - self.region.position.y) as usize;
        self.lines.get(local_idx)
    }
}

/// A horizontal region on a single screen row, owned by a specific layer.
///
/// The compositor cuts each row into non-overlapping regions based on
/// layer boundaries, then selects the topmost visible layer for each region.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompositorRegion {
    /// Start column.
    pub x: u16,
    /// Width in columns.
    pub width: u16,
    /// Which layer owns this region (None = background).
    pub source_layer_idx: Option<usize>,
}

impl CompositorRegion {
    /// Creates a new compositor region.
    pub fn new(x: u16, width: u16, source_layer_idx: Option<usize>) -> Self {
        Self {
            x,
            width,
            source_layer_idx,
        }
    }
}

/// Errors that can occur during compositing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompositorError {
    /// Layer validation error.
    InvalidLayer(String),
    /// Compositor output doesn't fit in the buffer.
    BufferTooSmall,
}

impl std::fmt::Display for CompositorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompositorError::InvalidLayer(msg) => write!(f, "Invalid layer: {}", msg),
            CompositorError::BufferTooSmall => write!(f, "Compositor buffer too small"),
        }
    }
}

impl std::error::Error for CompositorError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layer_construction() {
        let region = Rect::new(10, 20, 30, 40);
        let lines = vec![vec![Segment::new("hello")], vec![Segment::new("world")]];
        let layer = Layer::new(123, region, 5, lines.clone());

        assert!(layer.widget_id == 123);
        assert!(layer.region == region);
        assert!(layer.z_index == 5);
        assert!(layer.lines.len() == 2);
    }

    #[test]
    fn layer_empty_lines() {
        let region = Rect::new(0, 0, 10, 5);
        let layer = Layer::new(1, region, 0, vec![]);

        assert!(layer.lines.is_empty());
    }

    #[test]
    fn layer_contains_row() {
        let region = Rect::new(0, 10, 20, 5); // y=10, height=5 -> rows 10..15
        let layer = Layer::new(1, region, 0, vec![]);

        assert!(layer.contains_row(10)); // start
        assert!(layer.contains_row(14)); // end-1
        assert!(!layer.contains_row(9)); // before
        assert!(!layer.contains_row(15)); // after
    }

    #[test]
    fn layer_line_for_row() {
        let region = Rect::new(0, 10, 20, 3);
        let lines = vec![
            vec![Segment::new("line0")],
            vec![Segment::new("line1")],
            vec![Segment::new("line2")],
        ];
        let layer = Layer::new(1, region, 0, lines);

        let result0 = layer.line_for_row(10);
        assert!(result0.is_some());
        let segs0 = match result0 {
            Some(s) => s,
            None => unreachable!(),
        };
        assert!(segs0.len() == 1);
        assert!(segs0[0].text == "line0");

        let result1 = layer.line_for_row(11);
        assert!(result1.is_some());
        let segs1 = match result1 {
            Some(s) => s,
            None => unreachable!(),
        };
        assert!(segs1[0].text == "line1");

        let result2 = layer.line_for_row(12);
        assert!(result2.is_some());
        let segs2 = match result2 {
            Some(s) => s,
            None => unreachable!(),
        };
        assert!(segs2[0].text == "line2");
    }

    #[test]
    fn layer_line_for_row_outside() {
        let region = Rect::new(0, 10, 20, 2);
        let lines = vec![vec![Segment::new("a")], vec![Segment::new("b")]];
        let layer = Layer::new(1, region, 0, lines);

        assert!(layer.line_for_row(9).is_none()); // before
        assert!(layer.line_for_row(12).is_none()); // after
    }

    #[test]
    fn region_construction() {
        let region = CompositorRegion::new(5, 10, Some(3));

        assert!(region.x == 5);
        assert!(region.width == 10);
        assert!(region.source_layer_idx == Some(3));
    }

    #[test]
    fn region_with_no_source() {
        let region = CompositorRegion::new(0, 20, None);

        assert!(region.x == 0);
        assert!(region.width == 20);
        assert!(region.source_layer_idx.is_none());
    }

    #[test]
    fn compositor_error_display() {
        let err1 = CompositorError::InvalidLayer("bad widget".to_string());
        let display1 = format!("{}", err1);
        assert!(display1.contains("Invalid layer"));
        assert!(display1.contains("bad widget"));

        let err2 = CompositorError::BufferTooSmall;
        let display2 = format!("{}", err2);
        assert!(display2.contains("Compositor buffer too small"));
    }
}
