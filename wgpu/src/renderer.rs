use crate::{
    image, quad, text, triangle, Defaults, Image, Primitive, Quad, Settings,
    Target, Transformation,
};
use iced_native::{
    layout, Background, Color, Layout, MouseCursor, Point, Rectangle, TextParams,
    Vector, Widget,
};
use std::sync::Arc;

mod widget;

/// A [`wgpu`] renderer.
///
/// [`wgpu`]: https://github.com/gfx-rs/wgpu-rs
#[derive(Debug)]
pub struct Renderer {
    quad_pipeline: quad::Pipeline,
    image_pipeline: image::Pipeline,
    text_pipeline: text::Pipeline,
    triangle_pipeline: crate::triangle::Pipeline,
}

struct Layer<'a> {
    bounds: Rectangle<u32>,
    offset: Vector<u32>,
    quads: Vec<Quad>,
    images: Vec<Image>,
    meshes: Vec<(Point, Arc<triangle::Mesh2D>)>,
    text: Vec<wgpu_glyph::Section<'a>>,
}

impl<'a> Layer<'a> {
    pub fn new(bounds: Rectangle<u32>, offset: Vector<u32>) -> Self {
        Self {
            bounds,
            offset,
            quads: Vec::new(),
            images: Vec::new(),
            text: Vec::new(),
            meshes: Vec::new(),
        }
    }
}

impl Renderer {
    /// Creates a new [`Renderer`].
    ///
    /// [`Renderer`]: struct.Renderer.html
    pub fn new(device: &mut wgpu::Device, settings: Settings) -> Self {
        let text_pipeline = text::Pipeline::new(device, settings.default_font);
        let quad_pipeline = quad::Pipeline::new(device);
        let image_pipeline = crate::image::Pipeline::new(device);
        let triangle_pipeline =
            triangle::Pipeline::new(device, settings.antialiasing);

        Self {
            quad_pipeline,
            image_pipeline,
            text_pipeline,
            triangle_pipeline,
        }
    }

    /// Draws the provided primitives in the given [`Target`].
    ///
    /// The text provided as overlay will be renderer on top of the primitives.
    /// This is useful for rendering debug information.
    ///
    /// [`Target`]: struct.Target.html
    pub fn draw<T: AsRef<str>>(
        &mut self,
        device: &mut wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        target: Target<'_>,
        (primitive, mouse_cursor): &(Primitive, MouseCursor),
        scale_factor: f64,
        overlay: &[T],
    ) -> MouseCursor {
        log::debug!("Drawing");

        let (width, height) = target.viewport.dimensions();
        let scale_factor = scale_factor as f32;
        let transformation = target.viewport.transformation();

        let mut layers = Vec::new();

        layers.push(Layer::new(
            Rectangle {
                x: 0,
                y: 0,
                width: u32::from(width),
                height: u32::from(height),
            },
            Vector::new(0, 0),
        ));

        self.draw_primitive(primitive, &mut layers);
        self.draw_overlay(overlay, &mut layers);

        for layer in layers {
            self.flush(
                device,
                scale_factor,
                transformation,
                &layer,
                encoder,
                target.texture,
                width,
                height,
            );
        }

        self.image_pipeline.trim_cache();

        *mouse_cursor
    }

