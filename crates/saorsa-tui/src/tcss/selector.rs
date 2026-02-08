//! TCSS selector types.
//!
//! Defines the AST for CSS selectors used in TCSS stylesheets.

use std::fmt;

use cssparser::{Parser, ParserInput, Token};

use crate::tcss::error::TcssError;

/// A single simple selector component.
#[derive(Clone, Debug, PartialEq, Eq)]
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
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PseudoClass {
    /// `:focus` — widget has keyboard focus.
    Focus,
    /// `:hover` — mouse is over widget.
    Hover,
    /// `:disabled` — widget is disabled.
    Disabled,
    /// `:active` — widget is being activated.
    Active,
    /// `:first-child` — first child of parent.
    FirstChild,
    /// `:last-child` — last child of parent.
    LastChild,
    /// `:nth-child(n)` — nth child of parent.
    NthChild(i32),
    /// `:even` — even-positioned child.
    Even,
    /// `:odd` — odd-positioned child.
    Odd,
    /// `:root` — matches the root element (used for variable/theme definitions).
    Root,
}

impl PseudoClass {
    /// Parse a pseudo-class from its name.
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_ascii_lowercase().as_str() {
            "focus" => Some(Self::Focus),
            "hover" => Some(Self::Hover),
            "disabled" => Some(Self::Disabled),
            "active" => Some(Self::Active),
            "first-child" => Some(Self::FirstChild),
            "last-child" => Some(Self::LastChild),
            "even" => Some(Self::Even),
            "odd" => Some(Self::Odd),
            "root" => Some(Self::Root),
            _ => None,
        }
    }
}

impl fmt::Display for PseudoClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Focus => write!(f, ":focus"),
            Self::Hover => write!(f, ":hover"),
            Self::Disabled => write!(f, ":disabled"),
            Self::Active => write!(f, ":active"),
            Self::FirstChild => write!(f, ":first-child"),
            Self::LastChild => write!(f, ":last-child"),
            Self::NthChild(n) => write!(f, ":nth-child({n})"),
            Self::Even => write!(f, ":even"),
            Self::Odd => write!(f, ":odd"),
            Self::Root => write!(f, ":root"),
        }
    }
}

/// A compound selector (multiple simple selectors with no combinator).
///
/// For example: `Label.error#main:focus`
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompoundSelector {
    /// The simple selector components.
    pub components: Vec<SimpleSelector>,
}

impl CompoundSelector {
    /// Create a new compound selector from components.
    pub fn new(components: Vec<SimpleSelector>) -> Self {
        Self { components }
    }

    /// Create a compound selector with a single type selector.
    pub fn type_selector(name: impl Into<String>) -> Self {
        Self {
            components: vec![SimpleSelector::Type(name.into())],
        }
    }

    /// Calculate CSS specificity as (id_count, class_count, type_count).
    ///
    /// - ID selectors contribute to the first component.
    /// - Class selectors and pseudo-classes contribute to the second.
    /// - Type selectors contribute to the third.
    /// - Universal selector contributes nothing.
    pub fn specificity(&self) -> (u16, u16, u16) {
        let mut ids: u16 = 0;
        let mut classes: u16 = 0;
        let mut types: u16 = 0;

        for component in &self.components {
            match component {
                SimpleSelector::Id(_) => ids = ids.saturating_add(1),
                SimpleSelector::Class(_) | SimpleSelector::PseudoClass(_) => {
                    classes = classes.saturating_add(1);
                }
                SimpleSelector::Type(_) => types = types.saturating_add(1),
                SimpleSelector::Universal => {}
            }
        }

        (ids, classes, types)
    }
}

impl fmt::Display for CompoundSelector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for component in &self.components {
            match component {
                SimpleSelector::Type(name) => write!(f, "{name}")?,
                SimpleSelector::Class(name) => write!(f, ".{name}")?,
                SimpleSelector::Id(name) => write!(f, "#{name}")?,
                SimpleSelector::Universal => write!(f, "*")?,
                SimpleSelector::PseudoClass(pc) => write!(f, "{pc}")?,
            }
        }
        Ok(())
    }
}

