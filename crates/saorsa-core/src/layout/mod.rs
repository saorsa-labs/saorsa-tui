//! Layout system for splitting terminal areas.
//!
//! Provides constraint-based splitting, dock positioning, Taffy-based
//! CSS Flexbox/Grid layout, and scroll region management.

pub mod engine;
pub mod scroll;
pub mod style_converter;

pub use engine::{LayoutEngine, LayoutError, LayoutRect};
pub use scroll::{OverflowBehavior, ScrollManager, ScrollState};
pub use style_converter::computed_to_taffy;

use crate::geometry::Rect;

/// Direction of layout splitting.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    /// Stack children top to bottom.
    Vertical,
    /// Stack children left to right.
    Horizontal,
}

/// Constraint for a layout segment.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Constraint {
    /// Fixed size in cells.
    Fixed(u16),
    /// Minimum size in cells.
    Min(u16),
    /// Maximum size in cells.
    Max(u16),
    /// Percentage of available space (0-100).
    Percentage(u8),
    /// Fill remaining space (distributed equally among all Fill constraints).
    Fill,
}

/// Dock position for anchoring a widget to an edge.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dock {
    /// Dock to the top edge.
    Top,
    /// Dock to the bottom edge.
    Bottom,
    /// Dock to the left edge.
    Left,
    /// Dock to the right edge.
    Right,
}

/// Layout utilities for splitting terminal areas.
pub struct Layout;

impl Layout {
    /// Split an area into segments along the given direction using constraints.
    ///
    /// Returns a `Vec<Rect>` with one rect per constraint.
    pub fn split(area: Rect, direction: Direction, constraints: &[Constraint]) -> Vec<Rect> {
        if constraints.is_empty() {
            return Vec::new();
        }

        let total = match direction {
            Direction::Vertical => area.size.height,
            Direction::Horizontal => area.size.width,
        };

        let sizes = solve_constraints(total, constraints);

        let mut results = Vec::with_capacity(constraints.len());
        let mut offset: u16 = 0;

        for &size in &sizes {
            let rect = match direction {
                Direction::Vertical => Rect::new(
                    area.position.x,
                    area.position.y + offset,
                    area.size.width,
                    size,
                ),
                Direction::Horizontal => Rect::new(
                    area.position.x + offset,
                    area.position.y,
                    size,
                    area.size.height,
                ),
            };
            results.push(rect);
            offset = offset.saturating_add(size);
        }

        results
    }

    /// Dock a region to one edge of the area.
    ///
    /// Returns `(docked_rect, remaining_rect)`.
    pub fn dock(area: Rect, dock: Dock, size: u16) -> (Rect, Rect) {
        match dock {
            Dock::Top => {
                let s = size.min(area.size.height);
                (
                    Rect::new(area.position.x, area.position.y, area.size.width, s),
                    Rect::new(
                        area.position.x,
                        area.position.y + s,
                        area.size.width,
                        area.size.height.saturating_sub(s),
                    ),
                )
            }
            Dock::Bottom => {
                let s = size.min(area.size.height);
                (
                    Rect::new(
                        area.position.x,
                        area.position.y + area.size.height.saturating_sub(s),
                        area.size.width,
                        s,
                    ),
                    Rect::new(
                        area.position.x,
                        area.position.y,
                        area.size.width,
                        area.size.height.saturating_sub(s),
                    ),
                )
            }
            Dock::Left => {
                let s = size.min(area.size.width);
                (
                    Rect::new(area.position.x, area.position.y, s, area.size.height),
                    Rect::new(
                        area.position.x + s,
                        area.position.y,
                        area.size.width.saturating_sub(s),
                        area.size.height,
                    ),
                )
            }
            Dock::Right => {
                let s = size.min(area.size.width);
                (
                    Rect::new(
                        area.position.x + area.size.width.saturating_sub(s),
                        area.position.y,
                        s,
                        area.size.height,
                    ),
                    Rect::new(
                        area.position.x,
                        area.position.y,
                        area.size.width.saturating_sub(s),
                        area.size.height,
                    ),
                )
            }
        }
    }
}

