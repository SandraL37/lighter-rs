use crate::{
    core::{
        error::*,
        layout::{AvailableSpace, Rect, Size},
        style::{Color, Transform},
    },
    elements::text::TextProps,
};

pub mod piet;
pub mod tinyskia;

#[derive(Debug)]
pub enum RenderCommand {
    Rect {
        bounds: Rect<f32>,
        color: Color,
        opacity: f32,
        transform: Transform,
        z_index: i32,
    },

    Text {
        bounds: Rect<f32>,
        props: TextProps,
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

pub trait Renderer {
    fn render(&mut self, commands: &[RenderCommand]) -> Result<()>;
    fn get_size(&self) -> Size<usize>;
    fn measure_text(
        &mut self,
        text_props: &TextProps,
        available_width: AvailableSpace,
    ) -> Result<Size<f32>>;
}
