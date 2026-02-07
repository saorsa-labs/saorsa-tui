//! Hierarchical tree widget with expandable/collapsible nodes.
//!
//! Displays a tree of items as a flat list with indentation.
//! Supports keyboard navigation, expand/collapse, and lazy loading.

use crate::buffer::ScreenBuffer;
use crate::cell::Cell;
use crate::event::{Event, KeyCode, KeyEvent};
use crate::geometry::Rect;
use crate::segment::Segment;
use crate::style::Style;
use crate::text::truncate_to_display_width;
use unicode_width::UnicodeWidthStr;

use super::{BorderStyle, EventResult, InteractiveWidget, Widget};

/// A node in the tree hierarchy.
#[derive(Clone, Debug)]
pub struct TreeNode<T> {
    /// The data for this node.
    pub data: T,
    /// Children of this node.
    pub children: Vec<TreeNode<T>>,
    /// Whether this node is expanded (children visible).
    pub expanded: bool,
    /// Whether this node is a leaf (cannot have children).
    pub is_leaf: bool,
}

impl<T> TreeNode<T> {
    /// Create a new tree node with the given data.
    pub fn new(data: T) -> Self {
        Self {
            data,
            children: Vec::new(),
            expanded: false,
            is_leaf: true,
        }
    }

    /// Create a new branch node (not a leaf).
    pub fn branch(data: T) -> Self {
        Self {
            data,
            children: Vec::new(),
            expanded: false,
            is_leaf: false,
        }
    }

    /// Add a child node.
    pub fn with_child(mut self, child: TreeNode<T>) -> Self {
        self.is_leaf = false;
        self.children.push(child);
        self
    }

    /// Add multiple children.
    pub fn with_children(mut self, children: Vec<TreeNode<T>>) -> Self {
        if !children.is_empty() {
            self.is_leaf = false;
        }
        self.children = children;
        self
    }
}

/// A flattened visible node entry.
struct VisibleNode {
    /// Depth in the tree (0 = root).
    depth: usize,
    /// Path of indices to reach this node from roots.
    path: Vec<usize>,
    /// Whether expanded.
    expanded: bool,
    /// Whether leaf.
    is_leaf: bool,
}

/// Type alias for the node render function.
type NodeRenderFn<T> = Box<dyn Fn(&T, usize, bool, bool) -> Vec<Segment>>;

/// Type alias for the lazy load function.
type LazyLoadFn<T> = Option<Box<dyn Fn(&T) -> Vec<TreeNode<T>>>>;

/// A hierarchical tree widget with expandable/collapsible nodes.
///
/// Displays a forest of [`TreeNode`]s as an indented list. Supports
/// keyboard navigation, expand/collapse, and optional lazy loading.
pub struct Tree<T> {
    /// Root nodes (a forest).
    roots: Vec<TreeNode<T>>,
    /// Selected visible node index.
    selected: usize,
    /// Scroll offset (first visible line).
    scroll_offset: usize,
    /// Function to render a node as Segments.
    render_fn: NodeRenderFn<T>,
    /// Style for unselected nodes.
    node_style: Style,
    /// Style for the selected node.
    selected_style: Style,
    /// Border style.
    border: BorderStyle,
    /// Lazy load callback.
    lazy_load_fn: LazyLoadFn<T>,
}

impl<T> Tree<T> {
    /// Create a new tree with the given root nodes.
    pub fn new(roots: Vec<TreeNode<T>>) -> Self {
        Self {
            roots,
            selected: 0,
            scroll_offset: 0,
            render_fn: Box::new(|_, _, _, _| vec![Segment::new("???")]),
            node_style: Style::default(),
            selected_style: Style::default().reverse(true),
            border: BorderStyle::None,
            lazy_load_fn: None,
        }
    }

    /// Set a custom render function for nodes.
    ///
    /// Parameters: (data, depth, expanded, is_leaf) -> Segments
    #[must_use]
    pub fn with_render_fn<F>(mut self, f: F) -> Self
    where
        F: Fn(&T, usize, bool, bool) -> Vec<Segment> + 'static,
    {
        self.render_fn = Box::new(f);
        self
    }

    /// Set the style for unselected nodes.
    #[must_use]
    pub fn with_node_style(mut self, style: Style) -> Self {
        self.node_style = style;
        self
    }

    /// Set the style for the selected node.
    #[must_use]
    pub fn with_selected_style(mut self, style: Style) -> Self {
        self.selected_style = style;
        self
    }