/// Solve constraints to produce sizes that fit within `total`.
fn solve_constraints(total: u16, constraints: &[Constraint]) -> Vec<u16> {
    let n = constraints.len();
    let mut sizes = vec![0u16; n];
    let mut remaining = total;

    // Pass 1: allocate Fixed constraints
    for (i, c) in constraints.iter().enumerate() {
        if let Constraint::Fixed(s) = c {
            let s = (*s).min(remaining);
            sizes[i] = s;
            remaining = remaining.saturating_sub(s);
        }
    }

    // Pass 2: allocate Percentage constraints
    for (i, c) in constraints.iter().enumerate() {
        if let Constraint::Percentage(p) = c {
            let s = ((u32::from(total) * u32::from(*p)) / 100) as u16;
            let s = s.min(remaining);
            sizes[i] = s;
            remaining = remaining.saturating_sub(s);
        }
    }

    // Pass 3: allocate Min constraints (give at least min, but not more than remaining for now)
    for (i, c) in constraints.iter().enumerate() {
        if let Constraint::Min(min) = c {
            let s = (*min).min(remaining);
            sizes[i] = s;
            remaining = remaining.saturating_sub(s);
        }
    }

    // Pass 4: allocate Max constraints
    for (i, c) in constraints.iter().enumerate() {
        if let Constraint::Max(max) = c {
            let s = (*max).min(remaining);
            sizes[i] = s;
            remaining = remaining.saturating_sub(s);
        }
    }

    // Pass 5: distribute remaining among Fill constraints
    let fill_count = constraints
        .iter()
        .filter(|c| matches!(c, Constraint::Fill))
        .count();
    if fill_count > 0 {
        let each = remaining / fill_count as u16;
        let mut extra = remaining % fill_count as u16;
        for (i, c) in constraints.iter().enumerate() {
            if matches!(c, Constraint::Fill) {
                let bonus = if extra > 0 {
                    extra -= 1;
                    1
                } else {
                    0
                };
                sizes[i] = each + bonus;
            }
        }
    }

    sizes
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Rect;

    #[test]
    fn vertical_split_fixed() {
        let area = Rect::new(0, 0, 80, 24);
        let rects = Layout::split(
            area,
            Direction::Vertical,
            &[Constraint::Fixed(3), Constraint::Fixed(5)],
        );
        assert_eq!(rects.len(), 2);
        assert_eq!(rects[0], Rect::new(0, 0, 80, 3));
        assert_eq!(rects[1], Rect::new(0, 3, 80, 5));
    }

    #[test]
    fn horizontal_split_fixed() {
        let area = Rect::new(0, 0, 80, 24);
        let rects = Layout::split(
            area,
            Direction::Horizontal,
            &[Constraint::Fixed(20), Constraint::Fixed(30)],
        );
        assert_eq!(rects.len(), 2);
        assert_eq!(rects[0], Rect::new(0, 0, 20, 24));
        assert_eq!(rects[1], Rect::new(20, 0, 30, 24));
    }

    #[test]
    fn vertical_fixed_plus_fill() {
        let area = Rect::new(0, 0, 80, 24);
        let rects = Layout::split(
            area,
            Direction::Vertical,
            &[Constraint::Fixed(3), Constraint::Fill],
        );
        assert_eq!(rects.len(), 2);
        assert_eq!(rects[0], Rect::new(0, 0, 80, 3));
        assert_eq!(rects[1], Rect::new(0, 3, 80, 21));
    }

    #[test]
    fn multiple_fills_distribute_equally() {
        let area = Rect::new(0, 0, 80, 24);
        let rects = Layout::split(
            area,
            Direction::Vertical,
            &[Constraint::Fill, Constraint::Fill],
        );
        assert_eq!(rects.len(), 2);
        assert_eq!(rects[0].size.height, 12);
        assert_eq!(rects[1].size.height, 12);
    }

    #[test]
    fn percentage_split() {
        let area = Rect::new(0, 0, 100, 10);
        let rects = Layout::split(
            area,
            Direction::Horizontal,
            &[Constraint::Percentage(30), Constraint::Percentage(70)],
        );
        assert_eq!(rects[0].size.width, 30);
        assert_eq!(rects[1].size.width, 70);
    }

    #[test]
    fn empty_constraints() {
        let area = Rect::new(0, 0, 80, 24);
        let rects = Layout::split(area, Direction::Vertical, &[]);
        assert!(rects.is_empty());
    }

    #[test]
    fn dock_top() {
        let area = Rect::new(0, 0, 80, 24);
        let (docked, remaining) = Layout::dock(area, Dock::Top, 3);
        assert_eq!(docked, Rect::new(0, 0, 80, 3));
        assert_eq!(remaining, Rect::new(0, 3, 80, 21));
    }

    #[test]
    fn dock_bottom() {
        let area = Rect::new(0, 0, 80, 24);
        let (docked, remaining) = Layout::dock(area, Dock::Bottom, 3);
        assert_eq!(docked, Rect::new(0, 21, 80, 3));
        assert_eq!(remaining, Rect::new(0, 0, 80, 21));
    }

    #[test]
    fn dock_left() {
        let area = Rect::new(0, 0, 80, 24);
        let (docked, remaining) = Layout::dock(area, Dock::Left, 20);
        assert_eq!(docked, Rect::new(0, 0, 20, 24));
        assert_eq!(remaining, Rect::new(20, 0, 60, 24));
    }

    #[test]
    fn dock_right() {
        let area = Rect::new(0, 0, 80, 24);
        let (docked, remaining) = Layout::dock(area, Dock::Right, 20);
        assert_eq!(docked, Rect::new(60, 0, 20, 24));
        assert_eq!(remaining, Rect::new(0, 0, 60, 24));
    }

    #[test]
    fn dock_larger_than_area() {
        let area = Rect::new(0, 0, 80, 10);
        let (docked, remaining) = Layout::dock(area, Dock::Top, 20);
        assert_eq!(docked, Rect::new(0, 0, 80, 10));
        assert_eq!(remaining, Rect::new(0, 10, 80, 0));
    }

    #[test]
    fn offset_area_split() {
        let area = Rect::new(5, 10, 40, 20);
        let rects = Layout::split(
            area,
            Direction::Vertical,
            &[Constraint::Fixed(5), Constraint::Fill],
        );
        assert_eq!(rects[0], Rect::new(5, 10, 40, 5));
        assert_eq!(rects[1], Rect::new(5, 15, 40, 15));
    }
}

