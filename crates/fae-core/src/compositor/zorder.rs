//! Z-order selection for compositor regions.

use super::Layer;

/// Selects the topmost visible layer for a given horizontal interval on a given row.
///
/// Returns the index of the layer with the highest z-index that covers the interval
/// `[x_start, x_end)` on the given row. If multiple layers have the same z-index,
/// the later one in the layer list wins (insertion order).
///
/// Returns `None` if no layer covers this region (background shows through).
pub fn select_topmost(layers: &[Layer], row: u16, x_start: u16, x_end: u16) -> Option<usize> {
    let mut best_idx: Option<usize> = None;
    let mut best_z: i32 = i32::MIN;

    for (idx, layer) in layers.iter().enumerate() {
        // Check if layer intersects this row
        if !layer.contains_row(row) {
            continue;
        }

        // Check if layer horizontally overlaps [x_start, x_end)
        let layer_left = layer.region.position.x;
        let layer_right = layer.region.right();

        // Interval [x_start, x_end) overlaps [layer_left, layer_right) if:
        // x_start < layer_right AND x_end > layer_left
        if x_start >= layer_right || x_end <= layer_left {
            continue;
        }

        // This layer covers at least part of the interval
        // Select it if it has higher z-index, or same z-index but later insertion
        if layer.z_index > best_z || (layer.z_index == best_z && best_idx.is_some()) {
            best_idx = Some(idx);
            best_z = layer.z_index;
        }
    }

    best_idx
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Rect;

    #[test]
    fn no_layers_at_position() {
        let layers: Vec<Layer> = vec![];
        let result = select_topmost(&layers, 0, 10, 20);
        assert!(result.is_none());
    }

    #[test]
    fn single_layer_covers_region() {
        let layer = Layer::new(1, Rect::new(0, 0, 80, 24), 0, vec![]);
        let layers = vec![layer];

        let result = select_topmost(&layers, 5, 10, 20);
        assert!(result.is_some());
        let idx = match result {
            Some(i) => i,
            None => unreachable!(),
        };
        assert!(idx == 0);
    }

    #[test]
    fn two_overlapping_layers_higher_z_wins() {
        let layer1 = Layer::new(1, Rect::new(0, 0, 80, 24), 0, vec![]);
        let layer2 = Layer::new(2, Rect::new(10, 5, 30, 10), 10, vec![]);
        let layers = vec![layer1, layer2];

        let result = select_topmost(&layers, 7, 15, 25);
        assert!(result.is_some());
        let idx = match result {
            Some(i) => i,
            None => unreachable!(),
        };
        assert!(idx == 1); // layer2 has higher z_index
    }

    #[test]
    fn same_z_index_later_insertion_wins() {
        let layer1 = Layer::new(1, Rect::new(0, 0, 80, 24), 5, vec![]);
        let layer2 = Layer::new(2, Rect::new(10, 5, 30, 10), 5, vec![]);
        let layers = vec![layer1, layer2];

        let result = select_topmost(&layers, 7, 15, 25);
        assert!(result.is_some());
        let idx = match result {
            Some(i) => i,
            None => unreachable!(),
        };
        assert!(idx == 1); // same z_index, but layer2 inserted later
    }

    #[test]
    fn layer_partially_overlapping_still_selected() {
        let layer = Layer::new(1, Rect::new(10, 0, 20, 10), 0, vec![]);
        let layers = vec![layer];

        // Interval [5, 15) partially overlaps layer at [10, 30)
        let result = select_topmost(&layers, 5, 5, 15);
        assert!(result.is_some());
        let idx = match result {
            Some(i) => i,
            None => unreachable!(),
        };
        assert!(idx == 0);
    }

    #[test]
    fn layer_on_different_row_not_selected() {
        let layer = Layer::new(1, Rect::new(0, 10, 80, 5), 0, vec![]);
        let layers = vec![layer];

        // Layer is at y=10, asking for row 5
        let result = select_topmost(&layers, 5, 10, 20);
        assert!(result.is_none());
    }

    #[test]
    fn layer_before_interval_not_selected() {
        let layer = Layer::new(1, Rect::new(0, 0, 10, 10), 0, vec![]);
        let layers = vec![layer];

        // Layer ends at x=10, interval starts at x=10
        let result = select_topmost(&layers, 5, 10, 20);
        assert!(result.is_none());
    }

    #[test]
    fn layer_after_interval_not_selected() {
        let layer = Layer::new(1, Rect::new(30, 0, 20, 10), 0, vec![]);
        let layers = vec![layer];

        // Interval [10, 20) ends before layer starts at x=30
        let result = select_topmost(&layers, 5, 10, 20);
        assert!(result.is_none());
    }

    #[test]
    fn three_layers_different_z_indices() {
        let layer1 = Layer::new(1, Rect::new(0, 0, 80, 24), -5, vec![]);
        let layer2 = Layer::new(2, Rect::new(10, 5, 30, 10), 0, vec![]);
        let layer3 = Layer::new(3, Rect::new(20, 5, 20, 10), 15, vec![]);
        let layers = vec![layer1, layer2, layer3];

        let result = select_topmost(&layers, 7, 25, 35);
        assert!(result.is_some());
        let idx = match result {
            Some(i) => i,
            None => unreachable!(),
        };
        assert!(idx == 2); // layer3 has highest z_index (15)
    }

    #[test]
    fn negative_z_indices() {
        let layer1 = Layer::new(1, Rect::new(0, 0, 80, 24), -10, vec![]);
        let layer2 = Layer::new(2, Rect::new(10, 5, 30, 10), -5, vec![]);
        let layers = vec![layer1, layer2];

        let result = select_topmost(&layers, 7, 15, 25);
        assert!(result.is_some());
        let idx = match result {
            Some(i) => i,
            None => unreachable!(),
        };
        assert!(idx == 1); // layer2 has higher z_index (-5 > -10)
    }

    #[test]
    fn exact_interval_match() {
        let layer = Layer::new(1, Rect::new(10, 0, 20, 10), 0, vec![]);
        let layers = vec![layer];

        // Interval exactly matches layer bounds
        let result = select_topmost(&layers, 5, 10, 30);
        assert!(result.is_some());
        let idx = match result {
            Some(i) => i,
            None => unreachable!(),
        };
        assert!(idx == 0);
    }
}