    /// Set the border style.
    #[must_use]
    pub fn with_border(mut self, border: BorderStyle) -> Self {
        self.border = border;
        self
    }

    /// Set a lazy load function for deferred child loading.
    #[must_use]
    pub fn with_lazy_load<F>(mut self, f: F) -> Self
    where
        F: Fn(&T) -> Vec<TreeNode<T>> + 'static,
    {
        self.lazy_load_fn = Some(Box::new(f));
        self
    }

    /// Get a reference to the root nodes.
    pub fn roots(&self) -> &[TreeNode<T>] {
        &self.roots
    }

    /// Get the selected visible node index.
    pub fn selected(&self) -> usize {
        self.selected
    }

    /// Get the scroll offset.
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    /// Build the list of visible nodes by pre-order traversal.
    fn build_visible(&self) -> Vec<VisibleNode> {
        let mut result = Vec::new();
        for (idx, root) in self.roots.iter().enumerate() {
            self.collect_visible(root, 0, vec![idx], &mut result);
        }
        result
    }

    /// Recursively collect visible nodes.
    fn collect_visible(
        &self,
        node: &TreeNode<T>,
        depth: usize,
        path: Vec<usize>,
        result: &mut Vec<VisibleNode>,
    ) {
        result.push(VisibleNode {
            depth,
            path: path.clone(),
            expanded: node.expanded,
            is_leaf: node.is_leaf,
        });

        if node.expanded {
            for (child_idx, child) in node.children.iter().enumerate() {
                let mut child_path = path.clone();
                child_path.push(child_idx);
                self.collect_visible(child, depth + 1, child_path, result);
            }
        }
    }

    /// Get a mutable reference to a node by path.
    fn node_at_path_mut(&mut self, path: &[usize]) -> Option<&mut TreeNode<T>> {
        if path.is_empty() {
            return None;
        }
        let mut current = self.roots.get_mut(path[0])?;
        for &idx in &path[1..] {
            current = current.children.get_mut(idx)?;
        }
        Some(current)
    }

    /// Get an immutable reference to a node by path.
    fn node_at_path(&self, path: &[usize]) -> Option<&TreeNode<T>> {
        if path.is_empty() {
            return None;
        }
        let mut current = self.roots.get(path[0])?;
        for &idx in &path[1..] {
            current = current.children.get(idx)?;
        }
        Some(current)
    }

    /// Toggle expand/collapse at the selected node.
    pub fn toggle_selected(&mut self) {
        let visible = self.build_visible();
        if let Some(vnode) = visible.get(self.selected) {
            let path = vnode.path.clone();
            if let Some(node) = self.node_at_path_mut(&path)
                && !node.is_leaf
            {
                node.expanded = !node.expanded;
            }
        }
    }

    /// Expand the selected node (load children lazily if needed).
    pub fn expand_selected(&mut self) {
        let visible = self.build_visible();
        if let Some(vnode) = visible.get(self.selected) {
            let path = vnode.path.clone();
            let is_leaf = vnode.is_leaf;

            if is_leaf {
                return;
            }

            // Lazy load if needed
            if let Some(ref lazy_fn) = self.lazy_load_fn
                && let Some(node) = self.node_at_path(&path)
                && node.children.is_empty()
                && !node.is_leaf
            {
                let new_children = lazy_fn(&node.data);
                if let Some(node_mut) = self.node_at_path_mut(&path) {
                    node_mut.children = new_children;
                }
            }

            if let Some(node) = self.node_at_path_mut(&path) {
                node.expanded = true;
            }
        }
    }

    /// Collapse the selected node.
    pub fn collapse_selected(&mut self) {
        let visible = self.build_visible();
        if let Some(vnode) = visible.get(self.selected) {
            let path = vnode.path.clone();

            if let Some(node) = self.node_at_path_mut(&path)
                && node.expanded
            {
                node.expanded = false;
                return;
            }

            // If already collapsed, move to parent
            if path.len() > 1 {
                let parent_path = &path[..path.len() - 1];
                // Find parent in visible list
                for (idx, v) in visible.iter().enumerate() {
                    if v.path == parent_path {
                        self.selected = idx;
                        break;
                    }
                }
            }
        }
    }

    /// Get a reference to the data of the selected visible node.
    pub fn selected_node(&self) -> Option<&TreeNode<T>> {
        let visible = self.build_visible();
        visible
            .get(self.selected)
            .and_then(|v| self.node_at_path(&v.path))
    }

