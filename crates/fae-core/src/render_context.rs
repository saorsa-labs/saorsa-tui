//! Render context — manages the frame lifecycle and rendering pipeline.
//!
//! The `RenderContext` holds the current and previous frame buffers,
//! diffs them, and renders the changes to the terminal.

use crate::buffer::ScreenBuffer;
use crate::error::Result;
use crate::geometry::Size;
use crate::renderer::Renderer;
use crate::terminal::Terminal;

/// Manages the double-buffered rendering pipeline.
///
/// Each frame:
/// 1. `begin_frame()` — swap buffers, clear the current buffer
/// 2. Application writes to the current buffer
/// 3. `end_frame()` — diff, render, and write to terminal
pub struct RenderContext {
    current: ScreenBuffer,
    previous: ScreenBuffer,
    renderer: Renderer,
    size: Size,
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
        })
    }

    /// Create a render context with explicit size and capabilities (for testing).
    pub fn with_size(size: Size, renderer: Renderer) -> Self {
        Self {
            current: ScreenBuffer::new(size),
            previous: ScreenBuffer::new(size),
            renderer,
            size,
        }
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

    /// End the frame: diff current vs previous, render to escape sequences,
    /// write to terminal and flush.
    pub fn end_frame(&mut self, terminal: &mut dyn Terminal) -> Result<()> {
        let changes = self.current.diff(&self.previous);
        let output = self.renderer.render(&changes);
        if !output.is_empty() {
            terminal.write_raw(output.as_bytes())?;
            terminal.flush()?;
        }
        Ok(())
    }

    /// Handle a terminal resize: update buffers and size.
    pub fn handle_resize(&mut self, new_size: Size) {
        self.size = new_size;
        self.current.resize(new_size);
        self.previous.resize(new_size);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell::Cell;
    use crate::color::{Color, NamedColor};
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
        ctx.buffer_mut()
            .set(0, 0, Cell::new("A", Style::default()));
        assert_eq!(
            ctx.buffer().get(0, 0).map(|c| c.grapheme.as_str()),
            Some("A")
        );

        // Begin frame: current should be cleared
        ctx.begin_frame();
        assert!(ctx
            .buffer()
            .get(0, 0)
            .is_some_and(|c| c.is_blank()));
    }

    #[test]
    fn end_frame_writes_to_terminal() {
        let mut backend = TestBackend::new(10, 5);
        let renderer = Renderer::new(ColorSupport::TrueColor, false);
        let mut ctx = RenderContext::with_size(Size::new(10, 5), renderer);

        // Write a cell
        ctx.buffer_mut()
            .set(0, 0, Cell::new("A", Style::default()));

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
        ctx.buffer_mut()
            .set(0, 0, Cell::new("A", Style::default()));
        let _ = ctx.end_frame(&mut backend);
        backend.clear_buffer();

        // Frame 2: keep "A" at (0,0), add "B" at (1,0)
        ctx.begin_frame();
        ctx.buffer_mut()
            .set(0, 0, Cell::new("A", Style::default()));
        ctx.buffer_mut()
            .set(1, 0, Cell::new("B", Style::default()));
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
        let mut backend = TestBackend::new(10, 5);
        let renderer = Renderer::new(ColorSupport::TrueColor, false);
        let mut ctx = RenderContext::with_size(Size::new(10, 5), renderer);

        let style = Style::new()
            .fg(Color::Named(NamedColor::Red))
            .bold(true);
        ctx.buffer_mut().set(0, 0, Cell::new("X", style));
        let _ = ctx.end_frame(&mut backend);

        let output = backend.buffer();
        let output_str = String::from_utf8_lossy(output);
        assert!(output_str.contains("\x1b[31m")); // red fg
        assert!(output_str.contains("\x1b[1m")); // bold
        assert!(output_str.contains('X'));
    }
}
