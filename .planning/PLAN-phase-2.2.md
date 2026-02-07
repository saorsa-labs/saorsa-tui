# Phase 2.2: Selector Matching & Cascade

**Milestone**: 2 — CSS & Layout Engine
**Prerequisite**: Phase 2.1 (TCSS Parser) — Complete
**Estimated Tests**: 80+

---

## Overview

Build the CSS selector matching engine and cascade resolver. This connects the TCSS parser (Phase 2.1) to the widget system by introducing a widget tree with metadata, selector matching against that tree, cascade resolution for computing final styles, and match caching for performance.

### New Files
```
crates/fae-core/src/tcss/
├── tree.rs         # WidgetTree, WidgetNode, WidgetState
├── matcher.rs      # Selector matching engine
├── cascade.rs      # Cascade resolution, ComputedStyle
└── cache.rs        # Match result caching
```

### Modified Files
```
crates/fae-core/src/tcss/mod.rs   # Add new module declarations + re-exports
```

---

## Task 1: Widget Tree & Node Types (tree.rs)

**File**: `crates/fae-core/src/tcss/tree.rs`

Create the widget tree data structure that stores widget metadata needed for CSS selector matching.

### Types to Create

```rust
/// Pseudo-class state flags for a widget.
pub struct WidgetState {
    pub focused: bool,
    pub hovered: bool,
    pub disabled: bool,
    pub active: bool,
}

/// A node in the widget tree with CSS metadata.
pub struct WidgetNode {
    pub id: WidgetId,
    pub type_name: String,           // "Label", "Container", etc.
    pub classes: Vec<String>,        // CSS class names
    pub css_id: Option<String>,      // CSS ID (unique)
    pub state: WidgetState,          // Pseudo-class state
    pub parent: Option<WidgetId>,
    pub children: Vec<WidgetId>,
}

/// The widget tree — stores all nodes with parent/child relationships.
pub struct WidgetTree {
    nodes: HashMap<WidgetId, WidgetNode>,
    root: Option<WidgetId>,
}
```

### Methods on WidgetTree

- `new() -> Self`
- `add_node(node: WidgetNode)` — adds node, sets root if first
- `remove_node(id: WidgetId)` — removes node and unlinks from parent
- `get(id: WidgetId) -> Option<&WidgetNode>`
- `get_mut(id: WidgetId) -> Option<&mut WidgetNode>`
- `root() -> Option<WidgetId>`
- `parent(id: WidgetId) -> Option<&WidgetNode>`
- `children(id: WidgetId) -> &[WidgetId]`
- `ancestors(id: WidgetId) -> Vec<WidgetId>` — parent chain up to root
- `is_first_child(id: WidgetId) -> bool`
- `is_last_child(id: WidgetId) -> bool`
- `child_index(id: WidgetId) -> Option<usize>` — index among siblings
- `len() -> usize`
- `is_empty() -> bool`

### Methods on WidgetNode

- `new(id, type_name) -> Self` — with defaults
- `with_class(class) -> Self` — builder
- `with_id(css_id) -> Self` — builder
- `has_class(name) -> bool`

### Methods on WidgetState

- `new() -> Self` — all false
- `Default` impl

### Tests (8+)

1. `empty_tree` — new tree is empty, root is None
2. `add_root_node` — first node becomes root
3. `add_child_node` — child linked to parent
4. `remove_node` — removes and unlinks
5. `ancestors` — returns parent chain
6. `is_first_last_child` — sibling position queries
7. `child_index` — correct index among siblings
8. `widget_node_builder` — builder pattern works
9. `widget_state_default` — all false

---

## Task 2: Simple Selector Matching (matcher.rs)

**File**: `crates/fae-core/src/tcss/matcher.rs`

Implement the core selector matching logic — matching individual simple selectors and compound selectors against widget nodes.

### Functions/Methods

