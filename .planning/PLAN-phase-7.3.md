# Phase 7.3: Themes & Documentation

**Milestone 7: Polish & Release**
**Goal**: Add popular theme presets, complete API documentation, and create user guide.

---

## Task 1: Theme Data Structures

**Description**: Create theme data types and a theme registry for managing color themes.

**Files**:
- `crates/fae-core/src/css/themes/mod.rs` (NEW directory + module)
- `crates/fae-core/src/css/mod.rs` (add themes submodule)

**Acceptance Criteria**:
- `ThemeColors` struct with named color slots (background, foreground, accent, error, warning, success, etc.)
- `Theme` struct wrapping ThemeColors + metadata (name, author, variant: Light/Dark)
- `ThemeRegistry` for registering and retrieving themes by name
- `register_theme()`, `get_theme()`, `list_themes()` methods
- Default theme (dark) registered automatically
- Unit tests for registry operations
- All tests pass with zero warnings

---

## Task 2: Catppuccin Theme

**Description**: Implement the Catppuccin color theme with all four flavors.

**Files**:
- `crates/fae-core/src/css/themes/catppuccin.rs`

**Acceptance Criteria**:
- Four flavors: Latte (light), Frappe, Macchiato, Mocha (dark)
- Accurate color values from official Catppuccin palette
- Each flavor returns a `Theme` struct
- `register_catppuccin(registry)` registers all four flavors
- Unit tests verifying color accuracy for each flavor
- All tests pass with zero warnings

---

## Task 3: Dracula & Solarized Themes

**Description**: Implement Dracula and Solarized color themes.

**Files**:
- `crates/fae-core/src/css/themes/dracula.rs`
- `crates/fae-core/src/css/themes/solarized.rs`

**Acceptance Criteria**:
- Dracula: dark theme with official palette colors
- Solarized Light and Solarized Dark variants
- Each returns `Theme` struct with accurate colors
- `register_dracula(registry)` and `register_solarized(registry)` functions
- Unit tests for each theme
- All tests pass with zero warnings

---

## Task 4: Nord & Built-in Themes

**Description**: Implement Nord theme and bundle all themes with convenience registration.

**Files**:
- `crates/fae-core/src/css/themes/nord.rs`
- `crates/fae-core/src/css/themes/mod.rs` (add register_all_themes)

**Acceptance Criteria**:
- Nord: dark theme with official Nord palette
- `register_all_themes(registry)` convenience function
- Default theme included (fae-dark, fae-light)
- `ThemeRegistry::with_defaults()` constructor that pre-registers all themes
- Unit tests for Nord theme
- Integration test: register all themes, verify count and names
- All tests pass with zero warnings

---

## Task 5: API Documentation Audit

**Description**: Ensure all public items in fae-core have doc comments with examples.

**Files**:
- Multiple source files in `crates/fae-core/src/`

**Acceptance Criteria**:
- `cargo doc --workspace --no-deps` produces zero warnings
- All public structs, enums, traits, functions have `///` doc comments
- Key types have `# Examples` sections in their docs
- Module-level `//!` documentation for each module
- At least 5 doc-test examples that compile and run
- All tests pass with zero warnings

---

## Task 6: fae-ai & fae-agent Documentation

**Description**: Complete API documentation for the AI and agent crates.

**Files**:
- Multiple source files in `crates/fae-ai/src/` and `crates/fae-agent/src/`

**Acceptance Criteria**:
- All public items in fae-ai have doc comments
- All public items in fae-agent have doc comments
- Module-level documentation for provider, types, streaming modules
- Module-level documentation for tools, sessions, context modules
- `cargo doc --workspace --no-deps` produces zero warnings
- All tests pass with zero warnings

---

## Task 7: Architecture Documentation

**Description**: Create architecture overview documentation as module-level docs.

**Files**:
- `crates/fae-core/src/lib.rs` (enhance crate-level docs)
- `crates/fae-ai/src/lib.rs` (enhance crate-level docs)
- `crates/fae-agent/src/lib.rs` (enhance crate-level docs)

**Acceptance Criteria**:
- fae-core lib.rs: Overview of TUI framework architecture (rendering pipeline, widget system, CSS styling, layout, compositor)
- fae-ai lib.rs: Overview of multi-provider LLM API (provider abstraction, streaming, tool use)
- fae-agent lib.rs: Overview of agent runtime (tool execution, sessions, context engineering)
- Architecture diagrams in ASCII art where helpful
- `cargo doc --workspace --no-deps` produces zero warnings
- All tests pass with zero warnings

---

## Task 8: Theme Integration Tests

**Description**: Integration tests for theme system with CSS variables and styling.

**Files**:
- `crates/fae-core/tests/theme_integration.rs`

**Acceptance Criteria**:
- Test: Register and retrieve all built-in themes
- Test: Apply theme colors to CSS variables
- Test: Theme switching at runtime
- Test: Light vs dark variant detection
- Test: Custom theme registration
- Test: Theme color slot access
- At least 8 integration tests
- All pass with zero warnings
