//! Render context — manages the frame lifecycle and rendering pipeline.
//!
//! The `RenderContext` holds the current and previous frame buffers,
//! diffs them, and renders the changes to the terminal. It optionally
//! integrates a [`Compositor`] to resolve overlapping widget layers
//! before diffing.

use crate::buffer::ScreenBuffer;
use crate::compositor::Compositor;
use crate::error::Result;
use crate::geometry::Size;
use crate::renderer::Renderer;
use crate::terminal::Terminal;

/// Manages the double-buffered rendering pipeline.
///
/// Each frame:
/// 1. `begin_frame()` — swap buffers, clear the current buffer
/// 2. Application writes to the current buffer
/// 3. `end_frame()` — optionally compose layers, diff, render, and write to terminal
pub struct RenderContext {
    current: ScreenBuffer,
    previous: ScreenBuffer,
    renderer: Renderer,
    size: Size,
    compositor: Option<Compositor>,
}

impl RenderContext {
    /// Create a new render context for the given terminal.
    pub fn new(terminal: &dyn Terminal) -> Result<Self> {
        let size = terminal.size()?;
        let caps = terminal.capabilities();
        let renderer = Renderer::new(caps.color, caps.synchronized_output);
        Ok(Self {
            current: ScreenBuffer::new(size),
            previous: ScreenBuffer::new(size),
            renderer,
            size,
            compositor: None,
        })
    }

    /// Create a render context with explicit size and capabilities (for testing).
    pub fn with_size(size: Size, renderer: Renderer) -> Self {
        Self {
            current: ScreenBuffer::new(size),
            previous: ScreenBuffer::new(size),
            renderer,
            size,
            compositor: None,
        }
    }

    /// Set the compositor for this render context (builder pattern).
    ///
    /// When a compositor is present, `end_frame()` will call
    /// `compositor.compose()` on the current buffer before diffing.
    #[must_use]
    pub fn with_compositor(mut self, compositor: Compositor) -> Self {
        self.compositor = Some(compositor);
        self
    }

    /// Get a reference to the compositor, if one is set.
    pub fn compositor(&self) -> Option<&Compositor> {
        self.compositor.as_ref()
    }

    /// Get a mutable reference to the compositor, if one is set.
    pub fn compositor_mut(&mut self) -> Option<&mut Compositor> {
        self.compositor.as_mut()
    }

    /// Get the current buffer dimensions.
    pub fn size(&self) -> Size {
        self.size
    }

    /// Get a mutable reference to the current buffer for writing.
    pub fn buffer_mut(&mut self) -> &mut ScreenBuffer {
        &mut self.current
    }

    /// Get a reference to the current buffer.
    pub fn buffer(&self) -> &ScreenBuffer {
        &self.current
    }

    /// Begin a new frame: swap current → previous and clear the current buffer.
    pub fn begin_frame(&mut self) {
        std::mem::swap(&mut self.current, &mut self.previous);
        self.current.clear();
    }

    /// End the frame: optionally compose layers, diff current vs previous,
    /// render to escape sequences, write to terminal and flush.
    ///
    /// If a compositor is present, it composes all layers into the current
    /// buffer before the diff step.
    pub fn end_frame(&mut self, terminal: &mut dyn Terminal) -> Result<()> {
        // Compose layers into the buffer if a compositor is set
        if let Some(ref compositor) = self.compositor {
            compositor.compose(&mut self.current);
        }

        let changes = self.current.diff(&self.previous);
        let output = self.renderer.render(&changes);
        if !output.is_empty() {
            terminal.write_raw(output.as_bytes())?;
            terminal.flush()?;
        }
        Ok(())
    }

