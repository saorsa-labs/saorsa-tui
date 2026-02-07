//! CSS selector matching engine.
//!
//! Matches selectors from parsed TCSS stylesheets against widget nodes
//! in a [`WidgetTree`]. Supports simple, compound, and complex selectors
//! with child and descendant combinators.

use crate::focus::WidgetId;
use crate::tcss::ast::Stylesheet;
use crate::tcss::property::Declaration;
use crate::tcss::selector::{
    Combinator, CompoundSelector, PseudoClass, Selector, SelectorList, SimpleSelector,
};
use crate::tcss::tree::{WidgetNode, WidgetTree};

/// A matched rule with its specificity and source order.
#[derive(Clone, Debug)]
pub struct MatchedRule {
    /// CSS specificity as (id, class, type) counts.
    pub specificity: (u16, u16, u16),
    /// Index of the rule in the stylesheet (for cascade ordering).
    pub source_order: usize,
    /// The declarations from the matched rule.
    pub declarations: Vec<Declaration>,
}

/// The selector matcher — takes a stylesheet, matches against a tree.
pub struct StyleMatcher {
    /// (selector list, declarations, source order) for each rule.
    rules: Vec<(SelectorList, Vec<Declaration>, usize)>,
}

impl StyleMatcher {
    /// Create a new style matcher from a parsed stylesheet.
    ///
    /// Extracts all rules with their source ordering for later matching.
    pub fn new(stylesheet: &Stylesheet) -> Self {
        let rules = stylesheet
            .rules()
            .iter()
            .enumerate()
            .map(|(i, rule)| (rule.selectors.clone(), rule.declarations.clone(), i))
            .collect();
        Self { rules }
    }

    /// Return all rules that match a given widget in the tree.
    ///
    /// Each matched rule includes its specificity and source order
    /// for cascade resolution.
    pub fn match_widget(&self, tree: &WidgetTree, id: WidgetId) -> Vec<MatchedRule> {
        let mut matched = Vec::new();
        for (selectors, declarations, source_order) in &self.rules {
            if let Some(specificity) = Self::matches_any(tree, id, selectors) {
                matched.push(MatchedRule {
                    specificity,
                    source_order: *source_order,
                    declarations: declarations.clone(),
                });
            }
        }
        matched
    }

    /// Check if any selector in the list matches the widget.
    ///
    /// Returns the highest specificity among matching selectors,
    /// or `None` if no selector matches.
    pub fn matches_any(
        tree: &WidgetTree,
        id: WidgetId,
        selectors: &SelectorList,
    ) -> Option<(u16, u16, u16)> {
        let mut best: Option<(u16, u16, u16)> = None;
        for selector in &selectors.selectors {
            if matches_selector(tree, id, selector) {
                let spec = selector.specificity();
                best = Some(match best {
                    Some(current) if spec > current => spec,
                    Some(current) => current,
                    None => spec,
                });
            }
        }
        best
    }
}

// ---------------------------------------------------------------------------
// Core matching functions
// ---------------------------------------------------------------------------

/// Check if a simple selector matches a widget node in tree context.
pub fn matches_simple(tree: &WidgetTree, node: &WidgetNode, selector: &SimpleSelector) -> bool {
    match selector {
        SimpleSelector::Type(name) => node.type_name == *name,
        SimpleSelector::Class(name) => node.has_class(name),
        SimpleSelector::Id(name) => node.css_id.as_deref() == Some(name.as_str()),
        SimpleSelector::Universal => true,
        SimpleSelector::PseudoClass(pc) => matches_pseudo_class(tree, node, pc),
    }
}

/// Check if a compound selector matches a widget node.
///
/// All components must match (AND logic).
pub fn matches_compound(tree: &WidgetTree, node: &WidgetNode, selector: &CompoundSelector) -> bool {
    selector
        .components
        .iter()
        .all(|s| matches_simple(tree, node, s))
}