    /// Get the total number of visible nodes.
    pub fn visible_count(&self) -> usize {
        self.build_visible().len()
    }

    /// Ensure the selected item is visible by adjusting scroll_offset.
    fn ensure_selected_visible(&mut self, visible_height: usize) {
        if visible_height == 0 {
            return;
        }
        if self.selected < self.scroll_offset {
            self.scroll_offset = self.selected;
        }
        if self.selected >= self.scroll_offset + visible_height {
            self.scroll_offset = self
                .selected
                .saturating_sub(visible_height.saturating_sub(1));
        }
    }
}

impl<T> Widget for Tree<T> {
    fn render(&self, area: Rect, buf: &mut ScreenBuffer) {
        if area.size.width == 0 || area.size.height == 0 {
            return;
        }

        super::border::render_border(area, self.border, self.node_style.clone(), buf);

        let inner = super::border::inner_area(area, self.border);
        if inner.size.width == 0 || inner.size.height == 0 {
            return;
        }

        let height = inner.size.height as usize;
        let width = inner.size.width as usize;
        let visible = self.build_visible();
        let count = visible.len();

        let max_offset = count.saturating_sub(height.max(1));
        let scroll = self.scroll_offset.min(max_offset);
        let visible_end = (scroll + height).min(count);

        for (row, vis_idx) in (scroll..visible_end).enumerate() {
            let y = inner.position.y + row as u16;
            if let Some(vnode) = visible.get(vis_idx) {
                let is_selected = vis_idx == self.selected;
                let style = if is_selected {
                    &self.selected_style
                } else {
                    &self.node_style
                };

                // Fill row if selected
                if is_selected {
                    for col in 0..inner.size.width {
                        buf.set(inner.position.x + col, y, Cell::new(" ", style.clone()));
                    }
                }

                // Indentation (2 spaces per level)
                let indent = vnode.depth * 2;

                // Expand indicator
                let indicator = if vnode.is_leaf {
                    " "
                } else if vnode.expanded {
                    "\u{25bc}" // ▼
                } else {
                    "\u{25b6}" // ▶
                };

                // Render indent + indicator + content
                let mut col: u16 = 0;

                // Indent
                for _ in 0..indent {
                    if col as usize >= width {
                        break;
                    }
                    buf.set(inner.position.x + col, y, Cell::new(" ", style.clone()));
                    col += 1;
                }

                // Indicator
                if (col as usize) < width {
                    buf.set(
                        inner.position.x + col,
                        y,
                        Cell::new(indicator, style.clone()),
                    );
                    col += 1;
                }

                // Space after indicator
                if (col as usize) < width {
                    buf.set(inner.position.x + col, y, Cell::new(" ", style.clone()));
                    col += 1;
                }

                // Content from render_fn
                if let Some(node) = self.node_at_path(&vnode.path) {
                    let segments =
                        (self.render_fn)(&node.data, vnode.depth, vnode.expanded, vnode.is_leaf);
                    for segment in &segments {
                        if col as usize >= width {
                            break;
                        }
                        let remaining = width.saturating_sub(col as usize);
                        let truncated = truncate_to_display_width(&segment.text, remaining);
                        for ch in truncated.chars() {
                            let char_w =
                                UnicodeWidthStr::width(ch.encode_utf8(&mut [0; 4]) as &str);
                            if col as usize + char_w > width {
                                break;
                            }
                            buf.set(
                                inner.position.x + col,
                                y,
                                Cell::new(ch.to_string(), style.clone()),
                            );
                            col += char_w as u16;
                        }
                    }
                }
            }
        }
    }
}