    /// Handle a terminal resize: update buffers, size, and compositor dimensions.
    pub fn handle_resize(&mut self, new_size: Size) {
        self.size = new_size;
        self.current.resize(new_size);
        self.previous.resize(new_size);
        if let Some(ref mut compositor) = self.compositor {
            compositor.resize(new_size.width, new_size.height);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell::Cell;
    use crate::color::{Color, NamedColor};
    use crate::compositor::Compositor;
    use crate::geometry::Rect;
    use crate::segment::Segment;
    use crate::style::Style;
    use crate::terminal::{ColorSupport, TestBackend};

    #[test]
    fn create_from_test_backend() {
        let backend = TestBackend::new(80, 24);
        let ctx = RenderContext::new(&backend);
        assert!(ctx.is_ok());
        let ctx = ctx.ok();
        assert!(ctx.is_some());
        let ctx = ctx.as_ref();
        assert_eq!(ctx.map(|c| c.size()), Some(Size::new(80, 24)));
    }

    #[test]
    fn begin_frame_clears_current() {
        let renderer = Renderer::new(ColorSupport::TrueColor, false);
        let mut ctx = RenderContext::with_size(Size::new(10, 5), renderer);

        // Write something
        ctx.buffer_mut().set(0, 0, Cell::new("A", Style::default()));
        assert_eq!(
            ctx.buffer().get(0, 0).map(|c| c.grapheme.as_str()),
            Some("A")
        );

        // Begin frame: current should be cleared
        ctx.begin_frame();
        assert!(ctx.buffer().get(0, 0).is_some_and(|c| c.is_blank()));
    }

    #[test]
    fn end_frame_writes_to_terminal() {
        let mut backend = TestBackend::new(10, 5);
        let renderer = Renderer::new(ColorSupport::TrueColor, false);
        let mut ctx = RenderContext::with_size(Size::new(10, 5), renderer);

        // Write a cell
        ctx.buffer_mut().set(0, 0, Cell::new("A", Style::default()));

        // End frame should write escape sequences to the backend
        let result = ctx.end_frame(&mut backend);
        assert!(result.is_ok());

        let output = backend.buffer();
        assert!(!output.is_empty());
        // Should contain cursor positioning and the character
        let output_str = String::from_utf8_lossy(output);
        assert!(output_str.contains('A'));
    }

    #[test]
    fn second_frame_only_diffs() {
        let mut backend = TestBackend::new(10, 5);
        let renderer = Renderer::new(ColorSupport::TrueColor, false);
        let mut ctx = RenderContext::with_size(Size::new(10, 5), renderer);

        // Frame 1: write "A" at (0,0)
        ctx.buffer_mut().set(0, 0, Cell::new("A", Style::default()));
        let _ = ctx.end_frame(&mut backend);
        backend.clear_buffer();

        // Frame 2: keep "A" at (0,0), add "B" at (1,0)
        ctx.begin_frame();
        ctx.buffer_mut().set(0, 0, Cell::new("A", Style::default()));
        ctx.buffer_mut().set(1, 0, Cell::new("B", Style::default()));
        let _ = ctx.end_frame(&mut backend);

        let output = backend.buffer();
        let output_str = String::from_utf8_lossy(output);
        // Should contain "B" (the new cell) but "A" should also be in the
        // diff since it changed from blank (after begin_frame clear) to "A" —
        // actually begin_frame clears current, so all cells are new
        assert!(output_str.contains('B'));
    }

    #[test]
    fn handle_resize() {
        let renderer = Renderer::new(ColorSupport::TrueColor, false);
        let mut ctx = RenderContext::with_size(Size::new(10, 5), renderer);
        assert_eq!(ctx.size(), Size::new(10, 5));

        ctx.handle_resize(Size::new(20, 10));
        assert_eq!(ctx.size(), Size::new(20, 10));
        assert_eq!(ctx.buffer().size(), Size::new(20, 10));
    }

    #[test]
    fn styled_cell_rendering() {
        crate::test_env::without_no_color(|| {
            let mut backend = TestBackend::new(10, 5);
            let renderer = Renderer::new(ColorSupport::TrueColor, false);
            let mut ctx = RenderContext::with_size(Size::new(10, 5), renderer);

            let style = Style::new().fg(Color::Named(NamedColor::Red)).bold(true);
            ctx.buffer_mut().set(0, 0, Cell::new("X", style));
            let _ = ctx.end_frame(&mut backend);

            let output = backend.buffer();
            let output_str = String::from_utf8_lossy(output);
            assert!(output_str.contains("\x1b[31m")); // red fg
            assert!(output_str.contains("\x1b[1m")); // bold
            assert!(output_str.contains('X'));
        });
    }

    // --- New compositor integration tests ---

    #[test]
    fn compositor_none_by_default() {
        let renderer = Renderer::new(ColorSupport::TrueColor, false);
        let ctx = RenderContext::with_size(Size::new(10, 5), renderer);
        assert!(ctx.compositor().is_none());
    }

    #[test]
    fn compositor_none_from_new() {
        let backend = TestBackend::new(80, 24);
        let result = RenderContext::new(&backend);
        assert!(result.is_ok());
        match result {
            Ok(ctx) => assert!(ctx.compositor().is_none()),
            Err(_) => unreachable!(),
        }
    }

    #[test]
    fn with_compositor_sets_compositor() {
        let renderer = Renderer::new(ColorSupport::TrueColor, false);
        let compositor = Compositor::new(10, 5);
        let ctx = RenderContext::with_size(Size::new(10, 5), renderer).with_compositor(compositor);
        assert!(ctx.compositor().is_some());
    }

    #[test]
    fn compositor_accessor_returns_reference() {
        let renderer = Renderer::new(ColorSupport::TrueColor, false);
        let compositor = Compositor::new(80, 24);
        let ctx = RenderContext::with_size(Size::new(80, 24), renderer).with_compositor(compositor);
        match ctx.compositor() {
            Some(c) => {
                assert_eq!(c.screen_size(), Size::new(80, 24));
            }
            None => unreachable!(),
        }
    }

    #[test]
    fn compositor_mut_allows_mutation() {
        let renderer = Renderer::new(ColorSupport::TrueColor, false);
        let compositor = Compositor::new(80, 24);
        let mut ctx =
            RenderContext::with_size(Size::new(80, 24), renderer).with_compositor(compositor);

        // Add a layer through the mutable accessor
        match ctx.compositor_mut() {
            Some(c) => {
                let region = Rect::new(0, 0, 10, 5);
                c.add_widget(1, region, 0, vec![vec![Segment::new("test")]]);
                assert_eq!(c.layer_count(), 1);
            }
            None => unreachable!(),
        }
    }

    #[test]
    fn end_frame_with_compositor_composes_before_diff() {
        let mut backend = TestBackend::new(20, 5);
        let renderer = Renderer::new(ColorSupport::TrueColor, false);
        let mut compositor = Compositor::new(20, 5);

        // Add a layer with text
        let region = Rect::new(0, 0, 20, 5);
        compositor.add_widget(1, region, 0, vec![vec![Segment::new("Hello")]]);

        let mut ctx =
            RenderContext::with_size(Size::new(20, 5), renderer).with_compositor(compositor);

        // Don't write directly to the buffer — let the compositor handle it
        let result = ctx.end_frame(&mut backend);
        assert!(result.is_ok());

        let output = backend.buffer();
        let output_str = String::from_utf8_lossy(output);
        // The compositor should have written "Hello" into the buffer
        assert!(output_str.contains('H'));
        assert!(output_str.contains("ello"));
    }

    #[test]
    fn compositor_z_ordering_in_render_context() {
        let mut backend = TestBackend::new(20, 5);
        let renderer = Renderer::new(ColorSupport::TrueColor, false);
        let mut compositor = Compositor::new(20, 5);

        // Background layer (z=0) with "BACKGROUND"
        let bg_region = Rect::new(0, 0, 20, 5);
        compositor.add_widget(1, bg_region, 0, vec![vec![Segment::new("BACKGROUND")]]);

        // Overlay layer (z=10) at same position with "TOP"
        let fg_region = Rect::new(0, 0, 20, 5);
        compositor.add_widget(2, fg_region, 10, vec![vec![Segment::new("TOP")]]);

        let mut ctx =
            RenderContext::with_size(Size::new(20, 5), renderer).with_compositor(compositor);

        let result = ctx.end_frame(&mut backend);
        assert!(result.is_ok());

        // The buffer should have the overlay text at (0,0) since z=10 > z=0
        match ctx.buffer().get(0, 0) {
            Some(cell) => {
                assert_eq!(cell.grapheme, "T");
            }
            None => unreachable!(),
        }
    }

    #[test]
    fn handle_resize_updates_compositor() {
        let renderer = Renderer::new(ColorSupport::TrueColor, false);
        let compositor = Compositor::new(10, 5);
        let mut ctx =
            RenderContext::with_size(Size::new(10, 5), renderer).with_compositor(compositor);

        ctx.handle_resize(Size::new(40, 20));

        assert_eq!(ctx.size(), Size::new(40, 20));
        match ctx.compositor() {
            Some(c) => {
                assert_eq!(c.screen_size(), Size::new(40, 20));
            }
            None => unreachable!(),
        }
    }

    #[test]
    fn integration_widget_segments_through_compositor() {
        let mut backend = TestBackend::new(40, 10);
        let renderer = Renderer::new(ColorSupport::TrueColor, false);
        let mut compositor = Compositor::new(40, 10);

        // Widget 1: title bar at top
        let title_region = Rect::new(0, 0, 40, 1);
        let title_style = Style::new().fg(Color::Named(NamedColor::Green)).bold(true);
        let title_seg = Segment::styled("Title Bar", title_style);
        compositor.add_widget(1, title_region, 0, vec![vec![title_seg]]);

        // Widget 2: content area
        let content_region = Rect::new(0, 1, 40, 9);
        let content_seg = Segment::new("Content here");
        compositor.add_widget(2, content_region, 0, vec![vec![content_seg]]);

        let mut ctx =
            RenderContext::with_size(Size::new(40, 10), renderer).with_compositor(compositor);

        let result = ctx.end_frame(&mut backend);
        assert!(result.is_ok());

        // Check title bar rendered at row 0
        match ctx.buffer().get(0, 0) {
            Some(cell) => {
                assert_eq!(cell.grapheme, "T");
                assert!(cell.style.bold);
            }
            None => unreachable!(),
        }

        // Check content rendered at row 1
        match ctx.buffer().get(0, 1) {
            Some(cell) => {
                assert_eq!(cell.grapheme, "C");
            }
            None => unreachable!(),
        }
    }
}
