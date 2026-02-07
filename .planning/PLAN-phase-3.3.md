# Phase 3.3: Unicode & Double-Width Hardening

**Milestone**: 3 — Compositor & Advanced Rendering
**Phase**: 3.3 — Unicode & Double-Width
**Status**: Planned
**Estimated Tests**: 60+

## Overview

Phase 3.1 and 3.2 already implemented core Unicode support:
- `unicode-width` and `unicode-segmentation` crates integrated
- `Segment::split_at()` handles wide chars and combining marks
- `Cell` tracks grapheme + width, `ScreenBuffer::set()` auto-creates continuation cells
- Compositor `write_segments_to_buffer()` iterates graphemes with correct widths
- `grapheme_widths()`, `char_count()` helpers on Segment

This phase hardens edge cases, adds buffer-level Unicode protection, and creates
a comprehensive Unicode test suite for long-term regression prevention.

### What This Phase Adds

1. **Buffer-level wide char protection** — overwriting half a wide char, right-edge clipping
2. **Multi-codepoint emoji handling** — ZWJ sequences, flag sequences, skin tone modifiers
3. **Tab expansion** — convert tabs to spaces at configurable width
4. **Control character filtering** — strip or replace non-printable chars
5. **Unicode normalization awareness** — document NFC vs NFD handling
6. **Comprehensive regression test suite** — exhaustive Unicode edge cases

## Dependencies

- Phase 3.2 complete (739 tests)
- Existing `unicode-width` 0.2 and `unicode-segmentation` 1.12

---

## Task 1: Buffer-Level Wide Character Protection

**Files to modify:**
- `crates/saorsa-core/src/buffer.rs`

**Description**: Harden `ScreenBuffer::set()` for edge cases when overwriting cells
that are part of a wide character.

**Changes:**
- When setting a narrow cell over position X where X is a continuation cell (width=0),
  blank the preceding wide character's primary cell
- When setting a narrow cell over position X where X is a wide char (width=2),
  blank the continuation cell at X+1
- When setting a wide character at the last column (width would overflow), replace with space
- Add `pub fn set_safe(&mut self, x: u16, y: u16, cell: Cell)` that handles all these cases
  (keep existing `set()` for backward compat, have it delegate to `set_safe()`)

**Tests (10+):**
1. Overwrite continuation cell → preceding wide char becomes space
2. Overwrite wide char with narrow → continuation becomes space
3. Wide char at last column → replaced with space
4. Wide char at second-to-last column → fits correctly
5. Set narrow over narrow → no side effects
6. Set wide over wide → old continuation cleaned up
7. Multiple wide chars in sequence
8. Overwrite middle of sequence of wide chars
9. Set at column 0 with wide char
10. Set wide char where continuation would be out of bounds

---

## Task 2: Multi-Codepoint Emoji Handling

**Files to modify:**
- `crates/saorsa-core/src/segment.rs` — test coverage
- `crates/saorsa-core/src/cell.rs` — test coverage

**Description**: Verify and test handling of complex emoji sequences:
- ZWJ (Zero Width Joiner) families: 👨‍👩‍👧 (3+ codepoints)
- Flag sequences: 🇺🇸 (regional indicator pairs)
- Skin tone modifiers: 👍🏽 (base + modifier)
- Keycap sequences: #️⃣ (digit + VS16 + combining enclosing keycap)

The `unicode-width` and `unicode-segmentation` crates handle these correctly when
using grapheme cluster iteration, but we need tests proving our code doesn't break.

**Tests (10+):**
1. ZWJ family emoji width (should be 2)
2. ZWJ family emoji as single grapheme cluster
3. Flag emoji width (should be 2)
4. Flag emoji as single grapheme cluster
5. Skin tone modifier width
6. Split segment containing ZWJ emoji
7. Cell created from ZWJ emoji has correct width
8. Buffer set with ZWJ emoji creates continuation
9. Compositor handles ZWJ emoji in layers
10. Mixed text with complex emoji sequences

---

## Task 3: Tab Expansion and Control Character Handling

**Files to create:**
- `crates/saorsa-core/src/text.rs` (new module)

**Description**: Create a text preprocessing module that handles tab expansion
and control character filtering before text enters the rendering pipeline.

**Types and functions:**
```rust
/// Configuration for text preprocessing.
pub struct TextConfig {
    /// Tab stop width (default: 8).
    pub tab_width: u8,
}

/// Expand tabs to spaces according to tab stop positions.
pub fn expand_tabs(text: &str, tab_width: u8) -> String

/// Replace control characters (C0 except \t\n, C1, etc.) with a
/// replacement character or remove them entirely.
pub fn filter_control_chars(text: &str) -> String

/// Full preprocessing: expand tabs, filter controls.
pub fn preprocess(text: &str, config: &TextConfig) -> String
```

