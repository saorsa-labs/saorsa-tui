//! ANSI escape sequence renderer.
//!
//! Takes cell changes from the buffer diff and produces terminal output
//! with minimal escape sequences.

use std::fmt::Write;

use crate::buffer::CellChange;
use crate::color::{Color, NamedColor};
use crate::style::Style;
use crate::terminal::ColorSupport;

/// Renders cell changes into ANSI escape sequences.
pub struct Renderer {
    color_support: ColorSupport,
    synchronized_output: bool,
}

impl Renderer {
    /// Create a new renderer with the given color support level.
    pub fn new(color_support: ColorSupport, synchronized_output: bool) -> Self {
        Self {
            color_support,
            synchronized_output,
        }
    }

    /// Render a set of cell changes into a string of ANSI escape sequences.
    pub fn render(&self, changes: &[CellChange]) -> String {
        if changes.is_empty() {
            return String::new();
        }

        let mut output = String::with_capacity(changes.len() * 16);

        // Begin synchronized output if supported
        if self.synchronized_output {
            output.push_str("\x1b[?2026h");
        }

        let mut last_x: Option<u16> = None;
        let mut last_y: Option<u16> = None;
        let mut last_style = Style::default();
        let mut style_active = false;

        for change in changes {
            // Skip continuation cells — they don't produce output
            if change.cell.width == 0 {
                continue;
            }

            // Cursor positioning: only emit if not already at the right position
            let need_move = !matches!((last_x, last_y), (Some(lx), Some(ly)) if ly == change.y && lx == change.x);
            if need_move {
                // ANSI cursor position is 1-based
                let _ = write!(output, "\x1b[{};{}H", change.y + 1, change.x + 1);
            }

            // Style diffing: only emit changed attributes
            self.write_style_diff(&mut output, &last_style, &change.cell.style, style_active);
            last_style = change.cell.style.clone();
            style_active = true;

            // Write the grapheme
            output.push_str(&change.cell.grapheme);

            // Track cursor position (advances by cell width)
            last_x = Some(change.x + u16::from(change.cell.width));
            last_y = Some(change.y);
        }

        // Reset style at the end
        if style_active && !last_style.is_empty() {
            output.push_str("\x1b[0m");
        }

        // End synchronized output if supported
        if self.synchronized_output {
            output.push_str("\x1b[?2026l");
        }

        output
    }

    /// Write the minimal SGR sequence to transition from `prev` to `next` style.
    fn write_style_diff(&self, output: &mut String, prev: &Style, next: &Style, active: bool) {
        if !active || needs_reset(prev, next) {
            // Full reset if we turned off an attribute or if not yet active
            if active && !prev.is_empty() {
                output.push_str("\x1b[0m");
            }
            // Apply all attributes from next
            self.write_full_style(output, next);
            return;
        }

        // Incremental: only emit changed attributes
        if prev.fg != next.fg {
            self.write_fg(output, &next.fg);
        }
        if prev.bg != next.bg {
            self.write_bg(output, &next.bg);
        }
        if !prev.bold && next.bold {
            output.push_str("\x1b[1m");
        }
        if !prev.dim && next.dim {
            output.push_str("\x1b[2m");
        }
        if !prev.italic && next.italic {
            output.push_str("\x1b[3m");
        }
        if !prev.underline && next.underline {
            output.push_str("\x1b[4m");
        }
        if !prev.reverse && next.reverse {
            output.push_str("\x1b[7m");
        }
        if !prev.strikethrough && next.strikethrough {
            output.push_str("\x1b[9m");
        }
    }

    /// Write a full style (all attributes from scratch).
    fn write_full_style(&self, output: &mut String, style: &Style) {
        self.write_fg(output, &style.fg);
        self.write_bg(output, &style.bg);
        if style.bold {
            output.push_str("\x1b[1m");
        }
        if style.dim {
            output.push_str("\x1b[2m");
        }
        if style.italic {
            output.push_str("\x1b[3m");
        }
        if style.underline {
            output.push_str("\x1b[4m");
        }
        if style.reverse {
            output.push_str("\x1b[7m");
        }
        if style.strikethrough {
            output.push_str("\x1b[9m");
        }
    }

    /// Write a foreground color SGR sequence.
    fn write_fg(&self, output: &mut String, color: &Option<Color>) {
        match color {
            None => {}
            Some(c) => {
                let downgraded = self.downgrade_color(c);
                write_fg_color(output, &downgraded);
            }
        }
    }

