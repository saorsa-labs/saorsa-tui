# Phase 2.3: CSS Variables & Theming

**Milestone**: 2 — CSS & Layout Engine
**Prerequisite**: Phase 2.2 (Selector Matching & Cascade) — Complete
**Estimated Tests**: 70+

---

## Overview

Add CSS custom property (variable) support, a theme system, and live stylesheet reloading to the TCSS engine. Variables use `$variable` syntax (inspired by SCSS/Textual) rather than CSS `var(--name)` for terminal ergonomics. Themes are namespaced variable sets that can be switched at runtime.

### New Files
```
crates/fae-core/src/tcss/
├── variable.rs     # Variable storage, resolution, scoping
├── theme.rs        # Theme definitions, switching, built-in themes
└── reload.rs       # Live stylesheet file watching & reloading
```

### Modified Files
```
crates/fae-core/src/tcss/value.rs      # Add CssValue::Variable variant
crates/fae-core/src/tcss/parser.rs     # Parse $variable references
crates/fae-core/src/tcss/cascade.rs    # Resolve variables during cascade
crates/fae-core/src/tcss/mod.rs        # Add new module declarations + re-exports
crates/fae-core/Cargo.toml             # Add notify crate for file watching
```

---

## Task 1: Variable Value Type (value.rs, parser.rs)

**Files**: `crates/fae-core/src/tcss/value.rs`, `crates/fae-core/src/tcss/parser.rs`

Add a `Variable` variant to `CssValue` and parse `$variable-name` references.

### Changes to CssValue (value.rs)

Add new variant:
```rust
/// A variable reference ($name), to be resolved during cascade.
Variable(String),
```

### Changes to parser.rs

Add a `parse_variable` function and update `parse_property_value` to check for `$` prefix before delegating to property-specific parsers.

The cssparser tokenizer will see `$name` as a `Token::Delim('$')` followed by `Token::Ident(name)`. We need to try parsing a variable reference first in `parse_property_value`.

```rust
/// Try to parse a variable reference ($name).
/// Returns Ok(CssValue::Variable(name)) if found, Err otherwise.
fn try_parse_variable(input: &mut Parser<'_, '_>) -> Option<CssValue> {
    input.try_parse(|p| -> Result<CssValue, cssparser::ParseError<'_, ()>> {
        p.expect_delim('$')?;
        let name = p.expect_ident()?.to_string();
        Ok(CssValue::Variable(name))
    }).ok()
}
```

Update `parse_property_value` to attempt variable parsing before type-specific parsing:
```rust
pub fn parse_property_value(property: &PropertyName, input: &mut Parser<'_, '_>) -> Result<CssValue, TcssError> {
    // Try variable reference first.
    if let Some(var) = try_parse_variable(input) {
        return Ok(var);
    }
    // ... existing match arms ...
}
```

### Tests (6+)

1. `parse_variable_reference` — `$primary` parses to `CssValue::Variable("primary")`
2. `parse_variable_in_color` — `color: $fg;` parses as variable
3. `parse_variable_in_width` — `width: $sidebar-width;` parses as variable
4. `parse_variable_hyphenated` — `$my-var-name` preserves hyphens
5. `parse_non_variable_still_works` — `color: red;` still parses as color
6. `variable_clone_and_eq` — CssValue::Variable clone/eq works

---

## Task 2: Variable Storage & Scoping (variable.rs)

**File**: `crates/fae-core/src/tcss/variable.rs`

Create the variable storage system with scope support. Variables can be defined at different scopes (global/`:root`, theme, widget).

### Types

```rust
/// A set of CSS variable definitions.
#[derive(Clone, Debug, Default)]
pub struct VariableMap {
    vars: HashMap<String, CssValue>,
}

/// A scoped variable environment with layered lookups.
///
/// Resolution order: local → theme → global.
#[derive(Clone, Debug)]
pub struct VariableEnvironment {
    /// Global variables (from :root rules).
    global: VariableMap,
    /// Active theme variables (override global).
    theme: VariableMap,
    /// Local overrides (per-widget inline).
    local: VariableMap,
}
```

### VariableMap Methods

- `new() -> Self`
- `set(name: &str, value: CssValue)` — define a variable
- `get(name: &str) -> Option<&CssValue>` — lookup
- `remove(name: &str)` — undefine a variable
- `contains(name: &str) -> bool`
- `len() -> usize`
- `is_empty() -> bool`
- `iter() -> impl Iterator<Item = (&str, &CssValue)>`
- `merge(other: &VariableMap)` — copy all from other (other wins on conflict)

### VariableEnvironment Methods

