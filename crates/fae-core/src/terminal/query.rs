//! Runtime terminal capability detection via escape sequence queries.

use super::detect::{MultiplexerKind, TerminalKind};
use super::profiles::{merge_multiplexer_limits, profile_for};
use super::traits::{ColorSupport, TerminalCapabilities};
use std::io::{Read, Write};
use std::time::Duration;

/// Escape sequence constants for terminal queries.
mod sequences {
    /// DA1 - Primary Device Attributes query.
    /// Response format: CSI ? 1 ; 2 c (varies by terminal)
    pub const DA1: &[u8] = b"\x1b[c";

    /// DECRPM - Request Mode query for synchronized output (mode 2026).
    /// Response format: CSI ? 2026 ; Ps $ y where Ps is 0-4
    /// 0=not recognized, 1=set, 2=reset, 3=permanently set, 4=permanently reset
    pub const DECRPM_SYNC_OUTPUT: &[u8] = b"\x1b[?2026$p";

    /// Kitty keyboard protocol query.
    /// Response format: CSI ? flags u (if supported)
    pub const KITTY_KEYBOARD_QUERY: &[u8] = b"\x1b[?u";
}

/// Trait for querying terminal capabilities at runtime.
pub trait TerminalQuerier {
    /// Query color support via DA1 (Device Attributes).
    ///
    /// Sends CSI c and parses the response to determine color capabilities.
    /// Returns `None` on timeout or parse failure.
    fn query_color_support(&mut self) -> Option<ColorSupport>;

    /// Query synchronized output support via DECRPM.
    ///
    /// Sends CSI ? 2026 $ p and checks if mode 2026 is supported.
    /// Returns `None` on timeout or parse failure.
    fn query_synchronized_output(&mut self) -> Option<bool>;

    /// Query Kitty keyboard protocol support.
    ///
    /// Sends CSI ? u and checks for a valid response.
    /// Returns `None` on timeout or parse failure.
    fn query_kitty_keyboard(&mut self) -> Option<bool>;
}

/// Live terminal querier that talks to a real TTY.
pub struct LiveQuerier<R: Read, W: Write> {
    reader: R,
    writer: W,
    #[allow(dead_code)] // Used for future non-blocking I/O implementation
    timeout: Duration,
}

impl<R: Read, W: Write> LiveQuerier<R, W> {
    /// Create a new live querier with the given reader/writer and timeout.
    ///
    /// # Arguments
    ///
    /// * `reader` - Terminal input stream (typically stdin)
    /// * `writer` - Terminal output stream (typically stdout)
    /// * `timeout` - Maximum time to wait for responses (default: 50ms)
    pub fn new(reader: R, writer: W, timeout: Duration) -> Self {
        Self {
            reader,
            writer,
            timeout,
        }
    }

    /// Send a query and read response with timeout.
    ///
    /// Returns `None` if timeout occurs or I/O error.
    fn query(&mut self, sequence: &[u8]) -> Option<Vec<u8>> {
        // Write query sequence
        if self.writer.write_all(sequence).is_err() {
            return None;
        }
        if self.writer.flush().is_err() {
            return None;
        }

        // Read response with timeout (simplified: just read what's available)
        // In a real implementation, this would use non-blocking I/O or select/poll
        let mut buf = vec![0u8; 256];
        match self.reader.read(&mut buf) {
            Ok(n) if n > 0 => {
                buf.truncate(n);
                Some(buf)
            }
            _ => None,
        }
    }
}

impl<R: Read, W: Write> TerminalQuerier for LiveQuerier<R, W> {
    fn query_color_support(&mut self) -> Option<ColorSupport> {
        let response = self.query(sequences::DA1)?;

        // DA1 responses vary widely. Common patterns:
        // VT100: ESC [ ? 1 ; 2 c
        // xterm-256color: ESC [ ? 6 2 ; ... c
        // Most modern terminals: ESC [ ? 6 4 ; ... c (indicates color support)

        // Look for truecolor indicators in response
        let response_str = String::from_utf8_lossy(&response);
        if response_str.contains("64") || response_str.contains("62") {
            Some(ColorSupport::TrueColor)
        } else if response_str.contains("c") {
            // Got a response but no clear color indicator - assume 256 color
            Some(ColorSupport::Extended256)
        } else {
            None
        }
    }

