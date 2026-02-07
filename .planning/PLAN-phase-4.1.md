# Phase 4.1: Text Widgets

## Overview
Build the foundational text editing and rendering widgets: a TextBuffer with rope-based storage, a TextArea widget with cursor/selection, undo/redo, soft wrap, line numbers, and a streaming Markdown renderer.

**New dependencies:**
- `ropey = "1.6"` — efficient rope data structure for text editing
- `pulldown-cmark = "0.12"` — CommonMark parser for markdown rendering

**Deferred to Phase 4.1.5 or later:**
- tree-sitter syntax highlighting (requires native C library linkage, complex build)
- Autocomplete overlay (depends on more infrastructure)

Instead, Phase 4.1 provides a pluggable `Highlighter` trait so tree-sitter can be added later without changing the TextArea API.

---

## Task 1: Text Buffer with Rope Storage

**Files:**
- CREATE: `crates/saorsa-core/src/text_buffer.rs`
- MODIFY: `crates/saorsa-core/src/lib.rs` (add module + exports)
- MODIFY: `crates/saorsa-core/Cargo.toml` (add ropey dep)
- MODIFY: `Cargo.toml` (add ropey to workspace deps)

**Description:**
Create a `TextBuffer` struct wrapping `ropey::Rope` with a clean API for text editing operations.

Public API:
- `TextBuffer::new()` — empty buffer
- `TextBuffer::from_str(text: &str)` — from string
- `line_count() -> usize`
- `line(idx: usize) -> Option<String>` — get line content (without trailing newline)
- `line_len(idx: usize) -> Option<usize>` — character count of line
- `total_chars() -> usize`
- `insert_char(line: usize, col: usize, ch: char)` — insert at position
- `insert_str(line: usize, col: usize, text: &str)` — insert string
- `delete_range(start_line: usize, start_col: usize, end_line: usize, end_col: usize)`
- `delete_char(line: usize, col: usize)` — delete char at position
- `to_string() -> String` — full text content
- `lines_range(start: usize, end: usize) -> Vec<String>` — range of lines

**Tests (~12):**
- Empty buffer, from_str, line_count, line access
- Insert char, insert string, insert newline (splits line)
- Delete char, delete range, delete across lines
- Edge cases: empty lines, end of buffer, Unicode

---

## Task 2: Cursor & Selection Model

**Files:**
- CREATE: `crates/saorsa-core/src/cursor.rs`
- MODIFY: `crates/saorsa-core/src/lib.rs` (add module + exports)

**Description:**
Create cursor position and selection types for text editing.

Types:
- `CursorPosition { line: usize, col: usize }` — position in buffer
- `Selection { anchor: CursorPosition, head: CursorPosition }` — selection range
- `CursorState { position: CursorPosition, selection: Option<Selection>, preferred_col: Option<usize> }`

Methods on `CursorPosition`:
- `new(line, col)`, `beginning()`, equality/ordering

Methods on `Selection`:
- `new(anchor, head)`, `is_empty()`, `ordered() -> (CursorPosition, CursorPosition)` (start/end)
- `contains(pos)`, `line_range() -> (usize, usize)`

Methods on `CursorState`:
- `new(line, col)`, `move_left/right/up/down(buffer: &TextBuffer)` — with wrapping
- `move_to_line_start/end(buffer)`, `move_to_buffer_start/end()`
- `start_selection()`, `extend_selection()`, `clear_selection()`
- `selected_text(buffer: &TextBuffer) -> Option<String>`

**Tests (~15):**
- Cursor creation, movement in all directions
- Movement at buffer boundaries (wrap, clamp)
- Selection creation, ordering, containment
- Selected text extraction, multi-line selection
- Preferred column preservation on vertical movement

---

## Task 3: Undo/Redo System

**Files:**
- CREATE: `crates/saorsa-core/src/undo.rs`
- MODIFY: `crates/saorsa-core/src/lib.rs` (add module + exports)

**Description:**
Create an undo/redo stack for text editing operations.

Types:
- `EditOperation` enum: `Insert { pos, text }`, `Delete { pos, text }`, `Replace { pos, old_text, new_text }`
- `UndoStack` — bounded stack with redo support

Methods on `UndoStack`:
- `new(max_history: usize)` — create with capacity
- `push(op: EditOperation)` — push operation (clears redo stack)
- `undo() -> Option<EditOperation>` — pop and return inverse
- `redo() -> Option<EditOperation>` — pop from redo stack
- `can_undo() -> bool`, `can_redo() -> bool`
- `clear()` — reset both stacks

