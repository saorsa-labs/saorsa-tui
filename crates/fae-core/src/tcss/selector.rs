//! TCSS selector types.
//!
//! Defines the AST for CSS selectors used in TCSS stylesheets.

use std::fmt;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_type_selector() {
        let sel = CompoundSelector::type_selector("Label");
        assert_eq!(sel.components.len(), 1);
        assert_eq!(
            sel.components[0],
            SimpleSelector::Type("Label".into())
        );
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
}