/// Check if a pseudo-class matches a widget node in context.
pub fn matches_pseudo_class(tree: &WidgetTree, node: &WidgetNode, pseudo: &PseudoClass) -> bool {
    match pseudo {
        PseudoClass::Focus => node.state.focused,
        PseudoClass::Hover => node.state.hovered,
        PseudoClass::Disabled => node.state.disabled,
        PseudoClass::Active => node.state.active,
        PseudoClass::FirstChild => tree.is_first_child(node.id),
        PseudoClass::LastChild => tree.is_last_child(node.id),
        PseudoClass::NthChild(n) => {
            // CSS :nth-child is 1-indexed, child_index is 0-indexed.
            let target = *n - 1;
            if target < 0 {
                return false;
            }
            #[allow(clippy::cast_sign_loss)]
            let target_usize = target as usize;
            tree.child_index(node.id) == Some(target_usize)
        }
        PseudoClass::Even => tree.child_index(node.id).is_some_and(|i| (i + 1) % 2 == 0),
        PseudoClass::Odd => tree.child_index(node.id).is_some_and(|i| (i + 1) % 2 == 1),
        PseudoClass::Root => tree.root() == Some(node.id),
    }
}

/// Check if a full selector (with combinators) matches a widget.
///
/// Walks the selector chain from right (subject) to left, matching
/// each compound selector against the tree using the appropriate
/// combinator logic.
pub fn matches_selector(tree: &WidgetTree, node_id: WidgetId, selector: &Selector) -> bool {
    let node = match tree.get(node_id) {
        Some(n) => n,
        None => return false,
    };

    // The head (rightmost) compound must match the target node.
    if !matches_compound(tree, node, &selector.head) {
        return false;
    }

    // No chain means simple match — done.
    if selector.chain.is_empty() {
        return true;
    }

    // Walk the chain from right to left (chain[0] is leftmost ancestor).
    // We iterate from the end of the chain (closest to subject) backwards.
    let mut current_id = node_id;

    for (combinator, compound) in selector.chain.iter().rev() {
        match combinator {
            Combinator::Child => {
                // Must match the immediate parent.
                let parent = match tree.parent(current_id) {
                    Some(p) => p,
                    None => return false,
                };
                if !matches_compound(tree, parent, compound) {
                    return false;
                }
                current_id = parent.id;
            }
            Combinator::Descendant => {
                // Walk up all ancestors until one matches.
                let ancestors = tree.ancestors(current_id);
                let mut found = false;
                for ancestor_id in &ancestors {
                    let ancestor = match tree.get(*ancestor_id) {
                        Some(a) => a,
                        None => continue,
                    };
                    if matches_compound(tree, ancestor, compound) {
                        current_id = ancestor.id;
                        found = true;
                        break;
                    }
                }
                if !found {
                    return false;
                }
            }
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tcss::ast::Rule;
    use crate::tcss::property::PropertyName;
    use crate::tcss::value::CssValue;

    // -----------------------------------------------------------------------
    // Helper: build a tree with a root and optional children
    // -----------------------------------------------------------------------

    fn make_tree_with_root(type_name: &str) -> (WidgetTree, WidgetId) {
        let mut tree = WidgetTree::new();
        let root = WidgetNode::new(1, type_name);
        tree.add_node(root);
        (tree, 1)
    }

    fn add_child(tree: &mut WidgetTree, id: WidgetId, parent: WidgetId, type_name: &str) {
        let mut node = WidgetNode::new(id, type_name);
        node.parent = Some(parent);
        tree.add_node(node);
    }

    // -----------------------------------------------------------------------
    // Task 2: Simple selector matching
    // -----------------------------------------------------------------------

    #[test]
    fn match_type_selector() {
        let (tree, _) = make_tree_with_root("Label");
        let node = tree.get(1);
        assert!(node.is_some());
        let node = match node {
            Some(n) => n,
            None => unreachable!(),
        };
        assert!(matches_simple(
            &tree,
            node,
            &SimpleSelector::Type("Label".into())
        ));
    }

    #[test]
    fn match_type_mismatch() {
        let (tree, _) = make_tree_with_root("Label");
        let node = match tree.get(1) {
            Some(n) => n,
            None => unreachable!(),
        };
        assert!(!matches_simple(
            &tree,
            node,
            &SimpleSelector::Type("Container".into())
        ));
    }

    #[test]
    fn match_class_selector() {
        let (mut tree, _) = make_tree_with_root("Label");
        tree.get_mut(1)
            .iter_mut()
            .for_each(|n| n.classes.push("error".into()));
        let node = match tree.get(1) {
            Some(n) => n,
            None => unreachable!(),
        };
        assert!(matches_simple(
            &tree,
            node,
            &SimpleSelector::Class("error".into())
        ));
    }

    #[test]
    fn match_class_mismatch() {
        let (mut tree, _) = make_tree_with_root("Label");
        tree.get_mut(1)
            .iter_mut()
            .for_each(|n| n.classes.push("error".into()));
        let node = match tree.get(1) {
            Some(n) => n,
            None => unreachable!(),
        };
        assert!(!matches_simple(
            &tree,
            node,
            &SimpleSelector::Class("warning".into())
        ));
    }

    #[test]
    fn match_id_selector() {
        let (mut tree, _) = make_tree_with_root("Label");
        tree.get_mut(1)
            .iter_mut()
            .for_each(|n| n.css_id = Some("sidebar".into()));
        let node = match tree.get(1) {
            Some(n) => n,
            None => unreachable!(),
        };
        assert!(matches_simple(
            &tree,
            node,
            &SimpleSelector::Id("sidebar".into())
        ));
    }

    #[test]
    fn match_id_mismatch() {
        let (mut tree, _) = make_tree_with_root("Label");
        tree.get_mut(1)
            .iter_mut()
            .for_each(|n| n.css_id = Some("sidebar".into()));
        let node = match tree.get(1) {
            Some(n) => n,
            None => unreachable!(),
        };
        assert!(!matches_simple(
            &tree,
            node,
            &SimpleSelector::Id("header".into())
        ));
    }

    #[test]
    fn match_universal() {
        let (tree, _) = make_tree_with_root("Anything");
        let node = match tree.get(1) {
            Some(n) => n,
            None => unreachable!(),
        };
        assert!(matches_simple(&tree, node, &SimpleSelector::Universal));
    }

    #[test]
    fn match_compound_all() {
        let mut tree = WidgetTree::new();
        tree.add_node(
            WidgetNode::new(1, "Label")
                .with_class("error")
                .with_id("main"),
        );
        let node = match tree.get(1) {
            Some(n) => n,
            None => unreachable!(),
        };
        let compound = CompoundSelector::new(vec![
            SimpleSelector::Type("Label".into()),
            SimpleSelector::Class("error".into()),
            SimpleSelector::Id("main".into()),
        ]);
        assert!(matches_compound(&tree, node, &compound));
    }

    #[test]
    fn match_compound_partial_fail() {
        let mut tree = WidgetTree::new();
        tree.add_node(
            WidgetNode::new(1, "Label")
                .with_class("error")
                .with_id("main"),
        );
        let node = match tree.get(1) {
            Some(n) => n,
            None => unreachable!(),
        };
        // Type mismatch should cause failure even though class/id match.
        let compound = CompoundSelector::new(vec![
            SimpleSelector::Type("Container".into()),
            SimpleSelector::Class("error".into()),
            SimpleSelector::Id("main".into()),
        ]);
        assert!(!matches_compound(&tree, node, &compound));
    }

    // -----------------------------------------------------------------------
    // Task 3: Pseudo-class matching
    // -----------------------------------------------------------------------

    #[test]
    fn match_focus() {
        let (mut tree, _) = make_tree_with_root("Label");
        tree.get_mut(1)
            .iter_mut()
            .for_each(|n| n.state.focused = true);
        let node = match tree.get(1) {
            Some(n) => n,
            None => unreachable!(),
        };
        assert!(matches_pseudo_class(&tree, node, &PseudoClass::Focus));
    }

    #[test]
    fn match_focus_unfocused() {
        let (tree, _) = make_tree_with_root("Label");
        let node = match tree.get(1) {
            Some(n) => n,
            None => unreachable!(),
        };
        assert!(!matches_pseudo_class(&tree, node, &PseudoClass::Focus));
    }

    #[test]
    fn match_hover() {
        let (mut tree, _) = make_tree_with_root("Label");
        tree.get_mut(1)
            .iter_mut()
            .for_each(|n| n.state.hovered = true);
        let node = match tree.get(1) {
            Some(n) => n,
            None => unreachable!(),
        };
        assert!(matches_pseudo_class(&tree, node, &PseudoClass::Hover));
    }

    #[test]
    fn match_disabled() {
        let (mut tree, _) = make_tree_with_root("Label");
        tree.get_mut(1)
            .iter_mut()
            .for_each(|n| n.state.disabled = true);
        let node = match tree.get(1) {
            Some(n) => n,
            None => unreachable!(),
        };
        assert!(matches_pseudo_class(&tree, node, &PseudoClass::Disabled));
    }

    #[test]
    fn match_active() {
        let (mut tree, _) = make_tree_with_root("Label");
        tree.get_mut(1)
            .iter_mut()
            .for_each(|n| n.state.active = true);
        let node = match tree.get(1) {
            Some(n) => n,
            None => unreachable!(),
        };
        assert!(matches_pseudo_class(&tree, node, &PseudoClass::Active));
    }

    #[test]
    fn match_first_child() {
        let (mut tree, _) = make_tree_with_root("Root");
        add_child(&mut tree, 2, 1, "First");
        add_child(&mut tree, 3, 1, "Second");
        add_child(&mut tree, 4, 1, "Third");

        let first = match tree.get(2) {
            Some(n) => n,
            None => unreachable!(),
        };
        assert!(matches_pseudo_class(&tree, first, &PseudoClass::FirstChild));

        let second = match tree.get(3) {
            Some(n) => n,
            None => unreachable!(),
        };
        assert!(!matches_pseudo_class(
            &tree,
            second,
            &PseudoClass::FirstChild
        ));
    }

    #[test]
    fn match_last_child() {
        let (mut tree, _) = make_tree_with_root("Root");
        add_child(&mut tree, 2, 1, "First");
        add_child(&mut tree, 3, 1, "Last");

        let last = match tree.get(3) {
            Some(n) => n,
            None => unreachable!(),
        };
        assert!(matches_pseudo_class(&tree, last, &PseudoClass::LastChild));

        let first = match tree.get(2) {
            Some(n) => n,
            None => unreachable!(),
        };
        assert!(!matches_pseudo_class(&tree, first, &PseudoClass::LastChild));
    }

    #[test]
    fn match_nth_child() {
        let (mut tree, _) = make_tree_with_root("Root");
        add_child(&mut tree, 2, 1, "C1");
        add_child(&mut tree, 3, 1, "C2");
        add_child(&mut tree, 4, 1, "C3");

        // :nth-child(3) should match the third child (0-indexed = 2).
        let third = match tree.get(4) {
            Some(n) => n,
            None => unreachable!(),
        };
        assert!(matches_pseudo_class(
            &tree,
            third,
            &PseudoClass::NthChild(3)
        ));

        let first = match tree.get(2) {
            Some(n) => n,
            None => unreachable!(),
        };
        assert!(!matches_pseudo_class(
            &tree,
            first,
            &PseudoClass::NthChild(3)
        ));
    }

    #[test]
    fn match_even() {
        let (mut tree, _) = make_tree_with_root("Root");
        add_child(&mut tree, 2, 1, "C1");
        add_child(&mut tree, 3, 1, "C2");

        // C2 is at 0-indexed position 1, so (1+1) % 2 == 0 → even.
        let second = match tree.get(3) {
            Some(n) => n,
            None => unreachable!(),
        };
        assert!(matches_pseudo_class(&tree, second, &PseudoClass::Even));

        let first = match tree.get(2) {
            Some(n) => n,
            None => unreachable!(),
        };
        assert!(!matches_pseudo_class(&tree, first, &PseudoClass::Even));
    }

    #[test]
    fn match_odd() {
        let (mut tree, _) = make_tree_with_root("Root");
        add_child(&mut tree, 2, 1, "C1");
        add_child(&mut tree, 3, 1, "C2");

        // C1 is at 0-indexed position 0, so (0+1) % 2 == 1 → odd.
        let first = match tree.get(2) {
            Some(n) => n,
            None => unreachable!(),
        };
        assert!(matches_pseudo_class(&tree, first, &PseudoClass::Odd));

        let second = match tree.get(3) {
            Some(n) => n,
            None => unreachable!(),
        };
        assert!(!matches_pseudo_class(&tree, second, &PseudoClass::Odd));
    }

    #[test]
    fn match_compound_with_pseudo() {
        let (mut tree, _) = make_tree_with_root("Label");
        tree.get_mut(1)
            .iter_mut()
            .for_each(|n| n.state.focused = true);
        let node = match tree.get(1) {
            Some(n) => n,
            None => unreachable!(),
        };
        let compound = CompoundSelector::new(vec![
            SimpleSelector::Type("Label".into()),
            SimpleSelector::PseudoClass(PseudoClass::Focus),
        ]);
        assert!(matches_compound(&tree, node, &compound));
    }

    // -----------------------------------------------------------------------
    // Task 4: Combinator matching
    // -----------------------------------------------------------------------

    #[test]
    fn match_simple_selector_no_chain() {
        let (tree, _) = make_tree_with_root("Label");
        let sel = Selector::simple(CompoundSelector::type_selector("Label"));
        assert!(matches_selector(&tree, 1, &sel));
    }

    #[test]
    fn match_child_combinator() {
        let (mut tree, _) = make_tree_with_root("Container");
        add_child(&mut tree, 2, 1, "Label");

        // Container > Label
        let sel = Selector {
            head: CompoundSelector::type_selector("Label"),
            chain: vec![(
                Combinator::Child,
                CompoundSelector::type_selector("Container"),
            )],
        };
        assert!(matches_selector(&tree, 2, &sel));
    }

    #[test]
    fn match_child_wrong_parent() {
        let (mut tree, _) = make_tree_with_root("Header");
        add_child(&mut tree, 2, 1, "Label");

        // Container > Label — parent is Header, not Container.
        let sel = Selector {
            head: CompoundSelector::type_selector("Label"),
            chain: vec![(
                Combinator::Child,
                CompoundSelector::type_selector("Container"),
            )],
        };
        assert!(!matches_selector(&tree, 2, &sel));
    }

    #[test]
    fn match_descendant_combinator() {
        let (mut tree, _) = make_tree_with_root("Container");
        add_child(&mut tree, 2, 1, "Label");

        // Container Label (descendant)
        let sel = Selector {
            head: CompoundSelector::type_selector("Label"),
            chain: vec![(
                Combinator::Descendant,
                CompoundSelector::type_selector("Container"),
            )],
        };
        assert!(matches_selector(&tree, 2, &sel));
    }

    #[test]
    fn match_descendant_deep() {
        let (mut tree, _) = make_tree_with_root("Container");
        add_child(&mut tree, 2, 1, "Middle");
        add_child(&mut tree, 3, 2, "Label");

        // Container Label — Label is grandchild of Container.
        let sel = Selector {
            head: CompoundSelector::type_selector("Label"),
            chain: vec![(
                Combinator::Descendant,
                CompoundSelector::type_selector("Container"),
            )],
        };
        assert!(matches_selector(&tree, 3, &sel));
    }

    #[test]
    fn match_descendant_miss() {
        let (mut tree, _) = make_tree_with_root("Header");
        add_child(&mut tree, 2, 1, "Label");

        // Container Label — Container not in ancestors.
        let sel = Selector {
            head: CompoundSelector::type_selector("Label"),
            chain: vec![(
                Combinator::Descendant,
                CompoundSelector::type_selector("Container"),
            )],
        };
        assert!(!matches_selector(&tree, 2, &sel));
    }

    #[test]
    fn match_complex_chain() {
        // Tree: A(1) > B(2) > C(3)
        let (mut tree, _) = make_tree_with_root("A");
        add_child(&mut tree, 2, 1, "B");
        add_child(&mut tree, 3, 2, "C");

        // A > B > C
        let sel = Selector {
            head: CompoundSelector::type_selector("C"),
            chain: vec![
                (Combinator::Child, CompoundSelector::type_selector("A")),
                (Combinator::Child, CompoundSelector::type_selector("B")),
            ],
        };
        assert!(matches_selector(&tree, 3, &sel));
    }

    #[test]
    fn match_three_levels_wrong() {
        // Tree: A(1) > X(2) > C(3)
        let (mut tree, _) = make_tree_with_root("A");
        add_child(&mut tree, 2, 1, "X");
        add_child(&mut tree, 3, 2, "C");

        // A > B > C — B doesn't match X.
        let sel = Selector {
            head: CompoundSelector::type_selector("C"),
            chain: vec![
                (Combinator::Child, CompoundSelector::type_selector("A")),
                (Combinator::Child, CompoundSelector::type_selector("B")),
            ],
        };
        assert!(!matches_selector(&tree, 3, &sel));
    }

    #[test]
    fn match_mixed_combinators() {
        // Tree: A(1) > B(2) > C(3) > D(4)
        let (mut tree, _) = make_tree_with_root("A");
        add_child(&mut tree, 2, 1, "B");
        add_child(&mut tree, 3, 2, "C");
        add_child(&mut tree, 4, 3, "D");

        // A B > D — D's parent is C (child fails), but descendant of A through B.
        // Actually: A (descendant) B (child) D
        // chain[0] = (Descendant, A), chain[1] = (Child, B)
        // Process: D matches head. B must be child-parent of D → parent of D is C, not B → FAIL.
        let sel = Selector {
            head: CompoundSelector::type_selector("D"),
            chain: vec![
                (Combinator::Descendant, CompoundSelector::type_selector("A")),
                (Combinator::Child, CompoundSelector::type_selector("C")),
            ],
        };
        // D's parent is C (child match), C's ancestor includes A (descendant match).
        assert!(matches_selector(&tree, 4, &sel));
    }

    #[test]
    fn match_selector_list_any() {
        let (tree, _) = make_tree_with_root("Label");

        let list = SelectorList::new(vec![
            Selector::simple(CompoundSelector::type_selector("Container")),
            Selector::simple(CompoundSelector::type_selector("Label")),
        ]);
        assert!(StyleMatcher::matches_any(&tree, 1, &list).is_some());
    }

    #[test]
    fn match_selector_list_none() {
        let (tree, _) = make_tree_with_root("Label");

        let list = SelectorList::new(vec![
            Selector::simple(CompoundSelector::type_selector("Container")),
            Selector::simple(CompoundSelector::type_selector("Button")),
        ]);
        assert!(StyleMatcher::matches_any(&tree, 1, &list).is_none());
    }

    // -----------------------------------------------------------------------
    // Task 5: Rule matching engine
    // -----------------------------------------------------------------------

    fn make_rule(selector_css: &str, property: PropertyName, value: CssValue) -> Rule {
        let selectors = match SelectorList::parse(selector_css) {
            Ok(s) => s,
            Err(_) => unreachable!(),
        };
        Rule::new(selectors, vec![Declaration::new(property, value)])
    }

    #[test]
    fn match_no_rules() {
        let sheet = Stylesheet::new();
        let matcher = StyleMatcher::new(&sheet);
        let (tree, _) = make_tree_with_root("Label");
        let matched = matcher.match_widget(&tree, 1);
        assert!(matched.is_empty());
    }

    #[test]
    fn match_single_rule() {
        let mut sheet = Stylesheet::new();
        sheet.add_rule(make_rule(
            "Label",
            PropertyName::Color,
            CssValue::Keyword("red".into()),
        ));
        let matcher = StyleMatcher::new(&sheet);
        let (tree, _) = make_tree_with_root("Label");
        let matched = matcher.match_widget(&tree, 1);
        assert_eq!(matched.len(), 1);
    }

    #[test]
    fn match_multiple_rules() {
        let mut sheet = Stylesheet::new();
        sheet.add_rule(make_rule(
            "Label",
            PropertyName::Color,
            CssValue::Keyword("red".into()),
        ));
        sheet.add_rule(make_rule(
            "*",
            PropertyName::Background,
            CssValue::Keyword("blue".into()),
        ));
        let matcher = StyleMatcher::new(&sheet);
        let (tree, _) = make_tree_with_root("Label");
        let matched = matcher.match_widget(&tree, 1);
        assert_eq!(matched.len(), 2);
    }

    #[test]
    fn match_source_order() {
        let mut sheet = Stylesheet::new();
        sheet.add_rule(make_rule(
            "Label",
            PropertyName::Color,
            CssValue::Keyword("red".into()),
        ));
        sheet.add_rule(make_rule(
            "Label",
            PropertyName::Color,
            CssValue::Keyword("blue".into()),
        ));
        let matcher = StyleMatcher::new(&sheet);
        let (tree, _) = make_tree_with_root("Label");
        let matched = matcher.match_widget(&tree, 1);
        assert_eq!(matched.len(), 2);
        assert_eq!(matched[0].source_order, 0);
        assert_eq!(matched[1].source_order, 1);
    }

    #[test]
    fn match_specificity_attached() {
        let mut sheet = Stylesheet::new();
        // .error has specificity (0,1,0)
        sheet.add_rule(make_rule(
            ".error",
            PropertyName::Color,
            CssValue::Keyword("red".into()),
        ));
        let matcher = StyleMatcher::new(&sheet);
        let mut tree = WidgetTree::new();
        tree.add_node(WidgetNode::new(1, "Label").with_class("error"));
        let matched = matcher.match_widget(&tree, 1);
        assert_eq!(matched.len(), 1);
        assert_eq!(matched[0].specificity, (0, 1, 0));
    }

    #[test]
    fn match_no_match() {
        let mut sheet = Stylesheet::new();
        sheet.add_rule(make_rule(
            "Container",
            PropertyName::Color,
            CssValue::Keyword("red".into()),
        ));
        let matcher = StyleMatcher::new(&sheet);
        let (tree, _) = make_tree_with_root("Label");
        let matched = matcher.match_widget(&tree, 1);
        assert!(matched.is_empty());
    }

    #[test]
    fn match_selector_list_rule() {
        let mut sheet = Stylesheet::new();
        sheet.add_rule(make_rule(
            "Label, Container",
            PropertyName::Color,
            CssValue::Keyword("red".into()),
        ));
        let matcher = StyleMatcher::new(&sheet);
        let (tree, _) = make_tree_with_root("Label");
        let matched = matcher.match_widget(&tree, 1);
        assert_eq!(matched.len(), 1);
    }

    #[test]
    fn match_real_stylesheet() {
        let mut sheet = Stylesheet::new();
        sheet.add_rule(make_rule(
            "Label",
            PropertyName::Color,
            CssValue::Keyword("white".into()),
        ));
        sheet.add_rule(make_rule(
            ".error",
            PropertyName::Color,
            CssValue::Keyword("red".into()),
        ));
        sheet.add_rule(make_rule(
            "#main",
            PropertyName::Color,
            CssValue::Keyword("blue".into()),
        ));
        sheet.add_rule(make_rule(
            "Container",
            PropertyName::Background,
            CssValue::Keyword("black".into()),
        ));

        let matcher = StyleMatcher::new(&sheet);
        let mut tree = WidgetTree::new();
        tree.add_node(
            WidgetNode::new(1, "Label")
                .with_class("error")
                .with_id("main"),
        );

        let matched = matcher.match_widget(&tree, 1);
        // Should match Label, .error, and #main (not Container).
        assert_eq!(matched.len(), 3);
    }
}