- `new() -> Self` — empty environment
- `with_global(global: VariableMap) -> Self`
- `resolve(name: &str) -> Option<&CssValue>` — lookup with scope precedence (local > theme > global)
- `set_global(name: &str, value: CssValue)`
- `set_theme(name: &str, value: CssValue)`
- `set_local(name: &str, value: CssValue)`
- `set_theme_layer(theme: VariableMap)` — replace entire theme layer
- `clear_local()` — reset local overrides
- `global(&self) -> &VariableMap`
- `theme(&self) -> &VariableMap`

### Tests (10+)

1. `empty_variable_map` — new map is empty
2. `set_and_get` — set variable, get returns it
3. `get_missing` — returns None
4. `remove_variable` — variable removed after delete
5. `contains_check` — contains returns correct values
6. `merge_maps` — other overrides self on conflict
7. `environment_resolve_global` — falls through to global
8. `environment_resolve_theme_overrides_global` — theme wins over global
9. `environment_resolve_local_overrides_all` — local wins over theme and global
10. `environment_set_theme_layer` — replace entire theme layer
11. `environment_clear_local` — local cleared, falls through
12. `variable_map_iteration` — iter() returns all pairs

---

## Task 3: Variable Resolution in Cascade (cascade.rs)

**File**: `crates/fae-core/src/tcss/cascade.rs`

Update the cascade resolver to resolve `CssValue::Variable` references using a `VariableEnvironment`.

### Changes to CascadeResolver

Add a new method that accepts variables:

```rust
impl CascadeResolver {
    /// Resolve matched rules into a computed style, resolving variable references.
    pub fn resolve_with_variables(
        matches: &[MatchedRule],
        env: &VariableEnvironment,
    ) -> ComputedStyle {
        let mut style = Self::resolve(matches);
        style.resolve_variables(env);
        style
    }
}
```

### Changes to ComputedStyle

Add variable resolution:

```rust
impl ComputedStyle {
    /// Resolve all variable references in the computed style.
    pub fn resolve_variables(&mut self, env: &VariableEnvironment) {
        let resolved: Vec<(PropertyName, CssValue)> = self.properties
            .iter()
            .filter_map(|(prop, value)| {
                if let CssValue::Variable(name) = value {
                    env.resolve(name).map(|v| (prop.clone(), v.clone()))
                } else {
                    None
                }
            })
            .collect();
        for (prop, value) in resolved {
            self.properties.insert(prop, value);
        }
    }

    /// Check if any property has an unresolved variable.
    pub fn has_unresolved_variables(&self) -> bool {
        self.properties.values().any(|v| matches!(v, CssValue::Variable(_)))
    }
}
```

### Tests (8+)

1. `resolve_with_no_variables` — no variables, same as resolve()
2. `resolve_variable_from_global` — `$fg` resolved to global value
3. `resolve_variable_from_theme` — theme variable overrides global
4. `resolve_variable_missing` — unresolved variable stays as Variable
5. `resolve_multiple_variables` — multiple properties resolved
6. `resolve_mixed` — some variables, some concrete values
7. `has_unresolved_true` — detects unresolved variables
8. `has_unresolved_false` — no variables returns false
9. `full_pipeline_with_variables` — parse → match → cascade with variables

---

## Task 4: Variable Declaration Parsing (parser.rs)

**File**: `crates/fae-core/src/tcss/parser.rs`

Parse `:root` blocks and variable definitions. Variable declarations use the pattern `$name: value;`.

### Approach

TCSS variable definitions use `$name: value;` syntax inside selector blocks. The parser needs to detect when a declaration starts with `$` and treat it as a variable definition rather than a property declaration.

Add to parser.rs:

```rust
/// A parsed variable definition ($name: value).
#[derive(Clone, Debug, PartialEq)]
pub struct VariableDefinition {
    pub name: String,
    pub value: CssValue,
}
```

Update `Rule` to carry variable definitions:
- Add `pub variables: Vec<VariableDefinition>` to `Rule` in ast.rs

Update `parse_declaration_inner` to detect `$` prefix and route to variable parsing:
- If the next token is `$`, parse as variable definition
- Otherwise, parse as normal declaration

Add a function to extract variables from a parsed stylesheet:

```rust
/// Extract all variable definitions from :root rules in the stylesheet.
pub fn extract_root_variables(stylesheet: &Stylesheet) -> VariableMap {
    // Find rules with :root selector and collect their variable definitions.
}
```

### Tests (8+)

1. `parse_variable_definition` — `$primary: red;` parses correctly
2. `parse_variable_definition_hex` — `$bg: #1e1e2e;` parses to color
3. `parse_variable_definition_length` — `$width: 30;` parses to length
4. `parse_root_block` — `:root { $fg: white; $bg: #1e1e2e; }` parses
5. `extract_root_variables` — extracts variables from :root rules
6. `mixed_variables_and_properties` — block with both vars and props
7. `variable_in_non_root_block` — `.dark { $fg: white; }` parses (for theming)
8. `multiple_root_blocks` — later :root block variables override earlier

