use crate::core::{
    reactive::{bind::HasDeferredBindings, dirty::DirtyFlags, signal::MaybeSignal},
    state::Prop,
    style::Color,
};

#[derive(Clone, Debug)]
pub struct DivStyle {
    pub background_color: Prop<Color>,
    pub corner_radius: Prop<f32>,
}

impl Default for DivStyle {
    fn default() -> Self {
        DivStyle {
            background_color: Prop::new(Color::TRANSPARENT),
            corner_radius: Prop::new(0.0),
        }
    }
}

#[derive(Debug, Default)]
pub struct DivStylePatch {
    pub(crate) background_color: Option<Color>,
    pub(crate) corner_radius: Option<f32>,
}

pub trait DivStylePatcher: Sized {
    fn div_style(&mut self) -> &mut DivStylePatch;

    fn bg(mut self, color: Color) -> Self {
        self.div_style().background_color = Some(color);
        self
    }

    fn rounded(mut self, corner_radius: f32) -> Self {
        self.div_style().corner_radius = Some(corner_radius);
        self
    }
}

pub trait DivStyleBuilder: HasDeferredBindings + Sized {
    fn div_style(style: &mut Self::Style) -> &mut DivStyle;

    fn bg(mut self, color: impl Into<MaybeSignal<Color>>) -> Self {
        self.bind(
            |style| &mut Self::div_style(style).background_color.base,
            color,
            DirtyFlags::PAINT,
            |data, _, val| {
                if let Ok(div) = data.kind.as_div_mut() {
                    div.background_color.base = val;
                }
            },
        );
        self
    }

    fn rounded(mut self, radius: impl Into<MaybeSignal<f32>>) -> Self {
        self.bind(
            |style| &mut Self::div_style(style).corner_radius.base,
            radius,
            DirtyFlags::PAINT,
            |data, _, val| {
                if let Ok(div) = data.kind.as_div_mut() {
                    div.corner_radius.base = val;
                }
            },
        );
        self
    }
}
