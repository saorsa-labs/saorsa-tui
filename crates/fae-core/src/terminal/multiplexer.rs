//! Multiplexer escape sequence pass-through wrapping.
//!
//! Terminal multiplexers like tmux and screen require escape sequences to be wrapped
//! in DCS (Device Control String) pass-through sequences so they reach the underlying
//! terminal emulator instead of being interpreted by the multiplexer.

use super::detect::MultiplexerKind;

/// Trait for wrapping escape sequences for multiplexer pass-through.
pub trait EscapeWrapper {
    /// Wrap an escape sequence for pass-through to the underlying terminal.
    ///
    /// If no wrapping is needed, returns the original sequence unchanged.
    fn wrap(&self, seq: &str) -> String;
}

/// Wrapper for tmux multiplexer.
///
/// Tmux requires DCS tmux pass-through: `\x1bPtmux;\x1b{seq}\x1b\\`
/// All `\x1b` in the inner sequence must be doubled.
#[derive(Debug, Clone, Copy)]
pub struct TmuxWrapper;

impl EscapeWrapper for TmuxWrapper {
    fn wrap(&self, seq: &str) -> String {
        // Double all escape characters in the inner sequence
        let escaped = seq.replace('\x1b', "\x1b\x1b");
        format!("\x1bPtmux;\x1b{}\x1b\\", escaped)
    }
}

/// Wrapper for GNU Screen multiplexer.
///
/// Screen requires DCS pass-through: `\x1bP{seq}\x1b\\`
#[derive(Debug, Clone, Copy)]
pub struct ScreenWrapper;

impl EscapeWrapper for ScreenWrapper {
    fn wrap(&self, seq: &str) -> String {
        format!("\x1bP{}\x1b\\", seq)
    }
}

/// Wrapper for Zellij multiplexer.
///
/// Zellij is generally transparent and doesn't require wrapping.
#[derive(Debug, Clone, Copy)]
pub struct ZellijWrapper;

impl EscapeWrapper for ZellijWrapper {
    fn wrap(&self, seq: &str) -> String {
        seq.to_string()
    }
}

/// No-op wrapper when no multiplexer is detected.
#[derive(Debug, Clone, Copy)]
pub struct NoopWrapper;

impl EscapeWrapper for NoopWrapper {
    fn wrap(&self, seq: &str) -> String {
        seq.to_string()
    }
}

/// Get the appropriate wrapper for the given multiplexer kind.
pub fn wrapper_for(kind: MultiplexerKind) -> Box<dyn EscapeWrapper> {
    match kind {
        MultiplexerKind::Tmux => Box::new(TmuxWrapper),
        MultiplexerKind::Screen => Box::new(ScreenWrapper),
        MultiplexerKind::Zellij => Box::new(ZellijWrapper),
        MultiplexerKind::None => Box::new(NoopWrapper),
    }
}