```rust
/// Check if a simple selector matches a widget node.
pub fn matches_simple(node: &WidgetNode, selector: &SimpleSelector) -> bool

/// Check if a compound selector matches a widget node (all components must match).
pub fn matches_compound(node: &WidgetNode, selector: &CompoundSelector) -> bool
```

### Matching Rules

- `SimpleSelector::Type(name)` — matches if `node.type_name == name`
- `SimpleSelector::Class(name)` — matches if `node.has_class(name)`
- `SimpleSelector::Id(name)` — matches if `node.css_id == Some(name)`
- `SimpleSelector::Universal` — always matches
- `SimpleSelector::PseudoClass(pc)` — delegate to pseudo-class matching (Task 3)

For now, `PseudoClass` matching returns false (placeholder) — Task 3 fills this in.

Compound: ALL components must match (AND logic).

### Tests (8+)

1. `match_type_selector` — `Label` matches node with type_name "Label"
2. `match_type_mismatch` — `Container` doesn't match "Label"
3. `match_class_selector` — `.error` matches node with class "error"
4. `match_class_mismatch` — `.warning` doesn't match node with class "error"
5. `match_id_selector` — `#sidebar` matches node with css_id "sidebar"
6. `match_id_mismatch` — `#header` doesn't match node with css_id "sidebar"
7. `match_universal` — `*` matches any node
8. `match_compound_all` — `Label.error#main` requires all to match
9. `match_compound_partial_fail` — fails if any component doesn't match

---

## Task 3: Pseudo-class Matching

**File**: `crates/fae-core/src/tcss/matcher.rs` (extend)

Add pseudo-class matching support using widget state and tree position.

### Function

```rust
/// Check if a pseudo-class matches a widget node in context.
pub fn matches_pseudo_class(
    tree: &WidgetTree,
    node: &WidgetNode,
    pseudo: &PseudoClass,
) -> bool
```

### Matching Rules

- `:focus` → `node.state.focused`
- `:hover` → `node.state.hovered`
- `:disabled` → `node.state.disabled`
- `:active` → `node.state.active`
- `:first-child` → `tree.is_first_child(node.id)`
- `:last-child` → `tree.is_last_child(node.id)`
- `:nth-child(n)` → `tree.child_index(node.id) == Some(n - 1)` (1-indexed)
- `:even` → `tree.child_index(node.id).map(|i| (i + 1) % 2 == 0)`
- `:odd` → `tree.child_index(node.id).map(|i| (i + 1) % 2 == 1)`

### Update matches_simple

Update the `SimpleSelector::PseudoClass` branch to call `matches_pseudo_class`. This means `matches_simple` and `matches_compound` need to accept `&WidgetTree` as a parameter.

### Tests (10+)

1. `match_focus` — focused node matches :focus
2. `match_focus_unfocused` — unfocused node doesn't match :focus
3. `match_hover` — hovered node matches :hover
4. `match_disabled` — disabled node matches :disabled
5. `match_active` — active node matches :active
6. `match_first_child` — first sibling matches :first-child
7. `match_last_child` — last sibling matches :last-child
8. `match_nth_child` — third child matches :nth-child(3)
9. `match_even` — second child matches :even
10. `match_odd` — first child matches :odd
11. `match_compound_with_pseudo` — `Label:focus` matches focused Label

---

## Task 4: Combinator Matching

**File**: `crates/fae-core/src/tcss/matcher.rs` (extend)

Implement combinator matching by walking the widget tree from the subject element backwards through the selector chain.

### Function

```rust
/// Check if a full selector (with combinators) matches a widget.
pub fn matches_selector(
    tree: &WidgetTree,
    node_id: WidgetId,
    selector: &Selector,
) -> bool
```

### Algorithm

1. Check if `selector.head` matches the target node
2. If selector has no chain, return the result
3. Walk chain from right to left (each entry is `(Combinator, CompoundSelector)`)
4. For `Combinator::Child` — check if the immediate parent matches
5. For `Combinator::Descendant` — walk all ancestors until one matches (or none do)
6. Each step narrows the "current" node to the matched ancestor
7. All chain entries must match for the full selector to match