    fn draw_primitive<'a>(
        &mut self,
        primitive: &'a Primitive,
        layers: &mut Vec<Layer<'a>>,
    ) {
        let layer = layers.last_mut().unwrap();

        match primitive {
            Primitive::None => {}
            Primitive::Group { primitives } => {
                // TODO: Inspect a bit and regroup (?)
                for primitive in primitives {
                    self.draw_primitive(primitive, layers)
                }
            }
            Primitive::Text {
                bounds,
                text:
                    TextParams {
                        content,
                        size,
                        color,
                        font,
                        horizontal_alignment,
                        vertical_alignment,
                    },
            } => {
                let x = match horizontal_alignment {
                    iced_native::HorizontalAlignment::Left => bounds.x,
                    iced_native::HorizontalAlignment::Center => {
                        bounds.x + bounds.width / 2.0
                    }
                    iced_native::HorizontalAlignment::Right => {
                        bounds.x + bounds.width
                    }
                };

                let y = match vertical_alignment {
                    iced_native::VerticalAlignment::Top => bounds.y,
                    iced_native::VerticalAlignment::Center => {
                        bounds.y + bounds.height / 2.0
                    }
                    iced_native::VerticalAlignment::Bottom => {
                        bounds.y + bounds.height
                    }
                };

                layer.text.push(wgpu_glyph::Section {
                    text: &content,
                    screen_position: (
                        x - layer.offset.x as f32,
                        y - layer.offset.y as f32,
                    ),
                    bounds: (bounds.width, bounds.height),
                    scale: wgpu_glyph::Scale { x: *size, y: *size },
                    color: color.into_linear(),
                    font_id: self.text_pipeline.find_font(*font),
                    layout: wgpu_glyph::Layout::default()
                        .h_align(match horizontal_alignment {
                            iced_native::HorizontalAlignment::Left => {
                                wgpu_glyph::HorizontalAlign::Left
                            }
                            iced_native::HorizontalAlignment::Center => {
                                wgpu_glyph::HorizontalAlign::Center
                            }
                            iced_native::HorizontalAlignment::Right => {
                                wgpu_glyph::HorizontalAlign::Right
                            }
                        })
                        .v_align(match vertical_alignment {
                            iced_native::VerticalAlignment::Top => {
                                wgpu_glyph::VerticalAlign::Top
                            }
                            iced_native::VerticalAlignment::Center => {
                                wgpu_glyph::VerticalAlign::Center
                            }
                            iced_native::VerticalAlignment::Bottom => {
                                wgpu_glyph::VerticalAlign::Bottom
                            }
                        }),
                    ..Default::default()
                })
            }
            Primitive::Quad {
                bounds,
                background,
                border_radius,
                border_width,
                border_color,
            } => {
                // TODO: Move some of this computations to the GPU (?)
                layer.quads.push(Quad {
                    position: [
                        bounds.x - layer.offset.x as f32,
                        bounds.y - layer.offset.y as f32,
                    ],
                    scale: [bounds.width, bounds.height],
                    color: match background {
                        Background::Color(color) => color.into_linear(),
                    },
                    border_radius: *border_radius as f32,
                    border_width: *border_width as f32,
                    border_color: border_color.into_linear(),
                });
            }
            Primitive::Image { handle, bounds } => {
                layer.images.push(Image {
                    handle: image::Handle::Raster(handle.clone()),
                    position: [bounds.x, bounds.y],
                    scale: [bounds.width, bounds.height],
                });
            }
            Primitive::Svg { handle, bounds } => {
                layer.images.push(Image {
                    handle: image::Handle::Vector(handle.clone()),
                    position: [bounds.x, bounds.y],
                    scale: [bounds.width, bounds.height],
                });
            }
            Primitive::Mesh2D { origin, buffers } => {
                layer.meshes.push((*origin, buffers.clone()));
            }
            Primitive::Clip {
                bounds,
                offset,
                content,
            } => {
                let layer_bounds: Rectangle<f32> = layer.bounds.into();

                let clip = Rectangle {
                    x: bounds.x - layer.offset.x as f32,
                    y: bounds.y - layer.offset.y as f32,
                    ..*bounds
                };

                // Only draw visible content
                if let Some(clip_bounds) = layer_bounds.intersection(&clip) {
                    let clip_layer =
                        Layer::new(clip_bounds.into(), layer.offset + *offset);
                    let new_layer = Layer::new(layer.bounds, layer.offset);

                    layers.push(clip_layer);
                    self.draw_primitive(content, layers);
                    layers.push(new_layer);
                }
            }
        }
    }

    fn draw_overlay<'a, T: AsRef<str>>(
        &mut self,
        lines: &'a [T],
        layers: &mut Vec<Layer<'a>>,
    ) {
        let first = layers.first().unwrap();
        let mut overlay = Layer::new(first.bounds, Vector::new(0, 0));

        let font_id = self.text_pipeline.overlay_font();
        let scale = wgpu_glyph::Scale { x: 20.0, y: 20.0 };

        for (i, line) in lines.iter().enumerate() {
            overlay.text.push(wgpu_glyph::Section {
                text: line.as_ref(),
                screen_position: (11.0, 11.0 + 25.0 * i as f32),
                color: [0.9, 0.9, 0.9, 1.0],
                scale,
                font_id,
                ..wgpu_glyph::Section::default()
            });

            overlay.text.push(wgpu_glyph::Section {
                text: line.as_ref(),
                screen_position: (10.0, 10.0 + 25.0 * i as f32),
                color: [0.0, 0.0, 0.0, 1.0],
                scale,
                font_id,
                ..wgpu_glyph::Section::default()
            });
        }

        layers.push(overlay);
    }

    fn flush(
        &mut self,
        device: &mut wgpu::Device,
        scale_factor: f32,
        transformation: Transformation,
        layer: &Layer<'_>,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        target_width: u32,
        target_height: u32,
    ) {
        let bounds = layer.bounds * scale_factor;

        if layer.meshes.len() > 0 {
            let translated = transformation
                * Transformation::scale(scale_factor, scale_factor)
                * Transformation::translate(
                    -(layer.offset.x as f32),
                    -(layer.offset.y as f32),
                );

            self.triangle_pipeline.draw(
                device,
                encoder,
                target,
                target_width,
                target_height,
                translated,
                &layer.meshes,
                bounds,
            );
        }

        if layer.quads.len() > 0 {
            self.quad_pipeline.draw(
                device,
                encoder,
                &layer.quads,
                transformation,
                scale_factor,
                bounds,
                target,
            );
        }

        if layer.images.len() > 0 {
            let translated_and_scaled = transformation
                * Transformation::scale(scale_factor, scale_factor)
                * Transformation::translate(
                    -(layer.offset.x as f32),
                    -(layer.offset.y as f32),
                );

            self.image_pipeline.draw(
                device,
                encoder,
                &layer.images,
                translated_and_scaled,
                bounds,
                target,
                scale_factor,
            );
        }

        if layer.text.len() > 0 {
            for text in layer.text.iter() {
                // Target physical coordinates directly to avoid blurry text
                let text = wgpu_glyph::Section {
                    // TODO: We `round` here to avoid rerasterizing text when
                    // its position changes slightly. This can make text feel a
                    // bit "jumpy". We may be able to do better once we improve
                    // our text rendering/caching pipeline.
                    screen_position: (
                        (text.screen_position.0 * scale_factor).round(),
                        (text.screen_position.1 * scale_factor).round(),
                    ),
                    // TODO: Fix precision issues with some scale factors.
                    //
                    // The `ceil` here can cause some words to render on the
                    // same line when they should not.
                    //
                    // Ideally, `wgpu_glyph` should be able to compute layout
                    // using logical positions, and then apply the proper
                    // scaling when rendering. This would ensure that both
                    // measuring and rendering follow the same layout rules.
                    bounds: (
                        (text.bounds.0 * scale_factor).ceil(),
                        (text.bounds.1 * scale_factor).ceil(),
                    ),
                    scale: wgpu_glyph::Scale {
                        x: text.scale.x * scale_factor,
                        y: text.scale.y * scale_factor,
                    },
                    ..*text
                };

                self.text_pipeline.queue(text);
            }

            self.text_pipeline.draw_queued(
                device,
                encoder,
                target,
                transformation,
                wgpu_glyph::Region {
                    x: bounds.x,
                    y: bounds.y,
                    width: bounds.width,
                    height: bounds.height,
                },
            );
        }
    }
}

