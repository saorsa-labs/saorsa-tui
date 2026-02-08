//! Integration tests for the terminal compatibility system.

use saorsa_tui::buffer::{CellChange, ScreenBuffer};
use saorsa_tui::cell::Cell;
use saorsa_tui::color::{Color, NamedColor};
use saorsa_tui::geometry::Size;
use saorsa_tui::renderer::{Renderer, build_sgr_sequence};
use saorsa_tui::style::Style;
use saorsa_tui::terminal::multiplexer::wrap_sequence;
use saorsa_tui::terminal::{
    ColorSupport, MockQuerier, MultiplexerKind, TerminalKind, detect_capabilities, profile_for,
};

/// Test full render cycle with mock terminal capabilities.
#[test]
fn test_full_render_cycle_with_capabilities() {
    // Detect capabilities using mock querier
    let mut querier = MockQuerier::new()
        .with_color_support(ColorSupport::TrueColor)
        .with_synchronized_output(true)
        .with_kitty_keyboard(false);

    let caps = detect_capabilities(TerminalKind::Unknown, MultiplexerKind::None, &mut querier);

    // Create renderer with detected capabilities
    let renderer = Renderer::new(caps.color, caps.synchronized_output);

    // Create test screen buffers
    let mut current = ScreenBuffer::new(Size::new(80, 24));
    let previous = ScreenBuffer::new(Size::new(80, 24));

    let style = Style::new().fg(Color::Rgb {
        r: 255,
        g: 100,
        b: 50,
    });
    current.set(0, 0, Cell::new("H", style.clone()));
    current.set(1, 0, Cell::new("i", style));

    // Get buffer changes
    let changes = current.diff(&previous);

    // Render the changes
    let output = renderer.render(&changes);

    // Verify output contains expected elements
    assert!(output.starts_with("\x1b[?2026h")); // Synchronized output begin
    assert!(output.contains("\x1b[1;1H")); // Cursor position
    assert!(output.contains("\x1b[38;2;255;100;50m")); // TrueColor foreground
    assert!(output.contains('H'));
    assert!(output.contains('i'));
    assert!(output.ends_with("\x1b[?2026l")); // Synchronized output end
}

/// Test color downgrade chain: TrueColor → 256 → 16 → none.
#[test]
fn test_color_downgrade_chain() {
    let rgb_color = Color::Rgb { r: 255, g: 0, b: 0 };
    let style = Style::new().fg(rgb_color);
    let cell = Cell::new("X", style);
    let changes = vec![CellChange { x: 0, y: 0, cell }];

    // TrueColor: RGB passes through
    let renderer_true = Renderer::new(ColorSupport::TrueColor, false);
    let output_true = renderer_true.render(&changes);
    assert!(output_true.contains("\x1b[38;2;255;0;0m"));

    // 256-color: RGB converted to indexed
    let renderer_256 = Renderer::new(ColorSupport::Extended256, false);
    let output_256 = renderer_256.render(&changes);
    assert!(output_256.contains("\x1b[38;5;")); // Indexed color format
    assert!(!output_256.contains("\x1b[38;2;")); // No RGB

    // 16-color: RGB converted to named color
    let renderer_16 = Renderer::new(ColorSupport::Basic16, false);
    let output_16 = renderer_16.render(&changes);
    assert!(output_16.contains("\x1b[91m")); // Bright red
    assert!(!output_16.contains("\x1b[38;2;")); // No RGB
    assert!(!output_16.contains("\x1b[38;5;")); // No indexed

    // NoColor: All colors stripped
    let renderer_none = Renderer::new(ColorSupport::NoColor, false);
    let output_none = renderer_none.render(&changes);
    assert!(output_none.contains("\x1b[39m")); // Color reset
    assert!(!output_none.contains("\x1b[38;2;")); // No RGB
    assert!(!output_none.contains("\x1b[38;5;")); // No indexed
    assert!(!output_none.contains("\x1b[91m")); // No named color
}

/// Test capability override via dynamic queries.
#[test]
fn test_capability_override_works() {
    // Unknown terminal starts with conservative capabilities
    let static_caps = profile_for(TerminalKind::Unknown);
    assert_eq!(static_caps.color, ColorSupport::Basic16);
    assert!(!static_caps.synchronized_output);
    assert!(!static_caps.kitty_keyboard);

    // Query detects better capabilities
    let mut querier = MockQuerier::new()
        .with_color_support(ColorSupport::TrueColor)
        .with_synchronized_output(true)
        .with_kitty_keyboard(true);

    let detected_caps =
        detect_capabilities(TerminalKind::Unknown, MultiplexerKind::None, &mut querier);

    // Capabilities upgraded via queries
    assert_eq!(detected_caps.color, ColorSupport::TrueColor);
    assert!(detected_caps.synchronized_output);
    assert!(detected_caps.kitty_keyboard);

    // Static capabilities from profile still preserved
    assert!(detected_caps.unicode);
    assert!(detected_caps.mouse);
}