/// How selectors are combined.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Combinator {
    /// Descendant combinator: `A B` (whitespace).
    Descendant,
    /// Child combinator: `A > B`.
    Child,
}

impl fmt::Display for Combinator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Descendant => write!(f, " "),
            Self::Child => write!(f, " > "),
        }
    }
}

/// A complex selector: compound selectors joined by combinators.
///
/// For example: `Container > Label.error`
///
/// The `head` is the rightmost (subject) compound selector.
/// The `chain` contains (combinator, compound) pairs going left.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Selector {
    /// The rightmost compound selector (the subject).
    pub head: CompoundSelector,
    /// Chain of (combinator, compound) pairs going left from head.
    pub chain: Vec<(Combinator, CompoundSelector)>,
}

impl Selector {
    /// Create a simple selector with just a head and no chain.
    pub fn simple(head: CompoundSelector) -> Self {
        Self {
            head,
            chain: Vec::new(),
        }
    }

    /// Calculate the total specificity of this selector.
    ///
    /// Sums the specificity of all compound selectors in the chain.
    pub fn specificity(&self) -> (u16, u16, u16) {
        let (mut ids, mut classes, mut types) = self.head.specificity();

        for (_, compound) in &self.chain {
            let (i, c, t) = compound.specificity();
            ids = ids.saturating_add(i);
            classes = classes.saturating_add(c);
            types = types.saturating_add(t);
        }

        (ids, classes, types)
    }
}

impl fmt::Display for Selector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Write chain in reverse (leftmost first).
        for (combinator, compound) in self.chain.iter().rev() {
            write!(f, "{compound}{combinator}")?;
        }
        write!(f, "{}", self.head)
    }
}

/// A selector list (comma-separated selectors).
///
/// For example: `Label, Container, .error`
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectorList {
    /// The selectors in this list.
    pub selectors: Vec<Selector>,
}

impl SelectorList {
    /// Create a new selector list.
    pub fn new(selectors: Vec<Selector>) -> Self {
        Self { selectors }
    }

    /// Parse a comma-separated selector list from a CSS string.
    pub fn parse(input: &str) -> Result<Self, TcssError> {
        let mut parser_input = ParserInput::new(input);
        let mut parser = Parser::new(&mut parser_input);
        Self::parse_from(&mut parser)
    }

    /// Parse a selector list from a cssparser `Parser`.
    pub fn parse_from(input: &mut Parser<'_, '_>) -> Result<Self, TcssError> {
        let mut selectors = vec![parse_selector(input)?];

        while input.try_parse(|p| p.expect_comma()).is_ok() {
            selectors.push(parse_selector(input)?);
        }

        Ok(Self { selectors })
    }

    /// Return the highest specificity among all selectors in the list.
    pub fn max_specificity(&self) -> (u16, u16, u16) {
        self.selectors
            .iter()
            .map(Selector::specificity)
            .max()
            .unwrap_or((0, 0, 0))
    }
}

impl fmt::Display for SelectorList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, selector) in self.selectors.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{selector}")?;
        }
        Ok(())
    }
}

/// Parse a single complex selector from CSS input.
///
/// A complex selector is one or more compound selectors joined by combinators.
fn parse_selector(input: &mut Parser<'_, '_>) -> Result<Selector, TcssError> {
    let mut compounds = vec![parse_compound_selector(input)?];
    let mut combinators = Vec::new();

    loop {
        // Try to detect a combinator: `>` for child, whitespace for descendant.
        let combinator = input.try_parse(|input| {
            if input.try_parse(|p| p.expect_delim('>')).is_ok() {
                Ok(Combinator::Child)
            } else {
                // Check if there's another compound selector following (descendant combinator).
                // We peek at the next token — if it's an ident, `.`, `#`, `*`, or `:`,
                // there's a descendant relationship.
                let state = input.state();
                match input.next() {
                    Ok(
                        Token::Ident(_)
                        | Token::Delim('.')
                        | Token::Delim('*')
                        | Token::Colon
                        | Token::IDHash(_),
                    ) => {
                        input.reset(&state);
                        Ok(Combinator::Descendant)
                    }
                    _ => {
                        input.reset(&state);
                        Err(input.new_error_for_next_token::<()>())
                    }
                }
            }
        });

        match combinator {
            Ok(c) => {
                combinators.push(c);
                compounds.push(parse_compound_selector(input)?);
            }
            Err(_) => break,
        }
    }

    // Build the selector: the last compound is the head (subject),
    // earlier compounds form the chain with their combinators.
    let head = compounds.pop().ok_or_else(|| {
        TcssError::SelectorError("expected at least one selector component".into())
    })?;

    let chain: Vec<(Combinator, CompoundSelector)> =
        combinators.into_iter().zip(compounds).collect();

    Ok(Selector { head, chain })
}

