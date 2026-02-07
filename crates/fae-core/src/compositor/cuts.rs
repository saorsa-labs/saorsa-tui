//! Cut-finding algorithm for compositor row decomposition.
//!
//! For each screen row, finds all x-offsets where widget layer edges create
//! boundaries. These "cuts" define intervals `[cuts[i]..cuts[i+1]]` that
//! partition the row into regions where the same set of layers is visible.

use crate::compositor::layer::Layer;

/// Find cut points (x-offsets) for a given row across all layers.
///
/// Returns a sorted, deduplicated list of x-offsets where layer edges
/// create boundaries. These define intervals: `[cuts[0]..cuts[1]],
/// [cuts[1]..cuts[2]], ...`
///
/// Always includes screen boundaries (0 and `screen_width`) in the result.
/// For each layer that intersects the given row, includes its left and right
/// edges, clamped to `[0, screen_width]`.
///
/// # Example
///
/// ```
/// use fae_core::compositor::layer::Layer;
/// use fae_core::compositor::cuts::find_cuts;
/// use fae_core::geometry::Rect;
///
/// let layers = vec![
///     Layer::new(1, Rect::new(10, 0, 20, 5), 0, vec![]),
///     Layer::new(2, Rect::new(40, 0, 10, 5), 1, vec![]),
/// ];
///
/// let cuts = find_cuts(&layers, 0, 80);
/// // Expected: [0, 10, 30, 40, 50, 80]
/// assert_eq!(cuts, vec![0, 10, 30, 40, 50, 80]);
/// ```
pub fn find_cuts(layers: &[Layer], row: u16, screen_width: u16) -> Vec<u16> {
    let mut cuts = Vec::new();

    // Always include screen boundaries
    cuts.push(0);
    if screen_width > 0 {
        cuts.push(screen_width);
    }

    // Add left and right edges of layers that intersect this row
    for layer in layers {
        if layer.contains_row(row) {
            let left = layer.region.position.x;
            let right = left.saturating_add(layer.region.size.width);

            // Clamp to screen bounds and add unique cuts
            if left < screen_width {
                cuts.push(left);
            }
            if right <= screen_width {
                cuts.push(right);
            } else if right > screen_width {
                // If right extends beyond screen, screen_width is already added
                // Do nothing (screen_width already in cuts)
            }
        }
    }

    // Deduplicate and sort
    cuts.sort_unstable();
    cuts.dedup();
    cuts
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Rect;

    #[test]
    fn no_layers_returns_screen_bounds() {
        let layers: Vec<Layer> = vec![];
        let cuts = find_cuts(&layers, 0, 80);
        assert!(cuts == vec![0, 80]);
    }

    #[test]
    fn single_layer_full_width() {
        let layers = vec![Layer::new(1, Rect::new(0, 0, 80, 24), 0, vec![])];
        let cuts = find_cuts(&layers, 0, 80);
        assert!(cuts == vec![0, 80]);
    }

    #[test]
    fn single_layer_centered() {
        let layers = vec![Layer::new(1, Rect::new(10, 0, 20, 5), 0, vec![])];
        let cuts = find_cuts(&layers, 0, 80);
        assert!(cuts == vec![0, 10, 30, 80]);
    }

    #[test]
    fn two_non_overlapping() {
        let layers = vec![
            Layer::new(1, Rect::new(0, 0, 10, 5), 0, vec![]),
            Layer::new(2, Rect::new(20, 0, 10, 5), 1, vec![]),
        ];
        let cuts = find_cuts(&layers, 0, 80);
        assert!(cuts == vec![0, 10, 20, 30, 80]);
    }

    #[test]
    fn two_overlapping() {
        let layers = vec![
            Layer::new(1, Rect::new(0, 0, 20, 5), 0, vec![]),
            Layer::new(2, Rect::new(10, 0, 20, 5), 1, vec![]),
        ];
        let cuts = find_cuts(&layers, 0, 80);
        assert!(cuts == vec![0, 10, 20, 30, 80]);
    }

    #[test]
    fn layer_at_screen_edge() {
        let layers = vec![Layer::new(1, Rect::new(70, 0, 10, 5), 0, vec![])];
        let cuts = find_cuts(&layers, 0, 80);
        assert!(cuts == vec![0, 70, 80]);
    }

    #[test]
    fn layer_on_different_row() {
        let layers = vec![Layer::new(1, Rect::new(10, 5, 20, 5), 0, vec![])];
        let cuts = find_cuts(&layers, 0, 80);
        // Layer is at y=5, row 0 doesn't intersect it
        assert!(cuts == vec![0, 80]);
    }

    #[test]
    fn zero_width_screen() {
        let layers = vec![Layer::new(1, Rect::new(0, 0, 10, 5), 0, vec![])];
        let cuts = find_cuts(&layers, 0, 0);
        // screen_width = 0, only boundary is 0
        assert!(cuts == vec![0]);
    }

    #[test]
    fn layer_extends_beyond_screen() {
        let layers = vec![Layer::new(1, Rect::new(70, 0, 20, 5), 0, vec![])];
        let cuts = find_cuts(&layers, 0, 80);
        // Layer right edge (70 + 20 = 90) exceeds screen_width (80)
        // Should clamp to [0, 70, 80]
        assert!(cuts == vec![0, 70, 80]);
    }
}
