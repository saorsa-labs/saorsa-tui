# Phase 2.1: TCSS Parser

## Overview

Build a Terminal CSS (TCSS) parser for fae-core. TCSS is a subset of CSS tailored for terminal UIs. This phase creates the tokenizer, selector parser, property parser, and AST types. All parsing uses the `cssparser` crate from the Servo project.

## Dependencies to Add

- `cssparser = "0.34"` in workspace Cargo.toml and fae-core

## Module Structure

```
crates/fae-core/src/tcss/
├── mod.rs           # Module declarations and re-exports
├── ast.rs           # Stylesheet, Rule, Declaration AST
├── selector.rs      # Selector types and parser
├── property.rs      # Property name/value types
├── value.rs         # CSS value types (color, length, keyword)
├── parser.rs        # Full stylesheet parser
└── error.rs         # TCSS-specific error types
```

---

## Task 1: TCSS Module Scaffold & Value Types

**Files:** `crates/fae-core/src/tcss/mod.rs`, `crates/fae-core/src/tcss/value.rs`, `crates/fae-core/src/tcss/error.rs`, `Cargo.toml` (workspace + fae-core)

**Description:** Add cssparser dependency, create the tcss module structure, and define CSS value types used across properties.

### Value Types (value.rs)

```rust
/// A CSS length value.
pub enum Length {
    /// Fixed cell count.
    Cells(u16),
    /// Percentage of parent.
    Percent(f32),
    /// Auto sizing.
    Auto,
}

/// A CSS numeric value with unit.
pub enum CssValue {
    /// A color value.
    Color(Color),
    /// A length value.
    Length(Length),
    /// A keyword (e.g., "bold", "flex", "center").
    Keyword(String),
    /// An integer (e.g., flex-grow: 2).
    Integer(i32),
    /// A float (e.g., opacity: 0.5).
    Float(f32),
    /// A fractional unit (e.g., 1fr).
    Fr(f32),
    /// A string value.
    String(String),
}
```

### Error Types (error.rs)

```rust
pub enum TcssError {
    Parse(String),
    UnknownProperty(String),
    InvalidValue { property: String, value: String },
    SelectorError(String),
}
```

### Changes
- Add `cssparser = "0.34"` to workspace Cargo.toml and fae-core/Cargo.toml
- Create `crates/fae-core/src/tcss/mod.rs` with submodule declarations
- Create `crates/fae-core/src/tcss/value.rs` with CssValue, Length enums
- Create `crates/fae-core/src/tcss/error.rs` with TcssError
- Add `pub mod tcss;` to `crates/fae-core/src/lib.rs`

### Tests (6+)
- Length display/debug
- CssValue variants construction
- TcssError display messages
- Default values

---

## Task 2: Property Name Types

**Files:** `crates/fae-core/src/tcss/property.rs`

**Description:** Define the enum of all supported TCSS property names. This is a flat enum mapping CSS property names to typed variants.

### Property Enum (~40 properties for V1)

```rust
pub enum PropertyName {
    // Colors
    Color,
    Background,
    BorderColor,

    // Text decoration
    TextStyle,

    // Dimensions
    Width, Height,
    MinWidth, MaxWidth,
    MinHeight, MaxHeight,

    // Box model
    Margin, MarginTop, MarginRight, MarginBottom, MarginLeft,
    Padding, PaddingTop, PaddingRight, PaddingBottom, PaddingLeft,
    Border, BorderTop, BorderRight, BorderBottom, BorderLeft,

    // Layout
    Display,
    FlexDirection,
    FlexWrap,
    JustifyContent,
    AlignItems,
    AlignSelf,
    FlexGrow,
    FlexShrink,
    FlexBasis,
    Gap,

    // Grid
    GridTemplateColumns,
    GridTemplateRows,
    GridColumn,
    GridRow,

    // Positioning
    Dock,
    Overflow,
    OverflowX,
    OverflowY,

    // Visibility
    Visibility,
    Opacity,

    // Content alignment
    TextAlign,
    ContentAlign,
}

/// A parsed CSS declaration (property: value).
pub struct Declaration {
    pub property: PropertyName,
    pub value: CssValue,
    pub important: bool,
}
```

### Key Features
- `PropertyName::from_str(&str) -> Option<PropertyName>` for CSS name lookup
- `PropertyName::css_name(&self) -> &str` for reverse mapping
- `Declaration` struct pairing property name with value

### Tests (8+)
- from_str for each property category
- css_name round-trip
- Unknown property returns None
- Case-insensitive lookup
- Declaration construction
- Important flag

---

## Task 3: Selector Types

**Files:** `crates/fae-core/src/tcss/selector.rs`

**Description:** Define AST types for CSS selectors.

### Selector AST

