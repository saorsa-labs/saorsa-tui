//! Helper functions for snapshot testing.

/// Render segments to a text grid for snapshot testing.
///
/// Each line of segments is concatenated and padded to width with spaces.
/// The result is then padded to height with empty lines.
#[allow(dead_code)]
pub fn render_to_text(lines: &[Vec<saorsa_tui::Segment>], width: u16, height: u16) -> String {
    let mut result = String::new();
    let width = width as usize;
    let height = height as usize;

    for (row_idx, line) in lines.iter().enumerate() {
        if row_idx >= height {
            break;
        }

        let mut row_text = String::new();
        for segment in line {
            row_text.push_str(&segment.text);
        }

        // Pad to width
        let current_len = row_text.len();
        if current_len < width {
            for _ in 0..(width - current_len) {
                row_text.push(' ');
            }
        } else if current_len > width {
            row_text.truncate(width);
        }

        result.push_str(&row_text);
        result.push('\n');
    }

    // Pad to height with empty lines
    let rendered_rows = lines.len().min(height);
    for _ in rendered_rows..height {
        for _ in 0..width {
            result.push(' ');
        }
        result.push('\n');
    }

    result
}
