# Phase 2.4: Taffy Layout Integration

**Milestone**: 2 — CSS & Layout Engine
**Phase**: 2.4 — Taffy Layout Integration
**Status**: Planned
**Estimated Tests**: 92+

## Overview

Integrate the Taffy layout engine (from Servo) to provide CSS Flexbox and Grid
layout for the widget tree. Maps TCSS computed styles to Taffy styles, runs
layout computation, and produces integer-cell rects for terminal rendering.

**Key decisions:**
- Taffy v0.7 as layout backend
- `LayoutEngine` wraps `TaffyTree`, maps `WidgetId` ↔ `taffy::NodeId`
- f32→u16 rounding: floor for position, round for size
- New `CssValue::List` variant for grid template tracks
- Existing `layout.rs` refactored into `layout/` module directory
- Dock layout as pre-processing step before Taffy
- Scroll regions tracked separately via `ScrollManager`

## Dependencies

- Phases 2.1-2.3 complete (491 tests, zero warnings)
- `taffy = "0.7"` crate

---

## Task 1: Add Taffy Dependency and CssValue::List Variant

**Files to modify:**
- `Cargo.toml` (workspace) — add `taffy = "0.7"`
- `crates/saorsa-core/Cargo.toml` — add `taffy = { workspace = true }`
- `crates/saorsa-core/src/tcss/value.rs` — add `List(Vec<CssValue>)` variant

**New types:**
- `CssValue::List(Vec<CssValue>)` — ordered list of values for grid templates

**Tests (8+):**
1. `value_list_empty` — empty list
2. `value_list_single` — single element
3. `value_list_multiple` — `[Fr(1.0), Fr(2.0), Length(Cells(100))]`
4. `value_list_clone` — clone preserves contents
5. `value_list_eq` — equality works
6. `value_list_nested` — list containing list (edge case)
7. `value_fr_basic` — `CssValue::Fr(1.0)` coverage
8. `value_list_mixed_types` — Fr, Length, Keyword together

---

## Task 2: Grid Template and Fr Parsing

**Files to modify:**
- `crates/saorsa-core/src/tcss/parser.rs` — add grid template + fr parsing

**New functions:**
- `parse_grid_template()` — parse track list like `1fr 2fr 100` → `CssValue::List`
- `parse_grid_placement()` — parse placement like `span 2`, `1 / 3` → `CssValue`

**Modified:**
- `parse_property_value()` — route GridTemplate*/GridColumn/GridRow to new parsers

**Tests (10+):**
1. `parse_grid_template_single_fr` — `1fr` → `CssValue::Fr(1.0)`
2. `parse_grid_template_multiple_fr` — `1fr 2fr 1fr` → `CssValue::List`
3. `parse_grid_template_mixed` — `1fr 100 2fr`
4. `parse_grid_template_with_percent` — `25% 1fr 25%`
5. `parse_grid_template_auto` — `auto 1fr auto`
6. `parse_grid_template_single_cells` — `100` → `CssValue::Length(Cells(100))`
7. `parse_grid_placement_integer` — `2` → integer
8. `parse_grid_placement_span` — `span 3` → keyword
9. `parse_grid_placement_range` — `1 / 3` → keyword
10. `parse_fr_in_stylesheet` — full stylesheet roundtrip

---

## Task 3: Style Converter — ComputedStyle to Taffy Style

**New file:** `crates/saorsa-core/src/layout/style_converter.rs`

**New functions:**
- `computed_to_taffy(computed: &ComputedStyle) -> taffy::Style`
- `to_dimension(value: &CssValue) -> taffy::Dimension`
- `to_length_percentage(value: &CssValue) -> taffy::LengthPercentage`
- `to_length_percentage_auto(value: &CssValue) -> taffy::LengthPercentageAuto`
- `to_display(value: &CssValue) -> taffy::Display`
- `to_flex_direction(value: &CssValue) -> taffy::FlexDirection`
- `to_flex_wrap(value: &CssValue) -> taffy::FlexWrap`
- `to_justify_content(value: &CssValue) -> Option<taffy::JustifyContent>`
- `to_align_items(value: &CssValue) -> Option<taffy::AlignItems>`
- `to_align_self(value: &CssValue) -> Option<taffy::AlignSelf>`
- `to_overflow(value: &CssValue) -> taffy::Overflow`
- `to_grid_tracks(value: &CssValue) -> Vec<taffy::TrackSizingFunction>`
- `to_grid_placement(value: &CssValue) -> taffy::GridPlacement`

