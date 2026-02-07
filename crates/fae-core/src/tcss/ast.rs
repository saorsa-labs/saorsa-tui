//! Stylesheet AST types.
//!
//! Defines the top-level types that tie selectors to declarations
//! into a complete stylesheet.

use crate::tcss::property::Declaration;
use crate::tcss::selector::SelectorList;

/// A CSS rule: selector list paired with declarations.
#[derive(Clone, Debug, PartialEq)]
pub struct Rule {
    /// The selectors this rule applies to.
    pub selectors: SelectorList,
    /// The declarations (property-value pairs) in this rule.
    pub declarations: Vec<Declaration>,
}

impl Rule {
    /// Create a new rule.
    pub fn new(selectors: SelectorList, declarations: Vec<Declaration>) -> Self {
        Self {
            selectors,
            declarations,
        }
    }
}

/// A complete TCSS stylesheet.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Stylesheet {
    /// The rules in this stylesheet.
    rules: Vec<Rule>,
}

impl Stylesheet {
    /// Create an empty stylesheet.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a rule to the stylesheet.
    pub fn add_rule(&mut self, rule: Rule) {
        self.rules.push(rule);
    }

    /// Return all rules in the stylesheet.
    pub fn rules(&self) -> &[Rule] {
        &self.rules
    }

    /// Return the number of rules.
    pub fn len(&self) -> usize {
        self.rules.len()
    }

    /// Return whether the stylesheet is empty.
    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tcss::property::PropertyName;
    use crate::tcss::selector::{CompoundSelector, Selector};
    use crate::tcss::value::CssValue;

    fn label_rule() -> Rule {
        Rule::new(
            SelectorList::new(vec![Selector::simple(CompoundSelector::type_selector(
                "Label",
            ))]),
            vec![Declaration::new(
                PropertyName::Color,
                CssValue::Keyword("red".into()),
            )],
        )
    }

    #[test]
    fn empty_stylesheet() {
        let sheet = Stylesheet::new();
        assert!(sheet.is_empty());
        assert_eq!(sheet.len(), 0);
        assert!(sheet.rules().is_empty());
    }

    #[test]
    fn add_rule() {
        let mut sheet = Stylesheet::new();
        sheet.add_rule(label_rule());
        assert_eq!(sheet.len(), 1);
        assert!(!sheet.is_empty());
    }

    #[test]
    fn multiple_rules() {
        let mut sheet = Stylesheet::new();
        sheet.add_rule(label_rule());
        sheet.add_rule(label_rule());
        assert_eq!(sheet.len(), 2);
    }

    #[test]
    fn rule_with_multiple_selectors() {
        let rule = Rule::new(
            SelectorList::new(vec![
                Selector::simple(CompoundSelector::type_selector("Label")),
                Selector::simple(CompoundSelector::type_selector("Container")),
            ]),
            vec![Declaration::new(
                PropertyName::Color,
                CssValue::Keyword("blue".into()),
            )],
        );
        assert_eq!(rule.selectors.selectors.len(), 2);
    }

    #[test]
    fn rule_with_multiple_declarations() {
        let rule = Rule::new(
            SelectorList::new(vec![Selector::simple(CompoundSelector::type_selector(
                "Label",
            ))]),
            vec![
                Declaration::new(PropertyName::Color, CssValue::Keyword("red".into())),
                Declaration::new(
                    PropertyName::Background,
                    CssValue::Keyword("blue".into()),
                ),
                Declaration::new(PropertyName::TextStyle, CssValue::Keyword("bold".into())),
            ],
        );
        assert_eq!(rule.declarations.len(), 3);
    }
}
