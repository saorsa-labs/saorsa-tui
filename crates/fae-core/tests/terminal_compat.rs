//! Comprehensive terminal compatibility integration tests.
//!
//! This test suite validates terminal detection, capability profiles, multiplexer
//! wrapping, and NO_COLOR support across various terminal emulators.

use fae_core::terminal::{
    ColorSupport, MockQuerier, MultiplexerKind, TerminalKind, detect_capabilities,
    merge_multiplexer_limits, multiplexer::wrap_sequence, profile_for,
};

/// Test iTerm2 detection and capabilities.
#[test]
fn test_iterm2_capabilities() {
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

/// Test Kitty detection with keyboard protocol.
#[test]
fn test_kitty_capabilities_with_keyboard_protocol() {
    let caps = profile_for(TerminalKind::Kitty);

    assert_eq!(caps.color, ColorSupport::TrueColor);
    assert!(caps.unicode);
    assert!(caps.synchronized_output);
    assert!(caps.kitty_keyboard); // Kitty-specific
    assert!(caps.mouse);
    assert!(caps.bracketed_paste);
    assert!(caps.focus_events);
    assert!(caps.hyperlinks);
    assert!(!caps.sixel);
}

/// Test Kitty keyboard protocol runtime detection.
#[test]
fn test_kitty_keyboard_runtime_detection() {
    let mut querier = MockQuerier::new().with_kitty_keyboard(true);

    let caps = detect_capabilities(TerminalKind::Unknown, MultiplexerKind::None, &mut querier);

    // Unknown terminal upgraded by runtime detection
    assert!(caps.kitty_keyboard);
}

/// Test Alacritty detection and color support.
#[test]
fn test_alacritty_capabilities() {
    let caps = profile_for(TerminalKind::Alacritty);

    assert_eq!(caps.color, ColorSupport::TrueColor);
    assert!(caps.unicode);
    assert!(!caps.synchronized_output); // Alacritty doesn't support sync output
    assert!(!caps.kitty_keyboard);
    assert!(caps.mouse);
    assert!(caps.bracketed_paste);
    assert!(caps.focus_events);
    assert!(!caps.hyperlinks); // No hyperlink support
    assert!(!caps.sixel);
}

/// Test WezTerm detection with full feature set.
#[test]
fn test_wezterm_capabilities() {
    let caps = profile_for(TerminalKind::WezTerm);

    assert_eq!(caps.color, ColorSupport::TrueColor);
    assert!(caps.unicode);
    assert!(caps.synchronized_output);
    assert!(caps.kitty_keyboard);
    assert!(caps.mouse);
    assert!(caps.bracketed_paste);
    assert!(caps.focus_events);
    assert!(caps.hyperlinks);
    assert!(caps.sixel); // WezTerm supports Sixel graphics
}

/// Test Terminal.app with limited capabilities.
#[test]
fn test_terminal_app_limited_capabilities() {
    let caps = profile_for(TerminalKind::TerminalApp);

    assert_eq!(caps.color, ColorSupport::Basic16); // Only 16 colors
    assert!(caps.unicode);
    assert!(!caps.synchronized_output);
    assert!(!caps.kitty_keyboard);
    assert!(caps.mouse);
    assert!(!caps.bracketed_paste); // No bracketed paste
    assert!(!caps.focus_events);
    assert!(!caps.hyperlinks);
    assert!(!caps.sixel);
}

/// Test tmux pass-through wrapping.
#[test]
fn test_tmux_passthrough_wrapping() {
    let seq = "\x1b[?2026h";
    let wrapped = wrap_sequence(seq, MultiplexerKind::Tmux);

    // Tmux requires DCS tmux pass-through with doubled escapes
    assert_eq!(wrapped, "\x1bPtmux;\x1b\x1b\x1b[?2026h\x1b\\");

    // Verify multiple escape characters are all doubled
    let multi_escape = "\x1b[?2026h\x1b[?2027h";
    let multi_wrapped = wrap_sequence(multi_escape, MultiplexerKind::Tmux);
    assert_eq!(
        multi_wrapped,
        "\x1bPtmux;\x1b\x1b\x1b[?2026h\x1b\x1b[?2027h\x1b\\"
    );
}

/// Test tmux capability limits.
#[test]
fn test_tmux_capability_limits() {
    let kitty = profile_for(TerminalKind::Kitty);
    assert!(kitty.synchronized_output);

    let tmux_kitty = merge_multiplexer_limits(kitty, MultiplexerKind::Tmux);

    // Tmux disables synchronized output
    assert!(!tmux_kitty.synchronized_output);
    // But preserves other capabilities
    assert!(tmux_kitty.kitty_keyboard);
    assert!(tmux_kitty.hyperlinks);
    assert_eq!(tmux_kitty.color, ColorSupport::TrueColor);
}

/// Test screen pass-through wrapping.
#[test]
fn test_screen_passthrough_wrapping() {
    let seq = "\x1b[?2026h";
    let wrapped = wrap_sequence(seq, MultiplexerKind::Screen);

    // Screen uses simpler DCS pass-through (no escape doubling)
    assert_eq!(wrapped, "\x1bP\x1b[?2026h\x1b\\");
}

/// Test screen capability limits.
#[test]
fn test_screen_capability_limits() {
    let kitty = profile_for(TerminalKind::Kitty);

    let screen_kitty = merge_multiplexer_limits(kitty, MultiplexerKind::Screen);

    // Screen severely limits capabilities
    assert_eq!(screen_kitty.color, ColorSupport::Extended256); // Downgrade from TrueColor
    assert!(!screen_kitty.synchronized_output);
    assert!(!screen_kitty.kitty_keyboard);
    assert!(!screen_kitty.hyperlinks);
    assert!(!screen_kitty.sixel);
    // Basic features still work
    assert!(screen_kitty.unicode);
    assert!(screen_kitty.mouse);
}

/// Test nested multiplexer detection (tmux inside screen).
#[test]
fn test_nested_multiplexer_scenarios() {
    // Scenario: Kitty terminal running inside tmux
    let kitty = profile_for(TerminalKind::Kitty);
    let tmux_kitty = merge_multiplexer_limits(kitty.clone(), MultiplexerKind::Tmux);

    // Now imagine that's inside screen (double nesting)
    let screen_tmux_kitty = merge_multiplexer_limits(tmux_kitty.clone(), MultiplexerKind::Screen);

    // Screen is more limiting than tmux
    assert_eq!(screen_tmux_kitty.color, ColorSupport::Extended256);
    assert!(!screen_tmux_kitty.synchronized_output);
    assert!(!screen_tmux_kitty.kitty_keyboard);
    assert!(!screen_tmux_kitty.hyperlinks);

    // Verify that applying screen limits directly equals nested application
    let screen_kitty_direct = merge_multiplexer_limits(kitty, MultiplexerKind::Screen);
    assert_eq!(screen_tmux_kitty, screen_kitty_direct);
}

/// Test NO_COLOR environment variable respect.
#[test]
fn test_no_color_environment_variable() {
    use fae_core::buffer::CellChange;
    use fae_core::cell::Cell;
    use fae_core::color::Color;
    use fae_core::renderer::Renderer;
    use fae_core::style::Style;

    let style = Style::default().fg(Color::Rgb { r: 255, g: 0, b: 0 }); // Red
    let changes = vec![CellChange {
        x: 0,
        y: 0,
        cell: Cell::new("X", style),
    }];

    // Render without NO_COLOR
    let without_no_color = {
        // Ensure NO_COLOR is not set
        unsafe {
            std::env::remove_var("NO_COLOR");
        }
        let renderer = Renderer::new(ColorSupport::TrueColor, false);
        renderer.render(&changes)
    };

    // Render with NO_COLOR
    let with_no_color = {
        unsafe {
            std::env::set_var("NO_COLOR", "1");
        }
        let renderer = Renderer::new(ColorSupport::TrueColor, false);
        let result = renderer.render(&changes);
        unsafe {
            std::env::remove_var("NO_COLOR");
        }
        result
    };

    // With NO_COLOR, output should not contain RGB color codes
    assert_ne!(without_no_color, with_no_color);
    assert!(without_no_color.contains("\x1b[38;2;")); // Contains RGB truecolor
    assert!(!with_no_color.contains("\x1b[38;2;")); // NO_COLOR strips RGB codes
    assert!(with_no_color.contains("\x1b[39m")); // Uses reset instead
}

/// Test runtime capability detection upgrades unknown terminal.
#[test]
fn test_runtime_detection_upgrades_unknown_terminal() {
    let mut querier = MockQuerier::new()
        .with_color_support(ColorSupport::TrueColor)
        .with_synchronized_output(true)
        .with_kitty_keyboard(true);

    let caps = detect_capabilities(TerminalKind::Unknown, MultiplexerKind::None, &mut querier);

    // Unknown terminal should be upgraded by runtime detection
    assert_eq!(caps.color, ColorSupport::TrueColor);
    assert!(caps.synchronized_output);
    assert!(caps.kitty_keyboard);
}

/// Test partial query success with fallback to static profile.
#[test]
fn test_partial_query_success() {
    let mut querier = MockQuerier::new()
        .with_color_support(ColorSupport::Extended256)
        .with_kitty_keyboard(true);
    // synchronized_output query times out (not set)

    let caps = detect_capabilities(TerminalKind::Alacritty, MultiplexerKind::None, &mut querier);

    // Queried values applied
    assert_eq!(caps.color, ColorSupport::Extended256);
    assert!(caps.kitty_keyboard);
    // Non-queried value from static profile
    assert!(!caps.synchronized_output); // Alacritty static profile default
}

/// Test multiplexer limits override runtime detection.
#[test]
fn test_multiplexer_limits_override_runtime_detection() {
    // Even if runtime query says sync output is supported, tmux disables it
    let mut querier = MockQuerier::new().with_synchronized_output(true);

    let caps = detect_capabilities(TerminalKind::Kitty, MultiplexerKind::Tmux, &mut querier);

    // Multiplexer limit overrides both static and queried value
    assert!(!caps.synchronized_output);
}

/// Test Zellij multiplexer transparency (no wrapping needed).
#[test]
fn test_zellij_transparency() {
    let seq = "\x1b[?2026h";
    let wrapped = wrap_sequence(seq, MultiplexerKind::Zellij);

    // Zellij is transparent - no wrapping
    assert_eq!(wrapped, seq);

    // Verify capabilities are preserved
    let kitty = profile_for(TerminalKind::Kitty);
    let zellij_kitty = merge_multiplexer_limits(kitty.clone(), MultiplexerKind::Zellij);

    assert_eq!(kitty, zellij_kitty); // Identical
}

/// Test conservative unknown terminal profile.
#[test]
fn test_unknown_terminal_conservative_profile() {
    let caps = profile_for(TerminalKind::Unknown);

    // Unknown terminals get the most conservative profile
    assert_eq!(caps.color, ColorSupport::Basic16);
    assert!(caps.unicode); // Unicode is generally safe
    assert!(!caps.synchronized_output);
    assert!(!caps.kitty_keyboard);
    assert!(caps.mouse); // Mouse is usually safe
    assert!(!caps.bracketed_paste);
    assert!(!caps.focus_events);
    assert!(!caps.hyperlinks);
    assert!(!caps.sixel);
}

/// Test VTE-based terminal profile.
#[test]
fn test_vte_terminal_profile() {
    let caps = profile_for(TerminalKind::VTE);

    // VTE terminals (GNOME Terminal, Tilix, etc.)
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

/// Test xterm profile with 256 colors.
#[test]
fn test_xterm_profile() {
    let caps = profile_for(TerminalKind::Xterm);

    assert_eq!(caps.color, ColorSupport::Extended256); // Not TrueColor
    assert!(caps.unicode);
    assert!(!caps.synchronized_output);
    assert!(!caps.kitty_keyboard);
    assert!(caps.mouse);
    assert!(caps.bracketed_paste);
    assert!(caps.focus_events);
    assert!(!caps.hyperlinks);
    assert!(!caps.sixel);
}

/// Test screen preserves 256 color when not downgrading from TrueColor.
#[test]
fn test_screen_preserves_256_color() {
    let xterm = profile_for(TerminalKind::Xterm);
    assert_eq!(xterm.color, ColorSupport::Extended256);

    let screen_xterm = merge_multiplexer_limits(xterm, MultiplexerKind::Screen);

    // Screen should preserve 256 color (only downgrades TrueColor)
    assert_eq!(screen_xterm.color, ColorSupport::Extended256);
}

/// Test Windows Terminal profile.
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
