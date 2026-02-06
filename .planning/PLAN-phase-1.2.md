# Phase 1.2: Screen Buffer & Rendering

## Overview
Implement the screen buffer (Cell grid) and differential rendering pipeline.
The buffer holds the desired frame, the renderer diffs it against the previous
frame and emits minimal ANSI escape sequences to update the terminal.

## Tasks

### Task 1: ScreenBuffer — Cell Grid
- `crates/fae-core/src/buffer.rs`
- `ScreenBuffer` struct holding a `Vec<Cell>` with width/height
- `new(size)`, `resize(size)`, `clear()`, `get(x, y)`, `set(x, y, cell)`
- `get_row(y)` returning `&[Cell]`
- Wide character support: setting a wide char at (x, y) also sets continuation cell at (x+1, y)
- Out-of-bounds access returns None or is a no-op
- Tests: create, get/set, resize, wide chars, out of bounds

### Task 2: ScreenBuffer — Diff Algorithm
- `diff(&self, previous: &ScreenBuffer) -> Vec<CellChange>`
- `CellChange { x, y, cell }` — represents a single cell that changed
- Skip unchanged cells (same grapheme + same style + same width)
- Handle size mismatches (if sizes differ, full redraw)
- Optimize: skip trailing blanks on rows, coalesce cursor moves
- Tests: no changes returns empty, single cell change, full redraw on resize

### Task 3: ANSI Renderer — Escape Sequence Emission
- `crates/fae-core/src/renderer.rs`
- `Renderer` struct that takes a list of CellChanges and produces terminal output
- Cursor positioning: `\x1b[{row};{col}H`
- Style application: SGR sequences for fg, bg, bold, italic, etc.
- Style diffing: only emit changed attributes between consecutive cells
- Reset: `\x1b[0m` when needed
- Color encoding: truecolor (`\x1b[38;2;r;g;b]m`), 256 (`\x1b[38;5;N]m`), 16 (standard SGR)
- Tests: cursor positioning, style sequences, color encoding

### Task 4: ANSI Renderer — Color Downgrading
- When terminal supports only 256 colors, convert truecolor to nearest 256-color
- When terminal supports only 16 colors, convert to nearest basic color
- Color distance algorithm (simple Euclidean in RGB space)
- Lookup table for 256-color palette
- Tests: truecolor passthrough, truecolor→256, truecolor→16, named color mapping

### Task 5: Synchronized Output (CSI 2026)
- Begin synchronized update: `\x1b[?2026h`
- End synchronized update: `\x1b[?2026l`
- Detection: query terminal or use capability flag from TerminalCapabilities
- Wrap render output in sync markers when supported
- Tests: sync markers present/absent based on capabilities

### Task 6: RenderContext — Full Pipeline
- `crates/fae-core/src/render_context.rs`
- `RenderContext` holds current buffer, previous buffer, and renderer
- `begin_frame()`: swap current → previous, clear current
- `end_frame()`: diff, render, write to terminal
- `present()`: flush to terminal
- Terminal resize handling
- Tests: frame lifecycle, resize handling

### Task 7: Integration & Wire Up
- Add `buffer`, `renderer`, `render_context` modules to fae-core lib.rs
- Re-export key types: `ScreenBuffer`, `Renderer`, `RenderContext`
- Integration test: create buffer, write cells, diff, render to TestBackend
- Ensure all 48 existing tests still pass
- Run clippy, fmt, ensure zero warnings

## Dependencies
- Phase 1.1 complete (Cell, Style, Color, Terminal trait, TestBackend) ✅

## Acceptance Criteria
- ScreenBuffer correctly manages Cell grid with wide character support
- Diff algorithm produces minimal change set
- ANSI renderer emits correct escape sequences
- Color downgrading works for 256 and 16 color modes
- Synchronized output wraps render when supported
- Full pipeline: buffer → diff → render → terminal
- All tests pass, zero clippy warnings