    /// Write a background color SGR sequence.
    fn write_bg(&self, output: &mut String, color: &Option<Color>) {
        match color {
            None => {}
            Some(c) => {
                let downgraded = self.downgrade_color(c);
                write_bg_color(output, &downgraded);
            }
        }
    }

    /// Downgrade a color to match the terminal's color support level.
    fn downgrade_color<'a>(&self, color: &'a Color) -> std::borrow::Cow<'a, Color> {
        match self.color_support {
            ColorSupport::TrueColor => std::borrow::Cow::Borrowed(color),
            ColorSupport::Extended256 => match color {
                Color::Rgb { r, g, b } => {
                    std::borrow::Cow::Owned(Color::Indexed(rgb_to_256(*r, *g, *b)))
                }
                _ => std::borrow::Cow::Borrowed(color),
            },
            ColorSupport::Basic16 => match color {
                Color::Rgb { r, g, b } => {
                    std::borrow::Cow::Owned(Color::Named(rgb_to_named(*r, *g, *b)))
                }
                Color::Indexed(i) => std::borrow::Cow::Owned(Color::Named(index_to_named(*i))),
                _ => std::borrow::Cow::Borrowed(color),
            },
            ColorSupport::NoColor => std::borrow::Cow::Owned(Color::Reset),
        }
    }
}

/// Check if transitioning from `prev` to `next` requires a full SGR reset.
/// This is needed when we're turning OFF an attribute (e.g., bold was on, now off).
fn needs_reset(prev: &Style, next: &Style) -> bool {
    (prev.bold && !next.bold)
        || (prev.dim && !next.dim)
        || (prev.italic && !next.italic)
        || (prev.underline && !next.underline)
        || (prev.reverse && !next.reverse)
        || (prev.strikethrough && !next.strikethrough)
}

/// Write an SGR foreground color escape sequence.
fn write_fg_color(output: &mut String, color: &Color) {
    match color {
        Color::Rgb { r, g, b } => {
            let _ = write!(output, "\x1b[38;2;{r};{g};{b}m");
        }
        Color::Indexed(i) => {
            let _ = write!(output, "\x1b[38;5;{i}m");
        }
        Color::Named(n) => {
            let _ = write!(output, "\x1b[{}m", named_fg_code(n));
        }
        Color::Reset => {
            output.push_str("\x1b[39m");
        }
    }
}

/// Write an SGR background color escape sequence.
fn write_bg_color(output: &mut String, color: &Color) {
    match color {
        Color::Rgb { r, g, b } => {
            let _ = write!(output, "\x1b[48;2;{r};{g};{b}m");
        }
        Color::Indexed(i) => {
            let _ = write!(output, "\x1b[48;5;{i}m");
        }
        Color::Named(n) => {
            let _ = write!(output, "\x1b[{}m", named_bg_code(n));
        }
        Color::Reset => {
            output.push_str("\x1b[49m");
        }
    }
}

/// Get the SGR code for a named foreground color.
fn named_fg_code(color: &NamedColor) -> u8 {
    match color {
        NamedColor::Black => 30,
        NamedColor::Red => 31,
        NamedColor::Green => 32,
        NamedColor::Yellow => 33,
        NamedColor::Blue => 34,
        NamedColor::Magenta => 35,
        NamedColor::Cyan => 36,
        NamedColor::White => 37,
        NamedColor::BrightBlack => 90,
        NamedColor::BrightRed => 91,
        NamedColor::BrightGreen => 92,
        NamedColor::BrightYellow => 93,
        NamedColor::BrightBlue => 94,
        NamedColor::BrightMagenta => 95,
        NamedColor::BrightCyan => 96,
        NamedColor::BrightWhite => 97,
    }
}

/// Get the SGR code for a named background color.
fn named_bg_code(color: &NamedColor) -> u8 {
    match color {
        NamedColor::Black => 40,
        NamedColor::Red => 41,
        NamedColor::Green => 42,
        NamedColor::Yellow => 43,
        NamedColor::Blue => 44,
        NamedColor::Magenta => 45,
        NamedColor::Cyan => 46,
        NamedColor::White => 47,
        NamedColor::BrightBlack => 100,
        NamedColor::BrightRed => 101,
        NamedColor::BrightGreen => 102,
        NamedColor::BrightYellow => 103,
        NamedColor::BrightBlue => 104,
        NamedColor::BrightMagenta => 105,
        NamedColor::BrightCyan => 106,
        NamedColor::BrightWhite => 107,
    }
}

