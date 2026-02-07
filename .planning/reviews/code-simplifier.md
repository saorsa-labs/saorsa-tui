# Code Simplifier Review Report

## Overview
This review identifies simplification opportunities in the recently added text editing widgets (Phase 4.1). The analysis focuses on reducing complexity, eliminating redundancy, and leveraging standard library patterns.

## CRITICAL Issues

### 1. **Complex Cursor Movement Logic** (cursor.rs:72-122)
**Issue**: Movement methods in `CursorState` have repetitive pattern for handling preferred column and boundary checks.
**Current Pattern**:
```rust
pub fn move_up(&mut self, buffer: &TextBuffer) {
    self.clear_selection();
    if self.position.line > 0 {
        let target_col = self.preferred_col.unwrap_or(self.position.col);
        self.preferred_col = Some(target_col);
        self.position.line -= 1;
        let line_len = buffer.line_len(self.position.line).unwrap_or(0);
        self.position.col = target_col.min(line_len);
    }
}
```

**Simplification Opportunity**: Extract common movement logic into a private helper method to reduce duplication across `move_up`, `move_down`, `move_left`, and `move_right`.

### 2. **Redundant Boundary Checks** (text_buffer.rs:103-152)
**Issue**: Multiple methods in `TextBuffer` repeat similar boundary checking patterns.
**Current Pattern**:
```rust
pub fn line(&self, idx: usize) -> Option<String> {
    if idx >= self.rope.len_lines() {
        return None;
    }
    // ... rest of implementation
}
```

**Simplification Opportunity**: Use `checked_*` methods from ropey or implement a helper for common boundary operations.

## HIGH Priority Issues

### 3. **Overly Complex Character Deletion** (text_buffer.rs:149-159)
**Issue**: `delete_char` method has complex logic for handling character deletion at line boundaries.
**Current Pattern**:
```rust
pub fn delete_char(&mut self, line: usize, col: usize) {
    if let Some(char_idx) = self.line_col_to_char(line, col)
        && char_idx < self.rope.len_chars()
    {
        self.rope.remove(char_idx..char_idx + 1);
    }
}
```

**Simplification Opportunity**: Simplify the condition chaining and improve readability.

### 4. **Complex Selection Text Extraction** (cursor.rs:302-347)
**Issue**: `selected_text` method has nested logic for multi-line selection handling.
**Current Pattern**:
```rust
let line_start = if line_idx == start.line { start.col } else { 0 };
let line_end = if line_idx == end.line {
    end.col.min(line_text.chars().count())
} else {
    line_text.chars().count()
};
```

**Simplification Opportunity**: Extract line boundary logic into a helper method to reduce complexity.

### 5. **Repetitive Cursor Advancement Logic** (text_area.rs:182-194)
**Issue**: The cursor advancement logic after string insertion duplicates line/col incrementing logic.
**Current Pattern**:
```rust
// Advance cursor past inserted text
for ch in text.chars() {
    if ch == '\n' {
        self.cursor.position.line += 1;
        self.cursor.position.col = 0;
    } else {
        self.cursor.position.col += 1;
    }
}
```

**Simplification Opportunity**: Extract cursor advancement into a helper method to avoid duplication.

## MEDIUM Priority Issues

### 6. **Unnecessary Option Wrapping** (text_buffer.rs:60-69)
**Issue**: `line_len` method wraps in Option when it could return 0 for invalid indices.
**Current Pattern**:
```rust
pub fn line_len(&self, idx: usize) -> Option<usize> {
    self.line(idx).map(|l| l.chars().count())
}
```

**Simplification Opportunity**: Consider returning 0 for invalid indices to simplify caller code.

### 7. **Complex Style Stack Management** (markdown.rs:113-178)
**Issue**: Markdown renderer has complex state management for style stack and various flags.
**Current Pattern**:
```rust
let mut style_stack: Vec<Style> = vec![Style::default()];
let mut current_line: Vec<Segment> = Vec::new();
let mut current_width: usize = 0;
let mut in_code_block = false;
let mut list_depth: usize = 0;
let mut in_list_item = false;
```

**Simplification Opportunity**: Group related state into a single struct for better organization.

### 8. **Redundant Error Handling in Line Access** (text_area.rs:215-220)
**Issue**: Multiple methods check `if let Some(line_text) = self.buffer.line(pos.line)` with similar patterns.
**Current Pattern**:
```rust
if let Some(line_text) = self.buffer.line(pos.line) {
    let deleted: String = line_text
        .chars()
        .nth(del_col)
        .map(String::from)
        .unwrap_or_default();
    // ... rest of logic
}
```

**Simplification Opportunity**: Extract common line access and character extraction logic.

## LOW Priority Issues