```rust
/// A single simple selector component.
pub enum SimpleSelector {
    /// Type selector: `Label`, `Container`.
    Type(String),
    /// Class selector: `.error`.
    Class(String),
    /// ID selector: `#sidebar`.
    Id(String),
    /// Universal selector: `*`.
    Universal,
    /// Pseudo-class: `:focus`, `:hover`.
    PseudoClass(PseudoClass),
}

/// Supported pseudo-classes.
pub enum PseudoClass {
    Focus,
    Hover,
    Disabled,
    Active,
    FirstChild,
    LastChild,
    NthChild(i32),
    Even,
    Odd,
}

/// A compound selector (multiple simple selectors with no combinator).
/// e.g., `Label.error#main:focus`
pub struct CompoundSelector {
    pub components: Vec<SimpleSelector>,
}

/// How selectors are combined.
pub enum Combinator {
    /// Descendant: `A B`
    Descendant,
    /// Child: `A > B`
    Child,
}

/// A complex selector chain: compound selectors joined by combinators.
/// e.g., `Container > Label.error`
pub struct Selector {
    /// The rightmost compound selector (the subject).
    pub head: CompoundSelector,
    /// Chain of (combinator, compound) pairs going left.
    pub chain: Vec<(Combinator, CompoundSelector)>,
}

/// A selector list (comma-separated selectors).
pub struct SelectorList {
    pub selectors: Vec<Selector>,
}
```

### Key Features
- `CompoundSelector::specificity() -> (u16, u16, u16)` (id, class, type)
- `Selector::specificity()` sums components
- Display/Debug impls for all types

### Tests (10+)
- SimpleSelector construction
- CompoundSelector specificity: id=1,0,0; class=0,1,0; type=0,0,1
- Mixed specificity: `Label.error#main` = (1,1,1)
- PseudoClass variants
- Selector with chain
- SelectorList
- Universal selector specificity = (0,0,0)
- Display formatting

---

## Task 4: Stylesheet AST

**Files:** `crates/fae-core/src/tcss/ast.rs`

**Description:** Define the top-level stylesheet AST that ties selectors to declarations.

### AST Types

```rust
/// A CSS rule: selector(s) + declarations.
pub struct Rule {
    pub selectors: SelectorList,
    pub declarations: Vec<Declaration>,
}

/// A complete stylesheet.
pub struct Stylesheet {
    pub rules: Vec<Rule>,
}
```

### Key Features
- `Stylesheet::new() -> Self`
- `Stylesheet::add_rule(rule: Rule)`
- `Stylesheet::rules() -> &[Rule]`
- `Stylesheet::is_empty() -> bool`
- `Rule::new(selectors: SelectorList, declarations: Vec<Declaration>)`

### Tests (4+)
- Empty stylesheet
- Add rules
- Rule with multiple selectors
- Rule with multiple declarations

---

## Task 5: Value Parser

**Files:** `crates/fae-core/src/tcss/parser.rs` (partial — value parsing functions)

**Description:** Implement parsing of CSS values using cssparser. These are the building blocks that the property parser and full stylesheet parser will use.

### Parser Functions

```rust
/// Parse a color value from CSS.
/// Supports: #hex, named colors, rgb(r,g,b).
pub fn parse_color(input: &mut Parser) -> Result<Color, TcssError>

/// Parse a length value.
/// Supports: 10 (cells), 50% (percent), auto.
pub fn parse_length(input: &mut Parser) -> Result<Length, TcssError>

/// Parse a keyword from a set of valid options.
pub fn parse_keyword<'a>(input: &mut Parser, valid: &[&str]) -> Result<String, TcssError>

/// Parse an integer value.
pub fn parse_integer(input: &mut Parser) -> Result<i32, TcssError>

/// Parse a float value.
pub fn parse_float(input: &mut Parser) -> Result<f32, TcssError>

/// Parse a property value given its property name.
pub fn parse_property_value(
    property: &PropertyName,
    input: &mut Parser,
) -> Result<CssValue, TcssError>
```

### Key Features
- Color parsing: `#rgb`, `#rrggbb`, named colors (red, blue, etc.), `rgb(r,g,b)`
- Length parsing: bare numbers as cells, `%` suffix for percent, `auto` keyword
- Keyword parsing for enum-like properties (display, flex-direction, etc.)
- Property-specific value validation

