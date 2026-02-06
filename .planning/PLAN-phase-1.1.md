# Phase 1.1: Workspace & Core Types

## Overview
Convert from single crate to Cargo workspace with 5 crates. Define foundational types for the entire framework. Set up error handling, terminal abstraction, and project documentation.

---

## Task 1: Convert to Cargo Workspace

**Description**: Transform the single-crate project into a Cargo workspace with 5 member crates: fae-core, fae-ai, fae-agent, fae-app, fae-cli.

**Files**:
- `Cargo.toml` (workspace root - rewrite)
- `crates/fae-core/Cargo.toml` (new)
- `crates/fae-core/src/lib.rs` (new)
- `crates/fae-ai/Cargo.toml` (new)
- `crates/fae-ai/src/lib.rs` (new)
- `crates/fae-agent/Cargo.toml` (new)
- `crates/fae-agent/src/lib.rs` (new)
- `crates/fae-app/Cargo.toml` (new)
- `crates/fae-app/src/main.rs` (new)
- `crates/fae-cli/Cargo.toml` (new)
- `crates/fae-cli/src/main.rs` (new)

**Requirements**:
- Workspace root Cargo.toml with `[workspace]` section, resolver = "2"
- `[workspace.package]` with shared metadata (version, edition, authors, license, repository)
- `[workspace.dependencies]` for shared deps (thiserror, anyhow, tokio, serde, serde_json, tracing, crossterm)
- `[workspace.lints.clippy]` denying unwrap_used, expect_used in non-test code
- Each crate uses `{ workspace = true }` for shared deps
- fae-app depends on fae-core, fae-ai, fae-agent
- fae-cli depends on fae-app
- Remove old `src/main.rs`
- `cargo check --workspace` must pass with zero warnings

---

## Task 2: fae-core Error Types & Geometry Primitives

**Description**: Define the error types and basic geometry types for fae-core.

**Files**:
- `crates/fae-core/src/lib.rs` (update)
- `crates/fae-core/src/error.rs` (new)
- `crates/fae-core/src/geometry.rs` (new)

**Requirements**:
- `FaeCoreError` enum with thiserror: `Io`, `Terminal`, `Layout`, `Style`, `Render`, `Widget`, `Internal`
- `pub type Result<T> = std::result::Result<T, FaeCoreError>;`
- Geometry types: `Position { x: u16, y: u16 }`, `Size { width: u16, height: u16 }`, `Rect { position: Position, size: Size }`
- Implement `From<(u16, u16)>` for Position and Size
- Rect methods: `new()`, `contains()`, `intersects()`, `intersection()`, `area()`, `is_empty()`
- All types: `Clone, Copy, Debug, PartialEq, Eq, Hash`
- Doc comments on all public items
- Unit tests for geometry operations (at least 10 tests)

---

## Task 3: Color Type

**Description**: Define the Color type supporting truecolor, 256-color, 16-color, and named colors.

**Files**:
- `crates/fae-core/src/color.rs` (new)
- `crates/fae-core/src/lib.rs` (update - add module)

**Requirements**:
- `Color` enum: `Rgb { r: u8, g: u8, b: u8 }`, `Indexed(u8)`, `Named(NamedColor)`, `Reset`
- `NamedColor` enum: Black, Red, Green, Yellow, Blue, Magenta, Cyan, White, BrightBlack..BrightWhite (16 colors)
- `Color::from_hex("#rrggbb")` parser - returns Result
- `Color::from_css_name("red")` for named CSS colors
- Display impl for debug output
- Conversion to crossterm Color type
- `#[non_exhaustive]` on public enums
- Doc comments, unit tests (hex parsing, named color lookup, crossterm conversion)

---

## Task 4: Style Type

**Description**: Define the Style type representing text styling (foreground, background, decorations).

**Files**:
- `crates/fae-core/src/style.rs` (new)
- `crates/fae-core/src/lib.rs` (update - add module)