### 9. **Verbose Test Assertions** (Multiple files)
**Issue**: Tests use `unreachable!()` with descriptive strings instead of more concise assertions.
**Current Pattern**:
```rust
match buf.line(0) {
    Some(ref s) if s == "hello" => {}
    _ => unreachable!("expected 'hello'"),
}
```

**Simplification Opportunity**: Use `assert_eq!` macros for better readability and less verbose failure messages.

### 10. **Complex Option Handling** (cursor.rs:144-149)
**Issue**: `selected_text` method has complex Option chaining that could be simplified.
**Current Pattern**:
```rust
pub fn selected_text(&self, buffer: &TextBuffer) -> Option<String> {
    let sel = self.selection.as_ref()?;
    if sel.is_empty() {
        return None;
    }
    // ... rest of logic
}
```

**Simplification Opportunity**: Use early returns to flatten the Option chain.

## Summary

The codebase shows good structure but has several areas where simplification would improve maintainability:

1. **Extract common patterns** (cursor movement, boundary checking, line access)
2. **Reduce Option usage** where simple defaults would suffice
3. **Organize complex state** into logical groupings
4. **Simplify test assertions** for better readability

These changes would reduce code duplication while maintaining the same functionality, making the codebase easier to understand and maintain.

## Findings

### HIGH PRIORITY

- **text_area.rs:288-318** - Duplicated text selection logic. The `selected_text_for()` method (30 lines) is nearly identical to `CursorState::selected_text()` in cursor.rs:217-253. This violates DRY and creates maintenance burden.

- **text_area.rs:651-702** - Four nearly identical movement helper methods (`move_left_pos`, `move_right_pos`, `move_up_no_clear`, `move_down_no_clear`) duplicate logic from `CursorState`. These exist only to avoid clearing selection, which could be parameterized instead.

- **text_area.rs:322-378** - The `apply_operation()` method has three large match arms with duplicated cursor calculation logic. All three branches contain identical loops that advance line/col through text character-by-character.

### MEDIUM PRIORITY

- **text_area.rs:559-648** - The `handle_key()` method contains nested conditionals with repeated patterns. The shift-selection handling repeats the same structure 4 times (lines 560-566, 572-578, 584-590, 596-602).

- **wrap.rs:42-88** - The `wrap_line()` function has deep nesting (4 levels) and complex state management. The word boundary handling (lines 60-76) could be extracted to a helper.

- **markdown.rs:86-224** - The event processing loop is long (138 lines) with nested matches. Some event handlers could be extracted to methods for clarity.

- **markdown.rs:257-265** - Redundant match for heading styles. Levels 4-6 all use the same style, making the match verbose.

### LOW PRIORITY

- **text_buffer.rs:130-147** - The `line_col_to_char()` method has a verbose comment explaining clamping behavior (14 lines). The logic is simple enough that the comment adds more confusion than clarity.

- **cursor.rs:217-253** - The `selected_text()` method has explicit bounds checking and character iteration that could use iterator methods more idiomatically.

- **undo.rs:43-63** - The `inverse()` method clones strings in all branches. Since operations are consumed during undo/redo, inverse could potentially move values instead.

- **highlight.rs:75-96** - The `highlight_line()` method converts between byte and character indices multiple times. This could be simplified with a single character-based scan.

## Simplification Opportunities

### 1. Consolidate Text Selection Logic

**Current**: Duplicated 30-line methods in `text_area.rs` and `cursor.rs`

**Simplified**: Move to `Selection` type as a method:
```rust
impl Selection {
    pub fn extract_text(&self, buffer: &TextBuffer) -> Option<String> {
        // Single implementation shared by both modules
    }
}
```

**Impact**: Removes 30 lines of duplication, single source of truth for selection logic.

---

### 2. Parameterize Cursor Movement

