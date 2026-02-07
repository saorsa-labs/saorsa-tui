# Code Simplification Review: Phase 4.2 Data Widgets

**Review Date:** 2026-02-07
**Scope:** Phase 4.2 Data Widgets (rich_log.rs, select_list.rs, data_table.rs, tree.rs, directory_tree.rs, diff_view.rs)
**Commit Range:** affc93e..39704b0

---

## Executive Summary

Phase 4.2 introduces 6 new data widgets with 2,907 lines of code and 439 tests. The code is well-structured and follows project patterns, but contains **significant duplication** that should be consolidated. No code changes were made during this review.

### Key Findings

1. **CRITICAL: Duplicated `border_chars()` Function** - Identical implementation across 5 files (292 lines total)
2. **CRITICAL: Duplicated Border Rendering Logic** - Nearly identical `render_border()` across 5 files
3. **CRITICAL: Duplicated `inner_area()` Logic** - Identical border accounting across 5 files
4. **MODERATE: Repeated UTF-8 Safe Truncation Pattern** - Same character rendering loop in all widgets
5. **MODERATE: Duplicated `ensure_selected_visible()` Logic** - Identical across 3 widgets
6. **MINOR: Repeated Page Size Magic Number** - Hardcoded `20` appears 30+ times

---

## 1. CRITICAL: Duplicated `border_chars()` Function

### Current State

**Identical implementation in 5 files:**
- `rich_log.rs` (lines 292-318, 27 lines)
- `select_list.rs` (lines 534-560, 27 lines)
- `data_table.rs` (lines 599-625, 27 lines)
- `tree.rs` (lines 566-592, 27 lines)
- `diff_view.rs` (lines 495-521, 27 lines)

**Total duplication:** 135 lines (5 × 27)

### Code Example

```rust
// DUPLICATED 5 TIMES
fn border_chars(
    style: BorderStyle,
) -> Option<(
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    &'static str,
)> {
    match style {
        BorderStyle::None => None,
        BorderStyle::Single => Some((
            "\u{250c}", "\u{2510}", "\u{2514}", "\u{2518}", "\u{2500}", "\u{2502}",
        )),
        BorderStyle::Double => Some((
            "\u{2554}", "\u{2557}", "\u{255a}", "\u{255d}", "\u{2550}", "\u{2551}",
        )),
        BorderStyle::Rounded => Some((
            "\u{256d}", "\u{256e}", "\u{2570}", "\u{256f}", "\u{2500}", "\u{2502}",
        )),
        BorderStyle::Heavy => Some((
            "\u{250f}", "\u{2513}", "\u{2517}", "\u{251b}", "\u{2501}", "\u{2503}",
        )),
    }
}
```

### Recommendation

**Move to a shared module** (e.g., `widget/border.rs` or add to `widget/mod.rs`):

```rust
// In crates/fae-core/src/widget/border.rs (new file)
/// Border character set: (top-left, top-right, bottom-left, bottom-right, horizontal, vertical)
pub type BorderChars = (&'static str, &'static str, &'static str, &'static str, &'static str, &'static str);

impl BorderStyle {
    /// Get the Unicode characters for this border style.
    pub fn chars(self) -> Option<BorderChars> {
        match self {
            BorderStyle::None => None,
            BorderStyle::Single => Some((
                "\u{250c}", "\u{2510}", "\u{2514}", "\u{2518}", "\u{2500}", "\u{2502}",
            )),
            BorderStyle::Double => Some((
                "\u{2554}", "\u{2557}", "\u{255a}", "\u{255d}", "\u{2550}", "\u{2551}",
            )),
            BorderStyle::Rounded => Some((
                "\u{256d}", "\u{256e}", "\u{2570}", "\u{256f}", "\u{2500}", "\u{2502}",
            )),
            BorderStyle::Heavy => Some((
                "\u{250f}", "\u{2513}", "\u{2517}", "\u{251b}", "\u{2501}", "\u{2503}",
            )),
        }
    }
}
```

