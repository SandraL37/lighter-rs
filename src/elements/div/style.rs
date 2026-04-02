use crate::core::{
    reactive::{bind::HasDeferredBindings, dirty::DirtyFlags, signal::MaybeSignal},
    style::Color,
};

#[derive(Debug, Clone)]
pub struct DivStyle {
    pub background_color: Color,
    pub corner_radius: f32, // TODO: make it DefiniteDimension
}

pub trait DivStyleBuilder: HasDeferredBindings + Sized {
    fn div_style(style: &mut Self::Style) -> &mut DivStyle;

    fn bg(mut self, color: impl Into<MaybeSignal<Color>>) -> Self {
        self.bind(
            |style| &mut Self::div_style(style).background_color,
            color,
            DirtyFlags::PAINT,
            |data, _, val| {
                data.kind.as_div_mut().background_color = val;
            },
        );
        self
    }

    fn rounded(mut self, radius: impl Into<MaybeSignal<f32>>) -> Self {
        self.bind(
            |style| &mut Self::div_style(style).corner_radius,
            radius,
            DirtyFlags::PAINT,
            |data, _, val| {
                data.kind.as_div_mut().corner_radius = val;
            },
        );
        self
    }
}

impl Default for DivStyle {
    fn default() -> Self {
        DivStyle {
            background_color: Color::TRANSPARENT,
            corner_radius: 0.0,
        }
    }
}
