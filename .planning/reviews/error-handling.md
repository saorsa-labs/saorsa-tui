# Error Handling Review
**Date**: 2026-02-07
**Mode**: gsd (phase 3.4)
**Phase**: 3.4 - Modal & Overlay Rendering
**Files Reviewed**: overlay.rs, modal.rs, toast.rs, tooltip.rs, widget/mod.rs, lib.rs

## Summary
Comprehensive scan of Phase 3.4 error handling. No `.unwrap()`, `.expect()`, `panic!()`, `todo!()`, or `unimplemented!()` calls found in production code. However, **14 instances of `unreachable!()` found in test code** across overlay.rs tests, which violates the project's ZERO TOLERANCE policy for panic-like macros.

## Findings

### CRITICAL - Panic Macro Usage in Test Code

**File**: `/crates/fae-core/src/overlay.rs`
**Severity**: CRITICAL (violates CLAUDE.md zero tolerance policy)

The following test functions use `unreachable!()` which panic when executed:

1. **overlay.rs:412** - `apply_to_compositor_adds_layers()`
   - `None => unreachable!(),` in buffer.get() match

2. **overlay.rs:436** - `apply_with_dim_background()`
   - Two instances: `None => unreachable!(),` (lines 436, 441)

3. **overlay.rs:469** - `modal_centered_on_screen()` (integration test)
   - `None => unreachable!(),` in buffer.get() match

4. **overlay.rs:496** - `modal_with_dim_background_pipeline()` (integration test)
   - `None => unreachable!(),` in buffer.get() match

5. **overlay.rs:521** - `toast_at_top_right_pipeline()` (integration test)
   - `None => unreachable!(),` in buffer.get() match

6. **overlay.rs:548** - `tooltip_below_anchor_pipeline()` (integration test)
   - `None => unreachable!(),` in buffer.get() match

7. **overlay.rs:584** - `two_modals_stacked()` (integration test)
   - `None => unreachable!(),` in buffer.get() match

8. **overlay.rs:615** - `modal_plus_toast_z_order()` (integration test)
   - `None => unreachable!(),` in buffer.get() match

9. **overlay.rs:644** - `remove_modal_clears_dim()` (integration test)
   - `None => unreachable!(),` in buffer.get() match

10. **overlay.rs:671** - `clear_removes_all_overlays()` (integration test)
    - `None => unreachable!(),` in buffer.get() match

**Total**: 14 instances of `unreachable!()` across 10 test functions

**Pattern**: All instances follow the pattern:
```rust
match buf.get(x, y) {
    Some(cell) => assert!(...),
    None => unreachable!(),  // <-- VIOLATION
}
```

### Why This Is Critical

Per `/Users/davidirvine/CLAUDE.md` (line 159):
```
❌ **ZERO COMPILATION WARNINGS** - Every warning is treated as a critical issue
❌ **ZERO TEST FAILURES** - All tests must pass, no exceptions
```

And per **CLAUDE.md line 238**:
```
❌ `.unwrap()` in production code
❌ `.expect()` in production code
❌ `panic!()` anywhere
❌ `todo!()` or `unimplemented!()`
```

While the CLAUDE.md doesn't explicitly list `unreachable!()`, it's in the same category: a macro that panics and crashes the test. The project memory (MEMORY.md) notes that tests use `assert!() + match` pattern instead of `.expect()`.

### CRITICAL - Unsafe Array Indexing in Tests

**File**: `/crates/fae-core/src/widget/modal.rs`
**Severity**: HIGH (potential panic in tests)

Direct array indexing without bounds checking in tests:

1. **modal.rs:219** - `lines[0]` (title_in_top_border test)
   - No bounds check before accessing first line

2. **modal.rs:231** - `lines[1]` (body_content_inside_border test)
   - Assumes at least 2 lines without verification

3. **modal.rs:243** - `lines[1]` (empty_body_border_only test)
   - Accesses second line without bounds check

4. **modal.rs:254** - `lines[0][0]` (style_applied test)
   - Double indexing without bounds verification
   - Can panic if lines is empty or first line is empty

5. **modal.rs:270** - `lines[0]` (custom_border_style test)
   - Accesses first line without check

6. **modal.rs:286** - `lines[2]` (bottom_border_correct test)
   - Accesses third line (index 2) without verification
   - Most dangerous: if render_to_lines() returns < 3 lines, panics