**Impact:** Eliminates 135 lines of duplication, centralizes border logic in `BorderStyle` type.

---

## 2. CRITICAL: Duplicated Border Rendering Logic

### Current State

**Nearly identical `render_border()` method in 5 files:**
- `rich_log.rs` (lines 142-181, 40 lines)
- `select_list.rs` (lines 337-374, 38 lines)
- `data_table.rs` (lines 289-323, 35 lines)
- `tree.rs` (lines 354-388, 35 lines)
- `diff_view.rs` (lines 247-282, 36 lines)

**Total duplication:** ~184 lines

### Code Pattern

```rust
// DUPLICATED 5 TIMES (with minor style variations)
fn render_border(&self, area: Rect, buf: &mut ScreenBuffer) {
    let chars = border_chars(self.border);
    let (tl, tr, bl, br, h, v) = match chars {
        Some(c) => c,
        None => return,
    };

    let x1 = area.position.x;
    let y1 = area.position.y;
    let w = area.size.width;
    let h_val = area.size.height;

    if w == 0 || h_val == 0 {
        return;
    }

    let x2 = x1.saturating_add(w.saturating_sub(1));
    let y2 = y1.saturating_add(h_val.saturating_sub(1));

    // Corners
    buf.set(x1, y1, Cell::new(tl, style.clone()));
    buf.set(x2, y1, Cell::new(tr, style.clone()));
    buf.set(x1, y2, Cell::new(bl, style.clone()));
    buf.set(x2, y2, Cell::new(br, style.clone()));

    // Edges
    for x in (x1 + 1)..x2 {
        buf.set(x, y1, Cell::new(h, style.clone()));
        buf.set(x, y2, Cell::new(h, style.clone()));
    }

    for y in (y1 + 1)..y2 {
        buf.set(x1, y, Cell::new(v, style.clone()));
        buf.set(x2, y, Cell::new(v, style.clone()));
    }
}
```

### Recommendation

**Create a shared utility function:**

```rust
// In crates/fae-core/src/widget/border.rs
pub fn render_border(
    area: Rect,
    border_style: BorderStyle,
    cell_style: Style,
    buf: &mut ScreenBuffer,
) {
    let Some((tl, tr, bl, br, h, v)) = border_style.chars() else {
        return;
    };

    let x1 = area.position.x;
    let y1 = area.position.y;
    let w = area.size.width;
    let h_val = area.size.height;

    if w == 0 || h_val == 0 {
        return;
    }

    let x2 = x1.saturating_add(w.saturating_sub(1));
    let y2 = y1.saturating_add(h_val.saturating_sub(1));

    buf.set(x1, y1, Cell::new(tl, cell_style.clone()));
    buf.set(x2, y1, Cell::new(tr, cell_style.clone()));
    buf.set(x1, y2, Cell::new(bl, cell_style.clone()));
    buf.set(x2, y2, Cell::new(br, cell_style.clone()));

    for x in (x1 + 1)..x2 {
        buf.set(x, y1, Cell::new(h, cell_style.clone()));
        buf.set(x, y2, Cell::new(h, cell_style.clone()));
    }

    for y in (y1 + 1)..y2 {
        buf.set(x1, y, Cell::new(v, cell_style.clone()));
        buf.set(x2, y, Cell::new(v, cell_style.clone()));
    }
}
```

**Widget usage becomes:**

```rust
// Before (40 lines of duplicated code)
fn render_border(&self, area: Rect, buf: &mut ScreenBuffer) {
    // ... 40 lines ...
}

// After (1 line)
border::render_border(area, self.border, self.style.clone(), buf);
```

**Impact:** Eliminates ~184 lines of duplication.

---

## 3. CRITICAL: Duplicated `inner_area()` Logic

### Current State