#[cfg(test)]
mod integration_tests {
    use super::engine::LayoutEngine;
    use super::scroll::ScrollManager;
    use super::style_converter::computed_to_taffy;
    use crate::tcss::cascade::CascadeResolver;
    use crate::tcss::matcher::StyleMatcher;
    use crate::tcss::parser::parse_stylesheet;
    use crate::tcss::tree::{WidgetNode, WidgetTree};

    /// Parse TCSS, build tree, match, cascade, convert, compute layout.
    fn layout_from_css(
        css: &str,
        tree: &WidgetTree,
        root_id: u64,
        width: u16,
        height: u16,
    ) -> LayoutEngine {
        let result = parse_stylesheet(css);
        assert!(result.is_ok(), "parse failed: {result:?}");
        let stylesheet = match result {
            Ok(s) => s,
            Err(_) => unreachable!(),
        };
        let matcher = StyleMatcher::new(&stylesheet);
        let mut engine = LayoutEngine::new();

        // Build engine nodes bottom-up: leaves first, then parents
        build_engine_nodes(tree, root_id, &matcher, &mut engine);

        engine.set_root(root_id).ok();
        engine.compute(width, height).ok();
        engine
    }

    /// Recursively build engine nodes from widget tree.
    fn build_engine_nodes(
        tree: &WidgetTree,
        widget_id: u64,
        matcher: &StyleMatcher,
        engine: &mut LayoutEngine,
    ) {
        let node = tree.get(widget_id);
        assert!(node.is_some(), "widget {widget_id} not found");
        let node = match node {
            Some(n) => n,
            None => unreachable!(),
        };
        let children: Vec<u64> = node.children.clone();

        // Recurse into children first
        for &child_id in &children {
            build_engine_nodes(tree, child_id, matcher, engine);
        }

        // Match and cascade
        let matched = matcher.match_widget(tree, widget_id);
        let computed = CascadeResolver::resolve(&matched);
        let taffy_style = computed_to_taffy(&computed);

        if children.is_empty() {
            engine.add_node(widget_id, taffy_style).ok();
        } else {
            engine
                .add_node_with_children(widget_id, taffy_style, &children)
                .ok();
        }
    }

