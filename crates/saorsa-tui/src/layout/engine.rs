//! Taffy-based layout engine.
//!
//! Wraps a [`taffy::TaffyTree`] to compute CSS Flexbox and Grid layouts,
//! mapping [`WidgetId`] to Taffy nodes and producing integer-cell
//! [`LayoutRect`] results for terminal rendering.

use std::collections::HashMap;

use taffy::prelude::*;

use crate::focus::WidgetId;
use crate::geometry::Rect;

/// A layout rectangle in terminal cell coordinates.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LayoutRect {
    /// X position (column).
    pub x: u16,
    /// Y position (row).
    pub y: u16,
    /// Width in columns.
    pub width: u16,
    /// Height in rows.
    pub height: u16,
}

impl LayoutRect {
    /// Create a new layout rectangle.
    pub const fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Convert to a [`Rect`].
    pub const fn to_rect(self) -> Rect {
        Rect::new(self.x, self.y, self.width, self.height)
    }
}

/// Errors from layout operations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LayoutError {
    /// The widget was not found in the layout tree.
    WidgetNotFound(WidgetId),
    /// An error occurred in Taffy.
    TaffyError(String),
    /// No root node has been set.
    NoRoot,
}

impl std::fmt::Display for LayoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WidgetNotFound(id) => write!(f, "widget not found: {id}"),
            Self::TaffyError(e) => write!(f, "taffy error: {e}"),
            Self::NoRoot => write!(f, "no root node set"),
        }
    }
}

impl std::error::Error for LayoutError {}

/// Layout engine backed by Taffy.
///
/// Manages a tree of layout nodes associated with widget IDs, computes
/// CSS Flexbox and Grid layout, and returns integer-cell results.
pub struct LayoutEngine {
    taffy: TaffyTree<()>,
    widget_to_node: HashMap<WidgetId, NodeId>,
    node_to_widget: HashMap<NodeId, WidgetId>,
    root: Option<NodeId>,
}

impl LayoutEngine {
    /// Create a new empty layout engine.
    pub fn new() -> Self {
        Self {
            taffy: TaffyTree::new(),
            widget_to_node: HashMap::new(),
            node_to_widget: HashMap::new(),
            root: None,
        }
    }

    /// Add a leaf node with the given style.
    pub fn add_node(&mut self, widget_id: WidgetId, style: Style) -> Result<(), LayoutError> {
        let node = self
            .taffy
            .new_leaf(style)
            .map_err(|e| LayoutError::TaffyError(format!("{e}")))?;
        self.widget_to_node.insert(widget_id, node);
        self.node_to_widget.insert(node, widget_id);
        Ok(())
    }

    /// Add a node with children.
    pub fn add_node_with_children(
        &mut self,
        widget_id: WidgetId,
        style: Style,
        children: &[WidgetId],
    ) -> Result<(), LayoutError> {
        let child_nodes: Vec<NodeId> = children
            .iter()
            .map(|id| {
                self.widget_to_node
                    .get(id)
                    .copied()
                    .ok_or(LayoutError::WidgetNotFound(*id))
            })
            .collect::<Result<Vec<_>, _>>()?;

        let node = self
            .taffy
            .new_with_children(style, &child_nodes)
            .map_err(|e| LayoutError::TaffyError(format!("{e}")))?;
        self.widget_to_node.insert(widget_id, node);
        self.node_to_widget.insert(node, widget_id);
        Ok(())
    }

    /// Set the root node for layout computation.
    pub fn set_root(&mut self, widget_id: WidgetId) -> Result<(), LayoutError> {
        let node = self
            .widget_to_node
            .get(&widget_id)
            .copied()
            .ok_or(LayoutError::WidgetNotFound(widget_id))?;
        self.root = Some(node);
        Ok(())
    }

    /// Update the style of an existing node.
    pub fn update_style(&mut self, widget_id: WidgetId, style: Style) -> Result<(), LayoutError> {
        let node = self
            .widget_to_node
            .get(&widget_id)
            .copied()
            .ok_or(LayoutError::WidgetNotFound(widget_id))?;
        self.taffy
            .set_style(node, style)
            .map_err(|e| LayoutError::TaffyError(format!("{e}")))?;
        Ok(())
    }