/// Convert RGB to the nearest 256-color palette index.
///
/// The 256-color palette is:
/// - 0-7: standard colors
/// - 8-15: bright colors
/// - 16-231: 6x6x6 color cube
/// - 232-255: grayscale ramp
pub fn rgb_to_256(r: u8, g: u8, b: u8) -> u8 {
    // Check if it's close to a grayscale value
    if r == g && g == b {
        if r < 8 {
            return 16; // black
        }
        if r > 248 {
            return 231; // white
        }
        return (((u16::from(r) - 8) * 24 / 240) as u8) + 232;
    }

    // Map to 6x6x6 color cube (indices 16-231)
    let ri = color_cube_index(r);
    let gi = color_cube_index(g);
    let bi = color_cube_index(b);
    16 + 36 * ri + 6 * gi + bi
}

/// Map an 8-bit color channel to a 6-level color cube index.
fn color_cube_index(val: u8) -> u8 {
    if val < 48 {
        0
    } else if val < 115 {
        1
    } else {
        ((u16::from(val) - 35) / 40) as u8
    }
}

/// Convert RGB to the nearest named 16-color ANSI color.
pub fn rgb_to_named(r: u8, g: u8, b: u8) -> NamedColor {
    // Simple approach: find nearest ANSI color by Euclidean distance
    let candidates: [(NamedColor, (u8, u8, u8)); 16] = [
        (NamedColor::Black, (0, 0, 0)),
        (NamedColor::Red, (128, 0, 0)),
        (NamedColor::Green, (0, 128, 0)),
        (NamedColor::Yellow, (128, 128, 0)),
        (NamedColor::Blue, (0, 0, 128)),
        (NamedColor::Magenta, (128, 0, 128)),
        (NamedColor::Cyan, (0, 128, 128)),
        (NamedColor::White, (192, 192, 192)),
        (NamedColor::BrightBlack, (128, 128, 128)),
        (NamedColor::BrightRed, (255, 0, 0)),
        (NamedColor::BrightGreen, (0, 255, 0)),
        (NamedColor::BrightYellow, (255, 255, 0)),
        (NamedColor::BrightBlue, (0, 0, 255)),
        (NamedColor::BrightMagenta, (255, 0, 255)),
        (NamedColor::BrightCyan, (0, 255, 255)),
        (NamedColor::BrightWhite, (255, 255, 255)),
    ];

    let mut best = NamedColor::White;
    let mut best_dist = u32::MAX;
    for (name, (cr, cg, cb)) in &candidates {
        let dr = i32::from(r) - i32::from(*cr);
        let dg = i32::from(g) - i32::from(*cg);
        let db = i32::from(b) - i32::from(*cb);
        let dist = (dr * dr + dg * dg + db * db) as u32;
        if dist < best_dist {
            best_dist = dist;
            best = *name;
        }
    }
    best
}