/// Test multiplexer wrapping produces valid escape sequences.
#[test]
fn test_multiplexer_wrapping_valid_output() {
    // Synchronized output sequences
    let sync_begin = "\x1b[?2026h";
    let sync_end = "\x1b[?2026l";

    // Test Tmux wrapping
    let tmux_begin = wrap_sequence(sync_begin, MultiplexerKind::Tmux);
    let tmux_end = wrap_sequence(sync_end, MultiplexerKind::Tmux);

    // Tmux wrapping format: \x1bPtmux;\x1b{escaped_seq}\x1b\\
    // Escapes in seq are doubled
    assert_eq!(tmux_begin, "\x1bPtmux;\x1b\x1b\x1b[?2026h\x1b\\");
    assert_eq!(tmux_end, "\x1bPtmux;\x1b\x1b\x1b[?2026l\x1b\\");

    // Test Screen wrapping
    let screen_begin = wrap_sequence(sync_begin, MultiplexerKind::Screen);
    let screen_end = wrap_sequence(sync_end, MultiplexerKind::Screen);

    // Screen wrapping format: \x1bP{seq}\x1b\\
    assert_eq!(screen_begin, "\x1bP\x1b[?2026h\x1b\\");
    assert_eq!(screen_end, "\x1bP\x1b[?2026l\x1b\\");

    // Test Zellij (transparent)
    let zellij_begin = wrap_sequence(sync_begin, MultiplexerKind::Zellij);
    assert_eq!(zellij_begin, sync_begin);

    // Test None (no wrapping)
    let none_begin = wrap_sequence(sync_begin, MultiplexerKind::None);
    assert_eq!(none_begin, sync_begin);
}

/// Test combined SGR sequences with different color support levels.
#[test]
fn test_combined_sgr_with_color_downgrade() {
    let style = Style::new()
        .bold(true)
        .italic(true)
        .fg(Color::Rgb {
            r: 100,
            g: 200,
            b: 50,
        })
        .bg(Color::Named(NamedColor::Blue));

    // TrueColor: RGB preserved
    let sgr_true = build_sgr_sequence(&style, ColorSupport::TrueColor);
    assert!(sgr_true.contains("1;")); // Bold
    assert!(sgr_true.contains(";3;")); // Italic
    assert!(sgr_true.contains("38;2;100;200;50")); // RGB foreground
    assert!(sgr_true.contains("44")); // Blue background

    // 256-color: RGB downgraded to indexed
    let sgr_256 = build_sgr_sequence(&style, ColorSupport::Extended256);
    assert!(sgr_256.contains("1;")); // Bold preserved
    assert!(sgr_256.contains(";3;")); // Italic preserved
    assert!(sgr_256.contains("38;5;")); // Indexed foreground
    assert!(sgr_256.contains("44")); // Named color preserved

    // 16-color: RGB downgraded to named
    let sgr_16 = build_sgr_sequence(&style, ColorSupport::Basic16);
    assert!(sgr_16.contains("1;")); // Bold preserved
    assert!(sgr_16.contains(";3;")); // Italic preserved
    assert!(!sgr_16.contains("38;2;")); // No RGB
    assert!(!sgr_16.contains("38;5;")); // No indexed
    assert!(sgr_16.contains("44")); // Named color preserved

    // NoColor: Colors stripped, attributes preserved
    let sgr_none = build_sgr_sequence(&style, ColorSupport::NoColor);
    assert!(sgr_none.contains("1;")); // Bold preserved
    assert!(sgr_none.contains(";3;")); // Italic preserved
    assert!(sgr_none.contains("39")); // Foreground reset
    assert!(sgr_none.contains("49")); // Background reset
}

/// Test that static profiles give correct values for each terminal.
#[test]
fn test_static_profiles_correctness() {
    // Kitty: Modern terminal with all features
    let kitty = profile_for(TerminalKind::Kitty);
    assert_eq!(kitty.color, ColorSupport::TrueColor);
    assert!(kitty.synchronized_output);
    assert!(kitty.kitty_keyboard);
    assert!(kitty.hyperlinks);

    // Alacritty: Modern but missing some features
    let alacritty = profile_for(TerminalKind::Alacritty);
    assert_eq!(alacritty.color, ColorSupport::TrueColor);
    assert!(!alacritty.synchronized_output);
    assert!(!alacritty.kitty_keyboard);
    assert!(!alacritty.hyperlinks);

    // Terminal.app: Limited macOS terminal
    let terminal_app = profile_for(TerminalKind::TerminalApp);
    assert_eq!(terminal_app.color, ColorSupport::Basic16);
    assert!(!terminal_app.synchronized_output);
    assert!(!terminal_app.bracketed_paste);

    // Unknown: Conservative baseline
    let unknown = profile_for(TerminalKind::Unknown);
    assert_eq!(unknown.color, ColorSupport::Basic16);
    assert!(!unknown.synchronized_output);
    assert!(!unknown.kitty_keyboard);
    assert!(!unknown.hyperlinks);
}

