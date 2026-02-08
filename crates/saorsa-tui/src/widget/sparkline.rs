//! Sparkline widget for inline mini-charts.
//!
//! Displays a compact data series visualization using Unicode block
//! characters. Useful for CPU usage, memory trends, and other metrics.

use crate::buffer::ScreenBuffer;
use crate::cell::Cell;
use crate::geometry::Rect;
use crate::style::Style;

use super::Widget;

/// Block characters for bar rendering (from lowest to highest).
const BAR_CHARS: &[&str] = &["▁", "▂", "▃", "▄", "▅", "▆", "▇", "█"];

/// A sparkline mini-chart widget.
///
/// Renders a compact data series using Unicode block characters.
/// Each data point maps to a single column, with the block height
/// proportional to the value.
pub struct Sparkline {
    /// Data points.
    data: Vec<f32>,
    /// Maximum number of data points to display.
    max_width: usize,
    /// Chart height in lines (currently only 1 supported).
    height: u16,
    /// Visual style for the chart.
    chart_style: Style,
}

impl Sparkline {
    /// Create a new sparkline with the given data.
    pub fn new(data: Vec<f32>) -> Self {
        Self {
            data,
            max_width: 80,
            height: 1,
            chart_style: Style::default(),
        }
    }

    /// Set the maximum display width (oldest data dropped if exceeded).
    #[must_use]
    pub fn with_max_width(mut self, width: usize) -> Self {
        self.max_width = width;
        self
    }

    /// Set the chart height in lines.
    #[must_use]
    pub fn with_height(mut self, height: u16) -> Self {
        self.height = height.max(1);
        self
    }

    /// Set the chart visual style.
    #[must_use]
    pub fn with_chart_style(mut self, style: Style) -> Self {
        self.chart_style = style;
        self
    }

    /// Add a data point. Drops the oldest if exceeding `max_width`.
    pub fn push(&mut self, value: f32) {
        self.data.push(value);
        if self.data.len() > self.max_width {
            self.data.remove(0);
        }
    }

    /// Replace all data points.
    pub fn set_data(&mut self, data: Vec<f32>) {
        self.data = data;
    }

    /// Get the current data.
    pub fn data(&self) -> &[f32] {
        &self.data
    }

    /// Clear all data points.
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// Map a value to a bar character index (0-7).
    fn value_to_bar_index(value: f32, min: f32, range: f32) -> usize {
        if range <= 0.0 {
            return BAR_CHARS.len() / 2; // Middle level for flat data
        }
        let normalized = ((value - min) / range).clamp(0.0, 1.0);
        let idx = (normalized * (BAR_CHARS.len() - 1) as f32).round() as usize;
        idx.min(BAR_CHARS.len() - 1)
    }
}