impl<T> InteractiveWidget for Tree<T> {
    fn handle_event(&mut self, event: &Event) -> EventResult {
        let Event::Key(KeyEvent { code, .. }) = event else {
            return EventResult::Ignored;
        };

        let count = self.visible_count();

        match code {
            KeyCode::Up => {
                if self.selected > 0 {
                    self.selected -= 1;
                    self.ensure_selected_visible(20);
                }
                EventResult::Consumed
            }
            KeyCode::Down => {
                if count > 0 && self.selected < count.saturating_sub(1) {
                    self.selected += 1;
                    self.ensure_selected_visible(20);
                }
                EventResult::Consumed
            }
            KeyCode::Right => {
                self.expand_selected();
                EventResult::Consumed
            }
            KeyCode::Left => {
                self.collapse_selected();
                EventResult::Consumed
            }
            KeyCode::Enter => {
                self.toggle_selected();
                EventResult::Consumed
            }
            KeyCode::PageUp => {
                let page = 20;
                self.selected = self.selected.saturating_sub(page);
                self.ensure_selected_visible(20);
                EventResult::Consumed
            }
            KeyCode::PageDown => {
                let page = 20;
                if count > 0 {
                    self.selected = (self.selected + page).min(count.saturating_sub(1));
                    self.ensure_selected_visible(20);
                }
                EventResult::Consumed
            }
            KeyCode::Home => {
                self.selected = 0;
                self.scroll_offset = 0;
                EventResult::Consumed
            }
            KeyCode::End => {
                if count > 0 {
                    self.selected = count.saturating_sub(1);
                    self.ensure_selected_visible(20);
                }
                EventResult::Consumed
            }
            _ => EventResult::Ignored,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::geometry::Size;

    fn make_render_fn() -> impl Fn(&String, usize, bool, bool) -> Vec<Segment> {
        |data: &String, _depth, _expanded, _is_leaf| vec![Segment::new(data)]
    }

    fn make_test_tree() -> Tree<String> {
        let tree = TreeNode::branch("root".into())
            .with_child(TreeNode::new("leaf1".into()))
            .with_child(
                TreeNode::branch("branch1".into())
                    .with_child(TreeNode::new("child1".into()))
                    .with_child(TreeNode::new("child2".into())),
            )
            .with_child(TreeNode::new("leaf2".into()));

        Tree::new(vec![tree]).with_render_fn(make_render_fn())
    }

    #[test]
    fn create_tree_with_nodes() {
        let tree = make_test_tree();
        assert_eq!(tree.roots().len(), 1);
    }

    #[test]
    fn render_collapsed_tree_only_roots() {
        let tree = make_test_tree();
        // Only root is visible (collapsed)
        assert_eq!(tree.visible_count(), 1);

        let mut buf = ScreenBuffer::new(Size::new(30, 10));
        tree.render(Rect::new(0, 0, 30, 10), &mut buf);

        // Should show "▶ root" (▶ is the expand indicator for non-leaf)
        assert_eq!(buf.get(0, 0).map(|c| c.grapheme.as_str()), Some("\u{25b6}"));
    }

    #[test]
    fn expand_node_children_visible() {
        let mut tree = make_test_tree();
        // Expand root
        tree.toggle_selected();

        // Root + 3 children: leaf1, branch1, leaf2
        assert_eq!(tree.visible_count(), 4);
    }

    #[test]
    fn collapse_node_hides_children() {
        let mut tree = make_test_tree();
        tree.toggle_selected(); // expand
        assert_eq!(tree.visible_count(), 4);
        tree.toggle_selected(); // collapse
        assert_eq!(tree.visible_count(), 1);
    }

    #[test]
    fn navigate_visible_nodes() {
        let mut tree = make_test_tree();
        tree.toggle_selected(); // expand root

        let down = Event::Key(KeyEvent {
            code: KeyCode::Down,
            modifiers: crate::event::Modifiers::NONE,
        });
        let up = Event::Key(KeyEvent {
            code: KeyCode::Up,
            modifiers: crate::event::Modifiers::NONE,
        });

        assert_eq!(tree.selected(), 0);
        tree.handle_event(&down);
        assert_eq!(tree.selected(), 1); // leaf1
        tree.handle_event(&down);
        assert_eq!(tree.selected(), 2); // branch1
        tree.handle_event(&up);
        assert_eq!(tree.selected(), 1);
    }

    #[test]
    fn right_key_expands() {
        let mut tree = make_test_tree();
        let right = Event::Key(KeyEvent {
            code: KeyCode::Right,
            modifiers: crate::event::Modifiers::NONE,
        });

        tree.handle_event(&right); // expand root
        assert_eq!(tree.visible_count(), 4);
    }

    #[test]
    fn left_key_collapses() {
        let mut tree = make_test_tree();
        tree.toggle_selected(); // expand root
        let left = Event::Key(KeyEvent {
            code: KeyCode::Left,
            modifiers: crate::event::Modifiers::NONE,
        });

        tree.handle_event(&left); // collapse root
        assert_eq!(tree.visible_count(), 1);
    }

    #[test]
    fn enter_toggles() {
        let mut tree = make_test_tree();
        let enter = Event::Key(KeyEvent {
            code: KeyCode::Enter,
            modifiers: crate::event::Modifiers::NONE,
        });

        tree.handle_event(&enter); // expand
        assert_eq!(tree.visible_count(), 4);
        tree.handle_event(&enter); // collapse
        assert_eq!(tree.visible_count(), 1);
    }

    #[test]
    fn lazy_load_on_expand() {
        use std::cell::RefCell;
        use std::rc::Rc;

        let load_count = Rc::new(RefCell::new(0));
        let captured = Rc::clone(&load_count);

        let root = TreeNode::branch("root".into());
        let mut tree = Tree::new(vec![root])
            .with_render_fn(make_render_fn())
            .with_lazy_load(move |_data: &String| {
                *captured.borrow_mut() += 1;
                vec![
                    TreeNode::new("loaded1".into()),
                    TreeNode::new("loaded2".into()),
                ]
            });

        tree.expand_selected();
        assert_eq!(*load_count.borrow(), 1);
        // root (expanded) + 2 loaded children
        assert_eq!(tree.visible_count(), 3);
    }

    #[test]
    fn selected_node_retrieval() {
        let mut tree = make_test_tree();
        tree.toggle_selected(); // expand root

        // Select leaf1
        tree.selected = 1;
        match tree.selected_node() {
            Some(node) => assert_eq!(node.data, "leaf1"),
            None => unreachable!("should have selected node"),
        }
    }

    #[test]
    fn utf8_safe_node_labels() {
        let node = TreeNode::new("你好".into());
        let tree = Tree::new(vec![node]).with_render_fn(make_render_fn());

        let mut buf = ScreenBuffer::new(Size::new(10, 1));
        tree.render(Rect::new(0, 0, 10, 1), &mut buf);

        // " 你" — space indicator + space + CJK chars
        // Position 2 (after " " indicator + " " space) should be "你"
        assert_eq!(buf.get(2, 0).map(|c| c.grapheme.as_str()), Some("你"));
    }

    #[test]
    fn empty_tree() {
        let tree: Tree<String> = Tree::new(vec![]).with_render_fn(make_render_fn());
        assert_eq!(tree.visible_count(), 0);
        assert!(tree.selected_node().is_none());

        // Should not crash on render
        let mut buf = ScreenBuffer::new(Size::new(20, 5));
        tree.render(Rect::new(0, 0, 20, 5), &mut buf);
    }

    #[test]
    fn deep_tree_multiple_levels() {
        let deep = TreeNode::branch("L0".into()).with_child(
            TreeNode::branch("L1".into())
                .with_child(TreeNode::branch("L2".into()).with_child(TreeNode::new("L3".into()))),
        );

        let mut tree = Tree::new(vec![deep]).with_render_fn(make_render_fn());

        // Expand all levels
        tree.expand_selected(); // L0
        let down = Event::Key(KeyEvent {
            code: KeyCode::Down,
            modifiers: crate::event::Modifiers::NONE,
        });
        tree.handle_event(&down); // select L1
        tree.expand_selected(); // L1
        tree.handle_event(&down); // select L2
        tree.handle_event(&down); // stay or move to L2
        tree.selected = 2; // L2
        tree.expand_selected(); // L2

        // L0, L1, L2, L3 all visible
        assert_eq!(tree.visible_count(), 4);
    }

    #[test]
    fn mixed_expanded_collapsed() {
        let mut tree = make_test_tree();
        tree.toggle_selected(); // expand root
        // Navigate to branch1 (idx 2) and expand it
        tree.selected = 2;
        tree.toggle_selected(); // expand branch1

        // root, leaf1, branch1, child1, child2, leaf2
        assert_eq!(tree.visible_count(), 6);

        // Collapse branch1
        tree.toggle_selected();
        // root, leaf1, branch1, leaf2
        assert_eq!(tree.visible_count(), 4);
    }

    #[test]
    fn render_with_border() {
        let tree = make_test_tree();
        let tree = Tree {
            border: BorderStyle::Single,
            ..tree
        };

        let mut buf = ScreenBuffer::new(Size::new(20, 10));
        tree.render(Rect::new(0, 0, 20, 10), &mut buf);

        assert_eq!(buf.get(0, 0).map(|c| c.grapheme.as_str()), Some("\u{250c}"));
    }
}
