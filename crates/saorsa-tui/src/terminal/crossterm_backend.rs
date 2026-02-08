//! Crossterm-based terminal backend.

use std::io::{self, Write};

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};

use crate::error::Result;
use crate::geometry::Size;

use super::detect::{MultiplexerKind, detect_multiplexer, detect_terminal};
use super::profiles::{merge_multiplexer_limits, profile_for};
use super::traits::{Terminal, TerminalCapabilities};

/// Terminal backend using crossterm for real terminal I/O.
pub struct CrosstermBackend {
    capabilities: TerminalCapabilities,
    multiplexer: MultiplexerKind,
    raw_mode: bool,
}

impl CrosstermBackend {
    /// Create a new crossterm backend, detecting capabilities from the environment.
    ///
    /// This performs automatic detection of the terminal emulator and any running
    /// multiplexer (tmux, screen, zellij) and configures capabilities accordingly.
    pub fn new() -> Self {
        let kind = detect_terminal();
        let multiplexer = detect_multiplexer();
        let mut capabilities = profile_for(kind);
        capabilities = merge_multiplexer_limits(capabilities, multiplexer);

        Self {
            capabilities,
            multiplexer,
            raw_mode: false,
        }
    }

    /// Create a new crossterm backend with explicit capabilities.
    ///
    /// This bypasses automatic detection and uses the provided capabilities.
    /// Useful for testing or when capabilities are known in advance.
    pub fn with_capabilities(capabilities: TerminalCapabilities) -> Self {
        Self {
            capabilities,
            multiplexer: MultiplexerKind::None,
            raw_mode: false,
        }
    }

    /// Returns the detected multiplexer kind.
    ///
    /// This is used internally for escape sequence wrapping when sending
    /// terminal escape codes through multiplexers like tmux or screen.
    pub fn multiplexer(&self) -> MultiplexerKind {
        self.multiplexer
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terminal::detect::TerminalKind;
    use crate::terminal::traits::ColorSupport;

    #[test]
    fn test_new_creates_backend_with_detected_capabilities() {
        let backend = CrosstermBackend::new();
        // Just verify it created successfully with some capabilities
        assert!(!backend.raw_mode);
    }

    #[test]
    fn test_with_capabilities_uses_provided_caps() {
        let caps = TerminalCapabilities {
            color: ColorSupport::Extended256,
            unicode: true,
            synchronized_output: true,
            kitty_keyboard: true,
            mouse: true,
            bracketed_paste: true,
            focus_events: true,
            hyperlinks: true,
            sixel: false,
        };

        let backend = CrosstermBackend::with_capabilities(caps.clone());
        assert_eq!(backend.capabilities(), &caps);
        assert_eq!(backend.multiplexer, MultiplexerKind::None);
    }

    #[test]
    fn test_integration_kitty_profile() {
        // Verify that our detection integrates properly with profiles
        let kind = TerminalKind::Kitty;
        let profile = profile_for(kind);
        let backend = CrosstermBackend::with_capabilities(profile.clone());

        assert_eq!(backend.capabilities().color, ColorSupport::TrueColor);
        assert!(backend.capabilities().kitty_keyboard);
        assert!(backend.capabilities().synchronized_output);
    }

    #[test]
    fn test_integration_tmux_limits() {
        // Verify multiplexer limits are applied
        let kind = TerminalKind::Kitty;
        let multiplexer = MultiplexerKind::Tmux;
        let mut caps = profile_for(kind);
        caps = merge_multiplexer_limits(caps, multiplexer);

        let backend = CrosstermBackend::with_capabilities(caps);
        // Tmux should disable synchronized output even on Kitty
        assert!(!backend.capabilities().synchronized_output);
        // But other Kitty features should remain
        assert!(backend.capabilities().kitty_keyboard);
    }
}