The `EditOperation` must be invertible — `inverse()` returns the reverse operation.

**Tests (~10):**
- Push operations, undo single, redo single
- Undo multiple, redo after undo
- Push after undo clears redo
- Max history limit (oldest dropped)
- Operation inversion (insert↔delete, replace)

---

## Task 4: Soft Wrap & Line Number Calculation

**Files:**
- CREATE: `crates/saorsa-core/src/wrap.rs`
- MODIFY: `crates/saorsa-core/src/lib.rs` (add module + exports)

**Description:**
Create soft-wrap logic that splits logical lines into visual lines based on available width, accounting for display width (CJK, emoji).

Types:
- `WrapLine { text: String, logical_line: usize, start_col: usize }` — one visual line
- `WrapResult { lines: Vec<WrapLine>, line_number_width: u16 }`

Functions:
- `wrap_lines(buffer: &TextBuffer, width: usize) -> WrapResult` — wrap all lines
- `wrap_line(text: &str, width: usize) -> Vec<(String, usize)>` — wrap single line, returns (text, start_col) pairs
- `line_number_width(line_count: usize) -> u16` — digits needed for line numbers

The wrap function must handle:
- Break at word boundaries when possible (greedy, break on whitespace)
- Fall back to character boundary for long words
- Respect display width (CJK = 2 cells, emoji = 2 cells)
- Never split multi-byte characters

**Tests (~12):**
- Short line no wrap, exact width, overflow by one char
- Word wrap, long word break
- CJK characters (width 2), mixed content
- Empty lines, single char lines
- Line number width calculation
- Multi-line buffer wrapping

---

## Task 5: Highlighter Trait & Default Highlighter

**Files:**
- CREATE: `crates/saorsa-core/src/highlight.rs`
- MODIFY: `crates/saorsa-core/src/lib.rs` (add module + exports)

**Description:**
Create a pluggable highlighting trait and a default "no-op" highlighter. This allows tree-sitter to be added later.

Types:
- `HighlightSpan { start_col: usize, end_col: usize, style: Style }` — styled range within a line
- `trait Highlighter` — highlight provider
- `NoHighlighter` — default implementation (no styling)

Trait methods:
- `highlight_line(line_idx: usize, text: &str) -> Vec<HighlightSpan>`
- `on_edit(line_idx: usize)` — notification that a line changed (for incremental parsers)

`NoHighlighter` returns empty spans for all lines.

Also provide a `SimpleKeywordHighlighter` for testing:
- Takes a map of `keyword -> Style`
- Highlights exact keyword matches in each line
- Useful for testing the highlight integration without tree-sitter

**Tests (~8):**
- NoHighlighter returns empty spans
- SimpleKeywordHighlighter finds keywords
- Multiple keywords on same line
- No match, partial match (should not highlight)
- Unicode keyword matching

---

## Task 6: TextArea Widget — Core Rendering

**Files:**
- CREATE: `crates/saorsa-core/src/widget/text_area.rs`
- MODIFY: `crates/saorsa-core/src/widget/mod.rs` (add module + re-export)
- MODIFY: `crates/saorsa-core/src/lib.rs` (add TextArea to exports)

**Description:**
Create the TextArea widget that renders text with line numbers, soft wrap, cursor, selection highlighting, and syntax highlighting. Implements the `Widget` trait.

`TextArea` struct:
- `buffer: TextBuffer`
- `cursor: CursorState`
- `undo_stack: UndoStack`
- `highlighter: Box<dyn Highlighter>`
- `scroll_offset: usize` — first visible logical line
- `show_line_numbers: bool`
- `style: Style` (base text style)
- `cursor_style: Style`
- `selection_style: Style`
- `line_number_style: Style`

Builder methods:
- `new()`, `from_str(text)`, `with_highlighter(h)`, `with_style(s)`
- `with_line_numbers(bool)`, `with_cursor_style(s)`, `with_selection_style(s)`

Rendering:
- Implement `Widget::render(&self, area: Rect, buf: &mut ScreenBuffer)`
- Calculate visible lines based on `scroll_offset` and area height
- Render line numbers in left gutter (if enabled)
- Soft-wrap each visible line to available width
- Apply highlight spans as segment styles
- Show cursor position with cursor style
- Highlight selected text with selection style