**Identical implementation in 5 files:**
- `rich_log.rs` (lines 124-139, 16 lines)
- `select_list.rs` (lines 319-335, 17 lines)
- `data_table.rs` (lines 271-287, 17 lines)
- `tree.rs` (lines 336-352, 17 lines)
- `diff_view.rs` (lines 229-245, 17 lines)

**Total duplication:** 84 lines

### Code Pattern

```rust
// DUPLICATED 5 TIMES
fn inner_area(&self, area: Rect) -> Rect {
    match self.border {
        BorderStyle::None => area,
        _ => {
            if area.size.width < 2 || area.size.height < 2 {
                return Rect::new(area.position.x, area.position.y, 0, 0);
            }
            Rect::new(
                area.position.x + 1,
                area.position.y + 1,
                area.size.width.saturating_sub(2),
                area.size.height.saturating_sub(2),
            )
        }
    }
}
```

### Recommendation

**Add as a method on `Rect` or create utility function:**

```rust
// Option A: Add to Rect (in geometry.rs)
impl Rect {
    /// Calculate the inner area after accounting for a border.
    pub fn inner_with_border(self, border: BorderStyle) -> Rect {
        match border {
            BorderStyle::None => self,
            _ => {
                if self.size.width < 2 || self.size.height < 2 {
                    return Rect::new(self.position.x, self.position.y, 0, 0);
                }
                Rect::new(
                    self.position.x + 1,
                    self.position.y + 1,
                    self.size.width.saturating_sub(2),
                    self.size.height.saturating_sub(2),
                )
            }
        }
    }
}

// Option B: Utility function (in widget/border.rs)
pub fn inner_area(area: Rect, border: BorderStyle) -> Rect {
    // Same implementation
}
```

**Widget usage becomes:**

```rust
// Before
fn inner_area(&self, area: Rect) -> Rect {
    match self.border { /* 16 lines */ }
}
let inner = self.inner_area(area);

// After (Option A - preferred)
let inner = area.inner_with_border(self.border);

// After (Option B)
let inner = border::inner_area(area, self.border);
```

**Impact:** Eliminates 84 lines of duplication.

---

## 4. MODERATE: Repeated UTF-8 Safe Text Rendering Pattern

### Current State

**Similar character-by-character rendering loop in all 6 widgets:**

```rust
// PATTERN REPEATED 10+ TIMES across widgets
for ch in truncated.chars() {
    let char_w = UnicodeWidthStr::width(ch.encode_utf8(&mut [0; 4]) as &str);
    if col as usize + char_w > width {
        break;
    }
    buf.set(x, y, Cell::new(ch.to_string(), style.clone()));
    col += char_w as u16;
}
```

**Locations:**
- `rich_log.rs` (lines 223-231)
- `select_list.rs` (lines 442-450)
- `data_table.rs` (lines 380-387)
- `tree.rs` (lines 480-492)
- `diff_view.rs` (lines 296-303)

### Recommendation

**Create a shared rendering helper:**

```rust
// In crates/fae-core/src/widget/render_util.rs (new file)
use unicode_width::UnicodeWidthStr;

/// Render text into a buffer row, handling UTF-8 width and truncation.
///
/// Returns the number of columns consumed.
pub fn render_text(
    text: &str,
    x: u16,
    y: u16,
    max_width: usize,
    style: &Style,
    buf: &mut ScreenBuffer,
) -> u16 {
    let truncated = truncate_to_display_width(text, max_width);
    let mut col: u16 = 0;

    for ch in truncated.chars() {
        let char_w = UnicodeWidthStr::width(ch.encode_utf8(&mut [0; 4]) as &str);
        if col as usize + char_w > max_width {
            break;
        }
        buf.set(x + col, y, Cell::new(ch.to_string(), style.clone()));
        col += char_w as u16;
    }

    col
}
```

**Widget usage becomes:**

