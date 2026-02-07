//! Compositor — resolves overlapping widget layers into a flat cell grid.
//!
//! The compositor collects styled segment output from each widget,
//! finds cut boundaries where widget edges meet, selects the topmost
//! visible widget for each region, and writes the result to a screen buffer.

pub mod chop;
pub mod compose;
pub mod cuts;
pub mod layer;
pub mod zorder;

pub use layer::{CompositorError, CompositorRegion, Layer};

use crate::buffer::ScreenBuffer;
use crate::cell::Cell;
use crate::geometry::{Rect, Size};
use crate::segment::Segment;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

/// The compositor collects widget layers and resolves overlapping regions.
pub struct Compositor {
    layers: Vec<Layer>,
    screen_width: u16,
    screen_height: u16,
}

impl Compositor {
    /// Creates a new compositor with the given screen dimensions.
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            layers: Vec::new(),
            screen_width: width,
            screen_height: height,
        }
    }

    /// Removes all layers from the compositor.
    pub fn clear(&mut self) {
        self.layers.clear();
    }

    /// Adds a layer to the compositor stack.
    pub fn add_layer(&mut self, layer: Layer) {
        self.layers.push(layer);
    }

    /// Convenience method that creates and adds a layer.
    ///
    /// Creates a new layer from the given parameters and adds it to the stack.
    pub fn add_widget(
        &mut self,
        widget_id: u64,
        region: Rect,
        z_index: i32,
        lines: Vec<Vec<Segment>>,
    ) {
        let layer = Layer::new(widget_id, region, z_index, lines);
        self.add_layer(layer);
    }

    /// Returns the number of layers in the compositor.
    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }

    /// Returns the screen size.
    pub fn screen_size(&self) -> Size {
        Size::new(self.screen_width, self.screen_height)
    }

    /// Returns a slice of all layers in the compositor.
    pub fn layers(&self) -> &[Layer] {
        &self.layers
    }

    /// Compose all layers and write the result to the screen buffer.
    ///
    /// Processes each row by calling `compose_line` to resolve overlapping
    /// layers, then writes the resulting segments as cells to the buffer.
    pub fn compose(&self, buf: &mut ScreenBuffer) {
        for row in 0..self.screen_height {
            let segments = compose::compose_line(&self.layers, row, self.screen_width);
            self.write_segments_to_buffer(buf, row, &segments);
        }
    }

    /// Write segments to a row of the screen buffer, converting each
    /// segment's graphemes to cells with proper style and width handling.
    fn write_segments_to_buffer(&self, buf: &mut ScreenBuffer, row: u16, segments: &[Segment]) {
        let mut x = 0;

        for segment in segments {
            // Skip control segments (they don't render)
            if segment.is_control {
                continue;
            }

            // Process each grapheme in the segment
            for grapheme in segment.text.graphemes(true) {
                if x >= self.screen_width {
                    return; // Reached end of screen width
                }

                let width = UnicodeWidthStr::width(grapheme);
                let cell = Cell::new(grapheme, segment.style.clone());
                buf.set(x, row, cell);
                x += width as u16;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Rect;
    use crate::segment::Segment;

    #[test]
    fn new_compositor_empty() {
        let compositor = Compositor::new(80, 24);
        assert!(compositor.layer_count() == 0);
    }

    #[test]
    fn add_layer_increases_count() {
        let mut compositor = Compositor::new(80, 24);
        let region = Rect::new(0, 0, 10, 5);
        let layer = Layer::new(1, region, 0, vec![]);

        compositor.add_layer(layer);
        assert!(compositor.layer_count() == 1);
    }

    #[test]
    fn add_multiple_layers() {
        let mut compositor = Compositor::new(80, 24);
        let region1 = Rect::new(0, 0, 10, 5);
        let region2 = Rect::new(10, 10, 20, 10);
        let region3 = Rect::new(30, 5, 15, 8);

        compositor.add_layer(Layer::new(1, region1, 0, vec![]));
        compositor.add_layer(Layer::new(2, region2, 1, vec![]));
        compositor.add_layer(Layer::new(3, region3, 2, vec![]));

        assert!(compositor.layer_count() == 3);
    }

    #[test]
    fn add_widget_convenience() {
        let mut compositor = Compositor::new(80, 24);
        let region = Rect::new(5, 10, 20, 15);
        let lines = vec![vec![Segment::new("test")]];

        compositor.add_widget(42, region, 5, lines);

        assert!(compositor.layer_count() == 1);
        let layer_slice = compositor.layers();
        assert!(layer_slice.len() == 1);
        let layer = match layer_slice.first() {
            Some(l) => l,
            None => unreachable!(),
        };
        assert!(layer.widget_id == 42);
        assert!(layer.z_index == 5);
        assert!(layer.region == region);
    }

    #[test]
    fn clear_removes_all() {
        let mut compositor = Compositor::new(80, 24);
        let region1 = Rect::new(0, 0, 10, 5);
        let region2 = Rect::new(10, 10, 20, 10);

        compositor.add_layer(Layer::new(1, region1, 0, vec![]));
        compositor.add_layer(Layer::new(2, region2, 1, vec![]));
        assert!(compositor.layer_count() == 2);

        compositor.clear();
        assert!(compositor.layer_count() == 0);
    }

    #[test]
    fn screen_size_accessible() {
        let compositor = Compositor::new(100, 50);
        let size = compositor.screen_size();
        assert!(size.width == 100);
        assert!(size.height == 50);
    }

    #[test]
    fn layers_accessible() {
        let mut compositor = Compositor::new(80, 24);
        let region1 = Rect::new(0, 0, 10, 5);
        let region2 = Rect::new(10, 10, 20, 10);

        compositor.add_layer(Layer::new(1, region1, 0, vec![]));
        compositor.add_layer(Layer::new(2, region2, 1, vec![]));

        let layers = compositor.layers();
        assert!(layers.len() == 2);
        assert!(layers[0].widget_id == 1);
        assert!(layers[1].widget_id == 2);
    }

    #[test]
    fn compose_single_layer_to_buffer() {
        use crate::geometry::Size;

        let mut compositor = Compositor::new(80, 10);
        let region = Rect::new(0, 0, 80, 10);
        let lines = vec![vec![Segment::new("Hello, World!")]];
        compositor.add_layer(Layer::new(1, region, 0, lines));

        let mut buf = ScreenBuffer::new(Size::new(80, 10));
        compositor.compose(&mut buf);

        // Check that the text appears in the buffer
        assert!(buf.get(0, 0).is_some());
        match buf.get(0, 0) {
            Some(cell) => {
                assert!(cell.grapheme == "H");
            }
            None => unreachable!(),
        }

        // Check second character
        match buf.get(1, 0) {
            Some(cell) => {
                assert!(cell.grapheme == "e");
            }
            None => unreachable!(),
        }
    }

    #[test]
    fn compose_overlapping_layers_to_buffer() {
        use crate::geometry::Size;

        let mut compositor = Compositor::new(80, 10);

        // Background layer (z=0)
        let bg_region = Rect::new(0, 0, 80, 10);
        let bg_lines = vec![vec![Segment::new("Background")]];
        compositor.add_layer(Layer::new(1, bg_region, 0, bg_lines));

        // Overlay layer (z=10) at position (5, 0)
        let overlay_region = Rect::new(5, 0, 20, 10);
        let overlay_lines = vec![vec![Segment::new("Overlay")]];
        compositor.add_layer(Layer::new(2, overlay_region, 10, overlay_lines));

        let mut buf = ScreenBuffer::new(Size::new(80, 10));
        compositor.compose(&mut buf);

        // Position 0 should have 'B' from Background
        match buf.get(0, 0) {
            Some(cell) => {
                assert!(cell.grapheme == "B");
            }
            None => unreachable!(),
        }

        // Position 5 should have 'O' from Overlay (topmost)
        match buf.get(5, 0) {
            Some(cell) => {
                assert!(cell.grapheme == "O");
            }
            None => unreachable!(),
        }
    }

    #[test]
    fn compose_correct_cell_styles() {
        use crate::color::{Color, NamedColor};
        use crate::geometry::Size;
        use crate::style::Style;

        let mut compositor = Compositor::new(80, 10);
        let style = Style {
            fg: Some(Color::Named(NamedColor::Red)),
            bold: true,
            ..Default::default()
        };

        let mut seg = Segment::new("Styled");
        seg.style = style.clone();

        let region = Rect::new(0, 0, 20, 10);
        let lines = vec![vec![seg]];
        compositor.add_layer(Layer::new(1, region, 0, lines));

        let mut buf = ScreenBuffer::new(Size::new(80, 10));
        compositor.compose(&mut buf);

        // Check that the style is preserved
        match buf.get(0, 0) {
            Some(cell) => {
                assert!(cell.style.bold);
                assert!(matches!(cell.style.fg, Some(Color::Named(NamedColor::Red))));
            }
            None => unreachable!(),
        }
    }

    #[test]
    fn compose_empty_compositor_all_blank() {
        use crate::geometry::Size;

        let compositor = Compositor::new(80, 10);
        let mut buf = ScreenBuffer::new(Size::new(80, 10));

        compositor.compose(&mut buf);

        // All cells should be blank (space)
        for y in 0..10 {
            for x in 0..80 {
                match buf.get(x, y) {
                    Some(cell) => {
                        assert!(cell.is_blank());
                    }
                    None => unreachable!(),
                }
            }
        }
    }

    #[test]
    fn compose_wide_characters() {
        use crate::geometry::Size;

        let mut compositor = Compositor::new(80, 10);
        let region = Rect::new(0, 0, 20, 10);
        // 世 is a CJK character with width 2
        let lines = vec![vec![Segment::new("\u{4e16}界")]]; // 世界
        compositor.add_layer(Layer::new(1, region, 0, lines));

        let mut buf = ScreenBuffer::new(Size::new(80, 10));
        compositor.compose(&mut buf);

        // First cell should have the wide character
        match buf.get(0, 0) {
            Some(cell) => {
                assert!(cell.grapheme == "\u{4e16}");
                assert!(cell.width == 2);
            }
            None => unreachable!(),
        }

        // Second cell should be continuation (width 0)
        match buf.get(1, 0) {
            Some(cell) => {
                assert!(cell.width == 0);
            }
            None => unreachable!(),
        }

        // Third cell should have the second character
        match buf.get(2, 0) {
            Some(cell) => {
                assert!(cell.grapheme == "界");
                assert!(cell.width == 2);
            }
            None => unreachable!(),
        }
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::color::{Color, NamedColor};
    use crate::geometry::{Rect, Size};
    use crate::segment::Segment;
    use crate::style::Style;

    #[test]
    fn integration_chat_layout() {
        let mut compositor = Compositor::new(80, 24);

        // Header at top (z=0)
        let header_region = Rect::new(0, 0, 80, 1);
        let header_lines = vec![vec![Segment::new("Chat App Header")]];
        compositor.add_layer(Layer::new(1, header_region, 0, header_lines));

        // Messages area (z=0)
        let messages_region = Rect::new(0, 1, 80, 20);
        let messages_lines = vec![vec![Segment::new("Message 1")]];
        compositor.add_layer(Layer::new(2, messages_region, 0, messages_lines));

        // Input bar at bottom (z=0)
        let input_region = Rect::new(0, 21, 80, 3);
        let input_lines = vec![vec![Segment::new("Type here...")]];
        compositor.add_layer(Layer::new(3, input_region, 0, input_lines));

        // Modal overlay (z=10) centered
        let modal_region = Rect::new(20, 8, 40, 8);
        let modal_lines = vec![vec![Segment::new("Modal Dialog")]];
        compositor.add_layer(Layer::new(4, modal_region, 10, modal_lines));

        let mut buf = ScreenBuffer::new(Size::new(80, 24));
        compositor.compose(&mut buf);

        // Header text should be visible at row 0
        match buf.get(0, 0) {
            Some(cell) => {
                assert!(cell.grapheme == "C");
            }
            None => unreachable!(),
        }

        // Modal should overlay messages at row 8, column 20
        match buf.get(20, 8) {
            Some(cell) => {
                assert!(cell.grapheme == "M");
            }
            None => unreachable!(),
        }

        // Input bar should be visible at row 21
        match buf.get(0, 21) {
            Some(cell) => {
                assert!(cell.grapheme == "T");
            }
            None => unreachable!(),
        }
    }

    #[test]
    fn integration_three_overlapping_windows() {
        let mut compositor = Compositor::new(80, 24);

        // Bottom window (z=0)
        let window1_region = Rect::new(0, 0, 40, 20);
        let window1_lines = vec![vec![Segment::new("Window 1")]];
        compositor.add_layer(Layer::new(1, window1_region, 0, window1_lines));

        // Middle window (z=5)
        let window2_region = Rect::new(20, 5, 40, 15);
        let window2_lines = vec![vec![Segment::new("Window 2")]];
        compositor.add_layer(Layer::new(2, window2_region, 5, window2_lines));

        // Top window (z=10)
        let window3_region = Rect::new(30, 10, 30, 10);
        let window3_lines = vec![vec![Segment::new("Window 3")]];
        compositor.add_layer(Layer::new(3, window3_region, 10, window3_lines));

        let mut buf = ScreenBuffer::new(Size::new(80, 24));
        compositor.compose(&mut buf);

        // Position (0, 0) should have Window 1 (only window here)
        match buf.get(0, 0) {
            Some(cell) => {
                assert!(cell.grapheme == "W");
            }
            None => unreachable!(),
        }

        // Position (20, 5) should have Window 2 (overlaps Window 1)
        match buf.get(20, 5) {
            Some(cell) => {
                assert!(cell.grapheme == "W");
            }
            None => unreachable!(),
        }

        // Position (30, 10) should have Window 3 (highest z-index)
        match buf.get(30, 10) {
            Some(cell) => {
                assert!(cell.grapheme == "W");
            }
            None => unreachable!(),
        }
    }

    #[test]
    fn integration_styled_segments_preserved() {
        let mut compositor = Compositor::new(80, 24);

        let red_style = Style {
            fg: Some(Color::Named(NamedColor::Red)),
            bold: true,
            ..Default::default()
        };

        let blue_style = Style {
            fg: Some(Color::Named(NamedColor::Blue)),
            italic: true,
            ..Default::default()
        };

        let mut red_seg = Segment::new("Red ");
        red_seg.style = red_style.clone();

        let mut blue_seg = Segment::new("Blue");
        blue_seg.style = blue_style.clone();

        let region = Rect::new(0, 0, 40, 10);
        let lines = vec![vec![red_seg, blue_seg]];
        compositor.add_layer(Layer::new(1, region, 0, lines));

        let mut buf = ScreenBuffer::new(Size::new(80, 24));
        compositor.compose(&mut buf);

        // First character should have red style
        match buf.get(0, 0) {
            Some(cell) => {
                assert!(cell.style.bold);
                assert!(matches!(cell.style.fg, Some(Color::Named(NamedColor::Red))));
            }
            None => unreachable!(),
        }

        // Fifth character (index 4) should have blue style
        match buf.get(4, 0) {
            Some(cell) => {
                assert!(cell.style.italic);
                assert!(matches!(
                    cell.style.fg,
                    Some(Color::Named(NamedColor::Blue))
                ));
            }
            None => unreachable!(),
        }
    }

    #[test]
    fn integration_resize_recompose() {
        // Compose at 80x24
        let mut compositor1 = Compositor::new(80, 24);
        let region1 = Rect::new(0, 0, 40, 10);
        let lines1 = vec![vec![Segment::new("Test")]];
        compositor1.add_layer(Layer::new(1, region1, 0, lines1));

        let mut buf1 = ScreenBuffer::new(Size::new(80, 24));
        compositor1.compose(&mut buf1);

        match buf1.get(0, 0) {
            Some(cell) => {
                assert!(cell.grapheme == "T");
            }
            None => unreachable!(),
        }

        // Now compose at 120x30
        let mut compositor2 = Compositor::new(120, 30);
        let region2 = Rect::new(0, 0, 60, 15);
        let lines2 = vec![vec![Segment::new("Resized")]];
        compositor2.add_layer(Layer::new(1, region2, 0, lines2));

        let mut buf2 = ScreenBuffer::new(Size::new(120, 30));
        compositor2.compose(&mut buf2);

        match buf2.get(0, 0) {
            Some(cell) => {
                assert!(cell.grapheme == "R");
            }
            None => unreachable!(),
        }

        // Both buffers should work correctly
        assert!(buf1.width() == 80);
        assert!(buf2.width() == 120);
    }
}
