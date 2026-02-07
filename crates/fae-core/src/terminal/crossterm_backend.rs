//! Crossterm-based terminal backend.

use std::io::{self, Write};

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};

use crate::error::Result;
use crate::geometry::Size;

use super::traits::{ColorSupport, Terminal, TerminalCapabilities};

/// Terminal backend using crossterm for real terminal I/O.
pub struct CrosstermBackend {
    capabilities: TerminalCapabilities,
    raw_mode: bool,
}

impl CrosstermBackend {
    /// Create a new crossterm backend, detecting capabilities.
    pub fn new() -> Self {
        let capabilities = detect_capabilities();
        Self {
            capabilities,
            raw_mode: false,
        }
    }
}

impl Default for CrosstermBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl Terminal for CrosstermBackend {
    fn size(&self) -> Result<Size> {
        let (w, h) = terminal::size()?;
        Ok(Size::new(w, h))
    }

    fn capabilities(&self) -> &TerminalCapabilities {
        &self.capabilities
    }

    fn enter_raw_mode(&mut self) -> Result<()> {
        if !self.raw_mode {
            terminal::enable_raw_mode()?;
            execute!(io::stdout(), EnterAlternateScreen)?;
            self.raw_mode = true;
        }
        Ok(())
    }

    fn exit_raw_mode(&mut self) -> Result<()> {
        if self.raw_mode {
            execute!(io::stdout(), LeaveAlternateScreen)?;
            terminal::disable_raw_mode()?;
            self.raw_mode = false;
        }
        Ok(())
    }

    fn write_raw(&mut self, data: &[u8]) -> Result<()> {
        io::stdout().write_all(data)?;
        Ok(())
    }

    fn flush(&mut self) -> Result<()> {
        io::stdout().flush()?;
        Ok(())
    }

    fn enable_mouse(&mut self) -> Result<()> {
        execute!(io::stdout(), EnableMouseCapture)?;
        Ok(())
    }

    fn disable_mouse(&mut self) -> Result<()> {
        execute!(io::stdout(), DisableMouseCapture)?;
        Ok(())
    }
}

impl Drop for CrosstermBackend {
    fn drop(&mut self) {
        if self.raw_mode {
            let _ = execute!(io::stdout(), LeaveAlternateScreen);
            let _ = terminal::disable_raw_mode();
        }
    }
}

/// Detect terminal capabilities from the environment.
fn detect_capabilities() -> TerminalCapabilities {
    let color = detect_color_support();
    TerminalCapabilities {
        color,
        unicode: true,
        synchronized_output: false,
        kitty_keyboard: false,
        mouse: true,
        bracketed_paste: true,
        focus_events: true,
        hyperlinks: true,
        sixel: false,
    }
}

/// Detect color support from environment variables.
fn detect_color_support() -> ColorSupport {
    // Check COLORTERM for truecolor
    if let Ok(ct) = std::env::var("COLORTERM")
        && (ct == "truecolor" || ct == "24bit")
    {
        return ColorSupport::TrueColor;
    }
    // Check TERM for 256-color
    if let Ok(term) = std::env::var("TERM")
        && term.contains("256color")
    {
        return ColorSupport::Extended256;
    }
    // Check NO_COLOR
    if std::env::var("NO_COLOR").is_ok() {
        return ColorSupport::NoColor;
    }
    ColorSupport::Basic16
}
