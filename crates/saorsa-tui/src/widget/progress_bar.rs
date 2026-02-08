//! Progress bar widget with determinate and indeterminate modes.
//!
//! Supports a filled/empty bar with percentage display (determinate mode)
//! and an animated wave pattern (indeterminate mode).

use crate::buffer::ScreenBuffer;
use crate::cell::Cell;
use crate::geometry::Rect;
use crate::style::Style;

use super::{BorderStyle, Widget};

/// Progress bar mode.
#[derive(Clone, Debug, PartialEq)]
pub enum ProgressMode {
    /// Determinate progress (0.0 to 1.0).
    Determinate(f32),
    /// Indeterminate animated progress.
    Indeterminate {
        /// Current animation phase.
        phase: usize,
    },
}

/// A progress bar widget.
///
/// Displays a horizontal bar showing progress. In determinate mode,
/// the bar fills proportionally. In indeterminate mode, an animated
/// wave pattern moves across the bar.
pub struct ProgressBar {
    /// Current progress mode.
    mode: ProgressMode,
    /// Style for the filled portion.
    filled_style: Style,
    /// Style for the empty portion.
    empty_style: Style,
    /// Style for the percentage label.
    label_style: Style,
    /// Whether to show the percentage text.
    show_percentage: bool,
    /// Border style.
    border: BorderStyle,
}

/// Block characters for indeterminate animation (thin to thick).
const WAVE_CHARS: &[&str] = &["▏", "▎", "▍", "▌", "▋", "▊", "▉", "█"];

impl ProgressBar {
    /// Create a determinate progress bar.
    ///
    /// Progress is clamped to 0.0..=1.0.
    pub fn new(progress: f32) -> Self {
        Self {
            mode: ProgressMode::Determinate(progress.clamp(0.0, 1.0)),
            filled_style: Style::default().reverse(true),
            empty_style: Style::default(),
            label_style: Style::default(),
            show_percentage: true,
            border: BorderStyle::None,
        }
    }

    /// Create an indeterminate progress bar.
    pub fn indeterminate() -> Self {
        Self {
            mode: ProgressMode::Indeterminate { phase: 0 },
            filled_style: Style::default().reverse(true),
            empty_style: Style::default(),
            label_style: Style::default(),
            show_percentage: false,
            border: BorderStyle::None,
        }
    }

    /// Set the filled portion style.
    #[must_use]
    pub fn with_filled_style(mut self, style: Style) -> Self {
        self.filled_style = style;
        self
    }

    /// Set the empty portion style.
    #[must_use]
    pub fn with_empty_style(mut self, style: Style) -> Self {
        self.empty_style = style;
        self
    }

    /// Set the percentage label style.
    #[must_use]
    pub fn with_label_style(mut self, style: Style) -> Self {
        self.label_style = style;
        self
    }

    /// Enable or disable percentage display.
    #[must_use]
    pub fn with_show_percentage(mut self, show: bool) -> Self {
        self.show_percentage = show;
        self
    }

    /// Set the border style.
    #[must_use]
    pub fn with_border(mut self, border: BorderStyle) -> Self {
        self.border = border;
        self
    }

    /// Set the progress value (0.0 to 1.0, clamped).
    ///
    /// Switches to determinate mode if currently indeterminate.
    pub fn set_progress(&mut self, progress: f32) {
        self.mode = ProgressMode::Determinate(progress.clamp(0.0, 1.0));
    }

    /// Get the current progress value.
    ///
    /// Returns `None` if in indeterminate mode.
    pub fn progress(&self) -> Option<f32> {
        match self.mode {
            ProgressMode::Determinate(p) => Some(p),
            ProgressMode::Indeterminate { .. } => None,
        }
    }

    /// Advance the indeterminate animation phase.
    ///
    /// In determinate mode, this is a no-op.
    pub fn tick(&mut self) {
        if let ProgressMode::Indeterminate { ref mut phase } = self.mode {
            *phase = phase.wrapping_add(1);
        }
    }

    /// Get the current mode.
    pub fn mode(&self) -> &ProgressMode {
        &self.mode
    }
}

