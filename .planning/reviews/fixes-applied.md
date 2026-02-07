# Fixes Applied

**Date**: 2026-02-07
**Iteration**: 1

## Fix 1: overflow:auto maps to Scroll instead of Visible

**Source**: Codex review (P2 finding)
**File**: `crates/fae-core/src/layout/style_converter.rs`
**Line**: 243

**Before**:
```rust
"scroll" => Overflow::Scroll,
"clip" => Overflow::Clip,
_ => Overflow::Visible,  // "auto" fell through here
```

**After**:
```rust
"scroll" => Overflow::Scroll,
"auto" => Overflow::Scroll,
"clip" => Overflow::Clip,
_ => Overflow::Visible,
```

**Test Added**: `to_overflow_auto()` - verifies auto maps to Scroll

## Verification

- cargo clippy: PASS (0 warnings)
- cargo test: PASS (602 tests, 0 failures)
- cargo fmt: PASS
