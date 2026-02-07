//! Terminal CSS (TCSS) parser and stylesheet types.
//!
//! TCSS is a subset of CSS tailored for terminal user interfaces.
//! It supports selectors, properties, and values specific to
//! terminal rendering capabilities.

pub mod ast;
pub mod cache;
pub mod cascade;
pub mod error;
pub mod matcher;
pub mod parser;
pub mod property;
pub mod reload;
pub mod selector;
pub mod theme;
pub mod tree;
pub mod value;
pub mod variable;

pub use ast::{Rule, Stylesheet, VariableDefinition};
pub use cache::MatchCache;
pub use cascade::{CascadeResolver, ComputedStyle};
pub use error::TcssError;
pub use matcher::{MatchedRule, StyleMatcher};
pub use parser::{extract_root_variables, parse_declaration, parse_stylesheet};
pub use property::{Declaration, PropertyName};
pub use reload::{StylesheetEvent, StylesheetLoader};
pub use selector::{
    Combinator, CompoundSelector, PseudoClass, Selector, SelectorList, SimpleSelector,
};
pub use theme::{Theme, ThemeManager};
pub use tree::{WidgetNode, WidgetState, WidgetTree};
pub use value::{CssValue, Length};
pub use variable::{VariableEnvironment, VariableMap};

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

#[cfg(test)]
mod pipeline_tests {
    use super::*;
    use crate::Color;
    use crate::color::NamedColor;

    /// Helper: parse stylesheet, build tree, match, cascade → computed style.
    fn sheet(css: &str) -> Stylesheet {
        let result = parse_stylesheet(css);
        assert!(result.is_ok(), "parse failed: {result:?}");
        match result {
            Ok(s) => s,
            Err(_) => unreachable!(),
        }
    }

    #[test]
    fn full_pipeline_simple() {
        let css = "Label { color: red; width: 20; }";
        let stylesheet = sheet(css);
        let matcher = StyleMatcher::new(&stylesheet);

        let mut tree = WidgetTree::new();
        tree.add_node(WidgetNode::new(1, "Label"));

        let matched = matcher.match_widget(&tree, 1);
        let style = CascadeResolver::resolve(&matched);

        assert_eq!(style.len(), 2);
        assert!(style.has(&PropertyName::Color));
        assert!(style.has(&PropertyName::Width));
    }

    #[test]
    fn full_pipeline_specificity() {
        let css = r#"
            Label { color: white; }
            .error { color: red; }
        "#;
        let stylesheet = sheet(css);
        let matcher = StyleMatcher::new(&stylesheet);

        let mut tree = WidgetTree::new();
        tree.add_node(WidgetNode::new(1, "Label").with_class("error"));

        let matched = matcher.match_widget(&tree, 1);
        let style = CascadeResolver::resolve(&matched);

        // .error (0,1,0) beats Label (0,0,1).
        assert_eq!(
            style.get(&PropertyName::Color),
            Some(&CssValue::Color(Color::Named(NamedColor::Red)))
        );
    }

    #[test]
    fn full_pipeline_important() {
        let css = r#"
            .error { color: red; }
            Label { color: white !important; }
        "#;
        let stylesheet = sheet(css);
        let matcher = StyleMatcher::new(&stylesheet);

        let mut tree = WidgetTree::new();
        tree.add_node(WidgetNode::new(1, "Label").with_class("error"));

        let matched = matcher.match_widget(&tree, 1);
        let style = CascadeResolver::resolve(&matched);

        // !important overrides higher specificity.
        assert_eq!(
            style.get(&PropertyName::Color),
            Some(&CssValue::Color(Color::Named(NamedColor::White)))
        );
    }

