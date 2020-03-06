use iced_native::{Color, Font, HorizontalAlignment, Rectangle, VerticalAlignment};

/// Greetings, m'lord!
#[derive(Debug, Clone)]
pub struct TextNode {
    /// The contents of the text
    content: String,
    /// The bounds of the text
    bounds: Rectangle,
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
