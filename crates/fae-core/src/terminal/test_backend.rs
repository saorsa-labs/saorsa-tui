//! In-memory terminal backend for testing.

use crate::error::Result;
use crate::geometry::Size;

use super::traits::{Terminal, TerminalCapabilities};

/// In-memory terminal backend for testing.
///
/// All output is captured in a buffer that can be inspected.
pub struct TestBackend {
    size: Size,
    capabilities: TerminalCapabilities,
    buffer: Vec<u8>,
    raw_mode: bool,
    mouse_enabled: bool,
}

impl TestBackend {
    /// Create a new test backend with the given size.
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            size: Size::new(width, height),
            capabilities: TerminalCapabilities::default(),
            buffer: Vec::new(),
            raw_mode: false,
            mouse_enabled: false,
        }
    }

    /// Get the bytes written to this backend.
    pub fn buffer(&self) -> &[u8] {
        &self.buffer
    }

    /// Clear the output buffer.
    pub fn clear_buffer(&mut self) {
        self.buffer.clear();
    }

    /// Check if raw mode is active.
    pub fn is_raw_mode(&self) -> bool {
        self.raw_mode
    }

    /// Check if mouse capture is active.
    pub fn is_mouse_enabled(&self) -> bool {
        self.mouse_enabled
    }

    /// Set the terminal size (simulates a resize).
    pub fn set_size(&mut self, width: u16, height: u16) {
        self.size = Size::new(width, height);
    }
}

impl Terminal for TestBackend {
    fn size(&self) -> Result<Size> {
        Ok(self.size)
    }

    fn capabilities(&self) -> &TerminalCapabilities {
        &self.capabilities
    }

    fn enter_raw_mode(&mut self) -> Result<()> {
        self.raw_mode = true;
        Ok(())
    }

    fn exit_raw_mode(&mut self) -> Result<()> {
        self.raw_mode = false;
        Ok(())
    }

    fn write_raw(&mut self, data: &[u8]) -> Result<()> {
        self.buffer.extend_from_slice(data);
        Ok(())
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }

    fn enable_mouse(&mut self) -> Result<()> {
        self.mouse_enabled = true;
        Ok(())
    }

    fn disable_mouse(&mut self) -> Result<()> {
        self.mouse_enabled = false;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_size() {
        let tb = TestBackend::new(80, 24);
        let size = tb.size().ok();
        assert_eq!(size, Some(Size::new(80, 24)));
    }

    #[test]
    fn test_backend_write() {
        let mut tb = TestBackend::new(80, 24);
        tb.write_raw(b"hello").ok();
        assert_eq!(tb.buffer(), b"hello");
    }

    #[test]
    fn test_backend_clear() {
        let mut tb = TestBackend::new(80, 24);
        tb.write_raw(b"data").ok();
        tb.clear_buffer();
        assert!(tb.buffer().is_empty());
    }

    #[test]
    fn test_backend_raw_mode() {
        let mut tb = TestBackend::new(80, 24);
        assert!(!tb.is_raw_mode());
        tb.enter_raw_mode().ok();
        assert!(tb.is_raw_mode());
        tb.exit_raw_mode().ok();
        assert!(!tb.is_raw_mode());
    }

    #[test]
    fn test_backend_mouse() {
        let mut tb = TestBackend::new(80, 24);
        assert!(!tb.is_mouse_enabled());
        tb.enable_mouse().ok();
        assert!(tb.is_mouse_enabled());
        tb.disable_mouse().ok();
        assert!(!tb.is_mouse_enabled());
    }

    #[test]
    fn test_backend_resize() {
        let mut tb = TestBackend::new(80, 24);
        tb.set_size(120, 40);
        let size = tb.size().ok();
        assert_eq!(size, Some(Size::new(120, 40)));
    }
}