---

## Task 5: Theme System (theme.rs)

**File**: `crates/fae-core/src/tcss/theme.rs`

Implement the theme system that manages named themes with variable sets.

### Types

```rust
/// A named theme — a set of variable overrides.
#[derive(Clone, Debug)]
pub struct Theme {
    /// Theme name (e.g., "dark", "light", "catppuccin").
    name: String,
    /// Variable overrides for this theme.
    variables: VariableMap,
}

/// Manages multiple themes and the active theme.
#[derive(Clone, Debug)]
pub struct ThemeManager {
    /// Available themes.
    themes: HashMap<String, Theme>,
    /// Currently active theme name.
    active: Option<String>,
}
```

### Theme Methods

- `new(name: &str) -> Self` — empty theme
- `with_variables(name: &str, vars: VariableMap) -> Self`
- `name(&self) -> &str`
- `variables(&self) -> &VariableMap`
- `set_variable(name: &str, value: CssValue)`

### ThemeManager Methods

- `new() -> Self` — no themes, no active
- `register(theme: Theme)` — add a theme
- `set_active(name: &str) -> Result<(), TcssError>` — switch theme
- `active_theme(&self) -> Option<&Theme>` — get active theme
- `active_name(&self) -> Option<&str>` — get active theme name
- `theme_names(&self) -> Vec<&str>` — list available themes
- `has_theme(name: &str) -> bool`
- `remove(name: &str) -> bool` — remove a theme
- `build_environment(&self, global: &VariableMap) -> VariableEnvironment` — build env from global + active theme

### Built-in Themes

Provide a function to create default dark/light themes:

```rust
/// Create the built-in dark theme.
pub fn builtin_dark() -> Theme

/// Create the built-in light theme.
pub fn builtin_light() -> Theme
```

Dark theme variables:
- `$fg`: white
- `$bg`: #1e1e2e
- `$surface`: #313244
- `$primary`: #89b4fa
- `$secondary`: #a6e3a1
- `$error`: #f38ba8
- `$warning`: #f9e2af
- `$border`: #585b70

Light theme variables:
- `$fg`: #4c4f69
- `$bg`: #eff1f5
- `$surface`: #ccd0da
- `$primary`: #1e66f5
- `$secondary`: #40a02b
- `$error`: #d20f39
- `$warning`: #df8e1d
- `$border`: #9ca0b0

### Tests (10+)

1. `empty_theme` — new theme has no variables
2. `theme_with_variables` — created with variables
3. `theme_set_variable` — set works
4. `manager_register` — theme registered
5. `manager_set_active` — active theme set
6. `manager_set_active_missing` — error for non-existent theme
7. `manager_theme_names` — lists all registered names
8. `manager_build_environment` — environment built with global + theme
9. `builtin_dark_theme` — dark theme has expected variables
10. `builtin_light_theme` — light theme has expected variables
11. `theme_switch` — switch from dark to light changes active
12. `manager_remove_theme` — removed theme no longer available

---

## Task 6: Theme Extraction from Stylesheets (theme.rs, parser.rs)

**File**: `crates/fae-core/src/tcss/theme.rs`, `crates/fae-core/src/tcss/parser.rs`

Extract themes from stylesheet rules. Themes are defined as class-scoped variable blocks:
```css
:root { $fg: white; $bg: #1e1e2e; }
.dark { $fg: white; $bg: #1e1e2e; }
.light { $fg: #4c4f69; $bg: #eff1f5; }
```

### Functions

```rust
/// Extract themes from a stylesheet.
///
/// Rules with a single class selector (`.dark`, `.light`) that contain
/// variable definitions are treated as theme definitions.
/// The `:root` rule provides global defaults.
pub fn extract_themes(stylesheet: &Stylesheet) -> (VariableMap, Vec<Theme>)
```

This function:
1. Finds `:root` rules → global VariableMap
2. Finds rules with single class selectors containing only variable definitions → Theme per class name
3. Returns (globals, themes)

### Tests (6+)

1. `extract_no_themes` — stylesheet without themes returns empty
2. `extract_root_globals` — `:root` variables extracted
3. `extract_dark_theme` — `.dark` block becomes Theme
4. `extract_multiple_themes` — `.dark` and `.light` both extracted
5. `extract_ignores_property_rules` — rules with non-variable declarations ignored for theme extraction
6. `extract_full_themed_stylesheet` — realistic stylesheet with root + dark + light + widget rules

---

## Task 7: Live Stylesheet Reloading (reload.rs)

**File**: `crates/fae-core/src/tcss/reload.rs`

Implement file watching and live reloading of `.tcss` stylesheets.

### Dependencies