### Tests (12+)
- Parse hex colors (#fff, #1e1e2e)
- Parse named colors (red, blue, green)
- Parse rgb function: rgb(255, 0, 0)
- Parse cell length: 10
- Parse percent length: 50%
- Parse auto keyword
- Parse display keywords: flex, block, none
- Parse flex-direction keywords
- Parse integer (flex-grow)
- Parse float (opacity)
- Parse invalid values return error
- Parse property-specific values

---

## Task 6: Selector Parser

**Files:** `crates/fae-core/src/tcss/selector.rs` (add parsing methods)

**Description:** Implement CSS selector parsing using cssparser.

### Parser Functions

```rust
impl SelectorList {
    /// Parse a comma-separated selector list.
    pub fn parse(input: &str) -> Result<Self, TcssError>
}

impl Selector {
    /// Parse a single complex selector.
    fn parse(input: &mut Parser) -> Result<Self, TcssError>
}

impl CompoundSelector {
    /// Parse a compound selector (no combinators).
    fn parse(input: &mut Parser) -> Result<Self, TcssError>
}

impl SimpleSelector {
    /// Parse a single simple selector.
    fn parse(input: &mut Parser) -> Result<Self, TcssError>
}

impl PseudoClass {
    /// Parse a pseudo-class name.
    fn from_name(name: &str) -> Option<Self>
}
```

### Key Features
- Type selectors from ident tokens
- Class selectors from `.` + ident
- ID selectors from `#` + ident
- Pseudo-class from `:` + ident
- Universal from `*`
- `>` combinator detection
- Whitespace = descendant combinator
- Comma-separated selector lists

### Tests (12+)
- Parse type selector: `Label`
- Parse class selector: `.error`
- Parse ID selector: `#sidebar`
- Parse universal: `*`
- Parse pseudo-class: `:focus`, `:hover`, `:disabled`
- Parse compound: `Label.error`
- Parse child combinator: `Container > Label`
- Parse descendant combinator: `Container Label`
- Parse selector list: `Label, Container`
- Parse complex: `Container > Label.error:focus`
- Parse nth-child: `:nth-child(2)`
- Invalid selector error

---

## Task 7: Full Stylesheet Parser

**Files:** `crates/fae-core/src/tcss/parser.rs` (complete)

**Description:** Implement the full TCSS stylesheet parser that parses complete stylesheets into the AST.

### Parser Functions

```rust
impl Stylesheet {
    /// Parse a complete TCSS stylesheet.
    pub fn parse(input: &str) -> Result<Self, TcssError>
}

impl Rule {
    /// Parse a single CSS rule (selectors { declarations }).
    fn parse(input: &mut Parser) -> Result<Self, TcssError>
}

impl Declaration {
    /// Parse a single declaration (property: value;).
    fn parse(input: &mut Parser) -> Result<Self, TcssError>
}
```

### Key Features
- Parse multiple rules from input
- Handle `{ }` blocks for declaration lists
- Parse `;` separated declarations
- Handle `!important`
- Skip comments (`/* ... */`)
- Error recovery: skip invalid rules, continue parsing

### Tests (10+)
- Parse empty stylesheet
- Parse single rule: `Label { color: red; }`
- Parse multiple declarations: `Label { color: red; background: blue; }`
- Parse multiple rules
- Parse with !important: `Label { color: red !important; }`
- Parse with comments: `/* comment */ Label { color: red; }`
- Parse complex selector rule: `Container > Label.error { color: red; }`
- Parse selector list rule: `Label, Container { color: red; }`
- Parse all property types (color, length, keyword, integer, float)
- Invalid rule skipped, valid rules parsed
- Real-world stylesheet with mixed content

---

## Task 8: Wire Up & Integration

**Files:** `crates/fae-core/src/tcss/mod.rs`, `crates/fae-core/src/lib.rs`

**Description:** Wire up all TCSS modules, add re-exports, and write integration tests.

### Changes
- Complete `tcss/mod.rs` with all submodule declarations and re-exports
- Update `crates/fae-core/src/lib.rs` re-exports for TCSS types
- Add integration tests combining parsing → AST → inspection

### Re-exports
```rust
// In tcss/mod.rs
pub use ast::{Rule, Stylesheet};
pub use error::TcssError;
pub use property::{Declaration, PropertyName};
pub use selector::{
    Combinator, CompoundSelector, PseudoClass, Selector, SelectorList, SimpleSelector,
};
pub use value::{CssValue, Length};
```

### Integration Tests (6+)
- Parse a complete theme stylesheet (10+ rules)
- Verify selector specificity after parsing
- Verify property values are correctly typed
- Round-trip: parse → inspect → verify all fields
- Error recovery: mix of valid and invalid rules
- Empty input returns empty stylesheet

---

## Task Summary

| # | Task | Files | Tests | Depends On |
|---|------|-------|-------|------------|
| 1 | Module scaffold & value types | mod.rs, value.rs, error.rs, Cargo.toml | 6+ | — |
| 2 | Property name types | property.rs | 8+ | 1 |
| 3 | Selector types | selector.rs | 10+ | 1 |
| 4 | Stylesheet AST | ast.rs | 4+ | 2, 3 |
| 5 | Value parser | parser.rs (partial) | 12+ | 1, 2 |
| 6 | Selector parser | selector.rs (extend) | 12+ | 3 |
| 7 | Full stylesheet parser | parser.rs (complete) | 10+ | 4, 5, 6 |
| 8 | Wire up & integration | mod.rs, lib.rs | 6+ | 7 |

**Total: ~68+ tests**