    #[test]
    fn full_pipeline_child_combinator() {
        let css = "Container > Label { color: blue; }";
        let stylesheet = sheet(css);
        let matcher = StyleMatcher::new(&stylesheet);

        let mut tree = WidgetTree::new();
        tree.add_node(WidgetNode::new(1, "Container"));
        let mut label = WidgetNode::new(2, "Label");
        label.parent = Some(1);
        tree.add_node(label);

        let matched = matcher.match_widget(&tree, 2);
        let style = CascadeResolver::resolve(&matched);

        assert_eq!(
            style.get(&PropertyName::Color),
            Some(&CssValue::Color(Color::Named(NamedColor::Blue)))
        );
    }

    #[test]
    fn full_pipeline_descendant() {
        let css = "Container Label { color: green; }";
        let stylesheet = sheet(css);
        let matcher = StyleMatcher::new(&stylesheet);

        let mut tree = WidgetTree::new();
        tree.add_node(WidgetNode::new(1, "Container"));
        let mut mid = WidgetNode::new(2, "Middle");
        mid.parent = Some(1);
        tree.add_node(mid);
        let mut label = WidgetNode::new(3, "Label");
        label.parent = Some(2);
        tree.add_node(label);

        // Label is grandchild of Container — descendant match.
        let matched = matcher.match_widget(&tree, 3);
        let style = CascadeResolver::resolve(&matched);

        assert_eq!(
            style.get(&PropertyName::Color),
            Some(&CssValue::Color(Color::Named(NamedColor::Green)))
        );
    }

    #[test]
    fn full_pipeline_pseudo_class() {
        let css = r#"
            Label { color: white; }
            Label:focus { color: green; }
        "#;
        let stylesheet = sheet(css);
        let matcher = StyleMatcher::new(&stylesheet);

        // Unfocused Label — only Label rule matches.
        let mut tree = WidgetTree::new();
        tree.add_node(WidgetNode::new(1, "Label"));

        let matched = matcher.match_widget(&tree, 1);
        let style = CascadeResolver::resolve(&matched);
        assert_eq!(
            style.get(&PropertyName::Color),
            Some(&CssValue::Color(Color::Named(NamedColor::White)))
        );

        // Now focus the label — Label:focus overrides (higher specificity).
        tree.get_mut(1)
            .iter_mut()
            .for_each(|n| n.state.focused = true);
        let matched = matcher.match_widget(&tree, 1);
        let style = CascadeResolver::resolve(&matched);
        assert_eq!(
            style.get(&PropertyName::Color),
            Some(&CssValue::Color(Color::Named(NamedColor::Green)))
        );
    }

    #[test]
    fn full_pipeline_cached_match() {
        let css = "Label { color: red; }";
        let stylesheet = sheet(css);
        let matcher = StyleMatcher::new(&stylesheet);

        let mut tree = WidgetTree::new();
        tree.add_node(WidgetNode::new(1, "Label"));

        let mut cache = MatchCache::new();

        // First query — cache miss.
        assert!(cache.get(1).is_none());
        let matched = matcher.match_widget(&tree, 1);
        cache.insert(1, matched);

        // Second query — cache hit.
        let cached = cache.get(1);
        assert!(cached.is_some());
        let cached = match cached {
            Some(m) => m,
            None => unreachable!(),
        };
        assert_eq!(cached.len(), 1);
    }

    #[test]
    fn full_pipeline_invalidation() {
        let css = r#"
            Label { color: white; }
            Label:focus { color: green; }
        "#;
        let stylesheet = sheet(css);
        let matcher = StyleMatcher::new(&stylesheet);

        let mut tree = WidgetTree::new();
        tree.add_node(WidgetNode::new(1, "Label"));

        let mut cache = MatchCache::new();

        // Initial match (unfocused).
        let matched = matcher.match_widget(&tree, 1);
        let style = CascadeResolver::resolve(&matched);
        assert_eq!(
            style.get(&PropertyName::Color),
            Some(&CssValue::Color(Color::Named(NamedColor::White)))
        );
        cache.insert(1, matched);

        // State change — focus the label.
        tree.get_mut(1)
            .iter_mut()
            .for_each(|n| n.state.focused = true);
        cache.invalidate(1);
        assert!(cache.get(1).is_none());

        // Re-match after invalidation.
        let matched = matcher.match_widget(&tree, 1);
        let style = CascadeResolver::resolve(&matched);
        assert_eq!(
            style.get(&PropertyName::Color),
            Some(&CssValue::Color(Color::Named(NamedColor::Green)))
        );
        cache.insert(1, matched);
        assert!(cache.get(1).is_some());
    }
}

