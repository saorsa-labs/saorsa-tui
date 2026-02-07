//! Segment chopping for extracting sub-ranges from layer output.

use crate::segment::Segment;

/// Extracts a sub-range of segments from a layer's line.
///
/// Given a list of segments that represent a layer's content starting at `layer_x`,
/// this function extracts the portion that falls within the interval
/// `[cut_start, cut_start + cut_width)`.
///
/// If a segment straddles a cut boundary, it is split using `Segment::split_at`.
/// If the cut range extends beyond the segments, blank space is added.
///
/// # Arguments
///
/// * `segments` - The layer's segment list for this line
/// * `layer_x` - The x-offset where the layer starts on screen
/// * `cut_start` - The left edge of the cut interval
/// * `cut_width` - The width of the cut interval
///
/// # Returns
///
/// A list of segments covering exactly `cut_width` display columns,
/// starting at `cut_start`.
pub fn chop_segments(
    segments: &[Segment],
    layer_x: u16,
    cut_start: u16,
    cut_width: u16,
) -> Vec<Segment> {
    if cut_width == 0 {
        return vec![];
    }

    let cut_end = cut_start + cut_width;
    let mut result = Vec::new();
    let mut current_x = layer_x;

    for seg in segments {
        if seg.is_empty() || seg.is_control {
            continue;
        }

        let seg_width = seg.width() as u16;
        let seg_end = current_x + seg_width;

        // Segment entirely before cut range
        if seg_end <= cut_start {
            current_x = seg_end;
            continue;
        }

        // Segment entirely after cut range
        if current_x >= cut_end {
            break;
        }

        // Segment overlaps cut range
        let mut segment_to_add = seg.clone();

        // Trim left if segment starts before cut_start
        if current_x < cut_start {
            let trim_left = (cut_start - current_x) as usize;
            let (_left, right) = segment_to_add.split_at(trim_left);
            segment_to_add = right;
            current_x = cut_start;
        }

        // Trim right if segment extends beyond cut_end
        let remaining_width = (cut_end - current_x) as usize;
        if segment_to_add.width() > remaining_width {
            let (left, _right) = segment_to_add.split_at(remaining_width);
            segment_to_add = left;
        }

        if !segment_to_add.is_empty() {
            current_x += segment_to_add.width() as u16;
            result.push(segment_to_add);
        }

        if current_x >= cut_end {
            break;
        }
    }

    // Pad with blank if segments don't fill the cut range
    let total_width: usize = result.iter().map(|s| s.width()).sum();
    if (total_width as u16) < cut_width {
        let padding = " ".repeat((cut_width as usize) - total_width);
        result.push(Segment::new(padding));
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::style::Style;

    #[test]
    fn full_segment_within_cut_range() {
        let segments = vec![Segment::new("hello")];
        let result = chop_segments(&segments, 0, 0, 5);

        assert!(result.len() == 1);
        assert!(result[0].text == "hello");
    }

    #[test]
    fn segment_split_at_left_boundary() {
        let segments = vec![Segment::new("hello world")];
        let result = chop_segments(&segments, 0, 6, 5);

        // "hello world" starts at 0, cut starts at 6
        // Should extract "world"
        assert!(result.len() == 1);
        assert!(result[0].text == "world");
    }

    #[test]
    fn segment_split_at_right_boundary() {
        let segments = vec![Segment::new("hello world")];
        let result = chop_segments(&segments, 0, 0, 5);

        // Should extract "hello"
        assert!(result.len() == 1);
        assert!(result[0].text == "hello");
    }

    #[test]
    fn segment_split_at_both_boundaries() {
        let segments = vec![Segment::new("hello world testing")];
        let result = chop_segments(&segments, 0, 6, 5);

        // Extract "world"
        assert!(result.len() == 1);
        assert!(result[0].text == "world");
    }

    #[test]
    fn empty_segments_skipped() {
        let segments = vec![Segment::new(""), Segment::new("hello"), Segment::new("")];
        let result = chop_segments(&segments, 0, 0, 5);

        assert!(result.len() == 1);
        assert!(result[0].text == "hello");
    }

    #[test]
    fn cut_range_beyond_segment_end() {
        let segments = vec![Segment::new("hi")];
        let result = chop_segments(&segments, 0, 0, 10);

        // "hi" is 2 chars, need 10 → pad with 8 spaces
        assert!(result.len() == 2);
        assert!(result[0].text == "hi");
        assert!(result[1].text == "        ");
    }

    #[test]
    fn multiple_segments() {
        let segments = vec![Segment::new("hello "), Segment::new("world")];
        let result = chop_segments(&segments, 0, 0, 11);

        assert!(result.len() == 2);
        assert!(result[0].text == "hello ");
        assert!(result[1].text == "world");
    }

    #[test]
    fn layer_offset_before_cut() {
        let segments = vec![Segment::new("hello")];
        // Layer starts at x=10, cut is [15, 20)
        let result = chop_segments(&segments, 10, 15, 5);

        // "hello" at x=10 means "h"@10, "e"@11, "l"@12, "l"@13, "o"@14
        // Cut [15, 20) is beyond the segment, should be all padding
        assert!(result.len() == 1);
        assert!(result[0].text == "     ");
    }

    #[test]
    fn layer_offset_overlapping_cut() {
        let segments = vec![Segment::new("hello world")];
        // Layer starts at x=5, cut is [10, 15)
        let result = chop_segments(&segments, 5, 10, 5);

        // "hello world" at x=5 (11 chars total)
        // h=5, e=6, l=7, l=8, o=9, space=10, w=11, o=12, r=13, l=14, d=15
        // Position 10 is offset 5 in the segment (the space)
        // Extract 5 chars starting from position 10: " worl"
        assert!(result.len() == 1);
        assert!(result[0].text == " worl");
    }

    #[test]
    fn zero_width_cut() {
        let segments = vec![Segment::new("hello")];
        let result = chop_segments(&segments, 0, 0, 0);

        assert!(result.is_empty());
    }

    #[test]
    fn control_segments_ignored() {
        let segments = vec![
            Segment::new("hello"),
            Segment::control("ESC[1m"),
            Segment::new(" world"),
        ];
        let result = chop_segments(&segments, 0, 0, 11);

        // Control segment should be skipped
        assert!(result.len() == 2);
        assert!(result[0].text == "hello");
        assert!(result[1].text == " world");
    }

    #[test]
    fn styled_segment_preserved() {
        let style = Style::new().bold(true);
        let segments = vec![Segment::styled("hello", style.clone())];
        let result = chop_segments(&segments, 0, 0, 5);

        assert!(result.len() == 1);
        assert!(result[0].text == "hello");
        assert!(result[0].style.bold);
    }

    #[test]
    fn partial_overlap_at_start() {
        let segments = vec![Segment::new("testing")];
        // Segment at 0-6, cut at [5, 10)
        let result = chop_segments(&segments, 0, 5, 5);

        // Extract last 2 chars "ng" + pad with 3 spaces
        assert!(result.len() == 2);
        assert!(result[0].text == "ng");
        assert!(result[1].text == "   ");
    }

    #[test]
    fn partial_overlap_at_end() {
        let segments = vec![Segment::new("testing")];
        // "testing" is 7 chars. Layer at x=5, so segment spans [5, 12)
        // Caller should call chop for the actual overlap: [5, 7)
        let result = chop_segments(&segments, 5, 5, 2);

        // Extract first 2 chars: "te"
        assert!(result.len() == 1);
        assert!(result[0].text == "te");
    }
}