**Tests (~10):**
- Empty TextArea renders without crash
- Text renders with correct content
- Line numbers displayed correctly
- Soft wrap splits long lines
- Cursor position visible
- Scroll offset hides top lines

---

## Task 7: TextArea Widget — Editing & Input Handling

**Files:**
- MODIFY: `crates/saorsa-core/src/widget/text_area.rs`

**Description:**
Add editing operations and input handling to TextArea. Implement `InteractiveWidget`.

Editing methods:
- `insert_char(ch: char)` — insert at cursor, push to undo
- `insert_str(text: &str)` — insert string at cursor
- `delete_backward()` — backspace
- `delete_forward()` — delete key
- `delete_selection()` — delete selected text
- `new_line()` — insert newline at cursor
- `undo()`, `redo()` — undo/redo operations

Input handling (`InteractiveWidget::handle_event`):
- Arrow keys: cursor movement (with shift for selection)
- Home/End: line start/end
- Ctrl+Home/End: buffer start/end
- Backspace/Delete: delete operations
- Ctrl+Z: undo, Ctrl+Y/Ctrl+Shift+Z: redo
- Character input: insert char
- Enter: new line

Scroll adjustment:
- After cursor movement, ensure cursor is within visible area
- `ensure_cursor_visible(area_height: u16)` — adjust scroll_offset

**Tests (~12):**
- Insert char updates buffer and cursor
- Insert at end of line, middle of line
- Backspace at start of line joins lines
- Delete at end of line joins lines
- Undo reverses insert, redo reapplies
- Selection delete removes selected text
- Arrow keys with shift create selection
- Scroll adjusts when cursor moves off screen
- Enter splits line correctly

---

## Task 8: Streaming Markdown Renderer

**Files:**
- CREATE: `crates/saorsa-core/src/widget/markdown.rs`
- MODIFY: `crates/saorsa-core/src/widget/mod.rs` (add module + re-export)
- MODIFY: `crates/saorsa-core/src/lib.rs` (add exports)
- MODIFY: `crates/saorsa-core/Cargo.toml` (add pulldown-cmark dep)
- MODIFY: `Cargo.toml` (add pulldown-cmark to workspace deps)

**Description:**
Create a `MarkdownRenderer` that incrementally renders markdown to styled segments. Designed for streaming LLM output where text arrives in chunks.

Types:
- `MarkdownRenderer` — stateful incremental renderer
- `MarkdownBlock` enum: `Paragraph`, `Heading(u8)`, `CodeBlock(Option<String>)`, `ListItem(usize)`, `BlockQuote`, `ThematicBreak`, `Table`

`MarkdownRenderer` methods:
- `new() -> Self`
- `push_str(text: &str)` — append text chunk (streaming)
- `render_to_lines(width: u16) -> Vec<Vec<Segment>>` — render current state
- `clear()` — reset

The renderer:
- Uses pulldown-cmark to parse the accumulated text
- Applies styles: bold, italic, code (inline), headings (bold + color), code blocks (dimmed bg)
- Handles incomplete markdown gracefully (streaming — text may end mid-paragraph)
- Caches parsed blocks to avoid re-parsing unchanged content
- Word-wraps text to the given width

**Tests (~10):**
- Plain text renders as-is
- Bold/italic styling applied
- Heading styles (h1, h2, h3)
- Inline code styled
- Code block rendered with language hint
- List items with markers
- Incremental push_str builds correct output
- Width wrapping in paragraphs
- Empty input, whitespace-only input
- Mixed content (heading + paragraph + code)

---

## Summary

| Task | Name | Files | Est. Tests |
|------|------|-------|-----------|
| 1 | Text Buffer with Rope | text_buffer.rs, Cargo.toml | ~12 |
| 2 | Cursor & Selection | cursor.rs | ~15 |
| 3 | Undo/Redo System | undo.rs | ~10 |
| 4 | Soft Wrap & Line Numbers | wrap.rs | ~12 |
| 5 | Highlighter Trait | highlight.rs | ~8 |
| 6 | TextArea — Rendering | widget/text_area.rs | ~10 |
| 7 | TextArea — Editing | widget/text_area.rs | ~12 |
| 8 | Streaming Markdown | widget/markdown.rs | ~10 |
| **Total** | | **8 new files, 5 modified** | **~89** |

**New dependencies:** ropey 1.6, pulldown-cmark 0.12
**Deferred:** tree-sitter (later phase), autocomplete overlay (later phase)