#[cfg(test)]
mod themed_pipeline_tests {
    use super::*;
    use crate::Color;
    use crate::color::NamedColor;
    use crate::tcss::theme::extract_themes;

    fn sheet(css: &str) -> Stylesheet {
        let result = parse_stylesheet(css);
        assert!(result.is_ok(), "parse failed: {result:?}");
        match result {
            Ok(s) => s,
            Err(_) => unreachable!(),
        }
    }

    #[test]
    fn themed_pipeline_simple() {
        let css = r#"
            :root { $fg: white; $bg: #1e1e2e; }
            .dark { $fg: red; }
            Label { color: $fg; background: $bg; }
        "#;
        let stylesheet = sheet(css);
        let (globals, themes) = extract_themes(&stylesheet);
        let matcher = StyleMatcher::new(&stylesheet);

        let mut mgr = ThemeManager::new();
        for theme in themes {
            mgr.register(theme);
        }
        let result = mgr.set_active("dark");
        assert!(result.is_ok());

        let env = mgr.build_environment(&globals);

        let mut tree = WidgetTree::new();
        tree.add_node(WidgetNode::new(1, "Label"));

        let matched = matcher.match_widget(&tree, 1);
        let style = CascadeResolver::resolve_with_variables(&matched, &env);

        // $fg resolved from dark theme (red), $bg from global (#1e1e2e).
        assert_eq!(
            style.get(&PropertyName::Color),
            Some(&CssValue::Color(Color::Named(NamedColor::Red)))
        );
        assert!(style.has(&PropertyName::Background));
    }

    #[test]
    fn themed_pipeline_switch() {
        let css = r#"
            :root { $fg: white; }
            .dark { $fg: red; }
            .light { $fg: blue; }
            Label { color: $fg; }
        "#;
        let stylesheet = sheet(css);
        let (globals, themes) = extract_themes(&stylesheet);
        let matcher = StyleMatcher::new(&stylesheet);

        let mut mgr = ThemeManager::new();
        for theme in themes {
            mgr.register(theme);
        }

        let mut tree = WidgetTree::new();
        tree.add_node(WidgetNode::new(1, "Label"));
        let matched = matcher.match_widget(&tree, 1);

        // Dark theme.
        let result = mgr.set_active("dark");
        assert!(result.is_ok());
        let env = mgr.build_environment(&globals);
        let style = CascadeResolver::resolve_with_variables(&matched, &env);
        assert_eq!(
            style.get(&PropertyName::Color),
            Some(&CssValue::Color(Color::Named(NamedColor::Red)))
        );

        // Switch to light.
        let result = mgr.set_active("light");
        assert!(result.is_ok());
        let env = mgr.build_environment(&globals);
        let style = CascadeResolver::resolve_with_variables(&matched, &env);
        assert_eq!(
            style.get(&PropertyName::Color),
            Some(&CssValue::Color(Color::Named(NamedColor::Blue)))
        );
    }

    #[test]
    fn themed_pipeline_variable_in_property() {
        let css = r#"
            :root { $fg: green; }
            Label { color: $fg; }
        "#;
        let stylesheet = sheet(css);
        let globals = extract_root_variables(&stylesheet);
        let env = VariableEnvironment::with_global(globals);
        let matcher = StyleMatcher::new(&stylesheet);

        let mut tree = WidgetTree::new();
        tree.add_node(WidgetNode::new(1, "Label"));

        let matched = matcher.match_widget(&tree, 1);
        let style = CascadeResolver::resolve_with_variables(&matched, &env);

        assert_eq!(
            style.get(&PropertyName::Color),
            Some(&CssValue::Color(Color::Named(NamedColor::Green)))
        );
    }

    #[test]
    fn themed_pipeline_root_globals() {
        let css = r#"
            :root { $fg: yellow; }
            Label { color: $fg; }
        "#;
        let stylesheet = sheet(css);
        let globals = extract_root_variables(&stylesheet);
        let env = VariableEnvironment::with_global(globals);
        let matcher = StyleMatcher::new(&stylesheet);

        let mut tree = WidgetTree::new();
        tree.add_node(WidgetNode::new(1, "Label"));

        let matched = matcher.match_widget(&tree, 1);
        let style = CascadeResolver::resolve_with_variables(&matched, &env);

        assert_eq!(
            style.get(&PropertyName::Color),
            Some(&CssValue::Color(Color::Named(NamedColor::Yellow)))
        );
    }

    #[test]
    fn themed_pipeline_theme_overrides_root() {
        let css = r#"
            :root { $fg: white; }
            .dark { $fg: red; }
            Label { color: $fg; }
        "#;
        let stylesheet = sheet(css);
        let (globals, themes) = extract_themes(&stylesheet);

        let mut mgr = ThemeManager::new();
        for theme in themes {
            mgr.register(theme);
        }
        let result = mgr.set_active("dark");
        assert!(result.is_ok());
        let env = mgr.build_environment(&globals);
        let matcher = StyleMatcher::new(&stylesheet);

        let mut tree = WidgetTree::new();
        tree.add_node(WidgetNode::new(1, "Label"));

        let matched = matcher.match_widget(&tree, 1);
        let style = CascadeResolver::resolve_with_variables(&matched, &env);

        // Theme's $fg (red) overrides :root's $fg (white).
        assert_eq!(
            style.get(&PropertyName::Color),
            Some(&CssValue::Color(Color::Named(NamedColor::Red)))
        );
    }

    #[test]
    fn themed_pipeline_no_theme() {
        let css = r#"
            :root { $fg: white; }
            Label { color: $fg; }
        "#;
        let stylesheet = sheet(css);
        let (globals, _themes) = extract_themes(&stylesheet);

        // No active theme — only globals resolve.
        let mgr = ThemeManager::new();
        let env = mgr.build_environment(&globals);
        let matcher = StyleMatcher::new(&stylesheet);

        let mut tree = WidgetTree::new();
        tree.add_node(WidgetNode::new(1, "Label"));

        let matched = matcher.match_widget(&tree, 1);
        let style = CascadeResolver::resolve_with_variables(&matched, &env);

        assert_eq!(
            style.get(&PropertyName::Color),
            Some(&CssValue::Color(Color::Named(NamedColor::White)))
        );
    }

    #[test]
    fn themed_pipeline_loader() {
        let css = r#"
            :root { $fg: white; $bg: #1e1e2e; }
            .dark { $fg: red; }
            Label { color: $fg; }
        "#;
        let result = StylesheetLoader::load_string(css);
        assert!(result.is_ok());
        let loader = match result {
            Ok(l) => l,
            Err(_) => unreachable!(),
        };

        assert_eq!(loader.globals().len(), 2);
        assert_eq!(loader.themes().len(), 1);
        assert_eq!(loader.generation(), 1);
    }

    #[test]
    fn themed_pipeline_generation() {
        let css1 = ":root { $fg: white; }";
        let result = StylesheetLoader::load_string(css1);
        assert!(result.is_ok());
        let mut loader = match result {
            Ok(l) => l,
            Err(_) => unreachable!(),
        };
        assert_eq!(loader.generation(), 1);

        let css2 = ":root { $fg: red; $bg: black; }";
        let event = loader.reload_string(css2);
        assert!(event.is_ok());
        assert_eq!(loader.generation(), 2);
        assert_eq!(loader.globals().len(), 2);
    }
}