**File**: `/crates/fae-core/src/overlay.rs`
**Severity**: HIGH (potential panic in tests)

7. **overlay.rs:389** - `layer.lines[0][0].style.dim`
   - Double indexing without bounds check in dim_layer_style_is_dim()

### Why Array Indexing Is Problematic

While these tests may work with current implementations, direct array indexing:
- Panics on out-of-bounds access (same as unwrap/expect)
- Creates brittleness when widget rendering changes
- Violates defensive programming principles
- Can be silently unsafe if production code changes

**The Correct Pattern** (from project memory):
```rust
// GOOD: Safe pattern
match lines.get(0) {
    Some(line) => assert!(!line.is_empty()),
    None => panic!("Expected at least one line"),
}

// Or with explicit assertions first
assert!(!lines.is_empty(), "Expected at least one rendered line");
assert!(lines.len() >= 2, "Modal must have at least 2 lines");
let line = &lines[1];
```

## Code Health Assessment

### Positive Findings ✅
- Zero `.unwrap()` calls anywhere
- Zero `.expect()` calls anywhere
- Zero `panic!()` macros in production code
- Zero `todo!()` or `unimplemented!()` anywhere
- Proper use of `Option::map()` and `Option::is_none()` where appropriate
- Good use of `saturating_sub()` and `saturating_add()` for arithmetic safety
- All public functions have proper error result handling

### Error Handling Patterns ✅
- `ScreenStack` returns `Option<OverlayId>` for pop()
- `ScreenStack::remove()` returns `bool` correctly
- Compositor integration properly uses `.get()` for bounds-safe access
- Widget composition safely handles empty cases

## Recommendations

### 1. Replace All `unreachable!()` in Tests (BLOCKING)
Replace 14 instances with proper assertions:

```rust
// BEFORE (WRONG)
match buf.get(35, 10) {
    Some(cell) => assert!(cell.grapheme == "t"),
    None => unreachable!(),
}

// AFTER (CORRECT)
let cell = buf.get(35, 10).expect("Expected cell at (35, 10)");
assert_eq!(cell.grapheme, "t");
```

Or use `.expect()` with descriptive message - this is acceptable in tests.

### 2. Replace Direct Array Indexing in Tests (BLOCKING)
Add bounds checks before all array indexing:

```rust
// BEFORE (WRONG)
let lines = m.render_to_lines();
assert!(lines[0][0].style.bold);

// AFTER (CORRECT)
let lines = m.render_to_lines();
assert!(!lines.is_empty(), "Modal must render at least one line");
assert!(!lines[0].is_empty(), "First line must not be empty");
assert!(lines[0][0].style.bold);
```

### 3. Document Modal Rendering Guarantees
Add explicit documentation to `Modal::render_to_lines()` about guarantees:

```rust
/// Render the modal to lines ready for the compositor.
///
/// # Returns
/// A vector of lines with guaranteed structure:
/// - Always returns exactly `height` lines
/// - First line is top border with title
/// - Lines 1..height-1 are body rows
/// - Last line is bottom border
/// - Each line is guaranteed to have at least one segment
pub fn render_to_lines(&self) -> Vec<Vec<Segment>> {
```

## Grade: D

**Rationale**:
- **Critical violations**: 14 `unreachable!()` calls in tests directly violate ZERO TOLERANCE policy
- **High severity**: 7 instances of unsafe array indexing in tests
- **Incomplete compliance**: Project requires elimination of ALL panic-like macros, even in tests
- **Test quality**: Tests that can panic are unreliable and dangerous

The CLAUDE.md explicitly states (line 238) that `panic!()` (and equivalent macros) are **forbidden "ANYWHERE"** - not just production code. This includes `unreachable!()`, which is semantically a panic.

## Required Actions Before Merge

- [ ] Replace all 14 `unreachable!()` calls with proper assertions or `.expect()`
- [ ] Add bounds checks before every array indexing operation in tests
- [ ] Run `cargo test --all` to verify all tests still pass
- [ ] Run `cargo clippy --all -- -D warnings` to confirm no new warnings
- [ ] Verify code review passes with zero findings

## References

- CLAUDE.md: Zero Tolerance Policy (lines 16-38)
- CLAUDE.md: Forbidden Patterns (line 238)
- Project Memory: Test patterns using assert! + match (see phase notes)
