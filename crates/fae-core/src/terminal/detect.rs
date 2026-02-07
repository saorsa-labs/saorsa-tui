//! Terminal and multiplexer detection from environment variables.

use std::env;

/// The kind of terminal emulator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum TerminalKind {
    /// iTerm2 on macOS.
    ITerm2,
    /// Kitty terminal.
    Kitty,
    /// Alacritty terminal.
    Alacritty,
    /// WezTerm terminal.
    WezTerm,
    /// macOS Terminal.app.
    TerminalApp,
    /// Windows Terminal.
    WindowsTerminal,
    /// Xterm or compatible.
    Xterm,
    /// VTE-based terminal (GNOME Terminal, Tilix, etc.).
    VTE,
    /// Unknown or undetected terminal.
    Unknown,
}

/// The kind of terminal multiplexer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum MultiplexerKind {
    /// Tmux multiplexer.
    Tmux,
    /// GNU Screen multiplexer.
    Screen,
    /// Zellij multiplexer.
    Zellij,
    /// No multiplexer detected.
    None,
}

/// Combined information about the terminal environment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalInfo {
    /// The detected terminal emulator.
    pub kind: TerminalKind,
    /// The detected multiplexer, if any.
    pub multiplexer: MultiplexerKind,
    /// The version string, if available.
    pub version: Option<String>,
}

/// Detect the terminal emulator from environment variables.
///
/// Checks `TERM_PROGRAM`, `TERM`, `WT_SESSION`, and other environment variables
/// to identify the running terminal.
pub fn detect_terminal() -> TerminalKind {
    let term_program = env::var("TERM_PROGRAM").ok();
    let term = env::var("TERM").ok();
    let wt_session = env::var("WT_SESSION").ok();
    let vte_version = env::var("VTE_VERSION").ok();

    detect_terminal_from_vars(
        term_program.as_deref(),
        term.as_deref(),
        wt_session.as_deref(),
        vte_version.as_deref(),
    )
}

/// Internal function to detect terminal from explicit environment variable values.
///
/// This allows testing without modifying the global environment.
fn detect_terminal_from_vars(
    term_program: Option<&str>,
    term: Option<&str>,
    wt_session: Option<&str>,
    vte_version: Option<&str>,
) -> TerminalKind {
    // Check TERM_PROGRAM first (most specific)
    if let Some(program) = term_program {
        match program {
            "iTerm.app" => return TerminalKind::ITerm2,
            "Apple_Terminal" => return TerminalKind::TerminalApp,
            "WezTerm" => return TerminalKind::WezTerm,
            _ => {}
        }
    }

    // Check WT_SESSION for Windows Terminal
    if wt_session.is_some() {
        return TerminalKind::WindowsTerminal;
    }

    // Check VTE_VERSION for VTE-based terminals
    if vte_version.is_some() {
        return TerminalKind::VTE;
    }

    // Check TERM as fallback
    if let Some(term_value) = term {
        if term_value.contains("kitty") {
            return TerminalKind::Kitty;
        }
        if term_value.contains("alacritty") {
            return TerminalKind::Alacritty;
        }
        if term_value.starts_with("xterm") {
            return TerminalKind::Xterm;
        }
    }

    TerminalKind::Unknown
}

/// Detect the terminal multiplexer from environment variables.
///
/// Checks `TMUX`, `STY`, and `ZELLIJ` environment variables.
pub fn detect_multiplexer() -> MultiplexerKind {
    let tmux = env::var("TMUX").ok();
    let sty = env::var("STY").ok();
    let zellij = env::var("ZELLIJ").ok();

    detect_multiplexer_from_vars(tmux.as_deref(), sty.as_deref(), zellij.as_deref())
}

/// Internal function to detect multiplexer from explicit environment variable values.
///
/// This allows testing without modifying the global environment.
fn detect_multiplexer_from_vars(
    tmux: Option<&str>,
    sty: Option<&str>,
    zellij: Option<&str>,
) -> MultiplexerKind {
    if tmux.is_some() {
        return MultiplexerKind::Tmux;
    }
    if sty.is_some() {
        return MultiplexerKind::Screen;
    }
    if zellij.is_some() {
        return MultiplexerKind::Zellij;
    }
    MultiplexerKind::None
}

/// Detect terminal and multiplexer information.
///
/// Combines terminal emulator detection and multiplexer detection into a single
/// `TerminalInfo` structure. The version field is populated from `TERM_PROGRAM_VERSION`
/// if available.
pub fn detect() -> TerminalInfo {
    let kind = detect_terminal();
    let multiplexer = detect_multiplexer();
    let version = env::var("TERM_PROGRAM_VERSION").ok();

    TerminalInfo {
        kind,
        multiplexer,
        version,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_iterm2() {
        let kind = detect_terminal_from_vars(Some("iTerm.app"), None, None, None);
        assert!(matches!(kind, TerminalKind::ITerm2));
    }

    #[test]
    fn test_detect_terminal_app() {
        let kind = detect_terminal_from_vars(Some("Apple_Terminal"), None, None, None);
        assert!(matches!(kind, TerminalKind::TerminalApp));
    }

    #[test]
    fn test_detect_wezterm() {
        let kind = detect_terminal_from_vars(Some("WezTerm"), None, None, None);
        assert!(matches!(kind, TerminalKind::WezTerm));
    }

    #[test]
    fn test_detect_windows_terminal() {
        let kind = detect_terminal_from_vars(None, None, Some("session-id"), None);
        assert!(matches!(kind, TerminalKind::WindowsTerminal));
    }

    #[test]
    fn test_detect_vte() {
        let kind = detect_terminal_from_vars(None, None, None, Some("5200"));
        assert!(matches!(kind, TerminalKind::VTE));
    }

    #[test]
    fn test_detect_kitty_from_term() {
        let kind = detect_terminal_from_vars(None, Some("xterm-kitty"), None, None);
        assert!(matches!(kind, TerminalKind::Kitty));
    }

    #[test]
    fn test_detect_alacritty_from_term() {
        let kind = detect_terminal_from_vars(None, Some("alacritty"), None, None);
        assert!(matches!(kind, TerminalKind::Alacritty));
    }

    #[test]
    fn test_detect_xterm() {
        let kind = detect_terminal_from_vars(None, Some("xterm-256color"), None, None);
        assert!(matches!(kind, TerminalKind::Xterm));
    }

    #[test]
    fn test_detect_unknown() {
        let kind = detect_terminal_from_vars(None, None, None, None);
        assert!(matches!(kind, TerminalKind::Unknown));
    }

    #[test]
    fn test_detect_tmux() {
        let kind = detect_multiplexer_from_vars(Some("/tmp/tmux-1000/default,12345,0"), None, None);
        assert!(matches!(kind, MultiplexerKind::Tmux));
    }

    #[test]
    fn test_detect_screen() {
        let kind = detect_multiplexer_from_vars(None, Some("12345.pts-0.hostname"), None);
        assert!(matches!(kind, MultiplexerKind::Screen));
    }

    #[test]
    fn test_detect_zellij() {
        let kind = detect_multiplexer_from_vars(None, None, Some("0"));
        assert!(matches!(kind, MultiplexerKind::Zellij));
    }

    #[test]
    fn test_detect_no_multiplexer() {
        let kind = detect_multiplexer_from_vars(None, None, None);
        assert!(matches!(kind, MultiplexerKind::None));
    }
}