### Tests (10+)

1. `match_simple_selector` — `Label` with no chain matches Label
2. `match_child_combinator` — `Container > Label` matches Label with Container parent
3. `match_child_wrong_parent` — `Container > Label` doesn't match Label with Header parent
4. `match_descendant_combinator` — `Container Label` matches Label nested under Container
5. `match_descendant_deep` — `Container Label` matches even if not direct child
6. `match_descendant_miss` — `Container Label` doesn't match if Container not in ancestors
7. `match_complex_chain` — `A > B C` matches C with B parent somewhere under A grandparent
8. `match_three_levels` — `A > B > C` requires exact parent chain
9. `match_mixed_combinators` — `A B > C` descendant then child
10. `match_selector_list` — any selector in list matching means match

---

## Task 5: Rule Matching Engine

**File**: `crates/fae-core/src/tcss/matcher.rs` (extend)

Build the main matching engine that takes a stylesheet and finds all rules that apply to a given widget.

### Types

```rust
/// A matched rule with its specificity and source order.
#[derive(Clone, Debug)]
pub struct MatchedRule {
    pub specificity: (u16, u16, u16),
    pub source_order: usize,        // Index in stylesheet
    pub declarations: Vec<Declaration>,
}

/// The selector matcher — takes a stylesheet, matches against a tree.
pub struct StyleMatcher {
    rules: Vec<(SelectorList, Vec<Declaration>, usize)>,  // selectors, decls, source order
}
```

### Methods

- `StyleMatcher::new(stylesheet: &Stylesheet) -> Self` — extracts rules with source ordering
- `StyleMatcher::match_widget(tree: &WidgetTree, id: WidgetId) -> Vec<MatchedRule>` — returns all matching rules
- `StyleMatcher::matches_any(tree: &WidgetTree, id: WidgetId, selectors: &SelectorList) -> Option<(u16,u16,u16)>` — returns highest specificity match

### Tests (8+)

1. `match_no_rules` — empty stylesheet matches nothing
2. `match_single_rule` — one matching rule returned
3. `match_multiple_rules` — multiple rules can match same widget
4. `match_source_order` — source order is preserved
5. `match_specificity_attached` — specificity is correctly attached to each match
6. `match_no_match` — non-matching rules excluded
7. `match_selector_list_any` — rule matches if any selector in list matches
8. `match_real_stylesheet` — realistic stylesheet with mixed selectors

---

## Task 6: Cascade Resolution (cascade.rs)

**File**: `crates/fae-core/src/tcss/cascade.rs`

Implement the CSS cascade algorithm that resolves matched rules into a final computed style.

### Types

```rust
/// The computed style for a widget — final resolved property values.
#[derive(Clone, Debug, Default)]
pub struct ComputedStyle {
    properties: HashMap<PropertyName, CssValue>,
}

/// A cascade resolver.
pub struct CascadeResolver;
```

### ComputedStyle Methods

- `new() -> Self`
- `get(prop: PropertyName) -> Option<&CssValue>`
- `set(prop: PropertyName, value: CssValue)`
- `has(prop: PropertyName) -> bool`
- `len() -> usize`
- `is_empty() -> bool`
- `iter() -> impl Iterator<Item = (&PropertyName, &CssValue)>`

### CascadeResolver Methods

- `resolve(matches: &[MatchedRule]) -> ComputedStyle`

### Cascade Algorithm

1. Separate declarations into normal and !important
2. Sort normal declarations by (specificity, source_order) ascending
3. Sort !important declarations by (specificity, source_order) ascending
4. Apply normal declarations first (later entries override earlier)
5. Apply !important declarations last (they override everything)
6. Return final ComputedStyle

### Tests (10+)

