//! Line composition — combines cuts, z-order, and chopping into final segment lists.

use crate::compositor::Layer;
use crate::compositor::chop::chop_segments;
use crate::compositor::cuts::find_cuts;
use crate::compositor::zorder::select_topmost;
use crate::segment::Segment;

/// Compose a single screen row by combining cut-finding, z-order selection, and segment chopping.
///
/// Returns a list of segments representing the final composed output for the given row.
///
/// # Algorithm
///
/// 1. Find cut points (x-offsets where layer edges create boundaries)
/// 2. For each interval between cuts:
///    - Select the topmost visible layer (if any)
///    - Extract and chop the relevant segments from that layer
///    - Append to the result
/// 3. Fill gaps with blank segments where no layer is visible
///
/// # Example
///
/// ```
/// use fae_core::compositor::layer::Layer;
/// use fae_core::compositor::compose::compose_line;
/// use fae_core::geometry::Rect;
/// use fae_core::segment::Segment;
///
/// let layer1 = Layer::new(1, Rect::new(0, 0, 40, 10), 0, vec![
///     vec![Segment::new("Hello")],
/// ]);
/// let layer2 = Layer::new(2, Rect::new(50, 0, 30, 10), 1, vec![
///     vec![Segment::new("World")],
/// ]);
///
/// let layers = vec![layer1, layer2];
/// let segments = compose_line(&layers, 0, 80);
///
/// // Result contains segments from both layers, with blank gap in between
/// assert!(segments.len() >= 3); // layer1, blank, layer2
/// ```
pub fn compose_line(layers: &[Layer], row: u16, screen_width: u16) -> Vec<Segment> {
    let mut result = Vec::new();

    // Find all cut points for this row
    let cuts = find_cuts(layers, row, screen_width);

    // If no cuts or only screen boundaries, return blank line
    if cuts.len() <= 1 {
        if screen_width > 0 {
            result.push(Segment::new(" ".repeat(screen_width as usize)));
        }
        return result;
    }

    // Process each interval between consecutive cuts
    for i in 0..cuts.len() - 1 {
        let x_start = cuts[i];
        let x_end = cuts[i + 1];
        let width = x_end - x_start;

        if width == 0 {
            continue;
        }

        // Find the topmost layer that covers this interval
        match select_topmost(layers, row, x_start, x_end) {
            Some(layer_idx) => {
                // Get the layer and its line for this row
                let layer = &layers[layer_idx];

                match layer.line_for_row(row) {
                    Some(line_segments) => {
                        // Chop the segments to extract just the interval we need
                        let chopped =
                            chop_segments(line_segments, layer.region.position.x, x_start, width);
                        result.extend(chopped);
                    }
                    None => {
                        // Layer claims to contain this row but has no line data
                        result.push(Segment::new(" ".repeat(width as usize)));
                    }
                }
            }
            None => {
                // No layer covers this interval — fill with blank
                result.push(Segment::new(" ".repeat(width as usize)));
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Rect;

    #[test]
    fn single_layer_full_width() {
        let layer = Layer::new(
            1,
            Rect::new(0, 0, 80, 10),
            0,
            vec![vec![Segment::new("Hello, world!")]],
        );
        let layers = vec![layer];

        let segments = compose_line(&layers, 0, 80);

        // Should contain the text from the layer
        let found_text = segments.iter().any(|s| s.text.contains("Hello, world!"));
        assert!(found_text);
    }

    #[test]
    fn two_layers_side_by_side() {
        let layer1 = Layer::new(
            1,
            Rect::new(0, 0, 40, 10),
            0,
            vec![vec![Segment::new("Left")]],
        );
        let layer2 = Layer::new(
            2,
            Rect::new(40, 0, 40, 10),
            0,
            vec![vec![Segment::new("Right")]],
        );
        let layers = vec![layer1, layer2];

        let segments = compose_line(&layers, 0, 80);

        // Should have at least 2 segments (one from each layer)
        assert!(segments.len() >= 2);

        // First segment should be from layer1
        let found_left = segments.iter().any(|s| s.text.contains("Left"));
        assert!(found_left);

        // Second segment should be from layer2
        let found_right = segments.iter().any(|s| s.text.contains("Right"));
        assert!(found_right);
    }

    #[test]
    fn overlapping_layers_topmost_wins() {
        let layer1 = Layer::new(
            1,
            Rect::new(0, 0, 80, 10),
            0,
            vec![vec![Segment::new("Background")]],
        );
        let layer2 = Layer::new(
            2,
            Rect::new(10, 0, 20, 10),
            10,
            vec![vec![Segment::new("Overlay")]],
        );
        let layers = vec![layer1, layer2];

        let segments = compose_line(&layers, 0, 80);

        // Should contain "Overlay" from the higher z-index layer
        let found_overlay = segments.iter().any(|s| s.text.contains("Overlay"));
        assert!(found_overlay);
    }

    #[test]
    fn gap_between_layers_filled_with_blank() {
        let layer1 = Layer::new(1, Rect::new(0, 0, 10, 10), 0, vec![vec![Segment::new("A")]]);
        let layer2 = Layer::new(
            2,
            Rect::new(30, 0, 10, 10),
            0,
            vec![vec![Segment::new("B")]],
        );
        let layers = vec![layer1, layer2];

        let segments = compose_line(&layers, 0, 80);

        // Should have segments: layer1, blank gap, layer2, blank to screen end
        assert!(segments.len() >= 3);

        // Check for blank segment (either empty or all spaces)
        let has_blank = segments.iter().any(|s| s.text.trim().is_empty());
        assert!(has_blank);
    }

    #[test]
    fn layer_extends_beyond_screen_clipped() {
        let layer = Layer::new(
            1,
            Rect::new(70, 0, 20, 10),
            0,
            vec![vec![Segment::new("Very long text that exceeds screen")]],
        );
        let layers = vec![layer];

        let segments = compose_line(&layers, 0, 80);

        // Should clip at screen edge (x=80)
        let total_width: usize = segments.iter().map(|s| s.width()).sum();
        assert!(total_width <= 80);
    }

    #[test]
    fn empty_row_no_layers() {
        let layers: Vec<Layer> = vec![];

        let segments = compose_line(&layers, 0, 80);

        // Should return single blank segment
        assert!(segments.len() == 1);
        assert!(segments[0].text.trim().is_empty());
        assert!(segments[0].width() == 80);
    }

    #[test]
    fn layer_on_different_row_ignored() {
        let layer = Layer::new(
            1,
            Rect::new(0, 10, 80, 5),
            0,
            vec![vec![Segment::new("Not on row 0")]],
        );
        let layers = vec![layer];

        let segments = compose_line(&layers, 0, 80);

        // Layer doesn't intersect row 0, should get blank line
        assert!(segments.len() == 1);
        assert!(segments[0].text.trim().is_empty());
    }

    #[test]
    fn zero_width_screen() {
        let layer = Layer::new(1, Rect::new(0, 0, 10, 10), 0, vec![vec![Segment::new("X")]]);
        let layers = vec![layer];

        let segments = compose_line(&layers, 0, 0);

        // Zero width screen returns empty result
        assert!(segments.is_empty());
    }

    #[test]
    fn styled_segment_preserved() {
        use crate::color::{Color, NamedColor};
        use crate::style::Style;

        let style = Style {
            fg: Some(Color::Named(NamedColor::Red)),
            ..Default::default()
        };

        let mut seg = Segment::new("Styled");
        seg.style = style.clone();

        let layer = Layer::new(1, Rect::new(0, 0, 20, 10), 0, vec![vec![seg.clone()]]);
        let layers = vec![layer];

        let segments = compose_line(&layers, 0, 80);

        // Find the styled segment
        let found_styled = segments.iter().any(|s| {
            s.text.contains("Styled")
                && s.style.fg.is_some()
                && matches!(s.style.fg, Some(Color::Named(NamedColor::Red)))
        });
        assert!(found_styled);
    }

    #[test]
    fn multiple_segments_in_layer() {
        let layer = Layer::new(
            1,
            Rect::new(0, 0, 80, 10),
            0,
            vec![vec![
                Segment::new("Hello "),
                Segment::new("world"),
                Segment::new("!"),
            ]],
        );
        let layers = vec![layer];

        let segments = compose_line(&layers, 0, 80);

        // Should contain all parts
        let combined: String = segments.iter().map(|s| s.text.as_str()).collect();
        assert!(combined.contains("Hello"));
        assert!(combined.contains("world"));
    }

    #[test]
    fn three_overlapping_layers_z_order() {
        let layer1 = Layer::new(
            1,
            Rect::new(0, 0, 80, 10),
            0,
            vec![vec![Segment::new("Bottom")]],
        );
        let layer2 = Layer::new(
            2,
            Rect::new(10, 0, 60, 10),
            5,
            vec![vec![Segment::new("Middle")]],
        );
        let layer3 = Layer::new(
            3,
            Rect::new(20, 0, 40, 10),
            10,
            vec![vec![Segment::new("Top")]],
        );
        let layers = vec![layer1, layer2, layer3];

        let segments = compose_line(&layers, 0, 80);

        // Should contain "Top" from highest z-index layer in the center
        let found_top = segments.iter().any(|s| s.text.contains("Top"));
        assert!(found_top);
    }
}
