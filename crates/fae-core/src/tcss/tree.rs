//! Widget tree with CSS metadata for selector matching.
//!
//! Provides a tree structure that stores widgets with type names,
//! CSS classes, IDs, and pseudo-class state — everything needed
//! to match CSS selectors against the widget hierarchy.

use std::collections::HashMap;

use crate::focus::WidgetId;

/// Pseudo-class state flags for a widget.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WidgetState {
    /// Whether this widget currently has keyboard focus.
    pub focused: bool,
    /// Whether the mouse is hovering over this widget.
    pub hovered: bool,
    /// Whether this widget is disabled.
    pub disabled: bool,
    /// Whether this widget is being activated (e.g., pressed).
    pub active: bool,
}

/// A node in the widget tree carrying CSS metadata.
#[derive(Clone, Debug)]
pub struct WidgetNode {
    /// Unique identifier for this widget.
    pub id: WidgetId,
    /// Widget type name used for CSS type selectors (e.g., "Label").
    pub type_name: String,
    /// CSS class names applied to this widget.
    pub classes: Vec<String>,
    /// Optional CSS ID (should be unique within the tree).
    pub css_id: Option<String>,
    /// Current pseudo-class state.
    pub state: WidgetState,
    /// Parent node ID, if any.
    pub parent: Option<WidgetId>,
    /// Ordered list of child node IDs.
    pub children: Vec<WidgetId>,
}

impl WidgetNode {
    /// Create a new widget node with the given ID and type name.
    ///
    /// All other fields default to empty/none/false.
    pub fn new(id: WidgetId, type_name: impl Into<String>) -> Self {
        Self {
            id,
            type_name: type_name.into(),
            classes: Vec::new(),
            css_id: None,
            state: WidgetState::default(),
            parent: None,
            children: Vec::new(),
        }
    }

    /// Builder: add a CSS class.
    pub fn with_class(mut self, class: impl Into<String>) -> Self {
        self.classes.push(class.into());
        self
    }

    /// Builder: set the CSS ID.
    pub fn with_id(mut self, css_id: impl Into<String>) -> Self {
        self.css_id = Some(css_id.into());
        self
    }

    /// Check whether this node has a given CSS class.
    pub fn has_class(&self, name: &str) -> bool {
        self.classes.iter().any(|c| c == name)
    }
}

/// A tree of widget nodes with parent/child relationships.
///
/// Nodes are stored in a `HashMap` keyed by [`WidgetId`]. The tree
/// tracks a single root and provides traversal helpers needed for
/// CSS selector matching (ancestors, sibling queries, etc.).
pub struct WidgetTree {
    nodes: HashMap<WidgetId, WidgetNode>,
    root: Option<WidgetId>,
}

impl WidgetTree {
    /// Create an empty widget tree.
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            root: None,
        }
    }

    /// Add a node to the tree.
    ///
    /// If the node has a parent, its ID is appended to the parent's
    /// children list. If the tree is empty and the node has no parent,
    /// it becomes the root.
    pub fn add_node(&mut self, node: WidgetNode) {
        let id = node.id;
        let parent_id = node.parent;
        self.nodes.insert(id, node);

        // Link to parent's children list.
        if let Some(pid) = parent_id {
            if let Some(parent) = self.nodes.get_mut(&pid) {
                parent.children.push(id);
            }
        } else if self.root.is_none() {
            self.root = Some(id);
        }
    }

    /// Remove a node from the tree.
    ///
    /// Unlinks the node from its parent's children list.
    /// Does **not** recursively remove children.
    pub fn remove_node(&mut self, id: WidgetId) {
        if let Some(node) = self.nodes.remove(&id) {
            // Unlink from parent.
            if let Some(pid) = node.parent
                && let Some(parent) = self.nodes.get_mut(&pid)
            {
                parent.children.retain(|&c| c != id);
            }
            // Clear root if it was the root.
            if self.root == Some(id) {
                self.root = None;
            }
        }
    }

    /// Look up a node by ID.
    pub fn get(&self, id: WidgetId) -> Option<&WidgetNode> {
        self.nodes.get(&id)
    }

    /// Look up a node by ID (mutable).
    pub fn get_mut(&mut self, id: WidgetId) -> Option<&mut WidgetNode> {
        self.nodes.get_mut(&id)
    }

    /// Return the root node ID, if the tree is non-empty.
    pub fn root(&self) -> Option<WidgetId> {
        self.root
    }

    /// Return the parent node of the given widget.
    pub fn parent(&self, id: WidgetId) -> Option<&WidgetNode> {
        self.nodes
            .get(&id)
            .and_then(|n| n.parent)
            .and_then(|pid| self.nodes.get(&pid))
    }

    /// Return the children IDs of the given widget.
    ///
    /// Returns an empty slice if the node is not found.
    pub fn children(&self, id: WidgetId) -> &[WidgetId] {
        match self.nodes.get(&id) {
            Some(node) => &node.children,
            None => &[],
        }
    }

    /// Return the ancestor chain from the immediate parent up to the root.
    ///
    /// Does **not** include `id` itself. Nearest parent is first.
    pub fn ancestors(&self, id: WidgetId) -> Vec<WidgetId> {
        let mut result = Vec::new();
        let mut current = self.nodes.get(&id).and_then(|n| n.parent);
        while let Some(pid) = current {
            result.push(pid);
            current = self.nodes.get(&pid).and_then(|n| n.parent);
        }
        result
    }

    /// Is this node the first child of its parent?
    pub fn is_first_child(&self, id: WidgetId) -> bool {
        let parent = match self.parent(id) {
            Some(p) => p,
            None => return false,
        };
        parent.children.first() == Some(&id)
    }

    /// Is this node the last child of its parent?
    pub fn is_last_child(&self, id: WidgetId) -> bool {
        let parent = match self.parent(id) {
            Some(p) => p,
            None => return false,
        };
        parent.children.last() == Some(&id)
    }

    /// Return the 0-based index of this node among its parent's children.
    pub fn child_index(&self, id: WidgetId) -> Option<usize> {
        let parent = self.parent(id)?;
        parent.children.iter().position(|&c| c == id)
    }

    /// Number of nodes in the tree.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Is the tree empty?
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

