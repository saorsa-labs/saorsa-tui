# Phase 7.1: Testing & Quality

**Milestone 7: Polish & Release**
**Goal**: Add snapshot tests, property-based tests, integration tests, and benchmarks.

---

## Task 1: Snapshot Testing Infrastructure

**Description**: Add the `insta` crate for snapshot testing. Create test helpers for rendering widgets to text snapshots. Set up snapshot directory structure.

**Files**:
- `crates/fae-core/Cargo.toml` (add insta dev-dependency)
- `crates/fae-core/tests/snapshots/` (directory for snapshot files)
- `crates/fae-core/tests/snapshot_helpers.rs` (shared test utilities)
- `crates/fae-core/tests/snapshot_basic.rs` (Label, StaticWidget, Container snapshots)

**Acceptance Criteria**:
- `insta` crate added as dev-dependency with `glob` feature
- Helper function `render_widget_to_string(widget, width, height) -> String` that renders a widget to a Segment list and formats as plain text grid
- Snapshot tests for Label (empty, short, long text), StaticWidget, Container (empty, with children)
- `cargo insta test` works, snapshots auto-generated
- All tests pass with zero warnings

---

## Task 2: Text Widget Snapshots

**Description**: Snapshot tests for TextArea and MarkdownRenderer widgets.

**Files**:
- `crates/fae-core/tests/snapshot_text_widgets.rs`

**Acceptance Criteria**:
- TextArea snapshots: empty, single line, multi-line, with cursor position, with selection, word wrap, line numbers
- MarkdownRenderer snapshots: headings, bold/italic, code blocks, lists, links, mixed content
- At least 12 snapshot tests total
- All pass with zero warnings

---

## Task 3: Data Widget Snapshots

**Description**: Snapshot tests for data display widgets.

**Files**:
- `crates/fae-core/tests/snapshot_data_widgets.rs`

**Acceptance Criteria**:
- RichLog snapshots: empty, with entries, scrolled
- SelectList snapshots: items, with filter, with selection
- DataTable snapshots: basic table, sorted column, resized columns
- Tree snapshots: flat, nested, expanded/collapsed
- DiffView snapshots: unified mode, side-by-side mode
- At least 12 snapshot tests total

---

## Task 4: UI Widget Snapshots

**Description**: Snapshot tests for UI control widgets.

**Files**:
- `crates/fae-core/tests/snapshot_ui_widgets.rs`

**Acceptance Criteria**:
- Tabs snapshots: multiple tabs, selected tab, tab positions
- ProgressBar snapshots: 0%, 50%, 100%, indeterminate
- LoadingIndicator snapshots: each style (Spinner, Dots, Line, Box, Circle)
- Collapsible snapshots: expanded, collapsed
- Form controls: Switch on/off, RadioButton selected/unselected, Checkbox checked/unchecked
- Sparkline snapshots: various data patterns
- OptionList snapshots: items with selection
- At least 15 snapshot tests total

---

## Task 5: Property-Based Tests for CSS Parser

**Description**: Use proptest to generate random CSS inputs and verify the parser handles them without panicking. Test round-trip parsing of values and selectors.

**Files**:
- `Cargo.toml` (add proptest to workspace deps)
- `crates/fae-core/Cargo.toml` (add proptest dev-dependency)
- `crates/fae-core/tests/proptest_css.rs`

**Acceptance Criteria**:
- proptest crate added as dev-dependency
- Property tests for CssValue parsing: random colors, lengths, keywords don't panic
- Property tests for selector parsing: random selector strings don't panic
- Property tests for variable resolution: any variable name resolves or returns fallback
- Round-trip test: parse a color value, serialize, re-parse yields same result
- At least 8 property tests
- All pass with zero warnings

---

## Task 6: Property-Based Tests for Layout Engine

**Description**: Use proptest to verify layout engine invariants with random widget trees and constraints.

**Files**:
- `crates/fae-core/tests/proptest_layout.rs`

**Acceptance Criteria**:
- Property tests for flexbox: random children with random flex factors produce valid non-overlapping rects
- Property tests for grid: random grid definitions produce valid layouts
- Property tests for box model: margin + border + padding + content = total size
- Invariant: no child extends beyond parent bounds (with overflow hidden)
- Invariant: all computed sizes are non-negative
- At least 6 property tests
- All pass with zero warnings

---

## Task 7: Integration Tests with Mock LLM

**Description**: Test the fae-ai provider layer and fae-agent tool execution with mock HTTP responses.

**Files**:
- `crates/fae-ai/Cargo.toml` (add wiremock or mockito dev-dep if needed)
- `crates/fae-ai/tests/integration_provider.rs`
- `crates/fae-agent/tests/integration_tools.rs`

**Acceptance Criteria**:
- Mock LLM server test: send completion request, verify response parsing
- Mock streaming test: simulate SSE chunks, verify stream assembly
- Mock tool call test: LLM returns tool_use, verify tool call extraction
- Agent tool execution test: execute Read tool against temp files, verify output
- Agent tool execution test: execute Grep tool, verify results
- At least 8 integration tests
- All pass with zero warnings

---

## Task 8: Performance Benchmarks with Criterion

**Description**: Add criterion benchmarks for performance-critical paths.

**Files**:
- `Cargo.toml` (add criterion to workspace deps)
- `crates/fae-core/Cargo.toml` (add criterion dev-dep + [[bench]] entries)
- `crates/fae-core/benches/rendering.rs`
- `crates/fae-core/benches/layout.rs`
- `crates/fae-core/benches/css_parsing.rs`

**Acceptance Criteria**:
- Rendering benchmark: ScreenBuffer diff for 80x24, 120x40, 200x60 grids
- Layout benchmark: Taffy layout computation for 10, 50, 100 node trees
- CSS parsing benchmark: parse simple stylesheet, complex stylesheet
- Segment rendering benchmark: render 1000 styled segments
- All benchmarks compile and run with `cargo bench`
- Results printed with statistical analysis
- Zero warnings
