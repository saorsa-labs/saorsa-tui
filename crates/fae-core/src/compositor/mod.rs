//! Compositor — resolves overlapping widget layers into a flat cell grid.
//!
//! The compositor collects styled segment output from each widget,
//! finds cut boundaries where widget edges meet, selects the topmost
//! visible widget for each region, and writes the result to a screen buffer.

pub mod chop;
pub mod cuts;
pub mod layer;
pub mod zorder;

pub use layer::{CompositorError, CompositorRegion, Layer};

use crate::geometry::{Rect, Size};
use crate::segment::Segment;

/// The compositor collects widget layers and resolves overlapping regions.
pub struct Compositor {
    layers: Vec<Layer>,
    screen_width: u16,
    screen_height: u16,
}

impl Compositor {
    /// Creates a new compositor with the given screen dimensions.
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            layers: Vec::new(),
            screen_width: width,
            screen_height: height,
        }
    }

    /// Removes all layers from the compositor.
    pub fn clear(&mut self) {
        self.layers.clear();
    }

    /// Adds a layer to the compositor stack.
    pub fn add_layer(&mut self, layer: Layer) {
        self.layers.push(layer);
    }

    /// Convenience method that creates and adds a layer.
    ///
    /// Creates a new layer from the given parameters and adds it to the stack.
    pub fn add_widget(
        &mut self,
        widget_id: u64,
        region: Rect,
        z_index: i32,
        lines: Vec<Vec<Segment>>,
    ) {
        let layer = Layer::new(widget_id, region, z_index, lines);
        self.add_layer(layer);
    }

    /// Returns the number of layers in the compositor.
    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }

    /// Returns the screen size.
    pub fn screen_size(&self) -> Size {
        Size::new(self.screen_width, self.screen_height)
    }

    /// Returns a slice of all layers in the compositor.
    pub fn layers(&self) -> &[Layer] {
        &self.layers
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Rect;
    use crate::segment::Segment;

    #[test]
    fn new_compositor_empty() {
        let compositor = Compositor::new(80, 24);
        assert!(compositor.layer_count() == 0);
    }

    #[test]
    fn add_layer_increases_count() {
        let mut compositor = Compositor::new(80, 24);
        let region = Rect::new(0, 0, 10, 5);
        let layer = Layer::new(1, region, 0, vec![]);

        compositor.add_layer(layer);
        assert!(compositor.layer_count() == 1);
    }

    #[test]
    fn add_multiple_layers() {
        let mut compositor = Compositor::new(80, 24);
        let region1 = Rect::new(0, 0, 10, 5);
        let region2 = Rect::new(10, 10, 20, 10);
        let region3 = Rect::new(30, 5, 15, 8);

        compositor.add_layer(Layer::new(1, region1, 0, vec![]));
        compositor.add_layer(Layer::new(2, region2, 1, vec![]));
        compositor.add_layer(Layer::new(3, region3, 2, vec![]));

        assert!(compositor.layer_count() == 3);
    }

    #[test]
    fn add_widget_convenience() {
        let mut compositor = Compositor::new(80, 24);
        let region = Rect::new(5, 10, 20, 15);
        let lines = vec![vec![Segment::new("test")]];

        compositor.add_widget(42, region, 5, lines);

        assert!(compositor.layer_count() == 1);
        let layer_slice = compositor.layers();
        assert!(layer_slice.len() == 1);
        let layer = match layer_slice.first() {
            Some(l) => l,
            None => unreachable!(),
        };
        assert!(layer.widget_id == 42);
        assert!(layer.z_index == 5);
        assert!(layer.region == region);
    }

    #[test]
    fn clear_removes_all() {
        let mut compositor = Compositor::new(80, 24);
        let region1 = Rect::new(0, 0, 10, 5);
        let region2 = Rect::new(10, 10, 20, 10);

        compositor.add_layer(Layer::new(1, region1, 0, vec![]));
        compositor.add_layer(Layer::new(2, region2, 1, vec![]));
        assert!(compositor.layer_count() == 2);

        compositor.clear();
        assert!(compositor.layer_count() == 0);
    }

    #[test]
    fn screen_size_accessible() {
        let compositor = Compositor::new(100, 50);
        let size = compositor.screen_size();
        assert!(size.width == 100);
        assert!(size.height == 50);
    }

    #[test]
    fn layers_accessible() {
        let mut compositor = Compositor::new(80, 24);
        let region1 = Rect::new(0, 0, 10, 5);
        let region2 = Rect::new(10, 10, 20, 10);

        compositor.add_layer(Layer::new(1, region1, 0, vec![]));
        compositor.add_layer(Layer::new(2, region2, 1, vec![]));

        let layers = compositor.layers();
        assert!(layers.len() == 2);
        assert!(layers[0].widget_id == 1);
        assert!(layers[1].widget_id == 2);
    }
}