    fn query_synchronized_output(&mut self) -> Option<bool> {
        let response = self.query(sequences::DECRPM_SYNC_OUTPUT)?;

        // Response: CSI ? 2026 ; Ps $ y
        // Ps: 1=set, 2=reset, 3=permanently set (all mean supported)
        // Ps: 0=not recognized, 4=permanently reset (not supported)
        let response_str = String::from_utf8_lossy(&response);

        // Look for "?2026;1$y" or "?2026;2$y" or "?2026;3$y"
        if response_str.contains("2026;1")
            || response_str.contains("2026;2")
            || response_str.contains("2026;3")
        {
            Some(true)
        } else if response_str.contains("2026;0") || response_str.contains("2026;4") {
            Some(false)
        } else {
            None
        }
    }

    fn query_kitty_keyboard(&mut self) -> Option<bool> {
        let response = self.query(sequences::KITTY_KEYBOARD_QUERY)?;

        // Response: CSI ? flags u (if supported)
        // No response or error means not supported
        let response_str = String::from_utf8_lossy(&response);

        // Look for "?...u" pattern (flags can vary)
        if response_str.contains('?') && response_str.contains('u') {
            Some(true)
        } else {
            Some(false)
        }
    }
}

/// Mock querier for testing.
///
/// Allows pre-configuring responses to queries without needing a real TTY.
pub struct MockQuerier {
    color_support: Option<ColorSupport>,
    synchronized_output: Option<bool>,
    kitty_keyboard: Option<bool>,
}

impl MockQuerier {
    /// Create a new mock querier with all queries returning `None` (timeout).
    pub fn new() -> Self {
        Self {
            color_support: None,
            synchronized_output: None,
            kitty_keyboard: None,
        }
    }

    /// Set the response for color support query.
    pub fn with_color_support(mut self, support: ColorSupport) -> Self {
        self.color_support = Some(support);
        self
    }

    /// Set the response for synchronized output query.
    pub fn with_synchronized_output(mut self, supported: bool) -> Self {
        self.synchronized_output = Some(supported);
        self
    }

    /// Set the response for Kitty keyboard query.
    pub fn with_kitty_keyboard(mut self, supported: bool) -> Self {
        self.kitty_keyboard = Some(supported);
        self
    }
}

impl Default for MockQuerier {
    fn default() -> Self {
        Self::new()
    }
}

impl TerminalQuerier for MockQuerier {
    fn query_color_support(&mut self) -> Option<ColorSupport> {
        self.color_support
    }

    fn query_synchronized_output(&mut self) -> Option<bool> {
        self.synchronized_output
    }

    fn query_kitty_keyboard(&mut self) -> Option<bool> {
        self.kitty_keyboard
    }
}