/// Convert a 256-color index to the nearest named 16-color.
fn index_to_named(idx: u8) -> NamedColor {
    match idx {
        0 => NamedColor::Black,
        1 => NamedColor::Red,
        2 => NamedColor::Green,
        3 => NamedColor::Yellow,
        4 => NamedColor::Blue,
        5 => NamedColor::Magenta,
        6 => NamedColor::Cyan,
        7 => NamedColor::White,
        8 => NamedColor::BrightBlack,
        9 => NamedColor::BrightRed,
        10 => NamedColor::BrightGreen,
        11 => NamedColor::BrightYellow,
        12 => NamedColor::BrightBlue,
        13 => NamedColor::BrightMagenta,
        14 => NamedColor::BrightCyan,
        15 => NamedColor::BrightWhite,
        16..=231 => {
            // Color cube: convert index back to approximate RGB
            let idx = idx - 16;
            let b_idx = idx % 6;
            let g_idx = (idx / 6) % 6;
            let r_idx = idx / 36;
            let r = if r_idx == 0 { 0 } else { 55 + 40 * r_idx };
            let g = if g_idx == 0 { 0 } else { 55 + 40 * g_idx };
            let b = if b_idx == 0 { 0 } else { 55 + 40 * b_idx };
            rgb_to_named(r, g, b)
        }
        _ => {
            // Grayscale ramp: 232-255 → 8, 18, 28, ..., 238
            let gray = 8 + 10 * (idx - 232);
            rgb_to_named(gray, gray, gray)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::CellChange;
    use crate::cell::Cell;

    #[test]
    fn render_empty_changes() {
        let renderer = Renderer::new(ColorSupport::TrueColor, false);
        let output = renderer.render(&[]);
        assert!(output.is_empty());
    }

    #[test]
    fn render_cursor_position() {
        let renderer = Renderer::new(ColorSupport::TrueColor, false);
        let changes = vec![CellChange {
            x: 5,
            y: 3,
            cell: Cell::new("A", Style::default()),
        }];
        let output = renderer.render(&changes);
        // Row 4, Col 6 (1-based)
        assert!(output.contains("\x1b[4;6H"));
        assert!(output.contains('A'));
    }

    #[test]
    fn render_adjacent_cells_no_redundant_move() {
        let renderer = Renderer::new(ColorSupport::TrueColor, false);
        let changes = vec![
            CellChange {
                x: 0,
                y: 0,
                cell: Cell::new("A", Style::default()),
            },
            CellChange {
                x: 1,
                y: 0,
                cell: Cell::new("B", Style::default()),
            },
        ];
        let output = renderer.render(&changes);
        // Should have one cursor position, then A, then B without another position
        let move_count = output.matches("\x1b[").count();
        // One for the initial cursor position
        assert_eq!(move_count, 1, "output: {output:?}");
    }

    #[test]
    fn render_fg_truecolor() {
        let renderer = Renderer::new(ColorSupport::TrueColor, false);
        let style = Style::new().fg(Color::Rgb {
            r: 255,
            g: 128,
            b: 0,
        });
        let changes = vec![CellChange {
            x: 0,
            y: 0,
            cell: Cell::new("X", style),
        }];
        let output = renderer.render(&changes);
        assert!(output.contains("\x1b[38;2;255;128;0m"));
    }

    #[test]
    fn render_bg_truecolor() {
        let renderer = Renderer::new(ColorSupport::TrueColor, false);
        let style = Style::new().bg(Color::Rgb {
            r: 0,
            g: 128,
            b: 255,
        });
        let changes = vec![CellChange {
            x: 0,
            y: 0,
            cell: Cell::new("X", style),
        }];
        let output = renderer.render(&changes);
        assert!(output.contains("\x1b[48;2;0;128;255m"));
    }

    #[test]
    fn render_bold_italic() {
        let renderer = Renderer::new(ColorSupport::TrueColor, false);
        let style = Style::new().bold(true).italic(true);
        let changes = vec![CellChange {
            x: 0,
            y: 0,
            cell: Cell::new("X", style),
        }];
        let output = renderer.render(&changes);
        assert!(output.contains("\x1b[1m")); // bold
        assert!(output.contains("\x1b[3m")); // italic
    }

    #[test]
    fn render_named_color() {
        let renderer = Renderer::new(ColorSupport::TrueColor, false);
        let style = Style::new().fg(Color::Named(NamedColor::Red));
        let changes = vec![CellChange {
            x: 0,
            y: 0,
            cell: Cell::new("X", style),
        }];
        let output = renderer.render(&changes);
        assert!(output.contains("\x1b[31m")); // red fg
    }

    #[test]
    fn render_indexed_color() {
        let renderer = Renderer::new(ColorSupport::TrueColor, false);
        let style = Style::new().fg(Color::Indexed(42));
        let changes = vec![CellChange {
            x: 0,
            y: 0,
            cell: Cell::new("X", style),
        }];
        let output = renderer.render(&changes);
        assert!(output.contains("\x1b[38;5;42m"));
    }

    #[test]
    fn render_style_reset_at_end() {
        let renderer = Renderer::new(ColorSupport::TrueColor, false);
        let style = Style::new().bold(true);
        let changes = vec![CellChange {
            x: 0,
            y: 0,
            cell: Cell::new("X", style),
        }];
        let output = renderer.render(&changes);
        assert!(output.ends_with("\x1b[0m"));
    }

    #[test]
    fn render_no_reset_for_default_style() {
        let renderer = Renderer::new(ColorSupport::TrueColor, false);
        let changes = vec![CellChange {
            x: 0,
            y: 0,
            cell: Cell::new("X", Style::default()),
        }];
        let output = renderer.render(&changes);
        assert!(!output.contains("\x1b[0m"));
    }

    #[test]
    fn render_skip_continuation_cells() {
        let renderer = Renderer::new(ColorSupport::TrueColor, false);
        let changes = vec![
            CellChange {
                x: 0,
                y: 0,
                cell: Cell::new("\u{4e16}", Style::default()), // 世 width=2
            },
            CellChange {
                x: 1,
                y: 0,
                cell: Cell::continuation(), // width=0
            },
        ];
        let output = renderer.render(&changes);
        // The continuation cell should not appear in output
        assert!(output.contains("\u{4e16}"));
        // Should only have one cursor move
        let esc_count = output.matches("\x1b[").count();
        assert_eq!(esc_count, 1);
    }

    #[test]
    fn synchronized_output_wrapping() {
        let renderer = Renderer::new(ColorSupport::TrueColor, true);
        let changes = vec![CellChange {
            x: 0,
            y: 0,
            cell: Cell::new("A", Style::default()),
        }];
        let output = renderer.render(&changes);
        assert!(output.starts_with("\x1b[?2026h"));
        assert!(output.ends_with("\x1b[?2026l"));
    }

    #[test]
    fn no_sync_when_disabled() {
        let renderer = Renderer::new(ColorSupport::TrueColor, false);
        let changes = vec![CellChange {
            x: 0,
            y: 0,
            cell: Cell::new("A", Style::default()),
        }];
        let output = renderer.render(&changes);
        assert!(!output.contains("\x1b[?2026h"));
        assert!(!output.contains("\x1b[?2026l"));
    }

    // Color downgrading tests

    #[test]
    fn truecolor_passthrough() {
        let renderer = Renderer::new(ColorSupport::TrueColor, false);
        let style = Style::new().fg(Color::Rgb {
            r: 100,
            g: 200,
            b: 50,
        });
        let changes = vec![CellChange {
            x: 0,
            y: 0,
            cell: Cell::new("X", style),
        }];
        let output = renderer.render(&changes);
        assert!(output.contains("\x1b[38;2;100;200;50m"));
    }

    #[test]
    fn truecolor_to_256() {
        let renderer = Renderer::new(ColorSupport::Extended256, false);
        let style = Style::new().fg(Color::Rgb { r: 255, g: 0, b: 0 });
        let changes = vec![CellChange {
            x: 0,
            y: 0,
            cell: Cell::new("X", style),
        }];
        let output = renderer.render(&changes);
        // Should use 256-color index, not truecolor
        assert!(output.contains("\x1b[38;5;"));
        assert!(!output.contains("\x1b[38;2;"));
    }

    #[test]
    fn truecolor_to_16() {
        let renderer = Renderer::new(ColorSupport::Basic16, false);
        let style = Style::new().fg(Color::Rgb { r: 255, g: 0, b: 0 });
        let changes = vec![CellChange {
            x: 0,
            y: 0,
            cell: Cell::new("X", style),
        }];
        let output = renderer.render(&changes);
        // Should use named color code (bright red = 91)
        assert!(output.contains("\x1b[91m"));
    }

    #[test]
    fn no_color_strips_all() {
        let renderer = Renderer::new(ColorSupport::NoColor, false);
        let style = Style::new()
            .fg(Color::Rgb { r: 255, g: 0, b: 0 })
            .bg(Color::Named(NamedColor::Blue));
        let changes = vec![CellChange {
            x: 0,
            y: 0,
            cell: Cell::new("X", style),
        }];
        let output = renderer.render(&changes);
        // Should use reset colors, not any specific color
        assert!(output.contains("\x1b[39m")); // fg reset
        assert!(output.contains("\x1b[49m")); // bg reset
    }

    // Color conversion unit tests

    #[test]
    fn rgb_to_256_pure_red() {
        let idx = rgb_to_256(255, 0, 0);
        // Pure red in color cube: r=5, g=0, b=0 → 16 + 36*5 + 6*0 + 0 = 196
        assert_eq!(idx, 196);
    }

    #[test]
    fn rgb_to_256_grayscale() {
        let idx = rgb_to_256(128, 128, 128);
        // Grayscale: (128-8)*24/240 = 12 → 232 + 12 = 244
        assert_eq!(idx, 244);
    }

    #[test]
    fn rgb_to_256_black() {
        let idx = rgb_to_256(0, 0, 0);
        assert_eq!(idx, 16); // near-black in grayscale
    }

    #[test]
    fn rgb_to_named_pure_red() {
        let named = rgb_to_named(255, 0, 0);
        assert_eq!(named, NamedColor::BrightRed);
    }

    #[test]
    fn rgb_to_named_pure_black() {
        let named = rgb_to_named(0, 0, 0);
        assert_eq!(named, NamedColor::Black);
    }

    #[test]
    fn rgb_to_named_pure_white() {
        let named = rgb_to_named(255, 255, 255);
        assert_eq!(named, NamedColor::BrightWhite);
    }
}
