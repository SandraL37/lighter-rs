use std::sync::Arc;

use crate::core::{
    error::*,
    layout::{Rect, Size},
    style::{Color, Transform},
};

// pub mod piet;
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
        content: Arc<str>,
        color: Color,
        font_size: f32,
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
    fn resize(&mut self, size: Size<u32>) -> Result<()>;
    fn get_size(&self) -> Size<u32>;
}