1. `empty_matches_empty_style` — no rules = no properties
2. `single_rule_applied` — single rule sets properties
3. `later_rule_overrides` — lower-specificity rule overridden by later
4. `higher_specificity_wins` — higher specificity wins regardless of order
5. `important_overrides_specificity` — !important beats higher specificity
6. `important_vs_important` — when both are !important, specificity wins
7. `multiple_properties_merged` — different properties from different rules merge
8. `same_property_last_wins` — same property, same specificity, last source order wins
9. `computed_style_accessors` — get/set/has/len/is_empty work
10. `computed_style_iteration` — iter() returns all properties
11. `real_cascade_example` — realistic multi-rule cascade scenario

---

## Task 7: Match Caching (cache.rs)

**File**: `crates/fae-core/src/tcss/cache.rs`

Implement a cache for match results to avoid re-matching on every render.

### Types

```rust
/// Cache for matched rules per widget.
pub struct MatchCache {
    entries: HashMap<WidgetId, Vec<MatchedRule>>,
    dirty: HashSet<WidgetId>,
}
```

### Methods

- `new() -> Self`
- `get(id: WidgetId) -> Option<&Vec<MatchedRule>>` — returns None if dirty
- `insert(id: WidgetId, matches: Vec<MatchedRule>)`
- `invalidate(id: WidgetId)` — mark widget as dirty
- `invalidate_all()` — mark all widgets as dirty (stylesheet change)
- `invalidate_subtree(tree: &WidgetTree, id: WidgetId)` — invalidate widget and all descendants
- `is_dirty(id: WidgetId) -> bool`
- `len() -> usize`
- `is_empty() -> bool`
- `clear()` — remove all entries

### Tests (8+)

1. `empty_cache` — new cache is empty
2. `insert_and_get` — cached entry returned
3. `get_missing` — returns None for uncached widget
4. `invalidate_single` — dirty entry returns None
5. `invalidate_all` — all entries become dirty
6. `invalidate_subtree` — widget and descendants invalidated
7. `is_dirty_check` — reports correct dirty state
8. `clear_removes_all` — clear empties cache

---

## Task 8: Integration & Wire-Up

**Files**:
- `crates/fae-core/src/tcss/mod.rs` — add new module declarations and re-exports
- `crates/fae-core/src/tcss/tree.rs`, `matcher.rs`, `cascade.rs`, `cache.rs` — verify integration

### Module Declarations

Add to `tcss/mod.rs`:
```rust
pub mod cache;
pub mod cascade;
pub mod matcher;
pub mod tree;
```

### Re-exports

```rust
pub use cache::MatchCache;
pub use cascade::{CascadeResolver, ComputedStyle};
pub use matcher::{MatchedRule, StyleMatcher};
pub use tree::{WidgetNode, WidgetState, WidgetTree};
```

### Integration Tests (8+)

Write integration tests in `tcss/mod.rs` that test the full pipeline:

1. `full_pipeline_simple` — parse stylesheet → build tree → match → cascade → computed style
2. `full_pipeline_specificity` — higher specificity rule wins in full pipeline
3. `full_pipeline_important` — !important override works end-to-end
4. `full_pipeline_child_combinator` — `Container > Label` works in full pipeline
5. `full_pipeline_descendant` — `Container Label` matches deep nesting
6. `full_pipeline_pseudo_class` — `:focus` changes matching result
7. `full_pipeline_cached_match` — second query hits cache
8. `full_pipeline_invalidation` — state change invalidates cache and re-matches correctly

---

## Summary

| Task | Description | Tests | New File |
|------|-------------|-------|----------|
| 1 | Widget Tree & Node Types | 8+ | tree.rs |
| 2 | Simple Selector Matching | 8+ | matcher.rs |
| 3 | Pseudo-class Matching | 10+ | matcher.rs (extend) |
| 4 | Combinator Matching | 10+ | matcher.rs (extend) |
| 5 | Rule Matching Engine | 8+ | matcher.rs (extend) |
| 6 | Cascade Resolution | 10+ | cascade.rs |
| 7 | Match Caching | 8+ | cache.rs |
| 8 | Integration & Wire-Up | 8+ | mod.rs (extend) |
| **Total** | | **70+** | |