/// Test multiplexer limits are applied correctly.
#[test]
fn test_multiplexer_limits_applied() {
    use saorsa_tui::terminal::merge_multiplexer_limits;

    let kitty = profile_for(TerminalKind::Kitty);

    // Tmux: Disables synchronized output
    let tmux_kitty = merge_multiplexer_limits(kitty.clone(), MultiplexerKind::Tmux);
    assert!(!tmux_kitty.synchronized_output);
    assert!(tmux_kitty.kitty_keyboard); // Other features preserved
    assert_eq!(tmux_kitty.color, ColorSupport::TrueColor);

    // Screen: Downgrades color and disables many features
    let screen_kitty = merge_multiplexer_limits(kitty.clone(), MultiplexerKind::Screen);
    assert_eq!(screen_kitty.color, ColorSupport::Extended256);
    assert!(!screen_kitty.synchronized_output);
    assert!(!screen_kitty.kitty_keyboard);
    assert!(!screen_kitty.hyperlinks);

    // Zellij: Transparent (no limits)
    let zellij_kitty = merge_multiplexer_limits(kitty.clone(), MultiplexerKind::Zellij);
    assert_eq!(kitty, zellij_kitty);

    // None: No limits
    let none_kitty = merge_multiplexer_limits(kitty.clone(), MultiplexerKind::None);
    assert_eq!(kitty, none_kitty);
}

/// Test end-to-end: Detect → Render → Wrap for tmux scenario.
#[test]
fn test_end_to_end_tmux_scenario() {
    // Simulate detection of Kitty terminal inside tmux
    let mut querier = MockQuerier::new()
        .with_color_support(ColorSupport::TrueColor)
        .with_synchronized_output(true);

    let caps = detect_capabilities(TerminalKind::Kitty, MultiplexerKind::Tmux, &mut querier);

    // Tmux should have disabled synchronized output despite query saying yes
    assert!(!caps.synchronized_output);
    assert_eq!(caps.color, ColorSupport::TrueColor);

    // Create renderer
    let renderer = Renderer::new(caps.color, caps.synchronized_output);

    // Render a cell
    let style = Style::new().fg(Color::Rgb { r: 255, g: 0, b: 0 });
    let changes = vec![CellChange {
        x: 0,
        y: 0,
        cell: Cell::new("X", style),
    }];
    let output = renderer.render(&changes);

    // No synchronized output markers (disabled by tmux)
    assert!(!output.contains("\x1b[?2026h"));
    assert!(!output.contains("\x1b[?2026l"));

    // TrueColor still works
    assert!(output.contains("\x1b[38;2;255;0;0m"));

    // If we need to wrap escape sequences for tmux
    let color_seq = "\x1b[38;2;255;0;0m";
    let wrapped = wrap_sequence(color_seq, MultiplexerKind::Tmux);
    assert_eq!(wrapped, "\x1bPtmux;\x1b\x1b\x1b[38;2;255;0;0m\x1b\\");
}

/// Test partial query success scenario.
#[test]
fn test_partial_query_success() {
    // Only color query succeeds, others timeout
    let mut querier = MockQuerier::new().with_color_support(ColorSupport::Extended256);

    let caps = detect_capabilities(TerminalKind::Unknown, MultiplexerKind::None, &mut querier);

    // Queried value overrides static
    assert_eq!(caps.color, ColorSupport::Extended256);

    // Static profile values used for non-queried capabilities
    assert!(!caps.synchronized_output); // Unknown profile default
    assert!(!caps.kitty_keyboard); // Unknown profile default
}

/// Test capability detection upgrades conservative defaults.
#[test]
fn test_detection_upgrades_conservative_defaults() {
    // Unknown terminal with full query success
    let mut querier = MockQuerier::new()
        .with_color_support(ColorSupport::TrueColor)
        .with_synchronized_output(true)
        .with_kitty_keyboard(true);

    let caps = detect_capabilities(TerminalKind::Unknown, MultiplexerKind::None, &mut querier);

    // All capabilities upgraded from conservative defaults
    assert_eq!(caps.color, ColorSupport::TrueColor);
    assert!(caps.synchronized_output);
    assert!(caps.kitty_keyboard);
}

/// Test that rendering respects capability limits.
#[test]
fn test_rendering_respects_capability_limits() {
    let style = Style::new().fg(Color::Rgb {
        r: 255,
        g: 100,
        b: 50,
    });
    let changes = vec![CellChange {
        x: 0,
        y: 0,
        cell: Cell::new("T", style),
    }];

    // NoColor capability: renderer strips all colors
    let renderer_none = Renderer::new(ColorSupport::NoColor, false);
    let output = renderer_none.render(&changes);
    assert!(output.contains("\x1b[39m")); // Reset
    assert!(!output.contains("\x1b[38;2;")); // No colors

    // Basic16 capability: renderer converts RGB to named
    let renderer_16 = Renderer::new(ColorSupport::Basic16, false);
    let output_16 = renderer_16.render(&changes);
    assert!(!output_16.contains("\x1b[38;2;")); // No RGB
    // Should have a named color code (exact value depends on conversion)
    assert!(output_16.contains("\x1b[9")); // Named colors in 90-97 or 30-37 range
}