    /// Replace the children list for an existing node.
    pub fn set_children(
        &mut self,
        widget_id: WidgetId,
        children: &[WidgetId],
    ) -> Result<(), LayoutError> {
        let node = self
            .widget_to_node
            .get(&widget_id)
            .copied()
            .ok_or(LayoutError::WidgetNotFound(widget_id))?;
        let child_nodes: Vec<NodeId> = children
            .iter()
            .map(|id| {
                self.widget_to_node
                    .get(id)
                    .copied()
                    .ok_or(LayoutError::WidgetNotFound(*id))
            })
            .collect::<Result<Vec<_>, _>>()?;
        self.taffy
            .set_children(node, &child_nodes)
            .map_err(|e| LayoutError::TaffyError(format!("{e}")))?;
        Ok(())
    }

    /// Remove a node from the layout tree.
    pub fn remove_node(&mut self, widget_id: WidgetId) -> Result<(), LayoutError> {
        let node = self
            .widget_to_node
            .remove(&widget_id)
            .ok_or(LayoutError::WidgetNotFound(widget_id))?;
        self.node_to_widget.remove(&node);
        self.taffy
            .remove(node)
            .map_err(|e| LayoutError::TaffyError(format!("{e}")))?;
        if self.root == Some(node) {
            self.root = None;
        }
        Ok(())
    }

    /// Compute layout using the given available space.
    pub fn compute(
        &mut self,
        available_width: u16,
        available_height: u16,
    ) -> Result<(), LayoutError> {
        let root = self.root.ok_or(LayoutError::NoRoot)?;
        let available = taffy::Size {
            width: AvailableSpace::Definite(f32::from(available_width)),
            height: AvailableSpace::Definite(f32::from(available_height)),
        };
        self.taffy
            .compute_layout(root, available)
            .map_err(|e| LayoutError::TaffyError(format!("{e}")))?;
        Ok(())
    }

    /// Get the computed layout for a widget as a [`LayoutRect`].
    pub fn layout(&self, widget_id: WidgetId) -> Result<LayoutRect, LayoutError> {
        let node = self
            .widget_to_node
            .get(&widget_id)
            .copied()
            .ok_or(LayoutError::WidgetNotFound(widget_id))?;
        let layout = self
            .taffy
            .layout(node)
            .map_err(|e| LayoutError::TaffyError(format!("{e}")))?;

        Ok(LayoutRect {
            x: round_position(layout.location.x),
            y: round_position(layout.location.y),
            width: round_size(layout.size.width),
            height: round_size(layout.size.height),
        })
    }

    /// Get the computed layout for a widget as a [`Rect`].
    pub fn layout_rect(&self, widget_id: WidgetId) -> Result<Rect, LayoutError> {
        self.layout(widget_id).map(|lr| lr.to_rect())
    }

    /// Check if a widget has a layout node.
    pub fn has_node(&self, widget_id: WidgetId) -> bool {
        self.widget_to_node.contains_key(&widget_id)
    }

    /// Return the number of nodes in the tree.
    pub fn node_count(&self) -> usize {
        self.widget_to_node.len()
    }
}

impl Default for LayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Round a position value: floor to integer cells.
pub fn round_position(value: f32) -> u16 {
    if value < 0.0 {
        0
    } else if value > f32::from(u16::MAX) {
        u16::MAX
    } else {
        value.floor() as u16
    }
}