```rust
// Before (9 lines)
for ch in truncated.chars() {
    let char_w = UnicodeWidthStr::width(ch.encode_utf8(&mut [0; 4]) as &str);
    if col as usize + char_w > width {
        break;
    }
    buf.set(x, y, Cell::new(ch.to_string(), style.clone()));
    col += char_w as u16;
}

// After (1 line)
col += render_text(&segment.text, inner.position.x + col, y, remaining, &style, buf);
```

**Impact:** Reduces 90+ lines across all widgets, centralizes UTF-8 rendering logic.

---

## 5. MODERATE: Duplicated `ensure_selected_visible()` Logic

### Current State

**Identical implementation in 3 widgets:**
- `select_list.rs` (lines 376-389, 14 lines)
- `data_table.rs` (lines 405-418, 14 lines)
- `tree.rs` (lines 321-334, 14 lines)

**Total duplication:** 42 lines

### Code Pattern

```rust
// DUPLICATED 3 TIMES
fn ensure_selected_visible(&mut self, visible_height: usize) {
    if visible_height == 0 {
        return;
    }
    if self.selected < self.scroll_offset {
        self.scroll_offset = self.selected;
    }
    if self.selected >= self.scroll_offset + visible_height {
        self.scroll_offset = self
            .selected
            .saturating_sub(visible_height.saturating_sub(1));
    }
}
```

### Recommendation

**Create a utility function:**

```rust
// In crates/fae-core/src/widget/scroll.rs (new file)
/// Adjust scroll offset to ensure the selected index is visible.
pub fn ensure_visible(
    selected: usize,
    scroll_offset: &mut usize,
    visible_height: usize,
) {
    if visible_height == 0 {
        return;
    }
    if selected < *scroll_offset {
        *scroll_offset = selected;
    }
    if selected >= *scroll_offset + visible_height {
        *scroll_offset = selected.saturating_sub(visible_height.saturating_sub(1));
    }
}
```

**Widget usage becomes:**

```rust
// Before (14 lines)
fn ensure_selected_visible(&mut self, visible_height: usize) {
    if visible_height == 0 {
        return;
    }
    // ... 10 more lines
}

// After (1 line)
scroll::ensure_visible(self.selected, &mut self.scroll_offset, visible_height);
```

**Impact:** Eliminates 42 lines of duplication.

---

## 6. MINOR: Repeated Page Size Magic Number

### Current State

**Hardcoded `20` appears 30+ times across all widgets:**

```rust
// In keyboard event handlers
KeyCode::PageUp => {
    let page = 20;  // ← MAGIC NUMBER REPEATED 10+ TIMES
    self.selected = self.selected.saturating_sub(page);
    self.ensure_selected_visible(20);  // ← ALSO HERE
    EventResult::Consumed
}
```

**Locations:**
- `rich_log.rs`: lines 263, 269, 469, 470, etc.
- `select_list.rs`: lines 469, 481, 482, etc.
- `data_table.rs`: lines 506, 552, etc.
- `tree.rs`: lines 512, 536, etc.
- `diff_view.rs`: lines 446, 451

### Recommendation

**Define as a constant:**

```rust
// In crates/fae-core/src/widget/mod.rs or each widget
/// Default page size for Page Up/Down navigation.
const DEFAULT_PAGE_SIZE: usize = 20;
```

**Usage:**

```rust
// Before
let page = 20;
self.ensure_selected_visible(20);

// After
let page = DEFAULT_PAGE_SIZE;
self.ensure_selected_visible(DEFAULT_PAGE_SIZE);
```

**Impact:**
- Eliminates magic numbers
- Makes page size configurable in the future (could become a widget field)
- Clarifies intent

---

## 7. Additional Observations

### Positive Patterns (Do Not Change)

1. **Consistent Widget Architecture** - All widgets follow the same pattern:
   - Builder pattern with `with_*()` methods
   - `inner_area()` → `render_border()` → render content
   - Comprehensive tests (70-150 tests per widget)

