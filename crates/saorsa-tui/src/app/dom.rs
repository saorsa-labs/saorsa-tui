//! Retained widget tree ("DOM") with CSS metadata.

use std::collections::HashMap;

use crate::focus::{FocusManager, WidgetId};
use crate::tcss::{WidgetNode, WidgetTree};

use super::node_widget::NodeWidget;

/// Node identifier.
pub type NodeId = WidgetId;

/// A reference to a DOM node.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NodeRef(pub NodeId);

/// A node stored in the DOM.
pub struct DomNode {
    pub(crate) widget: Box<dyn NodeWidget>,
    pub(crate) focusable: bool,
}

/// Retained widget tree with CSS metadata.
///
/// Internally this keeps:
/// - a `WidgetTree` for selector matching and pseudo-class state
/// - widget objects for rendering and event handling
/// - a `FocusManager` for focus traversal
pub struct Dom {
    next_id: NodeId,
    root: Option<NodeId>,
    nodes: HashMap<NodeId, DomNode>,
    tree: WidgetTree,
    focus: FocusManager,
}

impl Dom {
    /// Create an empty DOM.
    pub fn new() -> Self {
        Self {
            next_id: 1,
            root: None,
            nodes: HashMap::new(),
            tree: WidgetTree::new(),
            focus: FocusManager::new(),
        }
    }

    /// Returns the root node id, if set.
    pub fn root(&self) -> Option<NodeId> {
        self.root
    }

    /// Access the CSS metadata tree.
    pub fn widget_tree(&self) -> &WidgetTree {
        &self.tree
    }

    /// Access the CSS metadata tree mutably.
    pub fn widget_tree_mut(&mut self) -> &mut WidgetTree {
        &mut self.tree
    }

    /// Access the focus manager.
    pub fn focus(&self) -> &FocusManager {
        &self.focus
    }

    /// Access the focus manager mutably.
    pub fn focus_mut(&mut self) -> &mut FocusManager {
        &mut self.focus
    }

    /// Create a node (not attached).
    pub fn create(&mut self, type_name: impl Into<String>, widget: Box<dyn NodeWidget>) -> NodeRef {
        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);

        let node = WidgetNode::new(id, type_name);
        self.tree.add_node(node);
        self.nodes.insert(
            id,
            DomNode {
                widget,
                focusable: false,
            },
        );
        NodeRef(id)
    }

    /// Mark a node as focusable (adds to tab order).
    pub fn set_focusable(&mut self, node: NodeRef, focusable: bool) {
        if let Some(n) = self.nodes.get_mut(&node.0) {
            n.focusable = focusable;
            if focusable {
                self.focus.register(node.0);
            } else {
                self.focus.unregister(node.0);
            }
        }
    }

    /// Set the root node.
    pub fn set_root(&mut self, node: NodeRef) {
        self.root = Some(node.0);
        self.tree.set_root(node.0);
    }

    /// Append `child` as the last child of `parent`.
    ///
    /// The parent must already exist.
    pub fn append_child(&mut self, parent: NodeRef, child: NodeRef) {
        if let Some(child_node) = self.tree.get_mut(child.0) {
            child_node.parent = Some(parent.0);
        }
        if let Some(parent_node) = self.tree.get_mut(parent.0)
            && !parent_node.children.contains(&child.0)
        {
            parent_node.children.push(child.0);
        }
    }

    /// Detach a node from its parent, if it has one.
    pub fn detach(&mut self, node: NodeRef) {
        let parent = self.tree.get(node.0).and_then(|n| n.parent);
        if let Some(pid) = parent
            && let Some(p) = self.tree.get_mut(pid)
        {
            p.children.retain(|&c| c != node.0);
        }
        if let Some(n) = self.tree.get_mut(node.0) {
            n.parent = None;
        }
    }

    /// Return the node ids in this node's subtree (including itself), in post-order.
    pub fn subtree_post_order(&self, node: NodeRef) -> Vec<NodeId> {
        let mut out = Vec::new();
        if self.tree.get(node.0).is_none() {
            return out;
        }
        subtree_post_order(self.widget_tree(), node.0, &mut out);
        out
    }

    /// Return the node ids in this node's subtree (including itself), in pre-order.
    pub fn subtree_pre_order(&self, node: NodeRef) -> Vec<NodeId> {
        let mut out = Vec::new();
        if self.tree.get(node.0).is_none() {
            return out;
        }
        subtree_pre_order(self.widget_tree(), node.0, &mut out);
        out
    }

    /// Remove a node and all descendants from the DOM.
    ///
    /// Returns the removed node ids in post-order (leaves first).
    pub fn remove_subtree(&mut self, node: NodeRef) -> Vec<NodeId> {
        let ids = self.subtree_post_order(node);
        if ids.is_empty() {
            return ids;
        }

        // Unlink root if removed.
        if self.root == Some(node.0) {
            self.root = None;
        } else {
            self.detach(node);
        }

        for id in &ids {
            // Unregister focus if it was focusable.
            if let Some(n) = self.nodes.get(id)
                && n.focusable
            {
                self.focus.unregister(*id);
            }
            self.nodes.remove(id);
            self.tree.remove_node(*id);
        }

        ids
    }

    /// Set the CSS id of a node.
    pub fn set_css_id(&mut self, node: NodeRef, css_id: impl Into<String>) {
        if let Some(n) = self.tree.get_mut(node.0) {
            n.css_id = Some(css_id.into());
        }
    }

    /// Add a CSS class to a node.
    pub fn add_class(&mut self, node: NodeRef, class: impl Into<String>) {
        if let Some(n) = self.tree.get_mut(node.0) {
            n.classes.push(class.into());
        }
    }

    /// Mutable access to a widget by node id.
    pub fn widget_mut(&mut self, node: NodeRef) -> Option<&mut (dyn NodeWidget + '_)> {
        match self.nodes.get_mut(&node.0) {
            Some(n) => Some(&mut *n.widget),
            None => None,
        }
    }

    /// Immutable access to a widget by node id.
    pub fn widget(&self, node: NodeRef) -> Option<&(dyn NodeWidget + '_)> {
        match self.nodes.get(&node.0) {
            Some(n) => Some(&*n.widget),
            None => None,
        }
    }

    /// Downcast mutable access to a concrete widget type.
    pub fn downcast_widget_mut<T: 'static>(&mut self, node: NodeRef) -> Option<&mut T> {
        let w = self.widget_mut(node)?;
        w.as_any_mut().downcast_mut::<T>()
    }

    /// Returns all node ids (unordered).
    pub fn node_ids(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.nodes.keys().copied()
    }

    /// Returns whether a node exists.
    pub fn contains(&self, id: NodeId) -> bool {
        self.nodes.contains_key(&id)
    }

    pub(crate) fn node_mut(&mut self, id: NodeId) -> Option<&mut DomNode> {
        self.nodes.get_mut(&id)
    }
}

impl Default for Dom {
    fn default() -> Self {
        Self::new()
    }
}

fn subtree_post_order(tree: &WidgetTree, id: NodeId, out: &mut Vec<NodeId>) {
    for &child in tree.children(id) {
        subtree_post_order(tree, child, out);
    }
    out.push(id);
}

fn subtree_pre_order(tree: &WidgetTree, id: NodeId, out: &mut Vec<NodeId>) {
    out.push(id);
    for &child in tree.children(id) {
        subtree_pre_order(tree, child, out);
    }
}
