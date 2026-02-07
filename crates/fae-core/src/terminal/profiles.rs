//! Terminal capability profiles for known terminal emulators.

use super::detect::{MultiplexerKind, TerminalKind};
use super::traits::{ColorSupport, TerminalCapabilities};

/// Returns the default capability profile for a known terminal emulator.
///
/// Each profile is based on documented capabilities and real-world testing.
/// Unknown terminals receive a conservative baseline profile.
pub fn profile_for(kind: TerminalKind) -> TerminalCapabilities {
    match kind {
        TerminalKind::ITerm2 => TerminalCapabilities {
            color: ColorSupport::TrueColor,
            unicode: true,
            synchronized_output: true,
            kitty_keyboard: false,
            mouse: true,
            bracketed_paste: true,
            focus_events: true,
            hyperlinks: true,
            sixel: false,
        },
        TerminalKind::Kitty => TerminalCapabilities {
            color: ColorSupport::TrueColor,
            unicode: true,
            synchronized_output: true,
            kitty_keyboard: true,
            mouse: true,
            bracketed_paste: true,
            focus_events: true,
            hyperlinks: true,
            sixel: false,
        },
        TerminalKind::Alacritty => TerminalCapabilities {
            color: ColorSupport::TrueColor,
            unicode: true,
            synchronized_output: false,
            kitty_keyboard: false,
            mouse: true,
            bracketed_paste: true,
            focus_events: true,
            hyperlinks: false,
            sixel: false,
        },
        TerminalKind::WezTerm => TerminalCapabilities {
            color: ColorSupport::TrueColor,
            unicode: true,
            synchronized_output: true,
            kitty_keyboard: true,
            mouse: true,
            bracketed_paste: true,
            focus_events: true,
            hyperlinks: true,
            sixel: true,
        },
        TerminalKind::TerminalApp => TerminalCapabilities {
            color: ColorSupport::Basic16,
            unicode: true,
            synchronized_output: false,
            kitty_keyboard: false,
            mouse: true,
            bracketed_paste: false,
            focus_events: false,
            hyperlinks: false,
            sixel: false,
        },
        TerminalKind::WindowsTerminal => TerminalCapabilities {
            color: ColorSupport::TrueColor,
            unicode: true,
            synchronized_output: false,
            kitty_keyboard: false,
            mouse: true,
            bracketed_paste: true,
            focus_events: false,
            hyperlinks: true,
            sixel: false,
        },
        TerminalKind::Xterm => TerminalCapabilities {
            color: ColorSupport::Extended256,
            unicode: true,
            synchronized_output: false,
            kitty_keyboard: false,
            mouse: true,
            bracketed_paste: true,
            focus_events: true,
            hyperlinks: false,
            sixel: false,
        },
        TerminalKind::VTE => TerminalCapabilities {
            color: ColorSupport::TrueColor,
            unicode: true,
            synchronized_output: false,
            kitty_keyboard: false,
            mouse: true,
            bracketed_paste: true,
            focus_events: true,
            hyperlinks: true,
            sixel: false,
        },
        TerminalKind::Unknown => TerminalCapabilities {
            color: ColorSupport::Basic16,
            unicode: true,
            synchronized_output: false,
            kitty_keyboard: false,
            mouse: true,
            bracketed_paste: false,
            focus_events: false,
            hyperlinks: false,
            sixel: false,
        },
    }
}