**Mapping table:**

| TCSS Property | Taffy Field | Notes |
|---|---|---|
| Display | display | flex/grid/block/none |
| FlexDirection | flex_direction | row/column/etc. |
| FlexWrap | flex_wrap | nowrap/wrap |
| JustifyContent | justify_content | flex-start/center/etc. |
| AlignItems | align_items | stretch/center/etc. |
| AlignSelf | align_self | same as AlignItems |
| FlexGrow | flex_grow | Int/Float → f32 |
| FlexShrink | flex_shrink | Int/Float → f32 |
| FlexBasis | flex_basis | Length → Dimension |
| Gap | gap | Length → Size<LP> |
| Width/Height | size | Length → Dimension |
| MinWidth/MinHeight | min_size | Length → Dimension |
| MaxWidth/MaxHeight | max_size | Length → Dimension |
| Margin* | margin | Length → Rect<LPA> |
| Padding* | padding | Length → Rect<LP> |
| Border* | border | 1 cell if present |
| Overflow* | overflow | hidden/scroll/visible |
| GridTemplate* | grid_template_* | List → Vec<TSF> |
| GridColumn/Row | grid_column/row | Placement |

**Tests (14+):**
1. `convert_empty_style` — default mapping
2. `convert_display_flex/grid/none` — display variants
3-8. Flex property conversions (direction, wrap, justify, align, grow/shrink)
9-11. Dimension conversions (width, height, min/max)
12. `convert_margin_all_sides` — margin shorthand + individual
13. `convert_padding_individual` — per-side padding
14. `convert_border_width` — border → 1 cell
15. `convert_overflow_hidden` — overflow mapping
16. `convert_grid_template_fr` — grid template tracks

---

## Task 4: Layout Engine Core

**New file:** `crates/saorsa-core/src/layout/engine.rs`

**New types:**
- `LayoutRect { x: u16, y: u16, width: u16, height: u16 }`
- `LayoutError { WidgetNotFound, TaffyError, NoRoot }`
- `LayoutEngine { taffy, widget_to_node, node_to_widget, root }`

**Methods:**
- `new() -> Self`
- `add_node(widget_id, style) -> Result<(), LayoutError>`
- `add_node_with_children(widget_id, style, children) -> Result<(), LayoutError>`
- `set_root(widget_id) -> Result<(), LayoutError>`
- `update_style(widget_id, style) -> Result<(), LayoutError>`
- `remove_node(widget_id) -> Result<(), LayoutError>`
- `compute(available_width, available_height) -> Result<(), LayoutError>`
- `layout(widget_id) -> Result<LayoutRect, LayoutError>`
- `layout_rect(widget_id) -> Result<Rect, LayoutError>`
- `has_node(widget_id) -> bool`
- `node_count() -> usize`

**Helper functions:**
- `round_position(f32) -> u16` — floor for position
- `round_size(f32) -> u16` — round for size

**Tests (12+):**
1-2. Empty engine, add leaf node
3-4. Add with children, set root
5-6. Remove node, update style
7-8. Compute single node, two children row
9-10. Two children column, rect conversion
11-12. Widget not found error, no root error
13-14. Rounding tests

---

## Task 5: Flexbox Layout Integration

**Files to modify:** `crates/saorsa-core/src/layout/engine.rs` (extend)

**New method:**
- `build_from_tree(widget_tree, styles) -> Result<(), LayoutError>`

**Tests (12+):**
1. `flex_row_equal_grow` — 3 children flex-grow:1 split equally
2. `flex_column_equal_grow` — column direction
3. `flex_row_unequal_grow` — 1:2:1 ratio
4. `flex_column_fixed_and_grow` — fixed + flexible
5. `flex_wrap_overflow` — children wrap
6. `flex_justify_center` — centered children
7. `flex_justify_space_between` — spaced evenly
8. `flex_align_items_center` — cross-axis center
9. `flex_align_self_override` — individual override
10. `flex_nested` — nested flex containers
11. `flex_with_gap` — gap property
12. `flex_full_pipeline` — TCSS→match→cascade→layout end-to-end

---

## Task 6: Grid Layout and Box Model

**Files to modify:** `crates/saorsa-core/src/layout/engine.rs` (extend)