impl Widget for Sparkline {
    fn render(&self, area: Rect, buf: &mut ScreenBuffer) {
        if area.size.width == 0 || area.size.height == 0 || self.data.is_empty() {
            return;
        }

        let w = area.size.width as usize;
        let x0 = area.position.x;
        let y = area.position.y;

        // Use only the most recent data that fits
        let display_count = self.data.len().min(w).min(self.max_width);
        let start = self.data.len().saturating_sub(display_count);
        let visible = &self.data[start..];

        // Find data range
        let mut min = f32::MAX;
        let mut max = f32::MIN;
        for &v in visible {
            if v < min {
                min = v;
            }
            if v > max {
                max = v;
            }
        }
        let range = max - min;

        // Render bars
        for (i, &value) in visible.iter().enumerate() {
            if i >= w {
                break;
            }
            let bar_idx = Self::value_to_bar_index(value, min, range);
            let ch = BAR_CHARS[bar_idx];
            buf.set(x0 + i as u16, y, Cell::new(ch, self.chart_style.clone()));
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::geometry::Size;

    #[test]
    fn create_with_data() {
        let s = Sparkline::new(vec![1.0, 2.0, 3.0]);
        assert_eq!(s.data().len(), 3);
    }

    #[test]
    fn render_bars() {
        let s = Sparkline::new(vec![0.0, 0.5, 1.0]);
        let mut buf = ScreenBuffer::new(Size::new(10, 1));
        s.render(Rect::new(0, 0, 10, 1), &mut buf);

        // First should be lowest bar, last should be highest
        let first = buf.get(0, 0).unwrap().grapheme.clone();
        let last = buf.get(2, 0).unwrap().grapheme.clone();
        assert_eq!(first, "▁"); // lowest
        assert_eq!(last, "█"); // highest
    }

    #[test]
    fn push_drops_oldest() {
        let mut s = Sparkline::new(vec![1.0, 2.0, 3.0]).with_max_width(3);
        s.push(4.0);
        assert_eq!(s.data().len(), 3);
        assert_eq!(s.data()[0], 2.0); // 1.0 was dropped
        assert_eq!(s.data()[2], 4.0);
    }

    #[test]
    fn set_data_replaces() {
        let mut s = Sparkline::new(vec![1.0, 2.0]);
        s.set_data(vec![10.0, 20.0, 30.0]);
        assert_eq!(s.data().len(), 3);
        assert_eq!(s.data()[0], 10.0);
    }

    #[test]
    fn clear_removes_all() {
        let mut s = Sparkline::new(vec![1.0, 2.0, 3.0]);
        s.clear();
        assert!(s.data().is_empty());
    }

    #[test]
    fn empty_renders_blank() {
        let s = Sparkline::new(vec![]);
        let mut buf = ScreenBuffer::new(Size::new(10, 1));
        s.render(Rect::new(0, 0, 10, 1), &mut buf);
        // All cells should be default space
        assert_eq!(buf.get(0, 0).unwrap().grapheme, " ");
    }

    #[test]
    fn scaling() {
        // All same values should render middle bars
        let s = Sparkline::new(vec![5.0, 5.0, 5.0]);
        let mut buf = ScreenBuffer::new(Size::new(10, 1));
        s.render(Rect::new(0, 0, 10, 1), &mut buf);

        let ch = buf.get(0, 0).unwrap().grapheme.clone();
        assert!(BAR_CHARS.contains(&ch.as_str()));
    }

    #[test]
    fn max_width_respected() {
        let s = Sparkline::new(vec![1.0; 100]).with_max_width(5);
        // max_width only affects push, not initial data
        // But when rendering, only area width matters
        let mut buf = ScreenBuffer::new(Size::new(3, 1));
        s.render(Rect::new(0, 0, 3, 1), &mut buf);

        // Should render most recent 3 data points
        let ch = buf.get(0, 0).unwrap().grapheme.clone();
        assert!(BAR_CHARS.contains(&ch.as_str()));
    }

    #[test]
    fn custom_style_applied() {
        let style = Style::default().bold(true);
        let s = Sparkline::new(vec![1.0, 2.0]).with_chart_style(style);
        let mut buf = ScreenBuffer::new(Size::new(10, 1));
        s.render(Rect::new(0, 0, 10, 1), &mut buf);

        assert!(buf.get(0, 0).unwrap().style.bold);
    }

    #[test]
    fn zero_values() {
        let s = Sparkline::new(vec![0.0, 0.0, 0.0]);
        let mut buf = ScreenBuffer::new(Size::new(5, 1));
        s.render(Rect::new(0, 0, 5, 1), &mut buf);
        // Should not panic; all same value renders middle bar
        let ch = buf.get(0, 0).unwrap().grapheme.clone();
        assert!(BAR_CHARS.contains(&ch.as_str()));
    }

    #[test]
    fn negative_values() {
        let s = Sparkline::new(vec![-10.0, 0.0, 10.0]);
        let mut buf = ScreenBuffer::new(Size::new(5, 1));
        s.render(Rect::new(0, 0, 5, 1), &mut buf);

        let first = buf.get(0, 0).unwrap().grapheme.clone();
        let last = buf.get(2, 0).unwrap().grapheme.clone();
        assert_eq!(first, "▁"); // lowest
        assert_eq!(last, "█"); // highest
    }

    #[test]
    fn single_data_point() {
        let s = Sparkline::new(vec![42.0]);
        let mut buf = ScreenBuffer::new(Size::new(5, 1));
        s.render(Rect::new(0, 0, 5, 1), &mut buf);
        let ch = buf.get(0, 0).unwrap().grapheme.clone();
        assert!(BAR_CHARS.contains(&ch.as_str()));
    }
}