2. **UTF-8 Safety** - All widgets use `truncate_to_display_width()` correctly

3. **Proper Error Handling** - No `.unwrap()` or `.expect()` in production code

4. **Good Test Coverage** - 439 tests total, covering edge cases and UTF-8 scenarios

### Non-Issues (No Action Needed)

1. **Different rendering implementations** - Each widget has unique rendering logic (unified vs. side-by-side, tree indentation, table columns). This is expected and should NOT be consolidated.

2. **Type-specific logic** - Generic `Tree<T>`, `SelectList<T>` vs. concrete `DirectoryTree`. This is appropriate specialization.

3. **Separate `border_chars()` functions** - While duplicated, they're small and self-contained. However, consolidation is still recommended for maintainability.

---

## Consolidation Plan

### Proposed New Files

1. **`crates/fae-core/src/widget/border.rs`** (NEW)
   - `BorderStyle::chars()` method (replaces 5 × `border_chars()`)
   - `render_border()` utility function (replaces 5 × `render_border()` methods)
   - `inner_area()` utility function (replaces 5 × `inner_area()` methods)

2. **`crates/fae-core/src/widget/render_util.rs`** (NEW)
   - `render_text()` function (consolidates UTF-8 rendering loops)

3. **`crates/fae-core/src/widget/scroll.rs`** (NEW)
   - `ensure_visible()` function (consolidates scroll adjustment logic)
   - `DEFAULT_PAGE_SIZE` constant

### Migration Order

1. **Phase 1 (Immediate):** Move `border_chars()` to `BorderStyle::chars()` method
2. **Phase 2 (Immediate):** Create `border::render_border()` utility
3. **Phase 3 (Immediate):** Create `border::inner_area()` or `Rect::inner_with_border()`
4. **Phase 4 (Optional):** Create `render_util::render_text()` helper
5. **Phase 5 (Optional):** Create `scroll::ensure_visible()` utility

### Impact Summary

| Refactoring | Lines Saved | Files Affected | Priority |
|-------------|-------------|----------------|----------|
| `border_chars()` consolidation | 135 | 5 | CRITICAL |
| `render_border()` consolidation | 184 | 5 | CRITICAL |
| `inner_area()` consolidation | 84 | 5 | CRITICAL |
| UTF-8 rendering helper | 90+ | 6 | MODERATE |
| `ensure_selected_visible()` | 42 | 3 | MODERATE |
| Page size constant | 30+ | 5 | MINOR |
| **TOTAL** | **565+** | **6** | |

---

## Recommendations

### Immediate Actions (CRITICAL)

1. **Create `crates/fae-core/src/widget/border.rs`** with:
   - `BorderStyle::chars()` method
   - `render_border()` utility function
   - `inner_area()` utility function (or add to `Rect`)

2. **Update all 5 widgets** to use the new border utilities:
   - `rich_log.rs`
   - `select_list.rs`
   - `data_table.rs`
   - `tree.rs`
   - `diff_view.rs`

3. **Run full test suite** - All 439 existing tests should pass unchanged

### Optional Follow-Up (MODERATE)

4. **Create `crates/fae-core/src/widget/render_util.rs`** with `render_text()` helper

5. **Create `crates/fae-core/src/widget/scroll.rs`** with `ensure_visible()` and constants

### Future Considerations (MINOR)

6. **Make page size configurable** - Add `page_size` field to widgets that support paging

---

## Conclusion

Phase 4.2 code is well-structured and follows project standards, but contains **565+ lines of duplication** that should be consolidated. The most critical issue is the duplicated border logic across 5 files (403 lines). Consolidating into shared utilities will:

- Reduce code size by ~19% (565 / 2,907 lines)
- Improve maintainability (single source of truth for borders)
- Simplify future border enhancements (new border styles, themes, etc.)
- Maintain existing test coverage (no behavioral changes)

**No code changes were made during this review.** All findings are recommendations for future refactoring.
