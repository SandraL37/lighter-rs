use std::sync::Arc;

use crate::{
    core::{
        error::*,
        layout::{
            AvailableSpace,
            types::{rect::Rect, size::Size},
        },
        style::{Color, Transform},
    },
    elements::text::TextProps,
};

pub mod d2d;

#[derive(Debug)]
pub enum RenderCommand {
    Rect {
        bounds: Rect<f32>,
        corner_radius: f32,
        color: Color,
        opacity: f32,
        transform: Transform,
        z_index: i32,
    },

    Text {
        bounds: Rect<f32>,
        props: Arc<TextProps>,
        opacity: f32,
        transform: Transform,
        z_index: i32,
    },
}

impl RenderCommand {
    pub fn z_index(&self) -> i32 {
        match self {
            RenderCommand::Rect { z_index, .. } => *z_index,
            RenderCommand::Text { z_index, .. } => *z_index,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Dpi {
    x: f32,
    y: f32,
}

impl Dpi {
    pub const fn uniform(dpi: f32) -> Self {
        Self { x: dpi, y: dpi }
    }
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

pub trait Renderer: Sized {
    fn render(&mut self, commands: &[RenderCommand]) -> Result<()>;
    fn resize(&mut self, size: Size<usize>) -> Result<()>;
    fn measure_text(
        &mut self,
        text_props: &TextProps,
        available_size: Size<AvailableSpace>,
    ) -> Result<Size<f32>>;
    fn set_dpi(&mut self, dpi: Dpi) -> Result<()>;
}