/// Round a size value: round to nearest integer cells.
pub fn round_size(value: f32) -> u16 {
    if value < 0.0 {
        0
    } else if value > f32::from(u16::MAX) {
        u16::MAX
    } else {
        value.round() as u16
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Rect;
    use taffy::prelude::{
        AlignItems, Dimension, Display, FlexDirection, GridPlacement, JustifyContent,
        LengthPercentage, LengthPercentageAuto, Line, Style, auto, fr, length,
    };

    fn wid(n: u64) -> WidgetId {
        n
    }

    #[test]
    fn empty_engine() {
        let engine = LayoutEngine::new();
        assert_eq!(engine.node_count(), 0);
        assert!(!engine.has_node(wid(1)));
    }

    #[test]
    fn add_leaf_node() {
        let mut engine = LayoutEngine::new();
        let result = engine.add_node(wid(1), Style::default());
        assert!(result.is_ok());
        assert!(engine.has_node(wid(1)));
        assert_eq!(engine.node_count(), 1);
    }

    #[test]
    fn add_with_children() {
        let mut engine = LayoutEngine::new();
        engine.add_node(wid(1), Style::default()).ok();
        engine.add_node(wid(2), Style::default()).ok();
        let result = engine.add_node_with_children(wid(3), Style::default(), &[wid(1), wid(2)]);
        assert!(result.is_ok());
        assert_eq!(engine.node_count(), 3);
    }

    #[test]
    fn set_root() {
        let mut engine = LayoutEngine::new();
        engine.add_node(wid(1), Style::default()).ok();
        let result = engine.set_root(wid(1));
        assert!(result.is_ok());
    }

    #[test]
    fn remove_node() {
        let mut engine = LayoutEngine::new();
        engine.add_node(wid(1), Style::default()).ok();
        assert!(engine.has_node(wid(1)));
        let result = engine.remove_node(wid(1));
        assert!(result.is_ok());
        assert!(!engine.has_node(wid(1)));
        assert_eq!(engine.node_count(), 0);
    }

    #[test]
    fn update_style() {
        let mut engine = LayoutEngine::new();
        engine.add_node(wid(1), Style::default()).ok();
        let new_style = Style {
            size: taffy::Size {
                width: Dimension::Length(50.0),
                height: Dimension::Length(25.0),
            },
            ..Default::default()
        };
        let result = engine.update_style(wid(1), new_style);
        assert!(result.is_ok());
    }

    #[test]
    fn compute_single_node() {
        let mut engine = LayoutEngine::new();
        engine
            .add_node(
                wid(1),
                Style {
                    size: taffy::Size {
                        width: Dimension::Length(80.0),
                        height: Dimension::Length(24.0),
                    },
                    ..Default::default()
                },
            )
            .ok();
        engine.set_root(wid(1)).ok();
        let result = engine.compute(80, 24);
        assert!(result.is_ok());

        let layout = engine.layout(wid(1));
        assert!(layout.is_ok());
        let rect = layout.ok();
        assert!(rect.is_some());
        let rect = rect.unwrap_or_default();
        assert_eq!(rect.width, 80);
        assert_eq!(rect.height, 24);
    }

    #[test]
    fn compute_two_children_row() {
        let mut engine = LayoutEngine::new();
        engine
            .add_node(
                wid(1),
                Style {
                    flex_grow: 1.0,
                    ..Default::default()
                },
            )
            .ok();
        engine
            .add_node(
                wid(2),
                Style {
                    flex_grow: 1.0,
                    ..Default::default()
                },
            )
            .ok();
        engine
            .add_node_with_children(
                wid(3),
                Style {
                    size: taffy::Size {
                        width: Dimension::Length(80.0),
                        height: Dimension::Length(24.0),
                    },
                    ..Default::default()
                },
                &[wid(1), wid(2)],
            )
            .ok();
        engine.set_root(wid(3)).ok();
        engine.compute(80, 24).ok();

        let l1 = engine.layout(wid(1)).unwrap_or_default();
        let l2 = engine.layout(wid(2)).unwrap_or_default();
        assert_eq!(l1.width, 40);
        assert_eq!(l2.width, 40);
        assert_eq!(l1.height, 24);
    }

    #[test]
    fn compute_two_children_column() {
        let mut engine = LayoutEngine::new();
        engine
            .add_node(
                wid(1),
                Style {
                    flex_grow: 1.0,
                    ..Default::default()
                },
            )
            .ok();
        engine
            .add_node(
                wid(2),
                Style {
                    flex_grow: 1.0,
                    ..Default::default()
                },
            )
            .ok();
        engine
            .add_node_with_children(
                wid(3),
                Style {
                    flex_direction: FlexDirection::Column,
                    size: taffy::Size {
                        width: Dimension::Length(80.0),
                        height: Dimension::Length(24.0),
                    },
                    ..Default::default()
                },
                &[wid(1), wid(2)],
            )
            .ok();
        engine.set_root(wid(3)).ok();
        engine.compute(80, 24).ok();

        let l1 = engine.layout(wid(1)).unwrap_or_default();
        let l2 = engine.layout(wid(2)).unwrap_or_default();
        assert_eq!(l1.height, 12);
        assert_eq!(l2.height, 12);
        assert_eq!(l1.width, 80);
    }

    #[test]
    fn layout_rect_conversion() {
        let lr = LayoutRect::new(5, 10, 40, 20);
        let rect = lr.to_rect();
        assert_eq!(rect, Rect::new(5, 10, 40, 20));
    }

    #[test]
    fn widget_not_found_error() {
        let engine = LayoutEngine::new();
        let result = engine.layout(wid(999));
        assert!(result.is_err());
        match result {
            Err(LayoutError::WidgetNotFound(id)) => assert_eq!(id, wid(999)),
            _ => unreachable!(),
        }
    }

    #[test]
    fn no_root_error() {
        let mut engine = LayoutEngine::new();
        let result = engine.compute(80, 24);
        assert!(result.is_err());
        match result {
            Err(LayoutError::NoRoot) => {}
            _ => unreachable!(),
        }
    }

    #[test]
    fn round_position_values() {
        assert_eq!(round_position(0.0), 0);
        assert_eq!(round_position(5.7), 5); // floor
        assert_eq!(round_position(10.99), 10); // floor
        assert_eq!(round_position(-1.0), 0); // negative clamped
    }

    #[test]
    fn round_size_values() {
        assert_eq!(round_size(0.0), 0);
        assert_eq!(round_size(5.4), 5); // round
        assert_eq!(round_size(5.5), 6); // round up
        assert_eq!(round_size(-1.0), 0); // negative clamped
    }

    #[test]
    fn children_not_found_error() {
        let mut engine = LayoutEngine::new();
        let result = engine.add_node_with_children(wid(1), Style::default(), &[wid(999)]);
        assert!(result.is_err());
        match result {
            Err(LayoutError::WidgetNotFound(id)) => assert_eq!(id, wid(999)),
            _ => unreachable!(),
        }
    }

    #[test]
    fn remove_root_clears_root() {
        let mut engine = LayoutEngine::new();
        engine.add_node(wid(1), Style::default()).ok();
        engine.set_root(wid(1)).ok();
        engine.remove_node(wid(1)).ok();
        let result = engine.compute(80, 24);
        assert!(matches!(result, Err(LayoutError::NoRoot)));
    }

    // --- Flexbox integration tests (Task 5) ---

    #[test]
    fn flex_row_equal_grow() {
        let mut engine = LayoutEngine::new();
        for i in 1..=3 {
            engine
                .add_node(
                    wid(i),
                    Style {
                        flex_grow: 1.0,
                        ..Default::default()
                    },
                )
                .ok();
        }
        engine
            .add_node_with_children(
                wid(10),
                Style {
                    size: taffy::Size {
                        width: Dimension::Length(90.0),
                        height: Dimension::Length(30.0),
                    },
                    ..Default::default()
                },
                &[wid(1), wid(2), wid(3)],
            )
            .ok();
        engine.set_root(wid(10)).ok();
        engine.compute(90, 30).ok();

        for i in 1..=3 {
            let l = engine.layout(wid(i)).unwrap_or_default();
            assert_eq!(l.width, 30, "child {i} width should be 30");
        }
    }

    #[test]
    fn flex_column_equal_grow() {
        let mut engine = LayoutEngine::new();
        for i in 1..=3 {
            engine
                .add_node(
                    wid(i),
                    Style {
                        flex_grow: 1.0,
                        ..Default::default()
                    },
                )
                .ok();
        }
        engine
            .add_node_with_children(
                wid(10),
                Style {
                    flex_direction: FlexDirection::Column,
                    size: taffy::Size {
                        width: Dimension::Length(60.0),
                        height: Dimension::Length(30.0),
                    },
                    ..Default::default()
                },
                &[wid(1), wid(2), wid(3)],
            )
            .ok();
        engine.set_root(wid(10)).ok();
        engine.compute(60, 30).ok();

        for i in 1..=3 {
            let l = engine.layout(wid(i)).unwrap_or_default();
            assert_eq!(l.height, 10, "child {i} height should be 10");
        }
    }

    #[test]
    fn flex_row_unequal_grow() {
        let mut engine = LayoutEngine::new();
        engine
            .add_node(
                wid(1),
                Style {
                    flex_grow: 1.0,
                    ..Default::default()
                },
            )
            .ok();
        engine
            .add_node(
                wid(2),
                Style {
                    flex_grow: 2.0,
                    ..Default::default()
                },
            )
            .ok();
        engine
            .add_node(
                wid(3),
                Style {
                    flex_grow: 1.0,
                    ..Default::default()
                },
            )
            .ok();
        engine
            .add_node_with_children(
                wid(10),
                Style {
                    size: taffy::Size {
                        width: Dimension::Length(80.0),
                        height: Dimension::Length(20.0),
                    },
                    ..Default::default()
                },
                &[wid(1), wid(2), wid(3)],
            )
            .ok();
        engine.set_root(wid(10)).ok();
        engine.compute(80, 20).ok();

        let l1 = engine.layout(wid(1)).unwrap_or_default();
        let l2 = engine.layout(wid(2)).unwrap_or_default();
        let l3 = engine.layout(wid(3)).unwrap_or_default();
        assert_eq!(l1.width, 20);
        assert_eq!(l2.width, 40);
        assert_eq!(l3.width, 20);
    }

    #[test]
    fn flex_column_fixed_and_grow() {
        let mut engine = LayoutEngine::new();
        engine
            .add_node(
                wid(1),
                Style {
                    size: taffy::Size {
                        width: auto(),
                        height: Dimension::Length(5.0),
                    },
                    ..Default::default()
                },
            )
            .ok();
        engine
            .add_node(
                wid(2),
                Style {
                    flex_grow: 1.0,
                    ..Default::default()
                },
            )
            .ok();
        engine
            .add_node_with_children(
                wid(10),
                Style {
                    flex_direction: FlexDirection::Column,
                    size: taffy::Size {
                        width: Dimension::Length(80.0),
                        height: Dimension::Length(25.0),
                    },
                    ..Default::default()
                },
                &[wid(1), wid(2)],
            )
            .ok();
        engine.set_root(wid(10)).ok();
        engine.compute(80, 25).ok();

        let l1 = engine.layout(wid(1)).unwrap_or_default();
        let l2 = engine.layout(wid(2)).unwrap_or_default();
        assert_eq!(l1.height, 5);
        assert_eq!(l2.height, 20);
    }

    #[test]
    fn flex_justify_center() {
        let mut engine = LayoutEngine::new();
        engine
            .add_node(
                wid(1),
                Style {
                    size: taffy::Size {
                        width: Dimension::Length(20.0),
                        height: Dimension::Length(10.0),
                    },
                    ..Default::default()
                },
            )
            .ok();
        engine
            .add_node_with_children(
                wid(10),
                Style {
                    justify_content: Some(JustifyContent::Center),
                    size: taffy::Size {
                        width: Dimension::Length(80.0),
                        height: Dimension::Length(10.0),
                    },
                    ..Default::default()
                },
                &[wid(1)],
            )
            .ok();
        engine.set_root(wid(10)).ok();
        engine.compute(80, 10).ok();

        let l = engine.layout(wid(1)).unwrap_or_default();
        assert_eq!(l.x, 30); // (80-20)/2 = 30
    }

    #[test]
    fn flex_justify_space_between() {
        let mut engine = LayoutEngine::new();
        engine
            .add_node(
                wid(1),
                Style {
                    size: taffy::Size {
                        width: Dimension::Length(10.0),
                        height: Dimension::Length(10.0),
                    },
                    ..Default::default()
                },
            )
            .ok();
        engine
            .add_node(
                wid(2),
                Style {
                    size: taffy::Size {
                        width: Dimension::Length(10.0),
                        height: Dimension::Length(10.0),
                    },
                    ..Default::default()
                },
            )
            .ok();
        engine
            .add_node_with_children(
                wid(10),
                Style {
                    justify_content: Some(JustifyContent::SpaceBetween),
                    size: taffy::Size {
                        width: Dimension::Length(80.0),
                        height: Dimension::Length(10.0),
                    },
                    ..Default::default()
                },
                &[wid(1), wid(2)],
            )
            .ok();
        engine.set_root(wid(10)).ok();
        engine.compute(80, 10).ok();

        let l1 = engine.layout(wid(1)).unwrap_or_default();
        let l2 = engine.layout(wid(2)).unwrap_or_default();
        assert_eq!(l1.x, 0);
        assert_eq!(l2.x, 70); // 80 - 10 = 70
    }

    #[test]
    fn flex_align_items_center() {
        let mut engine = LayoutEngine::new();
        engine
            .add_node(
                wid(1),
                Style {
                    size: taffy::Size {
                        width: Dimension::Length(20.0),
                        height: Dimension::Length(10.0),
                    },
                    ..Default::default()
                },
            )
            .ok();
        engine
            .add_node_with_children(
                wid(10),
                Style {
                    align_items: Some(AlignItems::Center),
                    size: taffy::Size {
                        width: Dimension::Length(80.0),
                        height: Dimension::Length(30.0),
                    },
                    ..Default::default()
                },
                &[wid(1)],
            )
            .ok();
        engine.set_root(wid(10)).ok();
        engine.compute(80, 30).ok();

        let l = engine.layout(wid(1)).unwrap_or_default();
        assert_eq!(l.y, 10); // (30-10)/2 = 10
    }

    #[test]
    fn flex_nested() {
        let mut engine = LayoutEngine::new();
        // Inner children
        engine
            .add_node(
                wid(1),
                Style {
                    flex_grow: 1.0,
                    ..Default::default()
                },
            )
            .ok();
        engine
            .add_node(
                wid(2),
                Style {
                    flex_grow: 1.0,
                    ..Default::default()
                },
            )
            .ok();
        // Inner container (column)
        engine
            .add_node_with_children(
                wid(3),
                Style {
                    flex_direction: FlexDirection::Column,
                    flex_grow: 1.0,
                    ..Default::default()
                },
                &[wid(1), wid(2)],
            )
            .ok();
        // Sibling
        engine
            .add_node(
                wid(4),
                Style {
                    flex_grow: 1.0,
                    ..Default::default()
                },
            )
            .ok();
        // Root (row)
        engine
            .add_node_with_children(
                wid(10),
                Style {
                    size: taffy::Size {
                        width: Dimension::Length(80.0),
                        height: Dimension::Length(20.0),
                    },
                    ..Default::default()
                },
                &[wid(3), wid(4)],
            )
            .ok();
        engine.set_root(wid(10)).ok();
        engine.compute(80, 20).ok();

        let l3 = engine.layout(wid(3)).unwrap_or_default();
        let l4 = engine.layout(wid(4)).unwrap_or_default();
        assert_eq!(l3.width, 40);
        assert_eq!(l4.width, 40);
        let l1 = engine.layout(wid(1)).unwrap_or_default();
        let l2 = engine.layout(wid(2)).unwrap_or_default();
        assert_eq!(l1.height, 10);
        assert_eq!(l2.height, 10);
    }

    #[test]
    fn flex_with_gap() {
        let mut engine = LayoutEngine::new();
        engine
            .add_node(
                wid(1),
                Style {
                    size: taffy::Size {
                        width: Dimension::Length(20.0),
                        height: Dimension::Length(10.0),
                    },
                    ..Default::default()
                },
            )
            .ok();
        engine
            .add_node(
                wid(2),
                Style {
                    size: taffy::Size {
                        width: Dimension::Length(20.0),
                        height: Dimension::Length(10.0),
                    },
                    ..Default::default()
                },
            )
            .ok();
        engine
            .add_node_with_children(
                wid(10),
                Style {
                    gap: taffy::Size {
                        width: LengthPercentage::Length(10.0),
                        height: LengthPercentage::Length(0.0),
                    },
                    size: taffy::Size {
                        width: Dimension::Length(80.0),
                        height: Dimension::Length(10.0),
                    },
                    ..Default::default()
                },
                &[wid(1), wid(2)],
            )
            .ok();
        engine.set_root(wid(10)).ok();
        engine.compute(80, 10).ok();

        let l1 = engine.layout(wid(1)).unwrap_or_default();
        let l2 = engine.layout(wid(2)).unwrap_or_default();
        assert_eq!(l1.x, 0);
        assert_eq!(l2.x, 30); // 20 + 10 gap
    }

    // --- Grid layout tests (Task 6) ---

    #[test]
    fn grid_two_columns_equal() {
        let mut engine = LayoutEngine::new();
        engine.add_node(wid(1), Style::default()).ok();
        engine.add_node(wid(2), Style::default()).ok();
        engine
            .add_node_with_children(
                wid(10),
                Style {
                    display: Display::Grid,
                    grid_template_columns: vec![fr(1.0), fr(1.0)],
                    size: taffy::Size {
                        width: Dimension::Length(80.0),
                        height: Dimension::Length(20.0),
                    },
                    ..Default::default()
                },
                &[wid(1), wid(2)],
            )
            .ok();
        engine.set_root(wid(10)).ok();
        engine.compute(80, 20).ok();

        let l1 = engine.layout(wid(1)).unwrap_or_default();
        let l2 = engine.layout(wid(2)).unwrap_or_default();
        assert_eq!(l1.width, 40);
        assert_eq!(l2.width, 40);
    }

    #[test]
    fn grid_three_columns_fr() {
        let mut engine = LayoutEngine::new();
        for i in 1..=3 {
            engine.add_node(wid(i), Style::default()).ok();
        }
        engine
            .add_node_with_children(
                wid(10),
                Style {
                    display: Display::Grid,
                    grid_template_columns: vec![fr(1.0), fr(2.0), fr(1.0)],
                    size: taffy::Size {
                        width: Dimension::Length(80.0),
                        height: Dimension::Length(20.0),
                    },
                    ..Default::default()
                },
                &[wid(1), wid(2), wid(3)],
            )
            .ok();
        engine.set_root(wid(10)).ok();
        engine.compute(80, 20).ok();

        let l1 = engine.layout(wid(1)).unwrap_or_default();
        let l2 = engine.layout(wid(2)).unwrap_or_default();
        let l3 = engine.layout(wid(3)).unwrap_or_default();
        assert_eq!(l1.width, 20);
        assert_eq!(l2.width, 40);
        assert_eq!(l3.width, 20);
    }

    #[test]
    fn grid_columns_mixed_units() {
        let mut engine = LayoutEngine::new();
        engine.add_node(wid(1), Style::default()).ok();
        engine.add_node(wid(2), Style::default()).ok();
        engine
            .add_node_with_children(
                wid(10),
                Style {
                    display: Display::Grid,
                    grid_template_columns: vec![length(20.0), fr(1.0)],
                    size: taffy::Size {
                        width: Dimension::Length(80.0),
                        height: Dimension::Length(20.0),
                    },
                    ..Default::default()
                },
                &[wid(1), wid(2)],
            )
            .ok();
        engine.set_root(wid(10)).ok();
        engine.compute(80, 20).ok();

        let l1 = engine.layout(wid(1)).unwrap_or_default();
        let l2 = engine.layout(wid(2)).unwrap_or_default();
        assert_eq!(l1.width, 20);
        assert_eq!(l2.width, 60);
    }

    #[test]
    fn grid_rows_and_columns() {
        let mut engine = LayoutEngine::new();
        for i in 1..=4 {
            engine.add_node(wid(i), Style::default()).ok();
        }
        engine
            .add_node_with_children(
                wid(10),
                Style {
                    display: Display::Grid,
                    grid_template_columns: vec![fr(1.0), fr(1.0)],
                    grid_template_rows: vec![fr(1.0), fr(1.0)],
                    size: taffy::Size {
                        width: Dimension::Length(80.0),
                        height: Dimension::Length(20.0),
                    },
                    ..Default::default()
                },
                &[wid(1), wid(2), wid(3), wid(4)],
            )
            .ok();
        engine.set_root(wid(10)).ok();
        engine.compute(80, 20).ok();

        let l1 = engine.layout(wid(1)).unwrap_or_default();
        let l4 = engine.layout(wid(4)).unwrap_or_default();
        assert_eq!(l1.width, 40);
        assert_eq!(l1.height, 10);
        assert_eq!(l4.x, 40);
        assert_eq!(l4.y, 10);
    }

    #[test]
    fn grid_placement_span() {
        let mut engine = LayoutEngine::new();
        // First child spans 2 columns
        engine
            .add_node(
                wid(1),
                Style {
                    grid_column: Line {
                        start: GridPlacement::from_span(2),
                        end: GridPlacement::Auto,
                    },
                    ..Default::default()
                },
            )
            .ok();
        engine.add_node(wid(2), Style::default()).ok();
        engine.add_node(wid(3), Style::default()).ok();
        engine
            .add_node_with_children(
                wid(10),
                Style {
                    display: Display::Grid,
                    grid_template_columns: vec![fr(1.0), fr(1.0)],
                    size: taffy::Size {
                        width: Dimension::Length(80.0),
                        height: Dimension::Length(30.0),
                    },
                    ..Default::default()
                },
                &[wid(1), wid(2), wid(3)],
            )
            .ok();
        engine.set_root(wid(10)).ok();
        engine.compute(80, 30).ok();

        let l1 = engine.layout(wid(1)).unwrap_or_default();
        assert_eq!(l1.width, 80); // spans full width
    }

    #[test]
    fn box_model_padding_shrinks_content() {
        let mut engine = LayoutEngine::new();
        engine
            .add_node(
                wid(1),
                Style {
                    flex_grow: 1.0,
                    ..Default::default()
                },
            )
            .ok();
        engine
            .add_node_with_children(
                wid(10),
                Style {
                    padding: taffy::Rect {
                        left: LengthPercentage::Length(5.0),
                        right: LengthPercentage::Length(5.0),
                        top: LengthPercentage::Length(2.0),
                        bottom: LengthPercentage::Length(2.0),
                    },
                    size: taffy::Size {
                        width: Dimension::Length(80.0),
                        height: Dimension::Length(24.0),
                    },
                    ..Default::default()
                },
                &[wid(1)],
            )
            .ok();
        engine.set_root(wid(10)).ok();
        engine.compute(80, 24).ok();

        let l = engine.layout(wid(1)).unwrap_or_default();
        assert_eq!(l.x, 5);
        assert_eq!(l.y, 2);
        assert_eq!(l.width, 70); // 80 - 5 - 5
        assert_eq!(l.height, 20); // 24 - 2 - 2
    }

    #[test]
    fn box_model_margin_creates_space() {
        let mut engine = LayoutEngine::new();
        engine
            .add_node(
                wid(1),
                Style {
                    size: taffy::Size {
                        width: Dimension::Length(30.0),
                        height: Dimension::Length(10.0),
                    },
                    margin: taffy::Rect {
                        left: LengthPercentageAuto::Length(0.0),
                        right: LengthPercentageAuto::Length(5.0),
                        top: LengthPercentageAuto::Length(0.0),
                        bottom: LengthPercentageAuto::Length(0.0),
                    },
                    ..Default::default()
                },
            )
            .ok();
        engine
            .add_node(
                wid(2),
                Style {
                    size: taffy::Size {
                        width: Dimension::Length(30.0),
                        height: Dimension::Length(10.0),
                    },
                    ..Default::default()
                },
            )
            .ok();
        engine
            .add_node_with_children(
                wid(10),
                Style {
                    size: taffy::Size {
                        width: Dimension::Length(80.0),
                        height: Dimension::Length(10.0),
                    },
                    ..Default::default()
                },
                &[wid(1), wid(2)],
            )
            .ok();
        engine.set_root(wid(10)).ok();
        engine.compute(80, 10).ok();

        let l2 = engine.layout(wid(2)).unwrap_or_default();
        assert_eq!(l2.x, 35); // 30 + 5 margin
    }

    #[test]
    fn box_model_border_width() {
        let mut engine = LayoutEngine::new();
        engine
            .add_node(
                wid(1),
                Style {
                    flex_grow: 1.0,
                    ..Default::default()
                },
            )
            .ok();
        engine
            .add_node_with_children(
                wid(10),
                Style {
                    border: taffy::Rect {
                        left: LengthPercentage::Length(1.0),
                        right: LengthPercentage::Length(1.0),
                        top: LengthPercentage::Length(1.0),
                        bottom: LengthPercentage::Length(1.0),
                    },
                    size: taffy::Size {
                        width: Dimension::Length(80.0),
                        height: Dimension::Length(24.0),
                    },
                    ..Default::default()
                },
                &[wid(1)],
            )
            .ok();
        engine.set_root(wid(10)).ok();
        engine.compute(80, 24).ok();

        let l = engine.layout(wid(1)).unwrap_or_default();
        assert_eq!(l.x, 1);
        assert_eq!(l.y, 1);
        assert_eq!(l.width, 78); // 80 - 1 - 1
        assert_eq!(l.height, 22); // 24 - 1 - 1
    }

    #[test]
    fn box_model_combined() {
        let mut engine = LayoutEngine::new();
        engine
            .add_node(
                wid(1),
                Style {
                    flex_grow: 1.0,
                    ..Default::default()
                },
            )
            .ok();
        engine
            .add_node_with_children(
                wid(10),
                Style {
                    padding: taffy::Rect {
                        left: LengthPercentage::Length(2.0),
                        right: LengthPercentage::Length(2.0),
                        top: LengthPercentage::Length(1.0),
                        bottom: LengthPercentage::Length(1.0),
                    },
                    border: taffy::Rect {
                        left: LengthPercentage::Length(1.0),
                        right: LengthPercentage::Length(1.0),
                        top: LengthPercentage::Length(1.0),
                        bottom: LengthPercentage::Length(1.0),
                    },
                    size: taffy::Size {
                        width: Dimension::Length(80.0),
                        height: Dimension::Length(24.0),
                    },
                    ..Default::default()
                },
                &[wid(1)],
            )
            .ok();
        engine.set_root(wid(10)).ok();
        engine.compute(80, 24).ok();

        let l = engine.layout(wid(1)).unwrap_or_default();
        // border(1) + padding(2) = 3 each side
        assert_eq!(l.x, 3);
        assert_eq!(l.y, 2); // border(1) + padding(1)
        assert_eq!(l.width, 74); // 80 - 3 - 3
        assert_eq!(l.height, 20); // 24 - 2 - 2
    }

    #[test]
    fn layout_error_display() {
        let e1 = LayoutError::WidgetNotFound(wid(42));
        assert!(format!("{e1}").contains("42"));

        let e2 = LayoutError::TaffyError("boom".into());
        assert!(format!("{e2}").contains("boom"));

        let e3 = LayoutError::NoRoot;
        assert!(format!("{e3}").contains("no root"));
    }
}