/// Detect terminal capabilities combining static profiles and dynamic queries.
///
/// This function first gets the static profile for the given terminal kind,
/// then attempts to enhance it with runtime queries. If queries fail or time out,
/// the static profile is used as fallback.
///
/// # Arguments
///
/// * `kind` - The detected terminal kind
/// * `multiplexer` - The detected multiplexer kind
/// * `querier` - A querier implementation for runtime detection
///
/// # Examples
///
/// ```
/// use fae_core::terminal::{TerminalKind, MultiplexerKind, MockQuerier, detect_capabilities, ColorSupport};
///
/// let mut querier = MockQuerier::new()
///     .with_color_support(ColorSupport::TrueColor)
///     .with_synchronized_output(true);
///
/// let caps = detect_capabilities(TerminalKind::Unknown, MultiplexerKind::None, &mut querier);
/// assert_eq!(caps.color, ColorSupport::TrueColor);
/// assert!(caps.synchronized_output);
/// ```
pub fn detect_capabilities(
    kind: TerminalKind,
    multiplexer: MultiplexerKind,
    querier: &mut dyn TerminalQuerier,
) -> TerminalCapabilities {
    // Start with static profile
    let mut caps = profile_for(kind);

    // Enhance with dynamic queries (only override if query succeeds)
    if let Some(color) = querier.query_color_support() {
        caps.color = color;
    }

    if let Some(sync_output) = querier.query_synchronized_output() {
        caps.synchronized_output = sync_output;
    }

    if let Some(kitty_kb) = querier.query_kitty_keyboard() {
        caps.kitty_keyboard = kitty_kb;
    }

    // Apply multiplexer limits last
    merge_multiplexer_limits(caps, multiplexer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_querier_default() {
        let mut querier = MockQuerier::new();
        assert!(querier.query_color_support().is_none());
        assert!(querier.query_synchronized_output().is_none());
        assert!(querier.query_kitty_keyboard().is_none());
    }

    #[test]
    fn test_mock_querier_with_responses() {
        let mut querier = MockQuerier::new()
            .with_color_support(ColorSupport::TrueColor)
            .with_synchronized_output(true)
            .with_kitty_keyboard(false);

        assert_eq!(querier.query_color_support(), Some(ColorSupport::TrueColor));
        assert_eq!(querier.query_synchronized_output(), Some(true));
        assert_eq!(querier.query_kitty_keyboard(), Some(false));
    }

    #[test]
    fn test_detect_capabilities_fallback_to_static() {
        // No query responses - should fall back to static profile
        let mut querier = MockQuerier::new();
        let caps = detect_capabilities(TerminalKind::Kitty, MultiplexerKind::None, &mut querier);

        // Should match static Kitty profile
        assert_eq!(caps.color, ColorSupport::TrueColor);
        assert!(caps.synchronized_output);
        assert!(caps.kitty_keyboard);
    }

    #[test]
    fn test_detect_capabilities_override_with_queries() {
        // Query says no sync output - should override static profile
        let mut querier = MockQuerier::new().with_synchronized_output(false);

        let caps = detect_capabilities(TerminalKind::Kitty, MultiplexerKind::None, &mut querier);

        // Queried value overrides static
        assert!(!caps.synchronized_output);
        // Other static values preserved
        assert!(caps.kitty_keyboard);
    }

    #[test]
    fn test_detect_capabilities_upgrade_unknown_terminal() {
        // Unknown terminal with query-detected capabilities
        let mut querier = MockQuerier::new()
            .with_color_support(ColorSupport::TrueColor)
            .with_synchronized_output(true)
            .with_kitty_keyboard(true);

        let caps = detect_capabilities(TerminalKind::Unknown, MultiplexerKind::None, &mut querier);

        // Query results upgrade the conservative Unknown profile
        assert_eq!(caps.color, ColorSupport::TrueColor);
        assert!(caps.synchronized_output);
        assert!(caps.kitty_keyboard);
    }

    #[test]
    fn test_detect_capabilities_multiplexer_limits_applied() {
        // Even if query says sync output works, tmux disables it
        let mut querier = MockQuerier::new().with_synchronized_output(true);

        let caps = detect_capabilities(TerminalKind::Kitty, MultiplexerKind::Tmux, &mut querier);

        // Multiplexer limit overrides both static and queried value
        assert!(!caps.synchronized_output);
    }

    #[test]
    fn test_detect_capabilities_screen_downgrades_color() {
        // Query says truecolor, but screen limits to 256
        let mut querier = MockQuerier::new().with_color_support(ColorSupport::TrueColor);

        let caps = detect_capabilities(TerminalKind::Kitty, MultiplexerKind::Screen, &mut querier);

        // Screen multiplexer downgrades to 256 color
        assert_eq!(caps.color, ColorSupport::Extended256);
    }

    #[test]
    fn test_detect_capabilities_partial_query_success() {
        // Only some queries succeed
        let mut querier = MockQuerier::new()
            .with_color_support(ColorSupport::Extended256)
            .with_kitty_keyboard(true);
        // synchronized_output query times out (None)

        let caps =
            detect_capabilities(TerminalKind::Alacritty, MultiplexerKind::None, &mut querier);

        // Queried values applied
        assert_eq!(caps.color, ColorSupport::Extended256);
        assert!(caps.kitty_keyboard);
        // Non-queried value from static profile
        assert!(!caps.synchronized_output); // Alacritty static profile
    }

    #[test]
    fn test_live_querier_creation() {
        let input: &[u8] = &[];
        let output: Vec<u8> = Vec::new();
        let timeout = Duration::from_millis(50);

        let _querier = LiveQuerier::new(input, output, timeout);
        // Just verify it compiles and constructs
    }
}
