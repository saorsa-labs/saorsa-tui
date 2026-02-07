//! CSS cascade resolution.
//!
//! Implements the CSS cascade algorithm that resolves matched rules
//! into a final [`ComputedStyle`] by applying specificity and source
//! order, with `!important` declarations overriding normal ones.

use std::collections::HashMap;

use crate::tcss::matcher::MatchedRule;
use crate::tcss::property::PropertyName;
use crate::tcss::value::CssValue;

/// The computed style for a widget — final resolved property values.
///
/// After cascade resolution, this contains the winning value for each
/// property from all matching rules.
#[derive(Clone, Debug, Default)]
pub struct ComputedStyle {
    properties: HashMap<PropertyName, CssValue>,
}

impl ComputedStyle {
    /// Create a new empty computed style.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get a property value.
    pub fn get(&self, prop: &PropertyName) -> Option<&CssValue> {
        self.properties.get(prop)
    }

    /// Set a property value.
    pub fn set(&mut self, prop: PropertyName, value: CssValue) {
        self.properties.insert(prop, value);
    }

    /// Check if a property is set.
    pub fn has(&self, prop: &PropertyName) -> bool {
        self.properties.contains_key(prop)
    }

    /// Return the number of set properties.
    pub fn len(&self) -> usize {
        self.properties.len()
    }

    /// Return whether no properties are set.
    pub fn is_empty(&self) -> bool {
        self.properties.is_empty()
    }

    /// Iterate over all property-value pairs.
    pub fn iter(&self) -> impl Iterator<Item = (&PropertyName, &CssValue)> {
        self.properties.iter()
    }
}

/// A cascade resolver.
///
/// Applies the CSS cascade algorithm to a list of matched rules,
/// producing a final [`ComputedStyle`].
pub struct CascadeResolver;

/// A declaration with its cascade ordering key (specificity + source order).
type CascadeEntry = (PropertyName, CssValue, (u16, u16, u16), usize);