**Don't forget:** Add `pub mod text;` and exports to `lib.rs`.

**Tests (10+):**
1. expand_tabs with single tab → correct spaces
2. expand_tabs with tab at various positions → aligned to stops
3. expand_tabs with no tabs → unchanged
4. expand_tabs with custom width (2, 4, 8)
5. filter_control_chars removes null, bell, escape
6. filter_control_chars preserves newline, tab (pre-expansion)
7. filter_control_chars with clean text → unchanged
8. preprocess combines both
9. Empty string handling
10. Unicode text with embedded control chars

---

## Task 4: Compositor Unicode Edge Cases

**Files to modify:**
- `crates/saorsa-core/src/compositor/chop.rs` — edge case tests
- `crates/saorsa-core/src/compositor/compose.rs` — edge case tests

**Description**: Add tests for Unicode edge cases specific to the compositor's
cut/chop/compose pipeline.

**Tests (8+):**
1. Wide char split at compositor cut boundary → space padding applied
2. Combining mark at layer boundary → stays with base char
3. ZWJ emoji at layer overlap boundary → handled as single unit
4. Layer with tab-expanded text composes correctly
5. Multiple wide chars across two overlapping layers
6. Layer containing only zero-width chars (combining marks without base)
7. Empty string segments in a layer
8. Very long grapheme cluster (many combining marks)

---

## Task 5: Segment Unicode Robustness

**Files to modify:**
- `crates/saorsa-core/src/segment.rs` — additional methods and tests

**Description**: Add methods for common Unicode text operations on segments.

**New methods:**
- `pub fn truncate_to_width(&self, max_width: usize) -> Segment` — truncate segment
  to at most `max_width` display columns, handling wide char boundaries
- `pub fn pad_to_width(&self, target_width: usize) -> Segment` — pad segment with
  spaces to reach `target_width` display columns

**Tests (8+):**
1. truncate_to_width with ASCII — exact fit
2. truncate_to_width cuts before wide char boundary
3. truncate_to_width with width=0 → empty
4. truncate_to_width with width >= segment width → unchanged
5. pad_to_width adds spaces
6. pad_to_width when already at target → unchanged
7. pad_to_width with width < current → unchanged (no truncation)
8. Style preserved through truncation and padding

---

## Task 6: Cell and Buffer Unicode Tests

**Files to modify:**
- `crates/saorsa-core/src/cell.rs` — additional tests
- `crates/saorsa-core/src/buffer.rs` — additional tests

**Description**: Comprehensive test coverage for Cell and ScreenBuffer Unicode handling.

**Tests (8+):**
1. Cell from emoji → correct width
2. Cell from combining mark → width 0
3. Cell from CJK → width 2
4. Cell from ASCII → width 1
5. Buffer: write CJK string across cells → correct continuation
6. Buffer: diff with wide char changes → correct change count
7. Buffer: get_row with mixed ASCII/CJK → correct cell widths
8. Buffer: clear preserves buffer dimensions after wide char writes

---

## Task 7: Integration Tests — Full Unicode Pipeline

**Files to modify:**
- `crates/saorsa-core/src/compositor/mod.rs` — new test module

**Description**: End-to-end tests verifying Unicode text flows correctly through
the entire pipeline: Segment → Compositor → ScreenBuffer → diff → Renderer.

**Tests (8+):**
1. CJK text through full pipeline → correct ANSI output
2. Emoji text through full pipeline → correct ANSI output
3. Mixed script (Latin + CJK + emoji) through pipeline
4. Wide char at screen edge → correct clipping
5. Combining marks through pipeline → preserved in output
6. Tab-expanded text through pipeline
7. Style applied to wide chars through pipeline
8. Overlapping layers with mixed Unicode scripts

---

## Task 8: Documentation and Module Cleanup

**Files to modify:**
- `crates/saorsa-core/src/lib.rs` — exports
- Add doc comments to any undocumented public items

**Description**: Ensure all Unicode-related public APIs are documented and exported.

**Verification:** `cargo doc --workspace --no-deps` produces zero warnings.

---

## Success Criteria

- ✅ Buffer-level wide char protection working
- ✅ Multi-codepoint emoji correctly handled
- ✅ Tab expansion and control char filtering implemented
- ✅ Compositor Unicode edge cases tested
- ✅ Segment truncation/padding working
- ✅ 60+ new tests added
- ✅ Zero clippy warnings
- ✅ Zero compilation errors
- ✅ 100% public API documentation
