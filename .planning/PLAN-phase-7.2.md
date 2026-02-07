# Phase 7.2: Terminal Compatibility

**Milestone 7: Polish & Release**
**Goal**: Detect terminal emulators, query capabilities, add multiplexer support, and ensure graceful degradation.

---

## Task 1: Terminal Detection Module

**Description**: Create a terminal detection module that identifies the running terminal emulator and multiplexer from environment variables.

**Files**:
- `crates/saorsa-core/src/terminal/detect.rs`
- `crates/saorsa-core/src/terminal/mod.rs` (re-export)

**Acceptance Criteria**:
- `TerminalKind` enum: iTerm2, Kitty, Alacritty, WezTerm, TerminalApp, WindowsTerminal, Xterm, VTE, Unknown
- `MultiplexerKind` enum: Tmux, Screen, Zellij, None
- `detect_terminal() -> TerminalKind` from env vars (TERM_PROGRAM, TERM, WT_SESSION, etc.)
- `detect_multiplexer() -> MultiplexerKind` from env vars (TMUX, STY, ZELLIJ)
- `TerminalInfo` struct combining kind, multiplexer, version if available
- Unit tests with mocked env vars
- All tests pass with zero warnings

---

## Task 2: Capability Profiles

**Description**: Define capability profiles for known terminals with sensible defaults.

**Files**:
- `crates/saorsa-core/src/terminal/profiles.rs`
- `crates/saorsa-core/src/terminal/traits.rs` (extend TerminalCapabilities)

**Acceptance Criteria**:
- Extend `TerminalCapabilities` with: `bracketed_paste`, `focus_events`, `hyperlinks`, `sixel`
- `CapabilityProfile` for each known terminal with documented capability sets
- `profile_for(kind: TerminalKind) -> TerminalCapabilities` returns defaults for known terminals
- Multiplexer adjustments: tmux reduces synchronized_output, screen limits colors, etc.
- `merge_multiplexer_limits(caps, multiplexer) -> TerminalCapabilities`
- Unit tests for each terminal profile
- All tests pass with zero warnings

---

## Task 3: Dynamic Capability Detection

**Description**: Query terminal capabilities at runtime using escape sequences with timeout fallback.

**Files**:
- `crates/saorsa-core/src/terminal/query.rs`

**Acceptance Criteria**:
- `query_color_support() -> ColorSupport` — send DA1 sequence, parse response
- `query_synchronized_output() -> bool` — send DECRPM query for mode 2026
- `query_kitty_keyboard() -> bool` — probe for kitty keyboard protocol
- Timeout mechanism (50ms default) for non-responsive terminals
- Fallback to static profile when queries fail
- `detect_capabilities(kind, multiplexer) -> TerminalCapabilities` combines static + dynamic
- Unit tests with mock terminal responses
- All tests pass with zero warnings

---

## Task 4: Enhanced Color Downgrading

**Description**: Improve color fallback with perceptual accuracy and terminal-specific palettes.

**Files**:
- `crates/saorsa-core/src/renderer.rs` (enhance existing color downgrade)

**Acceptance Criteria**:
- Improve `rgb_to_256()` with CIELAB perceptual distance instead of Euclidean RGB
- Add `rgb_to_16()` with terminal-specific 16-color palettes (default, solarized, etc.)
- Cache color mappings to avoid repeated computation
- `ColorMapper` struct that stores computed mappings
- Respect `NO_COLOR` environment variable per spec (https://no-color.org/)
- Unit tests comparing downgrade quality
- All tests pass with zero warnings

---

## Task 5: Multiplexer Pass-Through

**Description**: Handle escape sequence wrapping for tmux, screen, and Zellij multiplexers.

**Files**:
- `crates/saorsa-core/src/terminal/multiplexer.rs`

**Acceptance Criteria**:
- `wrap_for_tmux(seq: &str) -> String` — tmux DCS pass-through (`\x1bPtmux;\x1b{seq}\x1b\\`)
- `wrap_for_screen(seq: &str) -> String` — screen DCS pass-through
- `EscapeWrapper` trait with implementations for each multiplexer
- Auto-detect if wrapping needed based on `MultiplexerKind`
- Integration with renderer: wrap synchronized output sequences
- Unit tests for each multiplexer wrapping format
- All tests pass with zero warnings

---

## Task 6: CrosstermBackend Enhancement

**Description**: Enhance the crossterm backend to use the new detection and capability system.

**Files**:
- `crates/saorsa-core/src/terminal/crossterm_backend.rs`

**Acceptance Criteria**:
- Use `detect_terminal()` and `detect_capabilities()` in `CrosstermBackend::new()`
- Replace hardcoded capabilities with detected values
- Add `CrosstermBackend::with_capabilities(caps)` for manual override
- Wrap escape sequences for multiplexers automatically
- Enable synchronized output when detected as supported
- Enable kitty keyboard when detected
- Unit tests for capability detection integration
- All tests pass with zero warnings

---

## Task 7: Terminal Compatibility Tests

**Description**: Create a comprehensive test suite for terminal compatibility with mock terminals.

**Files**:
- `crates/saorsa-core/tests/terminal_compat.rs`

**Acceptance Criteria**:
- `MockTerminal` helper that simulates specific terminal environments
- Test: iTerm2 detection and capabilities
- Test: Kitty detection with keyboard protocol
- Test: Alacritty detection and color support
- Test: WezTerm detection
- Test: Terminal.app with limited capabilities
- Test: tmux pass-through wrapping
- Test: screen pass-through wrapping
- Test: Nested multiplexer detection (tmux inside screen)
- Test: NO_COLOR respect
- At least 12 terminal compatibility tests
- All pass with zero warnings

---

## Task 8: Integration Tests & Documentation

**Description**: End-to-end integration tests and inline documentation for the terminal compatibility system.

**Files**:
- `crates/saorsa-core/tests/terminal_integration.rs`
- `crates/saorsa-core/src/terminal/mod.rs` (module docs)

**Acceptance Criteria**:
- Integration test: full render cycle with mock iTerm2 backend
- Integration test: full render cycle with mock tmux backend
- Integration test: color downgrade chain (TrueColor → 256 → 16 → none)
- Integration test: capability override works correctly
- Module-level documentation explaining the terminal detection system
- Doc comments on all new public types and functions
- `cargo doc` produces no warnings for terminal module
- All tests pass with zero warnings