impl iced_native::Renderer for Renderer {
    type Output = (Primitive, MouseCursor);
    type Defaults = Defaults;

    fn layout<'a, Message>(
        &mut self,
        element: &iced_native::Element<'a, Message, Self>,
        limits: &iced_native::layout::Limits,
    ) -> iced_native::layout::Node {
        let node = element.layout(self, limits);

        self.text_pipeline.clear_measurement_cache();

        node
    }
}

impl layout::Debugger for Renderer {
    fn explain<Message>(
        &mut self,
        defaults: &Defaults,
        widget: &dyn Widget<Message, Self>,
        layout: Layout<'_>,
        cursor_position: Point,
        color: Color,
    ) -> Self::Output {
        let mut primitives = Vec::new();
        let (primitive, cursor) =
            widget.draw(self, defaults, layout, cursor_position);

        explain_layout(layout, color, &mut primitives);
        primitives.push(primitive);

        (Primitive::Group { primitives }, cursor)
    }
}

fn explain_layout(
    layout: Layout<'_>,
    color: Color,
    primitives: &mut Vec<Primitive>,
) {
    primitives.push(Primitive::Quad {
        bounds: layout.bounds(),
        background: Background::Color(Color::TRANSPARENT),
        border_radius: 0,
        border_width: 1,
        border_color: [0.6, 0.6, 0.6, 0.5].into(),
    });

    for child in layout.children() {
        explain_layout(child, color, primitives);
    }
}