impl Default for WidgetTree {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_tree() {
        let tree = WidgetTree::new();
        assert!(tree.is_empty());
        assert_eq!(tree.len(), 0);
        assert!(tree.root().is_none());
    }

    #[test]
    fn add_root_node() {
        let mut tree = WidgetTree::new();
        tree.add_node(WidgetNode::new(1, "Container"));
        assert_eq!(tree.len(), 1);
        assert_eq!(tree.root(), Some(1));
        assert!(tree.get(1).is_some());
    }

    #[test]
    fn add_child_node() {
        let mut tree = WidgetTree::new();
        tree.add_node(WidgetNode::new(1, "Container"));

        let mut child = WidgetNode::new(2, "Label");
        child.parent = Some(1);
        tree.add_node(child);

        assert_eq!(tree.len(), 2);
        // Parent's children list should contain the child.
        assert_eq!(tree.children(1), &[2]);
        // Child's parent lookup should work.
        let parent = tree.parent(2);
        assert!(parent.is_some());
        let parent = match parent {
            Some(p) => p,
            None => unreachable!(),
        };
        assert_eq!(parent.id, 1);
    }

    #[test]
    fn remove_node() {
        let mut tree = WidgetTree::new();
        tree.add_node(WidgetNode::new(1, "Container"));

        let mut child = WidgetNode::new(2, "Label");
        child.parent = Some(1);
        tree.add_node(child);

        tree.remove_node(2);
        assert_eq!(tree.len(), 1);
        assert!(tree.get(2).is_none());
        // Parent's children list should no longer contain 2.
        assert!(tree.children(1).is_empty());
    }

    #[test]
    fn ancestors() {
        let mut tree = WidgetTree::new();
        tree.add_node(WidgetNode::new(1, "Root"));

        let mut mid = WidgetNode::new(2, "Middle");
        mid.parent = Some(1);
        tree.add_node(mid);

        let mut leaf = WidgetNode::new(3, "Leaf");
        leaf.parent = Some(2);
        tree.add_node(leaf);

        let anc = tree.ancestors(3);
        assert_eq!(anc, vec![2, 1]);
    }

    #[test]
    fn is_first_last_child() {
        let mut tree = WidgetTree::new();
        tree.add_node(WidgetNode::new(1, "Root"));

        for id in 2..=4 {
            let mut child = WidgetNode::new(id, "Child");
            child.parent = Some(1);
            tree.add_node(child);
        }

        assert!(tree.is_first_child(2));
        assert!(!tree.is_first_child(3));
        assert!(!tree.is_first_child(4));

        assert!(!tree.is_last_child(2));
        assert!(!tree.is_last_child(3));
        assert!(tree.is_last_child(4));
    }

    #[test]
    fn child_index() {
        let mut tree = WidgetTree::new();
        tree.add_node(WidgetNode::new(1, "Root"));

        for id in 2..=4 {
            let mut child = WidgetNode::new(id, "Child");
            child.parent = Some(1);
            tree.add_node(child);
        }

        assert_eq!(tree.child_index(2), Some(0));
        assert_eq!(tree.child_index(3), Some(1));
        assert_eq!(tree.child_index(4), Some(2));
        // Root has no parent, so no index.
        assert_eq!(tree.child_index(1), None);
    }

    #[test]
    fn widget_node_builder() {
        let node = WidgetNode::new(1, "Label")
            .with_class("error")
            .with_class("bold")
            .with_id("main-title");

        assert_eq!(node.type_name, "Label");
        assert!(node.has_class("error"));
        assert!(node.has_class("bold"));
        assert!(!node.has_class("hidden"));
        assert_eq!(node.css_id.as_deref(), Some("main-title"));
    }

    #[test]
    fn widget_state_default() {
        let state = WidgetState::default();
        assert!(!state.focused);
        assert!(!state.hovered);
        assert!(!state.disabled);
        assert!(!state.active);
    }
}