**Requirements**:
- `Style` struct with: `fg: Option<Color>`, `bg: Option<Color>`, `bold: bool`, `italic: bool`, `underline: bool`, `strikethrough: bool`, `dim: bool`, `reverse: bool`, `link: Option<String>`
- Builder pattern: `Style::new().fg(Color::Red).bold(true).build()`
- `Style::merge(&self, other: &Style) -> Style` - overlay styles (other's values take priority if set)
- `Style::reset()` - default/empty style
- `Style::is_empty()` - returns true if no attributes set
- Conversion to crossterm ContentStyle
- All derives: Clone, Debug, Default, PartialEq, Eq
- Doc comments, unit tests (merge behavior, builder, empty checks)

---

## Task 5: Segment Type

**Description**: Define the Segment type - the fundamental rendering unit (styled text).

**Files**:
- `crates/fae-core/src/segment.rs` (new)
- `crates/fae-core/src/lib.rs` (update - add module)

**Requirements**:
- `Segment` struct: `text: String`, `style: Style`, `is_control: bool`
- `Segment::new(text: impl Into<String>) -> Self` (default style)
- `Segment::styled(text: impl Into<String>, style: Style) -> Self`
- `Segment::control(text: impl Into<String>) -> Self` (is_control = true)
- `Segment::width(&self) -> usize` using unicode-width for display width
- `Segment::split_at(&self, offset: usize) -> (Segment, Segment)` - split at display-width offset (handles multi-byte)
- `Segment::is_empty(&self) -> bool`
- Add `unicode-width` and `unicode-segmentation` to fae-core dependencies
- Doc comments, unit tests (width calculation, split at various offsets, empty checks, ASCII + multi-byte)

---

## Task 6: Cell Type & Screen Buffer Basics

**Description**: Define the Cell type (single terminal cell) used in the screen buffer.

**Files**:
- `crates/fae-core/src/cell.rs` (new)
- `crates/fae-core/src/lib.rs` (update - add module)

**Requirements**:
- `Cell` struct: `grapheme: String`, `style: Style`, `width: u8` (1 or 2 for CJK/emoji)
- `Cell::new(grapheme: impl Into<String>, style: Style) -> Self` (auto-detect width)
- `Cell::blank() -> Self` (space, default style, width 1)
- `Cell::is_blank(&self) -> bool`
- `Cell::is_wide(&self) -> bool` (width > 1)
- `Cell::continuation() -> Self` (placeholder for second half of wide char)
- All derives: Clone, Debug, PartialEq, Eq
- Doc comments, unit tests (blank, wide char detection, CJK width, emoji width)

---

## Task 7: Terminal Abstraction Trait

**Description**: Define the Terminal trait and crossterm backend implementation.

**Files**:
- `crates/fae-core/src/terminal.rs` (new)
- `crates/fae-core/src/terminal/traits.rs` (new)
- `crates/fae-core/src/terminal/crossterm_backend.rs` (new)
- `crates/fae-core/src/terminal/test_backend.rs` (new)
- `crates/fae-core/src/lib.rs` (update - add module)

**Requirements**:
- `ColorSupport` enum: `NoColor`, `Basic16`, `Extended256`, `TrueColor`
- `TerminalCapabilities` struct: `color: ColorSupport`, `unicode: bool`, `synchronized_output: bool`, `kitty_keyboard: bool`, `mouse: bool`
- `Terminal` trait (async): `size() -> Result<Size>`, `capabilities() -> TerminalCapabilities`, `enter_raw_mode() -> Result<()>`, `exit_raw_mode() -> Result<()>`, `write_raw(&mut self, data: &[u8]) -> Result<()>`, `flush(&mut self) -> Result<()>`, `enable_mouse() -> Result<()>`, `disable_mouse() -> Result<()>`
- `CrosstermBackend` implementing Terminal trait using crossterm
- `TestBackend` implementing Terminal trait with in-memory buffer (for testing)
- Doc comments, tests for TestBackend (write raw data, check buffer contents, size)

---

## Task 8: Error Types for fae-ai, fae-agent, fae-app

**Description**: Define error types for the remaining crates.

**Files**:
- `crates/fae-ai/src/error.rs` (new)
- `crates/fae-ai/src/lib.rs` (update)
- `crates/fae-agent/src/error.rs` (new)
- `crates/fae-agent/src/lib.rs` (update)
- `crates/fae-app/src/error.rs` (new)
- `crates/fae-app/src/main.rs` (update)

**Requirements**:
- `FaeAiError`: `Provider`, `Auth`, `Network`, `RateLimit`, `InvalidRequest`, `Streaming`, `TokenLimit`, `Internal`
- `FaeAgentError`: `Tool`, `Session`, `Context`, `Provider(FaeAiError)`, `Cancelled`, `Internal`
- `FaeAppError`: `Ui(FaeCoreError)`, `Agent(FaeAgentError)`, `Config`, `Internal`
- All use thiserror with descriptive messages
- Each crate has `pub type Result<T> = std::result::Result<T, CrateError>;`
- fae-app main.rs: use anyhow for top-level error handling, minimal main that returns `anyhow::Result<()>`
- Doc comments, basic tests

---

## Task 9: Workspace Lint Configuration & CI Validation

**Description**: Configure workspace-wide lints and verify the entire workspace compiles cleanly.

**Files**:
- `Cargo.toml` (update workspace lints)
- `crates/fae-core/src/lib.rs` (update - add lint attrs)
- All other lib.rs/main.rs (update - add lint attrs)

**Requirements**:
- `[workspace.lints.clippy]` in root Cargo.toml: `unwrap_used = "deny"`, `expect_used = "deny"`, `all = "warn"`, `correctness = "deny"`
- `[workspace.lints.rust]` - `missing_docs = "warn"`, `unreachable_pub = "warn"`
- Each crate's Cargo.toml: `[lints] workspace = true`
- Each lib.rs: `#![warn(missing_docs)]`, appropriate crate-level doc comment
- `cargo check --workspace` - zero errors
- `cargo clippy --workspace --all-targets -- -D warnings` - zero warnings
- `cargo fmt --all -- --check` - passes
- `cargo nextest run --workspace` - all tests pass (must have cargo-nextest installed)
- `cargo doc --workspace --no-deps` - zero warnings

---

## Task 10: Project Documentation (CLAUDE.md)

**Description**: Create the CLAUDE.md for the fae project with build commands, architecture overview, and quality standards.

**Files**:
- `CLAUDE.md` (new - project root)

**Requirements**:
- Project overview (what fae is, the 5 crates and their roles)
- Build commands (check, clippy, test, fmt, doc)
- Quality standards (zero warnings, no unwrap in prod, thiserror for errors)
- Architecture overview (retained-mode TUI, CSS styling, compositor, reactive)
- Crate dependency graph
- Development workflow notes
- Keep concise - under 150 lines