**Tests (12+):**
Grid:
1. `grid_two_columns_equal` — `1fr 1fr`
2. `grid_three_columns_fr` — `1fr 2fr 1fr`
3. `grid_columns_mixed_units` — `100 1fr`
4. `grid_rows_and_columns` — 2×2 grid
5. `grid_placement_span` — span 2 columns
6. `grid_placement_explicit` — specific row/column
7. `grid_auto_placement` — auto-placed children

Box model:
8. `box_model_padding_shrinks_content` — padding effect
9. `box_model_margin_creates_space` — margin between siblings
10. `box_model_border_width` — 1 cell per side
11. `box_model_combined` — margin + border + padding
12. `box_model_percentage_margin` — percentage-based

---

## Task 7: Dock Layout and Scroll Regions

**New files:**
- `crates/saorsa-core/src/layout/dock.rs`
- `crates/saorsa-core/src/layout/scroll.rs`

**dock.rs types:**
- `DockPosition { Top, Bottom, Left, Right }`
- `extract_dock(style) -> Option<DockPosition>`
- `partition_docked(children, styles) -> (Vec<(WidgetId, DockPosition)>, Vec<WidgetId>)`
- `compute_dock_layout(area, docked, styles) -> (Vec<(WidgetId, LayoutRect)>, Rect)`

**scroll.rs types:**
- `OverflowBehavior { Visible, Hidden, Scroll, Auto }`
- `ScrollState { offset_x, offset_y, content_width/height, viewport_width/height }`
- `ScrollManager { regions }`

**ScrollManager methods:**
- `new()`, `register()`, `scroll_by()`, `scroll_to()`
- `get()`, `can_scroll_x/y()`, `visible_rect()`, `remove()`

**Other:**
- `extract_overflow(style) -> (OverflowBehavior, OverflowBehavior)`

**Tests (14+):**
Dock: extract, partition, top/left/right, all four, with size (7)
Scroll: state, register, scroll_by clamp, scroll_to, can_scroll, visible_rect, extract_overflow (10)

---

## Task 8: Module Structure, Re-exports, and Integration Tests

**Files to modify/create:**
- `crates/saorsa-core/src/layout.rs` → refactor into `crates/saorsa-core/src/layout/mod.rs`
- `crates/saorsa-core/src/layout/mod.rs` — re-exports + existing code
- `crates/saorsa-core/src/lib.rs` — update re-exports

**Module structure:**
```
layout/
  mod.rs              # Existing Layout/Direction/Constraint/Dock + re-exports
  engine.rs           # LayoutEngine
  style_converter.rs  # ComputedStyle → taffy::Style
  dock.rs             # Dock layout
  scroll.rs           # Scroll regions
```

**Re-exports from layout/mod.rs:**
- `LayoutEngine, LayoutError, LayoutRect`
- `DockPosition, compute_dock_layout, extract_dock, partition_docked`
- `OverflowBehavior, ScrollManager, ScrollState, extract_overflow`
- `computed_to_taffy`

**Integration tests (10+):**
1. `integration_parse_to_layout` — TCSS → match → cascade → layout
2. `integration_flex_sidebar_layout` — sidebar + main
3. `integration_grid_dashboard` — 2×3 grid
4. `integration_dock_header_footer` — dock:top/bottom + flex content
5. `integration_nested_flex_grid` — flex parent, grid children
6. `integration_box_model_spacing` — margin/padding affect positions
7. `integration_scroll_region_setup` — overflow:scroll creates scroll state
8. `integration_theme_affects_layout` — theme variable changes width
9. `integration_zero_size_area` — no panic on zero area
10. `integration_large_tree` — 100-node tree

---

## Summary

| Task | Description | Tests | New Files |
|------|-------------|-------|-----------|
| 1 | Taffy dep + CssValue::List | 8+ | — |
| 2 | Grid template & fr parsing | 10+ | — |
| 3 | Style converter | 14+ | layout/style_converter.rs |
| 4 | Layout engine core | 12+ | layout/engine.rs |
| 5 | Flexbox integration | 12+ | — |
| 6 | Grid + box model | 12+ | — |
| 7 | Dock + scroll regions | 14+ | layout/dock.rs, layout/scroll.rs |
| 8 | Module structure + integration | 10+ | layout/mod.rs |
| **Total** | | **92+** | **4 new files** |