Add to `crates/fae-core/Cargo.toml`:
```toml
[dependencies]
notify = "7"
```

### Types

```rust
/// A stylesheet loader that watches files for changes.
pub struct StylesheetLoader {
    /// Parsed stylesheet.
    stylesheet: Stylesheet,
    /// Extracted global variables.
    globals: VariableMap,
    /// Extracted themes.
    themes: Vec<Theme>,
    /// File path being watched (if any).
    path: Option<PathBuf>,
    /// Generation counter (incremented on reload).
    generation: u64,
}

/// Events emitted by the stylesheet loader.
#[derive(Clone, Debug)]
pub enum StylesheetEvent {
    /// Stylesheet was reloaded successfully.
    Reloaded {
        generation: u64,
    },
    /// Stylesheet reload failed (parse error).
    Error(String),
}
```

### StylesheetLoader Methods

- `new() -> Self` — empty loader
- `load_string(css: &str) -> Result<Self, TcssError>` — parse from string
- `load_file(path: &Path) -> Result<Self, TcssError>` — parse from file
- `reload(&mut self) -> Result<StylesheetEvent, TcssError>` — re-read and parse file
- `stylesheet(&self) -> &Stylesheet`
- `globals(&self) -> &VariableMap`
- `themes(&self) -> &[Theme]`
- `generation(&self) -> u64`
- `path(&self) -> Option<&Path>`

### File Watcher

```rust
/// Start watching a stylesheet file for changes.
///
/// Returns a receiver that emits events when the file changes.
/// The watcher runs on a background thread.
pub fn watch_stylesheet(
    path: &Path,
) -> Result<(notify::RecommendedWatcher, std::sync::mpsc::Receiver<StylesheetEvent>), TcssError>
```

### Tests (8+)

1. `load_from_string` — parse CSS string into loader
2. `load_extracts_globals` — globals extracted on load
3. `load_extracts_themes` — themes extracted on load
4. `generation_increments` — reload bumps generation
5. `reload_from_string` — reload with new content
6. `loader_accessors` — stylesheet(), globals(), themes() work
7. `empty_loader` — new loader has empty stylesheet
8. `load_file_not_found` — error for missing file

Note: File watcher integration tests are deferred to avoid test flakiness from filesystem timing. The `watch_stylesheet` function is tested via the public API surface (loader methods) rather than actual file watching in unit tests.

---

## Task 8: Integration & Wire-Up

**Files**:
- `crates/fae-core/src/tcss/mod.rs` — add new module declarations and re-exports
- `crates/fae-core/src/tcss/ast.rs` — add variables field to Rule
- Verify all modules compile and integrate

### Module Declarations

Add to `tcss/mod.rs`:
```rust
pub mod reload;
pub mod theme;
pub mod variable;
```

### Re-exports

```rust
pub use reload::{StylesheetEvent, StylesheetLoader};
pub use theme::{Theme, ThemeManager};
pub use variable::{VariableEnvironment, VariableMap};
```

### Changes to Rule (ast.rs)

Add variable definitions to the Rule struct:
```rust
pub struct Rule {
    pub selectors: SelectorList,
    pub declarations: Vec<Declaration>,
    pub variables: Vec<VariableDefinition>,
}
```

Update `Rule::new()` to default variables to empty vec.
Add `Rule::with_variables()` builder method.

### Integration Tests (8+)

Write integration tests in `tcss/mod.rs` that test the full themed pipeline:

1. `themed_pipeline_simple` — parse themed stylesheet → extract themes → set active → resolve with variables
2. `themed_pipeline_switch` — switch from dark to light theme, verify computed styles change
3. `themed_pipeline_variable_in_property` — `color: $fg;` resolved through pipeline
4. `themed_pipeline_root_globals` — `:root` variables resolve without theme
5. `themed_pipeline_theme_overrides_root` — theme variables override :root
6. `themed_pipeline_no_theme` — without active theme, only globals resolve
7. `themed_pipeline_loader` — StylesheetLoader loads and extracts themes
8. `themed_pipeline_generation` — loader generation tracks changes

---

## Summary

| Task | Description | Tests | File |
|------|-------------|-------|------|
| 1 | Variable Value Type | 6+ | value.rs, parser.rs |
| 2 | Variable Storage & Scoping | 10+ | variable.rs |
| 3 | Variable Resolution in Cascade | 8+ | cascade.rs |
| 4 | Variable Declaration Parsing | 8+ | parser.rs |
| 5 | Theme System | 10+ | theme.rs |
| 6 | Theme Extraction from Stylesheets | 6+ | theme.rs, parser.rs |
| 7 | Live Stylesheet Reloading | 8+ | reload.rs |
| 8 | Integration & Wire-Up | 8+ | mod.rs, ast.rs |
| **Total** | | **64+** | |