/// Applies multiplexer-specific capability limits to a terminal profile.
///
/// Terminal multiplexers like tmux and screen may intercept or not support
/// certain terminal features. This function adjusts capabilities accordingly.
///
/// # Examples
///
/// ```
/// use fae_core::terminal::{TerminalKind, MultiplexerKind, profile_for, merge_multiplexer_limits};
///
/// let kitty = profile_for(TerminalKind::Kitty);
/// let tmux_kitty = merge_multiplexer_limits(kitty, MultiplexerKind::Tmux);
/// // tmux may limit synchronized output even if Kitty supports it
/// ```
pub fn merge_multiplexer_limits(
    mut caps: TerminalCapabilities,
    multiplexer: MultiplexerKind,
) -> TerminalCapabilities {
    match multiplexer {
        MultiplexerKind::None => {
            // No multiplexer, return capabilities unchanged
        }
        MultiplexerKind::Tmux => {
            // tmux may not forward synchronized output reliably
            caps.synchronized_output = false;
            // tmux generally passes through other features
        }
        MultiplexerKind::Screen => {
            // screen is quite limited
            if caps.color == ColorSupport::TrueColor {
                caps.color = ColorSupport::Extended256;
            }
            caps.synchronized_output = false;
            caps.kitty_keyboard = false;
            caps.hyperlinks = false;
            caps.sixel = false;
        }
        MultiplexerKind::Zellij => {
            // Zellij is modern and mostly transparent
            // No specific limitations known
        }
    }
    caps
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iterm2_profile() {
        let caps = profile_for(TerminalKind::ITerm2);
        assert_eq!(caps.color, ColorSupport::TrueColor);
        assert!(caps.unicode);
        assert!(caps.synchronized_output);
        assert!(!caps.kitty_keyboard);
        assert!(caps.mouse);
        assert!(caps.bracketed_paste);
        assert!(caps.focus_events);
        assert!(caps.hyperlinks);
        assert!(!caps.sixel);
    }

    #[test]
    fn test_kitty_profile() {
        let caps = profile_for(TerminalKind::Kitty);
        assert_eq!(caps.color, ColorSupport::TrueColor);
        assert!(caps.unicode);
        assert!(caps.synchronized_output);
        assert!(caps.kitty_keyboard);
        assert!(caps.mouse);
        assert!(caps.bracketed_paste);
        assert!(caps.focus_events);
        assert!(caps.hyperlinks);
        assert!(!caps.sixel);
    }

    #[test]
    fn test_alacritty_profile() {
        let caps = profile_for(TerminalKind::Alacritty);
        assert_eq!(caps.color, ColorSupport::TrueColor);
        assert!(caps.unicode);
        assert!(!caps.synchronized_output);
        assert!(!caps.kitty_keyboard);
        assert!(caps.mouse);
        assert!(caps.bracketed_paste);
        assert!(caps.focus_events);
        assert!(!caps.hyperlinks);
        assert!(!caps.sixel);
    }

    #[test]
    fn test_wezterm_profile() {
        let caps = profile_for(TerminalKind::WezTerm);
        assert_eq!(caps.color, ColorSupport::TrueColor);
        assert!(caps.unicode);
        assert!(caps.synchronized_output);
        assert!(caps.kitty_keyboard);
        assert!(caps.mouse);
        assert!(caps.bracketed_paste);
        assert!(caps.focus_events);
        assert!(caps.hyperlinks);
        assert!(caps.sixel);
    }

    #[test]
    fn test_terminal_app_profile() {
        let caps = profile_for(TerminalKind::TerminalApp);
        assert_eq!(caps.color, ColorSupport::Basic16);
        assert!(caps.unicode);
        assert!(!caps.synchronized_output);
        assert!(!caps.kitty_keyboard);
        assert!(caps.mouse);
        assert!(!caps.bracketed_paste);
        assert!(!caps.focus_events);
        assert!(!caps.hyperlinks);
        assert!(!caps.sixel);
    }

    #[test]
    fn test_windows_terminal_profile() {
        let caps = profile_for(TerminalKind::WindowsTerminal);
        assert_eq!(caps.color, ColorSupport::TrueColor);
        assert!(caps.unicode);
        assert!(!caps.synchronized_output);
        assert!(!caps.kitty_keyboard);
        assert!(caps.mouse);
        assert!(caps.bracketed_paste);
        assert!(!caps.focus_events);
        assert!(caps.hyperlinks);
        assert!(!caps.sixel);
    }

    #[test]
    fn test_xterm_profile() {
        let caps = profile_for(TerminalKind::Xterm);
        assert_eq!(caps.color, ColorSupport::Extended256);
        assert!(caps.unicode);
        assert!(!caps.synchronized_output);
        assert!(!caps.kitty_keyboard);
        assert!(caps.mouse);
        assert!(caps.bracketed_paste);
        assert!(caps.focus_events);
        assert!(!caps.hyperlinks);
        assert!(!caps.sixel);
    }

    #[test]
    fn test_vte_profile() {
        let caps = profile_for(TerminalKind::VTE);
        assert_eq!(caps.color, ColorSupport::TrueColor);
        assert!(caps.unicode);
        assert!(!caps.synchronized_output);
        assert!(!caps.kitty_keyboard);
        assert!(caps.mouse);
        assert!(caps.bracketed_paste);
        assert!(caps.focus_events);
        assert!(caps.hyperlinks);
        assert!(!caps.sixel);
    }

    #[test]
    fn test_unknown_profile_conservative() {
        let caps = profile_for(TerminalKind::Unknown);
        assert_eq!(caps.color, ColorSupport::Basic16);
        assert!(caps.unicode);
        assert!(!caps.synchronized_output);
        assert!(!caps.kitty_keyboard);
        assert!(caps.mouse);
        assert!(!caps.bracketed_paste);
        assert!(!caps.focus_events);
        assert!(!caps.hyperlinks);
        assert!(!caps.sixel);
    }

    #[test]
    fn test_tmux_limits() {
        let kitty = profile_for(TerminalKind::Kitty);
        assert!(kitty.synchronized_output);

        let tmux_kitty = merge_multiplexer_limits(kitty.clone(), MultiplexerKind::Tmux);
        assert!(!tmux_kitty.synchronized_output);
        // Other capabilities should be preserved
        assert!(tmux_kitty.kitty_keyboard);
        assert!(tmux_kitty.hyperlinks);
        assert_eq!(tmux_kitty.color, ColorSupport::TrueColor);
    }

    #[test]
    fn test_screen_limits() {
        let kitty = profile_for(TerminalKind::Kitty);
        let screen_kitty = merge_multiplexer_limits(kitty, MultiplexerKind::Screen);

        assert_eq!(screen_kitty.color, ColorSupport::Extended256);
        assert!(!screen_kitty.synchronized_output);
        assert!(!screen_kitty.kitty_keyboard);
        assert!(!screen_kitty.hyperlinks);
        assert!(!screen_kitty.sixel);
        // Basic features still work
        assert!(screen_kitty.unicode);
        assert!(screen_kitty.mouse);
    }

    #[test]
    fn test_screen_limits_256color_preserved() {
        let xterm = profile_for(TerminalKind::Xterm);
        assert_eq!(xterm.color, ColorSupport::Extended256);

        let screen_xterm = merge_multiplexer_limits(xterm, MultiplexerKind::Screen);
        assert_eq!(screen_xterm.color, ColorSupport::Extended256);
    }

    #[test]
    fn test_zellij_transparent() {
        let kitty = profile_for(TerminalKind::Kitty);
        let zellij_kitty = merge_multiplexer_limits(kitty.clone(), MultiplexerKind::Zellij);

        // Zellij should preserve all capabilities
        assert_eq!(kitty, zellij_kitty);
    }

    #[test]
    fn test_no_multiplexer() {
        let kitty = profile_for(TerminalKind::Kitty);
        let no_mux_kitty = merge_multiplexer_limits(kitty.clone(), MultiplexerKind::None);

        // No multiplexer should preserve all capabilities
        assert_eq!(kitty, no_mux_kitty);
    }
}
