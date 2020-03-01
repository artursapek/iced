//pub use crate::Primitive::Text;
use iced_native::{Color, Font, HorizontalAlignment, VerticalAlignment};

/// Text
#[derive(Debug)]
pub struct TextParams {
    /// The contents of the text
    content: String,
    /// The color of the text
    color: Color,
    /// The size of the text
    size: f32,
    /// The font of the text
    font: Font,
    /// The horizontal alignment of the text
    horizontal_alignment: HorizontalAlignment,
    /// The vertical alignment of the text
    vertical_alignment: VerticalAlignment,
}