    #[test]
    fn integration_parse_to_layout() {
        let css = r#"
            #root {
                display: flex;
                flex-direction: column;
                width: 80;
                height: 24;
            }
            Label {
                flex-grow: 1;
            }
        "#;
        let mut tree = WidgetTree::new();
        let mut root = WidgetNode::new(1, "Container");
        root.css_id = Some("root".into());
        tree.add_node(root);
        let mut label = WidgetNode::new(2, "Label");
        label.parent = Some(1);
        tree.add_node(label);

        let engine = layout_from_css(css, &tree, 1, 80, 24);

        let root_layout = engine.layout(1).unwrap_or_default();
        assert_eq!(root_layout.width, 80);
        assert_eq!(root_layout.height, 24);

        let label_layout = engine.layout(2).unwrap_or_default();
        // In column layout, label fills full width and grows vertically
        assert_eq!(label_layout.width, 80);
        assert_eq!(label_layout.height, 24);
    }

    #[test]
    fn integration_flex_sidebar_layout() {
        let css = r#"
            #root { display: flex; width: 80; height: 24; }
            #sidebar { width: 20; }
            #main { flex-grow: 1; }
        "#;
        let mut tree = WidgetTree::new();
        let mut root = WidgetNode::new(1, "Container");
        root.css_id = Some("root".into());
        tree.add_node(root);

        let mut sidebar = WidgetNode::new(2, "Container");
        sidebar.css_id = Some("sidebar".into());
        sidebar.parent = Some(1);
        tree.add_node(sidebar);

        let mut main = WidgetNode::new(3, "Container");
        main.css_id = Some("main".into());
        main.parent = Some(1);
        tree.add_node(main);

        tree.get_mut(1)
            .iter_mut()
            .for_each(|n| n.children = vec![2, 3]);

        let engine = layout_from_css(css, &tree, 1, 80, 24);

        let sidebar_layout = engine.layout(2).unwrap_or_default();
        let main_layout = engine.layout(3).unwrap_or_default();
        assert_eq!(sidebar_layout.width, 20);
        assert_eq!(main_layout.width, 60);
        assert_eq!(main_layout.x, 20);
    }

    #[test]
    fn integration_grid_dashboard() {
        let css = r#"
            #root {
                display: grid;
                grid-template-columns: 1fr 1fr 1fr;
                grid-template-rows: 1fr 1fr;
                width: 90;
                height: 20;
            }
        "#;
        let mut tree = WidgetTree::new();
        let mut root = WidgetNode::new(1, "Container");
        root.css_id = Some("root".into());
        tree.add_node(root);

        let mut child_ids = Vec::new();
        for i in 2..=7 {
            let mut child = WidgetNode::new(i, "Panel");
            child.parent = Some(1);
            tree.add_node(child);
            child_ids.push(i);
        }
        tree.get_mut(1)
            .iter_mut()
            .for_each(|n| n.children = child_ids.clone());

        let engine = layout_from_css(css, &tree, 1, 90, 20);

        let l1 = engine.layout(2).unwrap_or_default();
        let l2 = engine.layout(3).unwrap_or_default();
        let l4 = engine.layout(5).unwrap_or_default();
        assert_eq!(l1.width, 30);
        assert_eq!(l1.height, 10);
        assert_eq!(l2.x, 30);
        assert_eq!(l4.y, 10); // second row
    }

    #[test]
    fn integration_nested_flex_grid() {
        let css = r#"
            #root { display: flex; width: 80; height: 20; }
            #left { flex-grow: 1; display: grid; grid-template-columns: 1fr 1fr; }
            #right { flex-grow: 1; }
        "#;
        let mut tree = WidgetTree::new();
        let mut root = WidgetNode::new(1, "Container");
        root.css_id = Some("root".into());
        tree.add_node(root);

        let mut left = WidgetNode::new(2, "Container");
        left.css_id = Some("left".into());
        left.parent = Some(1);
        tree.add_node(left);

        let mut right = WidgetNode::new(3, "Container");
        right.css_id = Some("right".into());
        right.parent = Some(1);
        tree.add_node(right);

        // Grid children of left
        let mut g1 = WidgetNode::new(4, "Panel");
        g1.parent = Some(2);
        tree.add_node(g1);
        let mut g2 = WidgetNode::new(5, "Panel");
        g2.parent = Some(2);
        tree.add_node(g2);

        tree.get_mut(1)
            .iter_mut()
            .for_each(|n| n.children = vec![2, 3]);
        tree.get_mut(2)
            .iter_mut()
            .for_each(|n| n.children = vec![4, 5]);

        let engine = layout_from_css(css, &tree, 1, 80, 20);

        let left_layout = engine.layout(2).unwrap_or_default();
        let right_layout = engine.layout(3).unwrap_or_default();
        assert_eq!(left_layout.width, 40);
        assert_eq!(right_layout.width, 40);

        let g1_layout = engine.layout(4).unwrap_or_default();
        let g2_layout = engine.layout(5).unwrap_or_default();
        assert_eq!(g1_layout.width, 20);
        assert_eq!(g2_layout.width, 20);
    }

    #[test]
    fn integration_box_model_spacing() {
        let css = r#"
            #root { display: flex; width: 80; height: 24; padding: 2; }
            #child { flex-grow: 1; }
        "#;
        let mut tree = WidgetTree::new();
        let mut root = WidgetNode::new(1, "Container");
        root.css_id = Some("root".into());
        tree.add_node(root);

        let mut child = WidgetNode::new(2, "Container");
        child.css_id = Some("child".into());
        child.parent = Some(1);
        tree.add_node(child);

        tree.get_mut(1)
            .iter_mut()
            .for_each(|n| n.children = vec![2]);

        let engine = layout_from_css(css, &tree, 1, 80, 24);

        let child_layout = engine.layout(2).unwrap_or_default();
        assert_eq!(child_layout.x, 2);
        assert_eq!(child_layout.y, 2);
        assert_eq!(child_layout.width, 76); // 80 - 2 - 2
        assert_eq!(child_layout.height, 20); // 24 - 2 - 2
    }

    #[test]
    fn integration_scroll_region_setup() {
        let css = r#"
            #root { overflow: scroll; width: 80; height: 24; }
        "#;
        let result = parse_stylesheet(css);
        assert!(result.is_ok());
        let stylesheet = match result {
            Ok(s) => s,
            Err(_) => unreachable!(),
        };
        let matcher = StyleMatcher::new(&stylesheet);

        let mut tree = WidgetTree::new();
        let mut root = WidgetNode::new(1, "Container");
        root.css_id = Some("root".into());
        tree.add_node(root);

        let matched = matcher.match_widget(&tree, 1);
        let computed = CascadeResolver::resolve(&matched);

        let (ox, oy) = super::scroll::extract_overflow(&computed);
        assert_eq!(ox, super::scroll::OverflowBehavior::Scroll);
        assert_eq!(oy, super::scroll::OverflowBehavior::Scroll);

        let mut scroll_mgr = ScrollManager::new();
        scroll_mgr.register(1, 200, 100, 80, 24);
        assert!(scroll_mgr.can_scroll_x(1));
        assert!(scroll_mgr.can_scroll_y(1));
    }

    #[test]
    fn integration_zero_size_area() {
        let css = r#"
            #root { display: flex; width: 0; height: 0; }
            Label { flex-grow: 1; }
        "#;
        let mut tree = WidgetTree::new();
        let mut root = WidgetNode::new(1, "Container");
        root.css_id = Some("root".into());
        tree.add_node(root);

        let mut label = WidgetNode::new(2, "Label");
        label.parent = Some(1);
        tree.add_node(label);

        let engine = layout_from_css(css, &tree, 1, 0, 0);

        let root_layout = engine.layout(1).unwrap_or_default();
        assert_eq!(root_layout.width, 0);
        assert_eq!(root_layout.height, 0);
    }

    #[test]
    fn integration_large_tree() {
        let css = r#"
            #root { display: flex; flex-direction: column; width: 100; height: 100; }
            .item { flex-grow: 1; }
        "#;
        let mut tree = WidgetTree::new();
        let mut root = WidgetNode::new(1, "Container");
        root.css_id = Some("root".into());
        tree.add_node(root);

        let mut child_ids = Vec::new();
        for i in 2..=101 {
            let mut child = WidgetNode::new(i, "Container");
            child.classes.push("item".into());
            child.parent = Some(1);
            tree.add_node(child);
            child_ids.push(i);
        }
        tree.get_mut(1)
            .iter_mut()
            .for_each(|n| n.children = child_ids.clone());

        let engine = layout_from_css(css, &tree, 1, 100, 100);

        // 100 children in column layout across 100 height = 1 each
        let l = engine.layout(2).unwrap_or_default();
        assert_eq!(l.height, 1);
        assert_eq!(l.width, 100);
    }

    #[test]
    fn integration_theme_affects_layout() {
        // Test that variable resolution can affect layout properties
        let css = r#"
            :root { $sidebar-width: 30; }
            #root { display: flex; width: 80; height: 24; }
            #sidebar { width: $sidebar-width; }
            #main { flex-grow: 1; }
        "#;
        let result = parse_stylesheet(css);
        assert!(result.is_ok());
        let stylesheet = match result {
            Ok(s) => s,
            Err(_) => unreachable!(),
        };

        let globals = crate::tcss::parser::extract_root_variables(&stylesheet);
        let env = crate::tcss::variable::VariableEnvironment::with_global(globals);
        let matcher = StyleMatcher::new(&stylesheet);

        let mut tree = WidgetTree::new();
        let mut root = WidgetNode::new(1, "Container");
        root.css_id = Some("root".into());
        tree.add_node(root);

        let mut sidebar = WidgetNode::new(2, "Container");
        sidebar.css_id = Some("sidebar".into());
        sidebar.parent = Some(1);
        tree.add_node(sidebar);

        let mut main = WidgetNode::new(3, "Container");
        main.css_id = Some("main".into());
        main.parent = Some(1);
        tree.add_node(main);

        tree.get_mut(1)
            .iter_mut()
            .for_each(|n| n.children = vec![2, 3]);

        // Resolve styles with variables
        let mut engine = LayoutEngine::new();

        for &wid in &[2, 3] {
            let matched = matcher.match_widget(&tree, wid);
            let computed = CascadeResolver::resolve_with_variables(&matched, &env);
            let style = computed_to_taffy(&computed);
            engine.add_node(wid, style).ok();
        }

        let matched = matcher.match_widget(&tree, 1);
        let computed = CascadeResolver::resolve_with_variables(&matched, &env);
        let style = computed_to_taffy(&computed);
        engine.add_node_with_children(1, style, &[2, 3]).ok();

        engine.set_root(1).ok();
        engine.compute(80, 24).ok();

        let sidebar_layout = engine.layout(2).unwrap_or_default();
        let main_layout = engine.layout(3).unwrap_or_default();
        assert_eq!(sidebar_layout.width, 30);
        assert_eq!(main_layout.width, 50);
    }
}