/// Convenience function to wrap an escape sequence for a specific multiplexer.
///
/// # Examples
///
/// ```
/// use fae_core::terminal::{MultiplexerKind, multiplexer::wrap_sequence};
///
/// let seq = "\x1b[?2026h";
/// let wrapped = wrap_sequence(seq, MultiplexerKind::Tmux);
/// assert_eq!(wrapped, "\x1bPtmux;\x1b\x1b\x1b[?2026h\x1b\\");
/// ```
pub fn wrap_sequence(seq: &str, kind: MultiplexerKind) -> String {
    let wrapper = wrapper_for(kind);
    wrapper.wrap(seq)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tmux_wrapper_basic() {
        let wrapper = TmuxWrapper;
        let seq = "\x1b[?2026h";
        let wrapped = wrapper.wrap(seq);
        assert_eq!(wrapped, "\x1bPtmux;\x1b\x1b\x1b[?2026h\x1b\\");
    }

    #[test]
    fn test_tmux_wrapper_multiple_escapes() {
        let wrapper = TmuxWrapper;
        let seq = "\x1b[?2026h\x1b[?2027h";
        let wrapped = wrapper.wrap(seq);
        // Each \x1b in the input becomes \x1b\x1b
        // Input: \x1b[?2026h\x1b[?2027h (2 escape characters)
        // Escaped: \x1b\x1b[?2026h\x1b\x1b[?2027h (both escapes doubled)
        // Wrapped: \x1bPtmux;\x1b + escaped + \x1b\\
        // Result: \x1bPtmux;\x1b\x1b\x1b[?2026h\x1b\x1b[?2027h\x1b\\
        assert_eq!(
            wrapped,
            "\x1bPtmux;\x1b\x1b\x1b[?2026h\x1b\x1b[?2027h\x1b\\"
        );
    }

    #[test]
    fn test_tmux_wrapper_no_escapes() {
        let wrapper = TmuxWrapper;
        let seq = "plain text";
        let wrapped = wrapper.wrap(seq);
        assert_eq!(wrapped, "\x1bPtmux;\x1bplain text\x1b\\");
    }

    #[test]
    fn test_screen_wrapper_basic() {
        let wrapper = ScreenWrapper;
        let seq = "\x1b[?2026h";
        let wrapped = wrapper.wrap(seq);
        assert_eq!(wrapped, "\x1bP\x1b[?2026h\x1b\\");
    }

    #[test]
    fn test_screen_wrapper_multiple_escapes() {
        let wrapper = ScreenWrapper;
        let seq = "\x1b[?2026h\x1b[?2027h";
        let wrapped = wrapper.wrap(seq);
        assert_eq!(wrapped, "\x1bP\x1b[?2026h\x1b[?2027h\x1b\\");
    }

    #[test]
    fn test_zellij_wrapper_passthrough() {
        let wrapper = ZellijWrapper;
        let seq = "\x1b[?2026h";
        let wrapped = wrapper.wrap(seq);
        assert_eq!(wrapped, seq);
    }

    #[test]
    fn test_noop_wrapper_passthrough() {
        let wrapper = NoopWrapper;
        let seq = "\x1b[?2026h";
        let wrapped = wrapper.wrap(seq);
        assert_eq!(wrapped, seq);
    }

    #[test]
    fn test_wrapper_for_tmux() {
        let wrapper = wrapper_for(MultiplexerKind::Tmux);
        let seq = "\x1b[?2026h";
        let wrapped = wrapper.wrap(seq);
        assert_eq!(wrapped, "\x1bPtmux;\x1b\x1b\x1b[?2026h\x1b\\");
    }

    #[test]
    fn test_wrapper_for_screen() {
        let wrapper = wrapper_for(MultiplexerKind::Screen);
        let seq = "\x1b[?2026h";
        let wrapped = wrapper.wrap(seq);
        assert_eq!(wrapped, "\x1bP\x1b[?2026h\x1b\\");
    }

    #[test]
    fn test_wrapper_for_zellij() {
        let wrapper = wrapper_for(MultiplexerKind::Zellij);
        let seq = "\x1b[?2026h";
        let wrapped = wrapper.wrap(seq);
        assert_eq!(wrapped, seq);
    }

    #[test]
    fn test_wrapper_for_none() {
        let wrapper = wrapper_for(MultiplexerKind::None);
        let seq = "\x1b[?2026h";
        let wrapped = wrapper.wrap(seq);
        assert_eq!(wrapped, seq);
    }

    #[test]
    fn test_wrap_sequence_tmux() {
        let seq = "\x1b[?2026h";
        let wrapped = wrap_sequence(seq, MultiplexerKind::Tmux);
        assert_eq!(wrapped, "\x1bPtmux;\x1b\x1b\x1b[?2026h\x1b\\");
    }

    #[test]
    fn test_wrap_sequence_screen() {
        let seq = "\x1b[?2026h";
        let wrapped = wrap_sequence(seq, MultiplexerKind::Screen);
        assert_eq!(wrapped, "\x1bP\x1b[?2026h\x1b\\");
    }

    #[test]
    fn test_wrap_sequence_zellij() {
        let seq = "\x1b[?2026h";
        let wrapped = wrap_sequence(seq, MultiplexerKind::Zellij);
        assert_eq!(wrapped, seq);
    }

    #[test]
    fn test_wrap_sequence_none() {
        let seq = "\x1b[?2026h";
        let wrapped = wrap_sequence(seq, MultiplexerKind::None);
        assert_eq!(wrapped, seq);
    }

    #[test]
    fn test_synchronized_output_sequence() {
        // Test wrapping synchronized output begin sequence
        let begin = "\x1b[?2026h";
        let end = "\x1b[?2026l";

        let begin_wrapped = wrap_sequence(begin, MultiplexerKind::Tmux);
        let end_wrapped = wrap_sequence(end, MultiplexerKind::Tmux);

        assert_eq!(begin_wrapped, "\x1bPtmux;\x1b\x1b\x1b[?2026h\x1b\\");
        assert_eq!(end_wrapped, "\x1bPtmux;\x1b\x1b\x1b[?2026l\x1b\\");
    }

    #[test]
    fn test_empty_sequence() {
        let seq = "";
        let wrapped = wrap_sequence(seq, MultiplexerKind::Tmux);
        assert_eq!(wrapped, "\x1bPtmux;\x1b\x1b\\");
    }
}
