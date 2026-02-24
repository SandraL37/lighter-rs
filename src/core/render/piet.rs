use std::f64;

use piet_common::{RenderContext, Text, TextLayout, TextLayoutBuilder};

use crate::{
    core::{
        error::*,
        layout::{AvailableSpace, Point, Size},
        render::{RenderCommand, Renderer},
        style::Color,
    },
    elements::text::{FontWeight, TextProps},
};

pub struct PietRenderer {
    device: piet_common::Device,
    size: Size<usize>,
    output: Option<String>,
}

impl From<piet_common::Error> for Error {
    fn from(value: piet_common::Error) -> Self {
        return Error::PietRendererError(value);
    }
}

impl PietRenderer {
    pub fn new(size: Size<usize>) -> Result<Self> {
        let device = piet_common::Device::new()?;
        Ok(Self {
            device,
            size,
            output: None,
        })
    }

    pub fn set_output(&mut self, path: String) {
        self.output = Some(path);
    }
}

impl Renderer for PietRenderer {
    fn render(&mut self, commands: &[RenderCommand]) -> Result<()> {
        let mut bitmap = self
            .device
            .bitmap_target(self.size.width, self.size.height, 1.0)?;
        {
            let mut rc = bitmap.render_context();

            rc.clear(None, piet_common::Color::TRANSPARENT);

            for cmd in commands {
                match cmd {
                    RenderCommand::Rect {
                        bounds,
                        color,
                        // TODO: opacity,
                        ..
                    } => {
                        let brush = rc.solid_brush(piet_common::Color::from(*color));
                        let rect = piet_common::kurbo::Rect::from_origin_size(
                            bounds.location,
                            bounds.size,
                        );
                        rc.fill(rect, &brush);
                    }
                    RenderCommand::Text { bounds, props, .. } => {
                        let text = rc.text();
                        let layout = text
                            .new_text_layout(props.content.clone())
                            .text_color(props.color.into())
                            .font(
                                piet_common::FontFamily::new_unchecked(props.font_family.clone()),
                                props.font_size as f64,
                            )
                            .default_attribute(piet_common::FontWeight::from(props.font_weight))
                            .build()?;
                        rc.draw_text(&layout, bounds.location);
                    }
                }
            }

            rc.finish()?;
        }

        if let Some(output) = &self.output {
            bitmap.save_to_file(output)?;
        }


        Ok(())
    }

    fn get_size(&self) -> Size<usize> {
        self.size
    }

    fn measure_text(
        &mut self,
        text_props: &TextProps,
        available_width: AvailableSpace,
    ) -> Result<Size<f32>> {
        let mut bitmap = self.device.bitmap_target(1, 1, 1.0)?;
        let mut rc = bitmap.render_context();
        let text = rc.text();
        let layout = text
            .new_text_layout(text_props.content.clone())
            .font(
                piet_common::FontFamily::new_unchecked(text_props.font_family.clone()),
                text_props.font_size as f64,
            )
            .default_attribute(piet_common::FontWeight::from(text_props.font_weight))
            .max_width({
                match available_width {
                    AvailableSpace::Definite(pixels) => pixels as f64,
                    _ => f64::INFINITY,
                }
            })
            .build()?;
        let bounds = layout.size();
        rc.finish()?;
        Ok(Size::wh(bounds.width as f32, bounds.height as f32))
    }
}

impl From<FontWeight> for piet_common::FontWeight {
    fn from(value: FontWeight) -> Self {
        piet_common::FontWeight::new(value.0)
    }
}

impl From<Color> for piet_common::Color {
    fn from(color: Color) -> Self {
        Self::rgba(
            color.r as f64,
            color.g as f64,
            color.b as f64,
            color.a as f64,
        )
    }
}

impl From<Point<f32>> for piet_common::kurbo::Point {
    fn from(value: Point<f32>) -> Self {
        Self {
            x: value.x as f64,
            y: value.y as f64,
        }
    }
}

impl From<Size<f32>> for piet_common::kurbo::Size {
    fn from(value: Size<f32>) -> Self {
        Self {
            width: value.width as f64,
            height: value.height as f64,
        }
    }
}
