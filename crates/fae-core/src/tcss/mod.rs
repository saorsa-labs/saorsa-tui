//! Terminal CSS (TCSS) parser and stylesheet types.
//!
//! TCSS is a subset of CSS tailored for terminal user interfaces.
//! It supports selectors, properties, and values specific to
//! terminal rendering capabilities.

pub mod ast;
pub mod error;
pub mod parser;
pub mod property;
pub mod selector;
pub mod tree;
pub mod value;

pub use ast::{Rule, Stylesheet};
pub use error::TcssError;
pub use parser::{parse_declaration, parse_stylesheet};
pub use property::{Declaration, PropertyName};
pub use selector::{
    Combinator, CompoundSelector, PseudoClass, Selector, SelectorList, SimpleSelector,
};
pub use tree::{WidgetNode, WidgetState, WidgetTree};
pub use value::{CssValue, Length};

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::Color;
    use crate::color::NamedColor;

    /// Helper to parse a stylesheet, asserting success.
    fn sheet(css: &str) -> Stylesheet {
        let result = parse_stylesheet(css);
        assert!(result.is_ok(), "parse failed: {result:?}");
        match result {
            Ok(s) => s,
            Err(_) => unreachable!(),
        }
    }

    #[test]
    fn parse_theme_stylesheet() {
        let css = r#"
            /* Base theme */
            Label {
                color: white;
                text-style: bold;
            }

            Container {
                background: #1e1e2e;
                padding: 2;
            }

            .error {
                color: red;
            }

            .warning {
                color: yellow;
            }

            #sidebar {
                width: 30;
                background: #2e2e3e;
            }

            #header {
                height: 3;
                background: #313244;
                color: blue;
            }

            Container > Label.title {
                color: blue;
                text-style: bold;
            }

            Label:focus {
                color: green;
            }

            Container .status {
                color: white;
                opacity: 0.8;
            }

            Label, Container {
                display: flex;
            }
        "#;
        let s = sheet(css);
        assert_eq!(s.len(), 10);
    }

    #[test]
    fn verify_selector_specificity_after_parse() {
        let css = "Container > Label.error#main:focus { color: red; }";
        let s = sheet(css);
        assert_eq!(s.len(), 1);

        let selector = &s.rules()[0].selectors.selectors[0];
        // head = Label.error#main:focus => id=1, class=1(error)+1(focus)=2, type=1(Label)
        // chain = Container => type=1
        // total = (1, 2, 2)
        assert_eq!(selector.specificity(), (1, 2, 2));
    }

    #[test]
    fn verify_property_values_typed() {
        let css = r#"
            Label {
                color: red;
                width: 20;
                height: 50%;
                display: flex;
                flex-grow: 3;
                opacity: 0.7;
                padding: auto;
            }
        "#;
        let s = sheet(css);
        let decls = &s.rules()[0].declarations;

        assert_eq!(decls.len(), 7);
        assert!(matches!(
            decls[0].value,
            CssValue::Color(Color::Named(NamedColor::Red))
        ));
        assert!(matches!(
            decls[1].value,
            CssValue::Length(Length::Cells(20))
        ));
        assert!(
            matches!(decls[2].value, CssValue::Length(Length::Percent(p)) if (p - 50.0).abs() < f32::EPSILON)
        );
        assert!(matches!(&decls[3].value, CssValue::Keyword(k) if k == "flex"));
        assert!(matches!(decls[4].value, CssValue::Integer(3)));
        assert!(matches!(decls[5].value, CssValue::Float(f) if (f - 0.7).abs() < f32::EPSILON));
        assert!(matches!(decls[6].value, CssValue::Length(Length::Auto)));
    }

    #[test]
    fn round_trip_parse_inspect() {
        let css = ".sidebar { width: 30; background: #1e1e2e; color: white; }";
        let s = sheet(css);

        assert_eq!(s.len(), 1);
        let rule = &s.rules()[0];

        // Verify selector
        assert_eq!(rule.selectors.selectors.len(), 1);
        let head = &rule.selectors.selectors[0].head;
        assert_eq!(head.components.len(), 1);
        assert_eq!(head.components[0], SimpleSelector::Class("sidebar".into()));

        // Verify declarations
        assert_eq!(rule.declarations.len(), 3);
        assert_eq!(rule.declarations[0].property, PropertyName::Width);
        assert_eq!(
            rule.declarations[0].value,
            CssValue::Length(Length::Cells(30))
        );
        assert_eq!(rule.declarations[1].property, PropertyName::Background);
        assert!(matches!(
            rule.declarations[1].value,
            CssValue::Color(Color::Rgb {
                r: 30,
                g: 30,
                b: 46
            })
        ));
        assert_eq!(rule.declarations[2].property, PropertyName::Color);
        assert_eq!(
            rule.declarations[2].value,
            CssValue::Color(Color::Named(NamedColor::White))
        );
    }

    #[test]
    fn error_recovery_mixed_valid_invalid() {
        let css = r#"
            Label { color: red; }
            ??? invalid stuff ???
            Container { background: blue; }
            12345 { }
            .error { color: yellow; }
        "#;
        let s = sheet(css);
        // Should recover and parse at least some valid rules
        assert!(!s.is_empty());
    }

    #[test]
    fn empty_input_returns_empty() {
        let s = sheet("");
        assert!(s.is_empty());
        assert_eq!(s.len(), 0);
    }
}
