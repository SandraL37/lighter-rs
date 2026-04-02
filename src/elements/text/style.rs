use std::sync::Arc;

use crate::{
    core::{
        reactive::{bind::HasDeferredBindings, dirty::DirtyFlags, signal::MaybeSignal},
        style::Color,
    },
    elements::text::IntoTextContent,
};

#[derive(Debug, Clone)]
pub struct TextStyle {
    pub content: Arc<str>,
    pub color: Color,
    pub font_size: f32,
    pub font_family: Arc<str>,
    pub font_weight: FontWeight,
}

impl Default for TextStyle {
    fn default() -> Self {
        TextStyle {
            content: Arc::from(""),
            color: Color::BLACK,
            font_size: 16.0,
            font_family: Arc::from("Segoe UI"),
            font_weight: FontWeight::NORMAL,
        }
    }
}

pub trait TextStyleBuilder: HasDeferredBindings + Sized {
    fn text_style(style: &mut Self::Style) -> &mut TextStyle;

    fn content(mut self, value: impl IntoTextContent) -> Self {
        self.bind(
            |style| &mut Self::text_style(style).content,
            value.into_text_content(),
            DirtyFlags::PAINT | DirtyFlags::LAYOUT,
            |data, _, val| {
                if let Ok(text) = data.kind.as_text_mut() {
                    text.content = val;
                }
            },
        );
        self
    }

    fn color(mut self, value: impl Into<MaybeSignal<Color>>) -> Self {
        self.bind(
            |style| &mut Self::text_style(style).color,
            value,
            DirtyFlags::PAINT,
            |data, _, val| {
                if let Ok(text) = data.kind.as_text_mut() {
                    text.color = val;
                }
            },
        );
        self
    }

    fn font_family(mut self, value: impl IntoTextContent) -> Self {
        self.bind(
            |style| &mut Self::text_style(style).font_family,
            value.into_text_content(),
            DirtyFlags::PAINT | DirtyFlags::LAYOUT,
            |data, _, val| {
                if let Ok(text) = data.kind.as_text_mut() {
                    text.font_family = val;
                }
            },
        );
        self
    }

    fn font_size(mut self, value: impl Into<MaybeSignal<f32>>) -> Self {
        self.bind(
            |style| &mut Self::text_style(style).font_size,
            value,
            DirtyFlags::PAINT | DirtyFlags::LAYOUT,
            |data, _, val| {
                if let Ok(text) = data.kind.as_text_mut() {
                    text.font_size = val;
                }
            },
        );
        self
    }

    fn font_weight(mut self, value: impl Into<MaybeSignal<FontWeight>>) -> Self {
        self.bind(
            |style| &mut Self::text_style(style).font_weight,
            value,
            DirtyFlags::PAINT | DirtyFlags::LAYOUT,
            |data, _, val| {
                if let Ok(text) = data.kind.as_text_mut() {
                    text.font_weight = val;
                }
            },
        );
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
pub struct FontWeight(pub u16);

impl FontWeight {
    pub const THIN: FontWeight = FontWeight(100);
    pub const EXTRA_LIGHT: FontWeight = FontWeight(200);
    pub const LIGHT: FontWeight = FontWeight(300);
    pub const REGULAR: FontWeight = FontWeight(400);
    pub const NORMAL: FontWeight = FontWeight::REGULAR;
    pub const MEDIUM: FontWeight = FontWeight(500);
    pub const SEMI_BOLD: FontWeight = FontWeight(600);
    pub const BOLD: FontWeight = FontWeight(700);
    pub const EXTRA_BOLD: FontWeight = FontWeight(800);
    pub const BLACK: FontWeight = FontWeight(900);
    pub const EXTRA_BLACK: FontWeight = FontWeight(950);

    pub fn new(raw: u16) -> FontWeight {
        let raw = raw.clamp(1, 1000);
        FontWeight(raw)
    }
}
