use crate::core::{
    error::*,
    layout::{Rect, Size},
    style::{Color, Transform},
};

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
        content: String,
        color: Color,
        font_size: f32,
        opacity: f32,
        transform: Transform,
        z_index: i32,
    },
}

pub trait Renderer {
    fn render(&mut self, commands: &[RenderCommand]) -> Result<()>;
    fn resize(&mut self, size: Size<u32>) -> Result<()>;
    fn get_size(&self) -> Size<u32>;
}
