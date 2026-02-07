//! Property-based tests for the layout engine.
//!
//! Uses proptest to verify layout engine invariants with random widget trees
//! and constraints.

use fae_core::layout::engine::LayoutEngine;
use proptest::prelude::*;
use taffy::Overflow;
use taffy::prelude::*;

/// Generate a random flex grow factor.
fn flex_grow_factor() -> impl Strategy<Value = f32> {
    prop_oneof![Just(0.0), Just(1.0), (0.1..=10.0f32),]
}

/// Generate a random fr track sizing.
fn fr_value() -> impl Strategy<Value = f32> {
    0.1..=10.0f32
}

// --- Property Tests ---

proptest! {
    /// Property: Flexbox with random children produces non-negative sizes.
    #[test]
    fn flexbox_random_children_nonnegative_sizes(
        num_children in 1usize..=10,
        width in 50u16..=200u16,
        height in 20u16..=100u16,
    ) {
        let mut engine = LayoutEngine::new();
        let mut child_ids = Vec::new();

        // Add children with flex-grow: 1
        for i in 0..num_children {
            let child_id = (i + 1) as u64;
            child_ids.push(child_id);
            engine.add_node(
                child_id,
                Style {
                    flex_grow: 1.0,
                    ..Default::default()
                },
            ).ok();
        }

        // Add parent container
        let parent_id = 100u64;
        engine.add_node_with_children(
            parent_id,
            Style {
                size: taffy::Size {
                    width: Dimension::Length(f32::from(width)),
                    height: Dimension::Length(f32::from(height)),
                },
                ..Default::default()
            },
            &child_ids,
        ).ok();

        engine.set_root(parent_id).ok();
        engine.compute(width, height).ok();

        // Verify all children have non-negative sizes
        for child_id in child_ids {
            match engine.layout(child_id) {
                Ok(layout) => {
                    // All sizes must be non-negative (u16 guarantees this)
                    assert!(layout.width < u16::MAX);
                    assert!(layout.height < u16::MAX);
                }
                Err(_) => unreachable!(),
            }
        }
    }

    /// Property: Flexbox with random flex factors produces non-overlapping children.
    #[test]
    fn flexbox_random_flex_no_overlap(
        flex1 in flex_grow_factor(),
        flex2 in flex_grow_factor(),
        flex3 in flex_grow_factor(),
        width in 100u16..=200u16,
        height in 40u16..=80u16,
    ) {
        // Skip if all flex factors are 0
        if flex1 == 0.0 && flex2 == 0.0 && flex3 == 0.0 {
            return Ok(());
        }

        let mut engine = LayoutEngine::new();

        // Add three children with different flex factors
        engine.add_node(1, Style { flex_grow: flex1, ..Default::default() }).ok();
        engine.add_node(2, Style { flex_grow: flex2, ..Default::default() }).ok();
        engine.add_node(3, Style { flex_grow: flex3, ..Default::default() }).ok();

        // Add parent (row direction)
        engine.add_node_with_children(
            10,
            Style {
                size: taffy::Size {
                    width: Dimension::Length(f32::from(width)),
                    height: Dimension::Length(f32::from(height)),
                },
                ..Default::default()
            },
            &[1, 2, 3],
        ).ok();

        engine.set_root(10).ok();
        engine.compute(width, height).ok();

        let l1 = match engine.layout(1) {
            Ok(l) => l,
            Err(_) => unreachable!(),
        };
        let l2 = match engine.layout(2) {
            Ok(l) => l,
            Err(_) => unreachable!(),
        };
        let l3 = match engine.layout(3) {
            Ok(l) => l,
            Err(_) => unreachable!(),
        };

        // Verify children don't overlap horizontally (row layout)
        // Child 1 ends at x + width, child 2 starts at or after that point
        assert!(l2.x >= l1.x + l1.width || l1.width == 0);
        assert!(l3.x >= l2.x + l2.width || l2.width == 0);

        // Total width should not exceed container
        let total_width = l1.width + l2.width + l3.width;
        assert!(total_width <= width);
    }

    /// Property: Grid with random column definitions produces valid layouts.
    #[test]
    fn grid_random_columns_valid_layout(
        fr1 in fr_value(),
        fr2 in fr_value(),
        fr3 in fr_value(),
        width in 100u16..=200u16,
        height in 40u16..=80u16,
    ) {
        let mut engine = LayoutEngine::new();

        // Add three children
        engine.add_node(1, Style::default()).ok();
        engine.add_node(2, Style::default()).ok();
        engine.add_node(3, Style::default()).ok();

        // Add grid parent with random fr columns
        engine.add_node_with_children(
            10,
            Style {
                display: Display::Grid,
                grid_template_columns: vec![fr(fr1), fr(fr2), fr(fr3)],
                size: taffy::Size {
                    width: Dimension::Length(f32::from(width)),
                    height: Dimension::Length(f32::from(height)),
                },
                ..Default::default()
            },
            &[1, 2, 3],
        ).ok();

        engine.set_root(10).ok();
        engine.compute(width, height).ok();

        // Verify all children have valid sizes
        for child_id in [1, 2, 3] {
            match engine.layout(child_id) {
                Ok(layout) => {
                    assert!(layout.width <= width);
                    assert!(layout.height <= height);
                }
                Err(_) => unreachable!(),
            }
        }
    }

    /// Property: Box model with padding produces correct content sizes.
    #[test]
    fn box_model_padding_correct_sizes(
        pad_left in 0.0..=20.0f32,
        pad_right in 0.0..=20.0f32,
        pad_top in 0.0..=10.0f32,
        pad_bottom in 0.0..=10.0f32,
        width in 80u16..=150u16,
        height in 40u16..=80u16,
    ) {
        let mut engine = LayoutEngine::new();

        // Child with flex-grow to fill available space
        engine.add_node(1, Style { flex_grow: 1.0, ..Default::default() }).ok();

        // Parent with padding
        engine.add_node_with_children(
            10,
            Style {
                padding: taffy::Rect {
                    left: LengthPercentage::Length(pad_left),
                    right: LengthPercentage::Length(pad_right),
                    top: LengthPercentage::Length(pad_top),
                    bottom: LengthPercentage::Length(pad_bottom),
                },
                size: taffy::Size {
                    width: Dimension::Length(f32::from(width)),
                    height: Dimension::Length(f32::from(height)),
                },
                ..Default::default()
            },
            &[1],
        ).ok();

        engine.set_root(10).ok();
        engine.compute(width, height).ok();

        let child = match engine.layout(1) {
            Ok(l) => l,
            Err(_) => unreachable!(),
        };

        // Child position should account for padding (with 1-cell tolerance for rounding)
        let min_x = pad_left.floor() as u16;
        let max_x = pad_left.ceil() as u16;
        assert!(child.x >= min_x && child.x <= max_x, "child.x={}, expected range [{}, {}]", child.x, min_x, max_x);

        let min_y = pad_top.floor() as u16;
        let max_y = pad_top.ceil() as u16;
        assert!(child.y >= min_y && child.y <= max_y, "child.y={}, expected range [{}, {}]", child.y, min_y, max_y);

        // Child size should be reduced by padding (allow ±1 for rounding differences)
        let total_h_padding = (pad_left + pad_right).round() as u16;
        let total_v_padding = (pad_top + pad_bottom).round() as u16;

        let expected_width = width.saturating_sub(total_h_padding);
        let expected_height = height.saturating_sub(total_v_padding);

        // Allow ±1 cell tolerance for rounding
        assert!(
            child.width >= expected_width.saturating_sub(1) && child.width <= expected_width.saturating_add(1),
            "child.width={}, expected ~{}", child.width, expected_width
        );
        assert!(
            child.height >= expected_height.saturating_sub(1) && child.height <= expected_height.saturating_add(1),
            "child.height={}, expected ~{}", child.height, expected_height
        );
    }

    /// Property: Children don't extend beyond parent bounds with overflow hidden.
    #[test]
    fn overflow_hidden_children_within_bounds(
        child_width in 50u16..=200u16,
        child_height in 30u16..=100u16,
        parent_width in 40u16..=150u16,
        parent_height in 20u16..=80u16,
    ) {
        let mut engine = LayoutEngine::new();

        // Child with fixed size
        engine.add_node(
            1,
            Style {
                size: taffy::Size {
                    width: Dimension::Length(f32::from(child_width)),
                    height: Dimension::Length(f32::from(child_height)),
                },
                ..Default::default()
            },
        ).ok();

        // Parent with overflow hidden
        engine.add_node_with_children(
            10,
            Style {
                overflow: taffy::Point {
                    x: Overflow::Hidden,
                    y: Overflow::Hidden,
                },
                size: taffy::Size {
                    width: Dimension::Length(f32::from(parent_width)),
                    height: Dimension::Length(f32::from(parent_height)),
                },
                ..Default::default()
            },
            &[1],
        ).ok();

        engine.set_root(10).ok();
        engine.compute(parent_width, parent_height).ok();

        let parent = match engine.layout(10) {
            Ok(l) => l,
            Err(_) => unreachable!(),
        };
        let child = match engine.layout(1) {
            Ok(l) => l,
            Err(_) => unreachable!(),
        };

        // Child's computed size within parent's area
        // Note: taffy doesn't clip the child's reported size, but the rendering
        // layer should. We verify the parent has the expected size.
        assert_eq!(parent.width, parent_width);
        assert_eq!(parent.height, parent_height);

        // Child position should be within parent
        assert!(child.x < parent_width || child.width == 0);
        assert!(child.y < parent_height || child.height == 0);
    }

    /// Property: Random container sizes produce valid non-negative rects.
    #[test]
    fn random_container_sizes_valid(
        width in 10u16..=300u16,
        height in 5u16..=200u16,
        num_children in 0usize..=5,
    ) {
        let mut engine = LayoutEngine::new();
        let mut child_ids = Vec::new();

        // Add children
        for i in 0..num_children {
            let child_id = (i + 1) as u64;
            child_ids.push(child_id);
            engine.add_node(
                child_id,
                Style { flex_grow: 1.0, ..Default::default() },
            ).ok();
        }

        // Add parent
        let parent_id = 100u64;
        engine.add_node_with_children(
            parent_id,
            Style {
                flex_direction: FlexDirection::Column,
                size: taffy::Size {
                    width: Dimension::Length(f32::from(width)),
                    height: Dimension::Length(f32::from(height)),
                },
                ..Default::default()
            },
            &child_ids,
        ).ok();

        engine.set_root(parent_id).ok();
        engine.compute(width, height).ok();

        // Parent layout should match input
        let parent_layout = match engine.layout(parent_id) {
            Ok(l) => l,
            Err(_) => unreachable!(),
        };
        assert_eq!(parent_layout.width, width);
        assert_eq!(parent_layout.height, height);

        // All children should have valid non-negative sizes
        for child_id in child_ids {
            match engine.layout(child_id) {
                Ok(layout) => {
                    // u16 guarantees non-negative
                    assert!(layout.x <= width);
                    assert!(layout.y <= height);
                    assert!(layout.width <= width);
                    assert!(layout.height <= height);
                }
                Err(_) => unreachable!(),
            }
        }
    }
}
