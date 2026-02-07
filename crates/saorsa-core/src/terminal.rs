//! Terminal abstraction for backend-agnostic rendering.
//!
//! This module provides a comprehensive terminal compatibility system that combines
//! static capability profiles, runtime escape sequence queries, and multiplexer detection
//! to ensure correct rendering across diverse terminal environments.
//!
//! # Architecture
//!
//! The terminal compatibility system has three layers:
//!
//! ## 1. Terminal Detection ([`detect`])
//!
//! Identifies the terminal emulator and multiplexer from environment variables:
//! - [`TerminalKind`]: iTerm2, Kitty, Alacritty, WezTerm, Terminal.app, Windows Terminal, etc.
//! - [`MultiplexerKind`]: tmux, GNU Screen, Zellij, or none
//!
//! Detection uses `TERM_PROGRAM`, `TERM`, `TMUX`, `STY`, and other environment variables.
//!
//! ## 2. Capability Profiles
//!
//! Each detected terminal has a static capability profile documenting supported features:
//! - Color support: TrueColor (24-bit), 256-color, 16-color (ANSI), or none
//! - Unicode support
//! - Synchronized output (reduces flicker)
//! - Kitty keyboard protocol
//! - Mouse events, bracketed paste, focus events
//! - Hyperlinks, Sixel graphics
//!
//! Multiplexers apply limits to terminal capabilities (e.g., tmux disables synchronized
//! output, GNU Screen downgrades TrueColor to 256-color).
//!
//! ## 3. Runtime Query
//!
//! Optionally enhance static profiles by querying the terminal at runtime:
//! - Device Attributes (DA1) for color support
//! - DECRPM for synchronized output support
//! - Kitty keyboard protocol query
//!
//! Runtime queries override static profiles when they succeed, allowing detection of
//! unknown terminals or upgraded capabilities.
//!
//! # Usage Pattern
//!
//! ```
//! use saorsa_core::terminal::{detect, detect_capabilities, MockQuerier};
//!
//! // Detect terminal and multiplexer
//! let info = detect();
//!
//! // Use mock querier for testing (or LiveQuerier for real terminal)
//! let mut querier = MockQuerier::new();
//!
//! // Get capabilities: static profile + runtime queries + multiplexer limits
//! let caps = detect_capabilities(info.kind, info.multiplexer, &mut querier);
//!
//! // Use capabilities to configure renderer
//! // let renderer = Renderer::new(caps.color, caps.synchronized_output);
//! ```
//!
//! # Multiplexer Support ([`multiplexer`])
//!
//! Terminal multiplexers intercept escape sequences. Pass-through wrapping ensures
//! sequences reach the underlying terminal:
//!
//! - **tmux**: Wraps sequences in DCS pass-through with escape doubling
//! - **GNU Screen**: Wraps sequences in DCS pass-through
//! - **Zellij**: Transparent (no wrapping needed)
//!
//! ```
//! use saorsa_core::terminal::{MultiplexerKind, multiplexer::wrap_sequence};
//!
//! let seq = "\x1b[?2026h"; // Synchronized output begin
//! let wrapped = wrap_sequence(seq, MultiplexerKind::Tmux);
//! assert_eq!(wrapped, "\x1bPtmux;\x1b\x1b\x1b[?2026h\x1b\\");
//! ```
//!
//! # Color Downgrading
//!
//! The renderer module automatically downgrades colors to match terminal capabilities:
//! - **TrueColor** → RGB (24-bit) preserved
//! - **Extended256** → RGB converted to nearest 256-color palette index (CIELAB distance)
//! - **Basic16** → RGB/indexed converted to nearest ANSI named color (CIELAB distance)
//! - **NoColor** → All colors stripped (respects `NO_COLOR` env var)
//!
//! Downgrading uses CIELAB color space for perceptually accurate matching.
//!
//! # Testing
//!
//! The [`MockQuerier`] and [`TestBackend`] support testing without a real TTY:
//!
//! ```
//! use saorsa_core::terminal::{ColorSupport, MockQuerier, TerminalKind, MultiplexerKind, detect_capabilities};
//!
//! let mut querier = MockQuerier::new()
//!     .with_color_support(ColorSupport::TrueColor)
//!     .with_synchronized_output(true);
//!
//! let caps = detect_capabilities(TerminalKind::Unknown, MultiplexerKind::None, &mut querier);
//! assert_eq!(caps.color, ColorSupport::TrueColor);
//! assert!(caps.synchronized_output);
//! ```

mod crossterm_backend;
mod detect;
pub mod multiplexer;
mod profiles;
mod query;
mod test_backend;
mod traits;

pub use crossterm_backend::CrosstermBackend;
pub use detect::{
    MultiplexerKind, TerminalInfo, TerminalKind, detect, detect_multiplexer, detect_terminal,
};
pub use profiles::{merge_multiplexer_limits, profile_for};
pub use query::{LiveQuerier, MockQuerier, TerminalQuerier, detect_capabilities};
pub use test_backend::TestBackend;
pub use traits::{ColorSupport, Terminal, TerminalCapabilities};