impl Widget for ProgressBar {
    fn render(&self, area: Rect, buf: &mut ScreenBuffer) {
        if area.size.width == 0 || area.size.height == 0 {
            return;
        }

        super::border::render_border(area, self.border, self.empty_style.clone(), buf);
        let inner = super::border::inner_area(area, self.border);
        if inner.size.width == 0 || inner.size.height == 0 {
            return;
        }

        let w = inner.size.width as usize;
        let y = inner.position.y;
        let x0 = inner.position.x;

        match &self.mode {
            ProgressMode::Determinate(progress) => {
                let filled_count = ((progress * w as f32).round() as usize).min(w);

                // Render filled portion
                for i in 0..filled_count {
                    buf.set(x0 + i as u16, y, Cell::new("█", self.filled_style.clone()));
                }

                // Render empty portion
                for i in filled_count..w {
                    buf.set(x0 + i as u16, y, Cell::new("░", self.empty_style.clone()));
                }

                // Overlay percentage label
                if self.show_percentage {
                    let pct = (progress * 100.0).round() as u32;
                    let label = format!("{pct}%");
                    let label_len = label.len();
                    let start = w.saturating_sub(label_len) / 2;

                    for (i, ch) in label.chars().enumerate() {
                        let col = start + i;
                        if col < w {
                            buf.set(
                                x0 + col as u16,
                                y,
                                Cell::new(ch.to_string(), self.label_style.clone()),
                            );
                        }
                    }
                }
            }
            ProgressMode::Indeterminate { phase } => {
                let wave_len = WAVE_CHARS.len();
                for i in 0..w {
                    let char_idx = (i + phase) % (wave_len * 2);
                    let ch = if char_idx < wave_len {
                        WAVE_CHARS[char_idx]
                    } else {
                        WAVE_CHARS[wave_len * 2 - 1 - char_idx]
                    };
                    buf.set(x0 + i as u16, y, Cell::new(ch, self.filled_style.clone()));
                }
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::geometry::Size;

    #[test]
    fn create_determinate_zero() {
        let bar = ProgressBar::new(0.0);
        assert_eq!(bar.progress(), Some(0.0));
    }

    #[test]
    fn create_determinate_half() {
        let bar = ProgressBar::new(0.5);
        assert_eq!(bar.progress(), Some(0.5));
    }

    #[test]
    fn create_determinate_full() {
        let bar = ProgressBar::new(1.0);
        assert_eq!(bar.progress(), Some(1.0));
    }

    #[test]
    fn progress_clamped() {
        let bar = ProgressBar::new(2.0);
        assert_eq!(bar.progress(), Some(1.0));

        let bar2 = ProgressBar::new(-0.5);
        assert_eq!(bar2.progress(), Some(0.0));
    }

    #[test]
    fn render_determinate_half() {
        let bar = ProgressBar::new(0.5).with_show_percentage(false);
        let mut buf = ScreenBuffer::new(Size::new(10, 1));
        bar.render(Rect::new(0, 0, 10, 1), &mut buf);

        // First 5 should be filled, last 5 empty
        assert_eq!(buf.get(0, 0).unwrap().grapheme, "█");
        assert_eq!(buf.get(4, 0).unwrap().grapheme, "█");
        assert_eq!(buf.get(5, 0).unwrap().grapheme, "░");
        assert_eq!(buf.get(9, 0).unwrap().grapheme, "░");
    }

    #[test]
    fn render_determinate_full() {
        let bar = ProgressBar::new(1.0).with_show_percentage(false);
        let mut buf = ScreenBuffer::new(Size::new(10, 1));
        bar.render(Rect::new(0, 0, 10, 1), &mut buf);

        for i in 0..10 {
            assert_eq!(buf.get(i, 0).unwrap().grapheme, "█");
        }
    }

    #[test]
    fn percentage_label_shown() {
        let bar = ProgressBar::new(0.5).with_show_percentage(true);
        let mut buf = ScreenBuffer::new(Size::new(20, 1));
        bar.render(Rect::new(0, 0, 20, 1), &mut buf);

        // "50%" should appear somewhere in the row
        let row: String = (0..20)
            .map(|x| buf.get(x, 0).map(|c| c.grapheme.as_str()).unwrap_or(" "))
            .collect();
        assert!(row.contains("50%"));
    }

    #[test]
    fn set_progress_updates() {
        let mut bar = ProgressBar::new(0.0);
        bar.set_progress(0.75);
        assert_eq!(bar.progress(), Some(0.75));
    }

    #[test]
    fn indeterminate_mode() {
        let bar = ProgressBar::indeterminate();
        assert!(bar.progress().is_none());
        assert!(matches!(
            bar.mode(),
            ProgressMode::Indeterminate { phase: 0 }
        ));
    }

    #[test]
    fn tick_advances_indeterminate() {
        let mut bar = ProgressBar::indeterminate();
        bar.tick();
        assert!(matches!(
            bar.mode(),
            ProgressMode::Indeterminate { phase: 1 }
        ));
        bar.tick();
        assert!(matches!(
            bar.mode(),
            ProgressMode::Indeterminate { phase: 2 }
        ));
    }

    #[test]
    fn indeterminate_renders() {
        let bar = ProgressBar::indeterminate();
        let mut buf = ScreenBuffer::new(Size::new(10, 1));
        bar.render(Rect::new(0, 0, 10, 1), &mut buf);

        // Should render wave chars, not empty
        let first = buf.get(0, 0).unwrap().grapheme.clone();
        assert!(WAVE_CHARS.contains(&first.as_str()) || first == "█" || first == "▏");
    }

    #[test]
    fn border_rendering() {
        let bar = ProgressBar::new(0.5)
            .with_border(BorderStyle::Single)
            .with_show_percentage(false);
        let mut buf = ScreenBuffer::new(Size::new(12, 3));
        bar.render(Rect::new(0, 0, 12, 3), &mut buf);

        assert_eq!(buf.get(0, 0).unwrap().grapheme, "┌");
        assert_eq!(buf.get(11, 0).unwrap().grapheme, "┐");
        // Inner bar at row 1
        assert_eq!(buf.get(1, 1).unwrap().grapheme, "█");
    }
}
