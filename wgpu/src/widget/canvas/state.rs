use iced_native::{
    Layout, Clipboard, Event, Point,
};

/// hi!
pub trait State: std::fmt::Debug {
    /// hi!
    fn on_event(
        &mut self,
        event: Event,
        cursor_position: Point,
        layout: Layout<'_>,
        clipboard: Option<&dyn Clipboard>,
    );
}