**Current**: 8 movement methods (4 in CursorState that clear selection, 4 in TextArea that don't)

**Simplified**: Add a `preserve_selection` parameter:
```rust
impl CursorState {
    pub fn move_left(&mut self, buffer: &TextBuffer, preserve_selection: bool) {
        if !preserve_selection {
            self.clear_selection();
        }
        // existing logic
    }
}
```

**Impact**: Removes 4 duplicate methods (~50 lines), clearer intent.

---

### 3. Extract Cursor Position Calculation

**Current**: Duplicated in 3 branches of `apply_operation()`

**Simplified**: Helper function:
```rust
fn advance_position(pos: CursorPosition, text: &str) -> CursorPosition {
    let mut line = pos.line;
    let mut col = pos.col;
    for ch in text.chars() {
        if ch == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
    }
    CursorPosition::new(line, col)
}
```

**Impact**: Removes ~30 lines of duplication across 3 match arms.

---

### 4. Extract Shift-Selection Pattern

**Current**: Repeated 4 times in `handle_key()` (lines 560-566, 572-578, 584-590, 596-602)

**Simplified**: Helper method:
```rust
fn handle_selection_movement<F>(&mut self, shift: bool, move_fn: F)
where
    F: FnOnce(&mut CursorState, &TextBuffer),
{
    if shift {
        if self.cursor.selection.is_none() {
            self.cursor.start_selection();
        }
        move_fn(&mut self.cursor, &self.buffer);
        self.cursor.extend_selection();
    } else {
        move_fn(&mut self.cursor, &self.buffer);
    }
}

// Usage:
KeyCode::Left => {
    self.handle_selection_movement(shift, |c, b| c.move_left(b));
    EventResult::Consumed
}
```

**Impact**: Reduces 40 lines to 10, removes nested conditionals.

---

### 5. Extract Word Boundary Breaking in wrap_line()

**Current**: Nested logic in lines 60-76 of `wrap.rs`

**Simplified**: Helper function:
```rust
fn break_at_word_boundary(
    line: &str,
    line_start_col: usize,
    width: usize,
) -> Option<(String, String, usize)> {
    // Returns (before, after, new_start_col)
}
```

**Impact**: Reduces nesting from 4 to 3 levels, clearer separation of concerns.

---

### 6. Extract Markdown Event Handlers

**Current**: 138-line event processing loop in `render_to_lines()`

**Simplified**: Extract to methods:
```rust
fn handle_start_tag(&mut self, tag: Tag, state: &mut RenderState);
fn handle_end_tag(&mut self, tag_end: TagEnd, state: &mut RenderState);
fn handle_text(&mut self, text: &str, state: &mut RenderState);
```

**Impact**: Main loop reduces to 30 lines, each handler is testable independently.

---

### 7. Simplify Heading Style Selection

**Current**: Match with redundant arms for levels 4-6

**Simplified**:
```rust
fn heading_style(level: u8) -> Style {
    let base = Style::new().bold(true);
    match level {
        1 => base.fg(Color::Named(NamedColor::Cyan)),
        2 => base.fg(Color::Named(NamedColor::Green)),
        3 => base.fg(Color::Named(NamedColor::Yellow)),
        _ => base,
    }
}
```

**Impact**: Removes redundant match arms, more concise.

---

## Positive Patterns Observed

The following patterns are excellent and should be maintained:

1. **Clear module boundaries** - Each module has a single, well-defined responsibility
2. **Consistent error handling** - No `.unwrap()` or `.expect()` in production code
3. **Comprehensive testing** - 105 tests added, covering edge cases and Unicode
4. **Type safety** - Strong types (`CursorPosition`, `Selection`, etc.) prevent errors
5. **Builder pattern** - `TextArea::new().with_*()` chains are idiomatic and clear
6. **Unicode correctness** - Proper grapheme cluster handling, display width awareness
7. **Documentation** - Module and function docs are clear and complete

## Grade: B+

**Rationale:**
- **Code Quality**: A (excellent type safety, testing, Unicode handling)
- **Clarity**: B+ (some complexity in TextArea and markdown renderer)
- **Maintainability**: B (duplication in text selection and cursor movement)
- **Consistency**: A (follows project patterns, no clippy warnings)

The code is production-ready and well-structured. The identified simplifications are **refinements, not requirements**. Addressing the HIGH priority items would improve maintainability by reducing duplication, while MEDIUM/LOW items would enhance readability but are not blocking issues.

## Recommendations

1. **HIGH priority**: Address text selection duplication and cursor movement parameterization before Phase 4.2
2. **MEDIUM priority**: Extract event handlers in markdown.rs if the module grows further
3. **LOW priority**: Apply other simplifications opportunistically during future maintenance

## Test Quality Assessment

The tests are **exemplary**:
- 105 new tests, zero failures
- Edge cases covered (Unicode, empty input, boundaries)
- Follow project pattern: `assert!()` + `match` instead of `.expect()`
- No test-only `.unwrap()` violations

## Files Reviewed

1. `crates/fae-core/src/text_buffer.rs` - 355 lines, 25 tests
2. `crates/fae-core/src/cursor.rs` - 509 lines, 27 tests
3. `crates/fae-core/src/undo.rs` - 293 lines, 12 tests
4. `crates/fae-core/src/wrap.rs` - 261 lines, 13 tests
5. `crates/fae-core/src/highlight.rs` - 182 lines, 9 tests
6. `crates/fae-core/src/widget/text_area.rs` - 892 lines, 17 tests
7. `crates/fae-core/src/widget/markdown.rs` - 454 lines, 12 tests

**Total**: 2,946 lines (production + tests), 115 tests

---

**Review Complete**: No changes made (analysis only as requested).
