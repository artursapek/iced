use iced_native::{
    layout, Clipboard, Event, Point,
};

/// hi!
pub trait Handler: std::fmt::Debug {
    /// hi!
    fn on_event(
        &mut self,
        event: Event,
        cursor_position: Point,
        clipboard: Option<&dyn Clipboard>,
    );
}
