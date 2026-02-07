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

    /// Resizes the compositor screen dimensions.
    ///
    /// Clears all layers since they may no longer be valid for the new size.
    pub fn resize(&mut self, width: u16, height: u16) {
        self.screen_width = width;
        self.screen_height = height;
        self.layers.clear();
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

#[cfg(test)]
mod advanced_integration_tests {
    use super::*;
    use crate::color::{Color, NamedColor};
    use crate::geometry::{Rect, Size};
    use crate::segment::Segment;
    use crate::style::Style;

    /// Task 7, Test 1: Syntax-highlighted code with multiple styled segments.
    #[test]
    fn syntax_highlighted_code() {
        let mut compositor = Compositor::new(40, 5);

        let keyword_style = Style::new().fg(Color::Named(NamedColor::Blue)).bold(true);
        let ident_style = Style::new().fg(Color::Named(NamedColor::White));
        let paren_style = Style::new().fg(Color::Named(NamedColor::Yellow));

        let segments = vec![
            Segment::styled("fn", keyword_style.clone()),
            Segment::styled(" ", Style::default()),
            Segment::styled("main", ident_style.clone()),
            Segment::styled("()", paren_style.clone()),
        ];

        let region = Rect::new(0, 0, 40, 5);
        compositor.add_layer(Layer::new(1, region, 0, vec![segments]));

        let mut buf = ScreenBuffer::new(Size::new(40, 5));
        compositor.compose(&mut buf);

        // "fn" at positions 0-1 should have blue, bold
        match buf.get(0, 0) {
            Some(cell) => {
                assert!(cell.grapheme == "f");
                assert!(cell.style.bold);
                assert!(matches!(
                    cell.style.fg,
                    Some(Color::Named(NamedColor::Blue))
                ));
            }
            None => unreachable!(),
        }
        match buf.get(1, 0) {
            Some(cell) => {
                assert!(cell.grapheme == "n");
                assert!(cell.style.bold);
            }
            None => unreachable!(),
        }

        // Space at position 2 should have default style
        match buf.get(2, 0) {
            Some(cell) => {
                assert!(cell.grapheme == " ");
            }
            None => unreachable!(),
        }

        // "main" at positions 3-6 should have white fg
        match buf.get(3, 0) {
            Some(cell) => {
                assert!(cell.grapheme == "m");
                assert!(matches!(
                    cell.style.fg,
                    Some(Color::Named(NamedColor::White))
                ));
            }
            None => unreachable!(),
        }

        // "()" at positions 7-8 should have yellow fg
        match buf.get(7, 0) {
            Some(cell) => {
                assert!(cell.grapheme == "(");
                assert!(matches!(
                    cell.style.fg,
                    Some(Color::Named(NamedColor::Yellow))
                ));
            }
            None => unreachable!(),
        }
    }

    /// Task 7, Test 2: Overlapping styled windows with different background colors.
    #[test]
    fn overlapping_styled_windows() {
        let mut compositor = Compositor::new(20, 5);

        // Bottom window (z=0) with green bg, fills region 0,0 -> 20,5
        let green_bg = Style::new().bg(Color::Named(NamedColor::Green));
        let green_line = vec![Segment::styled("GGGGGGGGGGGGGGGGGGG", green_bg.clone())];
        let bottom_region = Rect::new(0, 0, 20, 5);
        let bottom_lines = vec![green_line.clone(); 5];
        compositor.add_layer(Layer::new(1, bottom_region, 0, bottom_lines));

        // Top window (z=5) with blue bg, covers region 5,1 -> 10,3
        let blue_bg = Style::new().bg(Color::Named(NamedColor::Blue));
        let blue_line = vec![Segment::styled("BBBBB", blue_bg.clone())];
        let top_region = Rect::new(5, 1, 10, 3);
        let top_lines = vec![blue_line.clone(); 3];
        compositor.add_layer(Layer::new(2, top_region, 5, top_lines));

        let mut buf = ScreenBuffer::new(Size::new(20, 5));
        compositor.compose(&mut buf);

        // Row 0, col 5: only bottom window here -> green bg
        match buf.get(5, 0) {
            Some(cell) => {
                assert!(matches!(
                    cell.style.bg,
                    Some(Color::Named(NamedColor::Green))
                ));
            }
            None => unreachable!(),
        }

        // Row 1, col 5: top window starts here -> blue bg
        match buf.get(5, 1) {
            Some(cell) => {
                assert!(matches!(
                    cell.style.bg,
                    Some(Color::Named(NamedColor::Blue))
                ));
            }
            None => unreachable!(),
        }

        // Row 1, col 0: still bottom window region -> green bg
        match buf.get(0, 1) {
            Some(cell) => {
                assert!(matches!(
                    cell.style.bg,
                    Some(Color::Named(NamedColor::Green))
                ));
            }
            None => unreachable!(),
        }
    }

    /// Task 7, Test 3: Full-width CJK text in compositor.
    #[test]
    fn cjk_text_in_compositor() {
        let mut compositor = Compositor::new(20, 3);
        let region = Rect::new(0, 0, 20, 3);
        // "世界" = two CJK chars, each width 2, total width 4
        let lines = vec![vec![Segment::new("\u{4e16}\u{754c}")]];
        compositor.add_layer(Layer::new(1, region, 0, lines));

        let mut buf = ScreenBuffer::new(Size::new(20, 3));
        compositor.compose(&mut buf);

        // First CJK char at column 0, width 2
        match buf.get(0, 0) {
            Some(cell) => {
                assert!(cell.grapheme == "\u{4e16}");
                assert!(cell.width == 2);
            }
            None => unreachable!(),
        }

        // Continuation cell at column 1
        match buf.get(1, 0) {
            Some(cell) => {
                assert!(cell.width == 0);
            }
            None => unreachable!(),
        }

        // Second CJK char at column 2
        match buf.get(2, 0) {
            Some(cell) => {
                assert!(cell.grapheme == "\u{754c}");
                assert!(cell.width == 2);
            }
            None => unreachable!(),
        }

        // Continuation cell at column 3
        match buf.get(3, 0) {
            Some(cell) => {
                assert!(cell.width == 0);
            }
            None => unreachable!(),
        }
    }

    /// Task 7, Test 4: Multiple rows in a layer.
    #[test]
    fn multiple_rows_in_layer() {
        let mut compositor = Compositor::new(40, 10);
        let region = Rect::new(0, 0, 40, 10);
        let lines = vec![
            vec![Segment::new("Row Zero")],
            vec![Segment::new("Row One")],
            vec![Segment::new("Row Two")],
        ];
        compositor.add_layer(Layer::new(1, region, 0, lines));

        let mut buf = ScreenBuffer::new(Size::new(40, 10));
        compositor.compose(&mut buf);

        // Row 0 starts with 'R' from "Row Zero"
        match buf.get(0, 0) {
            Some(cell) => {
                assert!(cell.grapheme == "R");
            }
            None => unreachable!(),
        }
        match buf.get(4, 0) {
            Some(cell) => {
                assert!(cell.grapheme == "Z");
            }
            None => unreachable!(),
        }

        // Row 1 starts with 'R' from "Row One"
        match buf.get(0, 1) {
            Some(cell) => {
                assert!(cell.grapheme == "R");
            }
            None => unreachable!(),
        }
        match buf.get(4, 1) {
            Some(cell) => {
                assert!(cell.grapheme == "O");
            }
            None => unreachable!(),
        }

        // Row 2 starts with 'R' from "Row Two"
        match buf.get(0, 2) {
            Some(cell) => {
                assert!(cell.grapheme == "R");
            }
            None => unreachable!(),
        }
        match buf.get(4, 2) {
            Some(cell) => {
                assert!(cell.grapheme == "T");
            }
            None => unreachable!(),
        }
    }

    /// Task 7, Test 5: Layer partially off-screen.
    #[test]
    fn layer_partially_off_screen() {
        // Screen is 10x5
        let mut compositor = Compositor::new(10, 5);

        // Layer starts at x=7, width=10 — extends to x=17 which is beyond screen width 10
        let region = Rect::new(7, 0, 10, 3);
        let lines = vec![vec![Segment::new("ABCDEFGHIJ")]];
        compositor.add_layer(Layer::new(1, region, 0, lines));

        let mut buf = ScreenBuffer::new(Size::new(10, 5));
        compositor.compose(&mut buf);

        // Column 7 should have 'A'
        match buf.get(7, 0) {
            Some(cell) => {
                assert!(cell.grapheme == "A");
            }
            None => unreachable!(),
        }

        // Column 9 (last column) should have 'C'
        match buf.get(9, 0) {
            Some(cell) => {
                assert!(cell.grapheme == "C");
            }
            None => unreachable!(),
        }

        // No out-of-bounds access — buffer should be fine
        assert!(buf.get(10, 0).is_none());
    }

    /// Task 7, Test 6: Zero-layer compositor produces all blank cells.
    #[test]
    fn zero_layer_compositor_all_blank() {
        let compositor = Compositor::new(20, 10);
        let mut buf = ScreenBuffer::new(Size::new(20, 10));
        compositor.compose(&mut buf);

        for y in 0..10 {
            for x in 0..20 {
                match buf.get(x, y) {
                    Some(cell) => {
                        assert!(cell.is_blank());
                    }
                    None => unreachable!(),
                }
            }
        }
    }

    /// Task 7, Test 7: Background layer with overlay covering middle portion.
    #[test]
    fn styled_segments_split_by_overlay() {
        let mut compositor = Compositor::new(20, 3);

        // Background: "Hello World" at (0,0), z=0
        let bg_region = Rect::new(0, 0, 20, 3);
        let bg_lines = vec![vec![Segment::new("Hello World")]];
        compositor.add_layer(Layer::new(1, bg_region, 0, bg_lines));

        // Overlay: "XXXXX" at x=3, covering positions 3-7, z=10
        let overlay_region = Rect::new(3, 0, 5, 3);
        let overlay_lines = vec![vec![Segment::new("XXXXX")]];
        compositor.add_layer(Layer::new(2, overlay_region, 10, overlay_lines));

        let mut buf = ScreenBuffer::new(Size::new(20, 3));
        compositor.compose(&mut buf);

        // Positions 0-2: "Hel" from background
        match buf.get(0, 0) {
            Some(cell) => {
                assert!(cell.grapheme == "H");
            }
            None => unreachable!(),
        }
        match buf.get(1, 0) {
            Some(cell) => {
                assert!(cell.grapheme == "e");
            }
            None => unreachable!(),
        }
        match buf.get(2, 0) {
            Some(cell) => {
                assert!(cell.grapheme == "l");
            }
            None => unreachable!(),
        }

        // Positions 3-7: "XXXXX" from overlay
        match buf.get(3, 0) {
            Some(cell) => {
                assert!(cell.grapheme == "X");
            }
            None => unreachable!(),
        }
        match buf.get(7, 0) {
            Some(cell) => {
                assert!(cell.grapheme == "X");
            }
            None => unreachable!(),
        }

        // Position 8: "W" from background ("Hello World" offset by 8 = 'W')
        // The background text "Hello World" has chars at positions:
        // H(0) e(1) l(2) l(3) o(4) (5) W(6) o(7) r(8) l(9) d(10)
        // But the overlay covers 3-7 of the layer, so background position 8
        // should show "o" (position 8 in "Hello World" = 'r')
        // Wait — "Hello World" is 11 chars, at layer origin x=0.
        // Background position 8 maps to "Hello World"[8] = 'r'
        match buf.get(8, 0) {
            Some(cell) => {
                assert!(cell.grapheme == "r");
            }
            None => unreachable!(),
        }
    }

    /// Task 7, Test 8: Large number of layers — topmost always wins.
    #[test]
    fn many_layers_topmost_wins() {
        let mut compositor = Compositor::new(20, 5);

        // Create 25 layers, all covering the same region, with increasing z-index
        // Each layer has its own single character
        for i in 0u64..25 {
            let ch = char::from(b'A' + (i as u8) % 26);
            let region = Rect::new(0, 0, 20, 5);
            let lines = vec![vec![Segment::new(ch.to_string())]];
            compositor.add_layer(Layer::new(i + 1, region, i as i32, lines));
        }

        let mut buf = ScreenBuffer::new(Size::new(20, 5));
        compositor.compose(&mut buf);

        // The topmost layer (z=24) has character 'Y' (b'A' + 24)
        match buf.get(0, 0) {
            Some(cell) => {
                assert!(cell.grapheme == "Y");
            }
            None => unreachable!(),
        }
    }
}

#[cfg(test)]
mod unicode_pipeline_tests {
    use super::*;
    use crate::color::{Color, NamedColor};
    use crate::geometry::{Rect, Size};
    use crate::segment::Segment;
    use crate::style::Style;

    /// Test 1: CJK text layer produces correct primary and continuation cells.
    #[test]
    fn cjk_text_layer_correct_cells() {
        let mut compositor = Compositor::new(20, 3);
        let region = Rect::new(0, 0, 20, 3);
        // Three CJK chars: 世界人 — each width 2, total width 6
        let lines = vec![vec![Segment::new("\u{4e16}\u{754c}\u{4eba}")]];
        compositor.add_layer(Layer::new(1, region, 0, lines));

        let mut buf = ScreenBuffer::new(Size::new(20, 3));
        compositor.compose(&mut buf);

        // Column 0: primary "世" width 2
        match buf.get(0, 0) {
            Some(c) => {
                assert_eq!(c.grapheme, "\u{4e16}");
                assert_eq!(c.width, 2);
            }
            None => unreachable!(),
        }
        // Column 1: continuation
        match buf.get(1, 0) {
            Some(c) => assert_eq!(c.width, 0),
            None => unreachable!(),
        }
        // Column 2: primary "界" width 2
        match buf.get(2, 0) {
            Some(c) => {
                assert_eq!(c.grapheme, "\u{754c}");
                assert_eq!(c.width, 2);
            }
            None => unreachable!(),
        }
        // Column 3: continuation
        match buf.get(3, 0) {
            Some(c) => assert_eq!(c.width, 0),
            None => unreachable!(),
        }
        // Column 4: primary "人" width 2
        match buf.get(4, 0) {
            Some(c) => {
                assert_eq!(c.grapheme, "\u{4eba}");
                assert_eq!(c.width, 2);
            }
            None => unreachable!(),
        }
        // Column 5: continuation
        match buf.get(5, 0) {
            Some(c) => assert_eq!(c.width, 0),
            None => unreachable!(),
        }
        // Column 6: blank
        match buf.get(6, 0) {
            Some(c) => assert!(c.is_blank()),
            None => unreachable!(),
        }
    }

    /// Test 2: Emoji text layer produces correct cells.
    #[test]
    fn emoji_text_layer_correct_cells() {
        let mut compositor = Compositor::new(20, 3);
        let region = Rect::new(0, 0, 20, 3);
        // Two emoji: 😀🎉 — each width 2
        let lines = vec![vec![Segment::new("\u{1f600}\u{1f389}")]];
        compositor.add_layer(Layer::new(1, region, 0, lines));

        let mut buf = ScreenBuffer::new(Size::new(20, 3));
        compositor.compose(&mut buf);

        // Column 0: primary emoji, width 2
        match buf.get(0, 0) {
            Some(c) => {
                assert_eq!(c.grapheme, "\u{1f600}");
                assert_eq!(c.width, 2);
            }
            None => unreachable!(),
        }
        // Column 1: continuation
        match buf.get(1, 0) {
            Some(c) => assert_eq!(c.width, 0),
            None => unreachable!(),
        }
        // Column 2: second emoji, width 2
        match buf.get(2, 0) {
            Some(c) => {
                assert_eq!(c.grapheme, "\u{1f389}");
                assert_eq!(c.width, 2);
            }
            None => unreachable!(),
        }
        // Column 3: continuation
        match buf.get(3, 0) {
            Some(c) => assert_eq!(c.width, 0),
            None => unreachable!(),
        }
    }

    /// Test 3: Mixed Latin + CJK + emoji in one layer — all widths correct.
    #[test]
    fn mixed_latin_cjk_emoji_widths() {
        let mut compositor = Compositor::new(20, 3);
        let region = Rect::new(0, 0, 20, 3);
        // "Hi" (2) + "世" (2) + "😀" (2) = total width 6
        let lines = vec![vec![Segment::new("Hi\u{4e16}\u{1f600}")]];
        compositor.add_layer(Layer::new(1, region, 0, lines));

        let mut buf = ScreenBuffer::new(Size::new(20, 3));
        compositor.compose(&mut buf);

        // Columns 0-1: "H", "i" (each width 1)
        match buf.get(0, 0) {
            Some(c) => {
                assert_eq!(c.grapheme, "H");
                assert_eq!(c.width, 1);
            }
            None => unreachable!(),
        }
        match buf.get(1, 0) {
            Some(c) => {
                assert_eq!(c.grapheme, "i");
                assert_eq!(c.width, 1);
            }
            None => unreachable!(),
        }
        // Columns 2-3: CJK "世" (width 2) + continuation
        match buf.get(2, 0) {
            Some(c) => {
                assert_eq!(c.grapheme, "\u{4e16}");
                assert_eq!(c.width, 2);
            }
            None => unreachable!(),
        }
        match buf.get(3, 0) {
            Some(c) => assert_eq!(c.width, 0),
            None => unreachable!(),
        }
        // Columns 4-5: emoji "😀" (width 2) + continuation
        match buf.get(4, 0) {
            Some(c) => {
                assert_eq!(c.grapheme, "\u{1f600}");
                assert_eq!(c.width, 2);
            }
            None => unreachable!(),
        }
        match buf.get(5, 0) {
            Some(c) => assert_eq!(c.width, 0),
            None => unreachable!(),
        }
    }

    /// Test 4: Wide char at screen right edge — clipped correctly (no crash).
    #[test]
    fn wide_char_at_screen_right_edge_clipped() {
        // Screen width 5
        let mut compositor = Compositor::new(5, 1);
        let region = Rect::new(0, 0, 5, 1);
        // "ABCD世" = width 6: the CJK char starts at column 4 but needs column 5 too
        // The compositor writes 'A'(0), 'B'(1), 'C'(2), 'D'(3), then tries "世" at col 4
        // Since "世" is width 2 and the screen width is 5, column 5 is out of bounds
        // The buffer's set() will handle this by replacing with blank
        let lines = vec![vec![Segment::new("ABCD\u{4e16}")]];
        compositor.add_layer(Layer::new(1, region, 0, lines));

        let mut buf = ScreenBuffer::new(Size::new(5, 1));
        compositor.compose(&mut buf);

        // Columns 0-3 should have A, B, C, D
        match buf.get(0, 0) {
            Some(c) => assert_eq!(c.grapheme, "A"),
            None => unreachable!(),
        }
        match buf.get(3, 0) {
            Some(c) => assert_eq!(c.grapheme, "D"),
            None => unreachable!(),
        }
        // Column 4: the wide char can't fit, should be blank (buffer protection)
        match buf.get(4, 0) {
            Some(c) => {
                // Buffer set() replaces wide chars at last column with blank
                assert!(c.is_blank());
            }
            None => unreachable!(),
        }
        // No crash, no out-of-bounds
        assert!(buf.get(5, 0).is_none());
    }

    /// Test 5: Combining marks in a layer — preserved in buffer cells.
    #[test]
    fn combining_marks_preserved_in_buffer() {
        let mut compositor = Compositor::new(20, 3);
        let region = Rect::new(0, 0, 20, 3);
        // "e\u{0301}" = e with combining acute accent = single grapheme, width 1
        let lines = vec![vec![Segment::new("e\u{0301}X")]];
        compositor.add_layer(Layer::new(1, region, 0, lines));

        let mut buf = ScreenBuffer::new(Size::new(20, 3));
        compositor.compose(&mut buf);

        // Column 0: the composed grapheme "e\u{0301}" should be there
        match buf.get(0, 0) {
            Some(c) => {
                assert_eq!(c.grapheme, "e\u{0301}");
                assert_eq!(c.width, 1);
            }
            None => unreachable!(),
        }
        // Column 1: "X"
        match buf.get(1, 0) {
            Some(c) => assert_eq!(c.grapheme, "X"),
            None => unreachable!(),
        }
    }

    /// Test 6: Styled wide chars — style preserved in buffer.
    #[test]
    fn styled_wide_chars_preserved() {
        let mut compositor = Compositor::new(20, 3);
        let region = Rect::new(0, 0, 20, 3);
        let style = Style::new().fg(Color::Named(NamedColor::Red)).bold(true);
        // CJK text with style
        let lines = vec![vec![Segment::styled("\u{4e16}\u{754c}", style.clone())]];
        compositor.add_layer(Layer::new(1, region, 0, lines));

        let mut buf = ScreenBuffer::new(Size::new(20, 3));
        compositor.compose(&mut buf);

        // Column 0: "世" with red+bold style
        match buf.get(0, 0) {
            Some(c) => {
                assert_eq!(c.grapheme, "\u{4e16}");
                assert!(c.style.bold);
                assert!(matches!(c.style.fg, Some(Color::Named(NamedColor::Red))));
            }
            None => unreachable!(),
        }
        // Column 2: "界" with same style
        match buf.get(2, 0) {
            Some(c) => {
                assert_eq!(c.grapheme, "\u{754c}");
                assert!(c.style.bold);
                assert!(matches!(c.style.fg, Some(Color::Named(NamedColor::Red))));
            }
            None => unreachable!(),
        }
    }

    /// Test 7: Overlapping layers with different Unicode scripts — topmost wins.
    #[test]
    fn overlapping_unicode_scripts_topmost_wins() {
        let mut compositor = Compositor::new(20, 3);

        // Bottom layer (z=0): CJK text
        let bottom_region = Rect::new(0, 0, 20, 3);
        let bottom_lines = vec![vec![Segment::new("\u{4e16}\u{754c}\u{4eba}\u{6c11}")]];
        compositor.add_layer(Layer::new(1, bottom_region, 0, bottom_lines));

        // Top layer (z=10): Latin text at same position, overlapping first 4 columns
        let top_region = Rect::new(0, 0, 4, 3);
        let top_lines = vec![vec![Segment::new("ABCD")]];
        compositor.add_layer(Layer::new(2, top_region, 10, top_lines));

        let mut buf = ScreenBuffer::new(Size::new(20, 3));
        compositor.compose(&mut buf);

        // Columns 0-3: should have "ABCD" from top layer
        match buf.get(0, 0) {
            Some(c) => assert_eq!(c.grapheme, "A"),
            None => unreachable!(),
        }
        match buf.get(1, 0) {
            Some(c) => assert_eq!(c.grapheme, "B"),
            None => unreachable!(),
        }
        match buf.get(2, 0) {
            Some(c) => assert_eq!(c.grapheme, "C"),
            None => unreachable!(),
        }
        match buf.get(3, 0) {
            Some(c) => assert_eq!(c.grapheme, "D"),
            None => unreachable!(),
        }

        // Column 4 onwards: should have CJK from bottom layer
        // CJK "世界人民": 世(0-1), 界(2-3), 人(4-5), 民(6-7)
        // Since top layer covers 0-3, bottom layer columns 4-5 should show 人
        match buf.get(4, 0) {
            Some(c) => {
                assert_eq!(c.grapheme, "\u{4eba}");
                assert_eq!(c.width, 2);
            }
            None => unreachable!(),
        }
    }

    /// Test 8: Multiple rows of CJK text — correct row-by-row rendering.
    #[test]
    fn multiple_rows_cjk_text() {
        let mut compositor = Compositor::new(20, 5);
        let region = Rect::new(0, 0, 20, 5);
        let lines = vec![
            vec![Segment::new("\u{4e16}\u{754c}")], // 世界 (row 0)
            vec![Segment::new("\u{4eba}\u{6c11}")], // 人民 (row 1)
            vec![Segment::new("\u{5927}\u{5b66}")], // 大学 (row 2)
        ];
        compositor.add_layer(Layer::new(1, region, 0, lines));

        let mut buf = ScreenBuffer::new(Size::new(20, 5));
        compositor.compose(&mut buf);

        // Row 0: "世" at col 0, "界" at col 2
        match buf.get(0, 0) {
            Some(c) => {
                assert_eq!(c.grapheme, "\u{4e16}");
                assert_eq!(c.width, 2);
            }
            None => unreachable!(),
        }
        match buf.get(2, 0) {
            Some(c) => {
                assert_eq!(c.grapheme, "\u{754c}");
                assert_eq!(c.width, 2);
            }
            None => unreachable!(),
        }

        // Row 1: "人" at col 0, "民" at col 2
        match buf.get(0, 1) {
            Some(c) => {
                assert_eq!(c.grapheme, "\u{4eba}");
                assert_eq!(c.width, 2);
            }
            None => unreachable!(),
        }
        match buf.get(2, 1) {
            Some(c) => {
                assert_eq!(c.grapheme, "\u{6c11}");
                assert_eq!(c.width, 2);
            }
            None => unreachable!(),
        }

        // Row 2: "大" at col 0, "学" at col 2
        match buf.get(0, 2) {
            Some(c) => {
                assert_eq!(c.grapheme, "\u{5927}");
                assert_eq!(c.width, 2);
            }
            None => unreachable!(),
        }
        match buf.get(2, 2) {
            Some(c) => {
                assert_eq!(c.grapheme, "\u{5b66}");
                assert_eq!(c.width, 2);
            }
            None => unreachable!(),
        }

        // Row 3: should be all blank (no content)
        match buf.get(0, 3) {
            Some(c) => assert!(c.is_blank()),
            None => unreachable!(),
        }
    }
}