/// Parse a compound selector (no combinators).
fn parse_compound_selector(input: &mut Parser<'_, '_>) -> Result<CompoundSelector, TcssError> {
    let mut components = Vec::new();

    // Parse the first component (required).
    components.push(parse_simple_selector(input)?);

    // Parse additional components that are directly attached (no whitespace).
    loop {
        let result: Result<SimpleSelector, cssparser::ParseError<'_, ()>> =
            input.try_parse(|input| {
                let token = input.next_including_whitespace()?.clone();

                match &token {
                    Token::Delim('.') => {
                        let name = input.expect_ident()?.to_string();
                        Ok(SimpleSelector::Class(name))
                    }
                    Token::Colon => {
                        let name = input.expect_ident()?.to_string();
                        PseudoClass::from_name(&name)
                            .map(SimpleSelector::PseudoClass)
                            .ok_or_else(|| input.new_error_for_next_token::<()>())
                    }
                    Token::IDHash(name) => Ok(SimpleSelector::Id(name.to_string())),
                    _ => Err(input.new_error_for_next_token::<()>()),
                }
            });

        match result {
            Ok(component) => components.push(component),
            Err(_) => break,
        }
    }

    Ok(CompoundSelector::new(components))
}

/// Parse a single simple selector.
fn parse_simple_selector(input: &mut Parser<'_, '_>) -> Result<SimpleSelector, TcssError> {
    let token = input
        .next()
        .map_err(|e| TcssError::SelectorError(format!("{e:?}")))?
        .clone();

    match &token {
        Token::Ident(name) => Ok(SimpleSelector::Type(name.to_string())),
        Token::Delim('*') => Ok(SimpleSelector::Universal),
        Token::Delim('.') => {
            let name = input
                .expect_ident()
                .map_err(|e| TcssError::SelectorError(format!("{e:?}")))?
                .to_string();
            Ok(SimpleSelector::Class(name))
        }
        Token::IDHash(name) => Ok(SimpleSelector::Id(name.to_string())),
        Token::Colon => {
            let name = input
                .expect_ident()
                .map_err(|e| TcssError::SelectorError(format!("{e:?}")))?
                .to_string();
            PseudoClass::from_name(&name)
                .map(SimpleSelector::PseudoClass)
                .ok_or_else(|| TcssError::SelectorError(format!("unknown pseudo-class: {name}")))
        }
        other => Err(TcssError::SelectorError(format!(
            "expected selector, got {other:?}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_type_selector() {
        let sel = CompoundSelector::type_selector("Label");
        assert_eq!(sel.components.len(), 1);
        assert_eq!(sel.components[0], SimpleSelector::Type("Label".into()));
    }

    #[test]
    fn compound_specificity_type() {
        let sel = CompoundSelector::type_selector("Label");
        assert_eq!(sel.specificity(), (0, 0, 1));
    }

    #[test]
    fn compound_specificity_class() {
        let sel = CompoundSelector::new(vec![SimpleSelector::Class("error".into())]);
        assert_eq!(sel.specificity(), (0, 1, 0));
    }

    #[test]
    fn compound_specificity_id() {
        let sel = CompoundSelector::new(vec![SimpleSelector::Id("sidebar".into())]);
        assert_eq!(sel.specificity(), (1, 0, 0));
    }

    #[test]
    fn compound_specificity_mixed() {
        let sel = CompoundSelector::new(vec![
            SimpleSelector::Type("Label".into()),
            SimpleSelector::Class("error".into()),
            SimpleSelector::Id("main".into()),
        ]);
        assert_eq!(sel.specificity(), (1, 1, 1));
    }

    #[test]
    fn compound_specificity_universal() {
        let sel = CompoundSelector::new(vec![SimpleSelector::Universal]);
        assert_eq!(sel.specificity(), (0, 0, 0));
    }

    #[test]
    fn compound_specificity_pseudo_class() {
        let sel = CompoundSelector::new(vec![
            SimpleSelector::Type("Label".into()),
            SimpleSelector::PseudoClass(PseudoClass::Focus),
        ]);
        assert_eq!(sel.specificity(), (0, 1, 1));
    }

    #[test]
    fn pseudo_class_from_name() {
        assert_eq!(PseudoClass::from_name("focus"), Some(PseudoClass::Focus));
        assert_eq!(PseudoClass::from_name("hover"), Some(PseudoClass::Hover));
        assert_eq!(
            PseudoClass::from_name("disabled"),
            Some(PseudoClass::Disabled)
        );
        assert_eq!(
            PseudoClass::from_name("first-child"),
            Some(PseudoClass::FirstChild)
        );
        assert_eq!(
            PseudoClass::from_name("last-child"),
            Some(PseudoClass::LastChild)
        );
        assert_eq!(PseudoClass::from_name("even"), Some(PseudoClass::Even));
        assert_eq!(PseudoClass::from_name("odd"), Some(PseudoClass::Odd));
        assert_eq!(PseudoClass::from_name("unknown"), None);
    }

    #[test]
    fn pseudo_class_case_insensitive() {
        assert_eq!(PseudoClass::from_name("FOCUS"), Some(PseudoClass::Focus));
        assert_eq!(PseudoClass::from_name("Hover"), Some(PseudoClass::Hover));
    }

    #[test]
    fn selector_simple() {
        let sel = Selector::simple(CompoundSelector::type_selector("Label"));
        assert_eq!(sel.specificity(), (0, 0, 1));
        assert!(sel.chain.is_empty());
    }

    #[test]
    fn selector_with_chain() {
        let sel = Selector {
            head: CompoundSelector::new(vec![
                SimpleSelector::Type("Label".into()),
                SimpleSelector::Class("error".into()),
            ]),
            chain: vec![(
                Combinator::Child,
                CompoundSelector::type_selector("Container"),
            )],
        };
        // Container > Label.error
        // Container = (0,0,1), Label.error = (0,1,1), total = (0,1,2)
        assert_eq!(sel.specificity(), (0, 1, 2));
    }

    #[test]
    fn selector_list() {
        let list = SelectorList::new(vec![
            Selector::simple(CompoundSelector::type_selector("Label")),
            Selector::simple(CompoundSelector::new(vec![SimpleSelector::Id(
                "main".into(),
            )])),
        ]);
        assert_eq!(list.selectors.len(), 2);
        assert_eq!(list.max_specificity(), (1, 0, 0));
    }

    #[test]
    fn display_compound() {
        let sel = CompoundSelector::new(vec![
            SimpleSelector::Type("Label".into()),
            SimpleSelector::Class("error".into()),
            SimpleSelector::Id("main".into()),
            SimpleSelector::PseudoClass(PseudoClass::Focus),
        ]);
        assert_eq!(sel.to_string(), "Label.error#main:focus");
    }

    #[test]
    fn display_selector_with_chain() {
        let sel = Selector {
            head: CompoundSelector::type_selector("Label"),
            chain: vec![(
                Combinator::Child,
                CompoundSelector::type_selector("Container"),
            )],
        };
        assert_eq!(sel.to_string(), "Container > Label");
    }

    #[test]
    fn display_selector_list() {
        let list = SelectorList::new(vec![
            Selector::simple(CompoundSelector::type_selector("Label")),
            Selector::simple(CompoundSelector::type_selector("Container")),
        ]);
        assert_eq!(list.to_string(), "Label, Container");
    }

    #[test]
    fn display_pseudo_class() {
        assert_eq!(PseudoClass::Focus.to_string(), ":focus");
        assert_eq!(PseudoClass::NthChild(3).to_string(), ":nth-child(3)");
        assert_eq!(PseudoClass::FirstChild.to_string(), ":first-child");
    }

    // --- Parsing tests ---

    /// Helper to parse and assert success, returning the selector list.
    fn parse_ok(input: &str) -> SelectorList {
        let result = SelectorList::parse(input);
        assert!(result.is_ok(), "parse failed for '{input}': {result:?}");
        // Safe: we just asserted Ok above.
        match result {
            Ok(list) => list,
            Err(_) => unreachable!(),
        }
    }

    #[test]
    fn parse_type_selector() {
        let list = parse_ok("Label");
        assert_eq!(list.selectors.len(), 1);
        assert_eq!(list.selectors[0].head.components.len(), 1);
        assert_eq!(
            list.selectors[0].head.components[0],
            SimpleSelector::Type("Label".into())
        );
    }

    #[test]
    fn parse_class_selector() {
        let list = parse_ok(".error");
        assert_eq!(
            list.selectors[0].head.components[0],
            SimpleSelector::Class("error".into())
        );
    }

    #[test]
    fn parse_id_selector() {
        let list = parse_ok("#sidebar");
        assert_eq!(
            list.selectors[0].head.components[0],
            SimpleSelector::Id("sidebar".into())
        );
    }

    #[test]
    fn parse_universal_selector() {
        let list = parse_ok("*");
        assert_eq!(
            list.selectors[0].head.components[0],
            SimpleSelector::Universal
        );
    }

    #[test]
    fn parse_pseudo_class_focus() {
        let list = parse_ok(":focus");
        assert_eq!(
            list.selectors[0].head.components[0],
            SimpleSelector::PseudoClass(PseudoClass::Focus)
        );
    }

    #[test]
    fn parse_pseudo_class_hover() {
        let list = parse_ok(":hover");
        assert_eq!(
            list.selectors[0].head.components[0],
            SimpleSelector::PseudoClass(PseudoClass::Hover)
        );
    }

    #[test]
    fn parse_compound_type_and_class() {
        let list = parse_ok("Label.error");
        let head = &list.selectors[0].head;
        assert_eq!(head.components.len(), 2);
        assert_eq!(head.components[0], SimpleSelector::Type("Label".into()));
        assert_eq!(head.components[1], SimpleSelector::Class("error".into()));
    }

    #[test]
    fn parse_child_combinator() {
        let list = parse_ok("Container > Label");
        let sel = &list.selectors[0];
        assert_eq!(sel.head.components[0], SimpleSelector::Type("Label".into()));
        assert_eq!(sel.chain.len(), 1);
        assert_eq!(sel.chain[0].0, Combinator::Child);
        assert_eq!(
            sel.chain[0].1.components[0],
            SimpleSelector::Type("Container".into())
        );
    }

    #[test]
    fn parse_descendant_combinator() {
        let list = parse_ok("Container Label");
        let sel = &list.selectors[0];
        assert_eq!(sel.head.components[0], SimpleSelector::Type("Label".into()));
        assert_eq!(sel.chain.len(), 1);
        assert_eq!(sel.chain[0].0, Combinator::Descendant);
    }

    #[test]
    fn parse_selector_list_comma() {
        let list = parse_ok("Label, Container");
        assert_eq!(list.selectors.len(), 2);
    }

    #[test]
    fn parse_complex_selector() {
        let list = parse_ok("Container > Label.error:focus");
        let sel = &list.selectors[0];
        // head = Label.error:focus
        assert_eq!(sel.head.components.len(), 3);
        assert_eq!(sel.head.components[0], SimpleSelector::Type("Label".into()));
        assert_eq!(
            sel.head.components[1],
            SimpleSelector::Class("error".into())
        );
        assert_eq!(
            sel.head.components[2],
            SimpleSelector::PseudoClass(PseudoClass::Focus)
        );
        // chain = Container >
        assert_eq!(sel.chain.len(), 1);
        assert_eq!(sel.chain[0].0, Combinator::Child);
    }

    #[test]
    fn parse_invalid_selector() {
        let result = SelectorList::parse("123");
        assert!(result.is_err());
    }

    #[test]
    fn parse_specificity_after_parse() {
        let list = parse_ok("Container > Label.error#main");
        // Container(0,0,1) + Label(0,0,1) + .error(0,1,0) + #main(1,0,0) = (1,1,2)
        assert_eq!(list.selectors[0].specificity(), (1, 1, 2));
    }
}
