use iced_native::{Clipboard, Event, Layout, Point};

use super::Layer;

/// hi!
pub trait State: std::fmt::Debug {
    /// hi!
    fn on_event<'a>(
        &mut self,
        event: Event,
        cursor_position: Point,
        layout: Layout<'_>,
        clipboard: Option<&dyn Clipboard>,
        layers: &mut Vec<Box<dyn Layer<Self> + 'a>>,
    );
}