impl CascadeResolver {
    /// Resolve matched rules into a computed style.
    ///
    /// # Algorithm
    ///
    /// 1. Separate declarations into normal and `!important`.
    /// 2. Sort normal declarations by (specificity, source_order) ascending.
    /// 3. Sort `!important` declarations by (specificity, source_order) ascending.
    /// 4. Apply normal declarations first (later entries override earlier).
    /// 5. Apply `!important` declarations last (they override everything).
    /// 6. Return the final [`ComputedStyle`].
    pub fn resolve(matches: &[MatchedRule]) -> ComputedStyle {
        let mut normal: Vec<CascadeEntry> = Vec::new();
        let mut important: Vec<CascadeEntry> = Vec::new();

        for matched in matches {
            for decl in &matched.declarations {
                let entry = (
                    decl.property.clone(),
                    decl.value.clone(),
                    matched.specificity,
                    matched.source_order,
                );
                if decl.important {
                    important.push(entry);
                } else {
                    normal.push(entry);
                }
            }
        }

        // Sort ascending by (specificity, source_order).
        // Later entries in the sorted list override earlier ones.
        normal.sort_by_key(|&(_, _, spec, order)| (spec, order));
        important.sort_by_key(|&(_, _, spec, order)| (spec, order));

        let mut style = ComputedStyle::new();

        // Apply normal declarations (last wins).
        for (prop, value, _, _) in normal {
            style.set(prop, value);
        }

        // Apply !important declarations (override everything).
        for (prop, value, _, _) in important {
            style.set(prop, value);
        }

        style
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Color;
    use crate::color::NamedColor;
    use crate::tcss::property::Declaration;
    use crate::tcss::value::Length;

    fn matched_rule(
        specificity: (u16, u16, u16),
        source_order: usize,
        declarations: Vec<Declaration>,
    ) -> MatchedRule {
        MatchedRule {
            specificity,
            source_order,
            declarations,
        }
    }

    #[test]
    fn empty_matches_empty_style() {
        let style = CascadeResolver::resolve(&[]);
        assert!(style.is_empty());
        assert_eq!(style.len(), 0);
    }

    #[test]
    fn single_rule_applied() {
        let rules = vec![matched_rule(
            (0, 0, 1),
            0,
            vec![Declaration::new(
                PropertyName::Color,
                CssValue::Color(Color::Named(NamedColor::Red)),
            )],
        )];
        let style = CascadeResolver::resolve(&rules);
        assert_eq!(style.len(), 1);
        assert!(style.has(&PropertyName::Color));
    }

    #[test]
    fn later_rule_overrides() {
        // Same specificity: later source order wins.
        let rules = vec![
            matched_rule(
                (0, 0, 1),
                0,
                vec![Declaration::new(
                    PropertyName::Color,
                    CssValue::Keyword("red".into()),
                )],
            ),
            matched_rule(
                (0, 0, 1),
                1,
                vec![Declaration::new(
                    PropertyName::Color,
                    CssValue::Keyword("blue".into()),
                )],
            ),
        ];
        let style = CascadeResolver::resolve(&rules);
        assert_eq!(
            style.get(&PropertyName::Color),
            Some(&CssValue::Keyword("blue".into()))
        );
    }

    #[test]
    fn higher_specificity_wins() {
        // Higher specificity wins regardless of order.
        let rules = vec![
            matched_rule(
                (0, 1, 0),
                0,
                vec![Declaration::new(
                    PropertyName::Color,
                    CssValue::Keyword("class-wins".into()),
                )],
            ),
            matched_rule(
                (0, 0, 1),
                1,
                vec![Declaration::new(
                    PropertyName::Color,
                    CssValue::Keyword("type-loses".into()),
                )],
            ),
        ];
        let style = CascadeResolver::resolve(&rules);
        assert_eq!(
            style.get(&PropertyName::Color),
            Some(&CssValue::Keyword("class-wins".into()))
        );
    }

    #[test]
    fn important_overrides_specificity() {
        let rules = vec![
            matched_rule(
                (1, 0, 0),
                0,
                vec![Declaration::new(
                    PropertyName::Color,
                    CssValue::Keyword("high-spec".into()),
                )],
            ),
            matched_rule(
                (0, 0, 1),
                1,
                vec![Declaration::important(
                    PropertyName::Color,
                    CssValue::Keyword("important-wins".into()),
                )],
            ),
        ];
        let style = CascadeResolver::resolve(&rules);
        assert_eq!(
            style.get(&PropertyName::Color),
            Some(&CssValue::Keyword("important-wins".into()))
        );
    }

    #[test]
    fn important_vs_important() {
        // Both !important: higher specificity wins.
        let rules = vec![
            matched_rule(
                (0, 1, 0),
                0,
                vec![Declaration::important(
                    PropertyName::Color,
                    CssValue::Keyword("class-important".into()),
                )],
            ),
            matched_rule(
                (0, 0, 1),
                1,
                vec![Declaration::important(
                    PropertyName::Color,
                    CssValue::Keyword("type-important".into()),
                )],
            ),
        ];
        let style = CascadeResolver::resolve(&rules);
        assert_eq!(
            style.get(&PropertyName::Color),
            Some(&CssValue::Keyword("class-important".into()))
        );
    }

    #[test]
    fn multiple_properties_merged() {
        let rules = vec![
            matched_rule(
                (0, 0, 1),
                0,
                vec![Declaration::new(
                    PropertyName::Color,
                    CssValue::Keyword("red".into()),
                )],
            ),
            matched_rule(
                (0, 0, 1),
                1,
                vec![Declaration::new(
                    PropertyName::Background,
                    CssValue::Keyword("blue".into()),
                )],
            ),
        ];
        let style = CascadeResolver::resolve(&rules);
        assert_eq!(style.len(), 2);
        assert!(style.has(&PropertyName::Color));
        assert!(style.has(&PropertyName::Background));
    }

    #[test]
    fn same_property_last_wins() {
        let rules = vec![
            matched_rule(
                (0, 0, 1),
                0,
                vec![Declaration::new(
                    PropertyName::Color,
                    CssValue::Keyword("first".into()),
                )],
            ),
            matched_rule(
                (0, 0, 1),
                1,
                vec![Declaration::new(
                    PropertyName::Color,
                    CssValue::Keyword("second".into()),
                )],
            ),
            matched_rule(
                (0, 0, 1),
                2,
                vec![Declaration::new(
                    PropertyName::Color,
                    CssValue::Keyword("third".into()),
                )],
            ),
        ];
        let style = CascadeResolver::resolve(&rules);
        assert_eq!(
            style.get(&PropertyName::Color),
            Some(&CssValue::Keyword("third".into()))
        );
    }

    #[test]
    fn computed_style_accessors() {
        let mut style = ComputedStyle::new();
        assert!(style.is_empty());
        assert_eq!(style.len(), 0);
        assert!(!style.has(&PropertyName::Color));
        assert!(style.get(&PropertyName::Color).is_none());

        style.set(PropertyName::Color, CssValue::Keyword("red".into()));
        assert!(!style.is_empty());
        assert_eq!(style.len(), 1);
        assert!(style.has(&PropertyName::Color));
        assert_eq!(
            style.get(&PropertyName::Color),
            Some(&CssValue::Keyword("red".into()))
        );
    }

    #[test]
    fn computed_style_iteration() {
        let mut style = ComputedStyle::new();
        style.set(PropertyName::Color, CssValue::Keyword("red".into()));
        style.set(PropertyName::Width, CssValue::Length(Length::Cells(10)));
        let pairs: Vec<_> = style.iter().collect();
        assert_eq!(pairs.len(), 2);
    }

    #[test]
    fn real_cascade_example() {
        // Simulate: Label { color: white; } .error { color: red; } #main { width: 30; }
        let rules = vec![
            matched_rule(
                (0, 0, 1),
                0,
                vec![
                    Declaration::new(PropertyName::Color, CssValue::Keyword("white".into())),
                    Declaration::new(PropertyName::TextStyle, CssValue::Keyword("bold".into())),
                ],
            ),
            matched_rule(
                (0, 1, 0),
                1,
                vec![Declaration::new(
                    PropertyName::Color,
                    CssValue::Keyword("red".into()),
                )],
            ),
            matched_rule(
                (1, 0, 0),
                2,
                vec![Declaration::new(
                    PropertyName::Width,
                    CssValue::Length(Length::Cells(30)),
                )],
            ),
        ];
        let style = CascadeResolver::resolve(&rules);
        assert_eq!(style.len(), 3);
        // .error overrides Label for color (higher specificity).
        assert_eq!(
            style.get(&PropertyName::Color),
            Some(&CssValue::Keyword("red".into()))
        );
        // text-style from Label rule persists.
        assert_eq!(
            style.get(&PropertyName::TextStyle),
            Some(&CssValue::Keyword("bold".into()))
        );
        // width from #main rule.
        assert_eq!(
            style.get(&PropertyName::Width),
            Some(&CssValue::Length(Length::Cells(30)))
        );
    }
}
