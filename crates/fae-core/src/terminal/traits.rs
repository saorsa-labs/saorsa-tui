//! Terminal trait and capability types.

use crate::error::Result;
use crate::geometry::Size;

/// Level of color support available.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ColorSupport {
    /// No color.
    NoColor,
    /// 16 ANSI colors.
    Basic16,
    /// 256 color palette.
    Extended256,
    /// 24-bit true color.
    TrueColor,
}

/// Capabilities detected for the terminal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TerminalCapabilities {
    /// Color support level.
    pub color: ColorSupport,
    /// Whether the terminal supports Unicode.
    pub unicode: bool,
    /// Whether CSI 2026 synchronized output is supported.
    pub synchronized_output: bool,
    /// Whether the Kitty keyboard protocol is supported.
    pub kitty_keyboard: bool,
    /// Whether mouse events are available.
    pub mouse: bool,
}

impl Default for TerminalCapabilities {
    fn default() -> Self {
        Self {
            color: ColorSupport::TrueColor,
            unicode: true,
            synchronized_output: false,
            kitty_keyboard: false,
            mouse: true,
        }
    }
}

/// Abstraction over terminal backends.
pub trait Terminal: Send {
    /// Get the current terminal size.
    fn size(&self) -> Result<Size>;

    /// Get the terminal's capabilities.
    fn capabilities(&self) -> &TerminalCapabilities;

    /// Enter raw mode (disable line buffering, echo, etc.).
    fn enter_raw_mode(&mut self) -> Result<()>;

    /// Exit raw mode (restore normal terminal state).
    fn exit_raw_mode(&mut self) -> Result<()>;

    /// Write raw bytes to the terminal.
    fn write_raw(&mut self, data: &[u8]) -> Result<()>;

    /// Flush buffered output to the terminal.
    fn flush(&mut self) -> Result<()>;

    /// Enable mouse event capture.
    fn enable_mouse(&mut self) -> Result<()>;

    /// Disable mouse event capture.
    fn disable_mouse(&mut self) -> Result<()>;
}
