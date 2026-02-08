//! Match result caching.
//!
//! Caches matched rules per widget to avoid re-matching on every render.
//! Supports fine-grained invalidation by widget, subtree, or the entire
//! cache (e.g., when the stylesheet changes).

use std::collections::{HashMap, HashSet};

use crate::focus::WidgetId;
use crate::tcss::matcher::MatchedRule;
use crate::tcss::tree::WidgetTree;

/// Cache for matched rules per widget.
///
/// Entries are stored by [`WidgetId`]. A dirty set tracks which entries
/// need re-matching. Dirty entries return `None` from [`get`](Self::get).
pub struct MatchCache {
    entries: HashMap<WidgetId, Vec<MatchedRule>>,
    dirty: HashSet<WidgetId>,
}

impl MatchCache {
    /// Create a new empty cache.
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            dirty: HashSet::new(),
        }
    }

    /// Get cached match results for a widget.
    ///
    /// Returns `None` if the widget is not cached or is dirty.
    pub fn get(&self, id: WidgetId) -> Option<&Vec<MatchedRule>> {
        if self.dirty.contains(&id) {
            return None;
        }
        self.entries.get(&id)
    }

    /// Insert match results for a widget, clearing its dirty flag.
    pub fn insert(&mut self, id: WidgetId, matches: Vec<MatchedRule>) {
        self.dirty.remove(&id);
        self.entries.insert(id, matches);
    }

    /// Mark a single widget as dirty (needs re-matching).
    pub fn invalidate(&mut self, id: WidgetId) {
        self.dirty.insert(id);
    }

    /// Mark all cached widgets as dirty.
    ///
    /// Use this when the stylesheet changes and all matches must be
    /// recomputed.
    pub fn invalidate_all(&mut self) {
        for &id in self.entries.keys() {
            self.dirty.insert(id);
        }
    }

    /// Mark a widget and all its descendants as dirty.
    pub fn invalidate_subtree(&mut self, tree: &WidgetTree, id: WidgetId) {
        self.dirty.insert(id);
        for &child_id in tree.children(id) {
            self.invalidate_subtree(tree, child_id);
        }
    }

    /// Check if a widget is dirty (needs re-matching).
    pub fn is_dirty(&self, id: WidgetId) -> bool {
        self.dirty.contains(&id)
    }

    /// Return the number of cached entries (including dirty ones).
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Return whether the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Remove all entries and dirty flags.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.dirty.clear();
    }
}

impl Default for MatchCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tcss::property::{Declaration, PropertyName};
    use crate::tcss::tree::WidgetNode;
    use crate::tcss::value::CssValue;

    fn sample_matches() -> Vec<MatchedRule> {
        vec![MatchedRule {
            specificity: (0, 0, 1),
            source_order: 0,
            declarations: vec![Declaration::new(
                PropertyName::Color,
                CssValue::Keyword("red".into()),
            )],
        }]
    }

    #[test]
    fn empty_cache() {
        let cache = MatchCache::new();
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn insert_and_get() {
        let mut cache = MatchCache::new();
        cache.insert(1, sample_matches());
        assert!(!cache.is_empty());
        assert_eq!(cache.len(), 1);
        assert!(cache.get(1).is_some());
        let matches = match cache.get(1) {
            Some(m) => m,
            None => unreachable!(),
        };
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn get_missing() {
        let cache = MatchCache::new();
        assert!(cache.get(42).is_none());
    }

    #[test]
    fn invalidate_single() {
        let mut cache = MatchCache::new();
        cache.insert(1, sample_matches());
        assert!(cache.get(1).is_some());

        cache.invalidate(1);
        assert!(cache.get(1).is_none());
        assert!(cache.is_dirty(1));
        // Entry still exists, just dirty.
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn invalidate_all() {
        let mut cache = MatchCache::new();
        cache.insert(1, sample_matches());
        cache.insert(2, sample_matches());
        cache.insert(3, sample_matches());

        cache.invalidate_all();
        assert!(cache.get(1).is_none());
        assert!(cache.get(2).is_none());
        assert!(cache.get(3).is_none());
        assert!(cache.is_dirty(1));
        assert!(cache.is_dirty(2));
        assert!(cache.is_dirty(3));
    }

    #[test]
    fn invalidate_subtree() {
        let mut tree = WidgetTree::new();
        tree.add_node(WidgetNode::new(1, "Root"));
        let mut child = WidgetNode::new(2, "Child");
        child.parent = Some(1);
        tree.add_node(child);
        let mut grandchild = WidgetNode::new(3, "Grandchild");
        grandchild.parent = Some(2);
        tree.add_node(grandchild);

        let mut cache = MatchCache::new();
        cache.insert(1, sample_matches());
        cache.insert(2, sample_matches());
        cache.insert(3, sample_matches());

        // Invalidate node 1 and descendants.
        cache.invalidate_subtree(&tree, 1);
        assert!(cache.is_dirty(1));
        assert!(cache.is_dirty(2));
        assert!(cache.is_dirty(3));
    }

    #[test]
    fn is_dirty_check() {
        let mut cache = MatchCache::new();
        assert!(!cache.is_dirty(1));

        cache.insert(1, sample_matches());
        assert!(!cache.is_dirty(1));

        cache.invalidate(1);
        assert!(cache.is_dirty(1));

        // Re-insert clears dirty.
        cache.insert(1, sample_matches());
        assert!(!cache.is_dirty(1));
    }

    #[test]
    fn clear_removes_all() {
        let mut cache = MatchCache::new();
        cache.insert(1, sample_matches());
        cache.insert(2, sample_matches());
        cache.invalidate(1);

        cache.clear();
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
        assert!(!cache.is_dirty(1));
    }
}
