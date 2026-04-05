use crate::core::{
    reactive::{bind::HasDeferredBindings, dirty::DirtyFlags, signal::MaybeSignal},
    style::Color,
};

#[derive(Clone)]
pub struct DivStyle {
    pub background_color: Color,
    pub corner_radius: f32, // TODO: make it DefiniteDimension
}

#[derive(Default)]
pub struct DivStylePatch {
    pub(crate) patches: Vec<Box<dyn Fn(&mut DivStyle)>>,
}

impl DivStylePatch {
    pub fn bg(mut self, color: Color) -> Self {
        self.patches.push(Box::new(move |style| {
            style.background_color = color;
        }));
        self
    }

    pub fn rounded(mut self, corner_radius: f32) -> Self {
        self.patches.push(Box::new(move |style| {
            style.corner_radius = corner_radius;
        }));
        self
    }
}

impl std::fmt::Debug for DivStylePatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DivStylePatch")
            .field("patches", &self.patches.len())
            .finish()
    }
}

pub trait DivStyleBuilder: HasDeferredBindings + Sized {
    fn div_style(style: &mut Self::Style) -> &mut DivStyle;

    fn bg(mut self, color: impl Into<MaybeSignal<Color>>) -> Self {
        self.bind(
            |style| &mut Self::div_style(style).background_color,
            color,
            DirtyFlags::PAINT,
            |data, _, val| {
                if let Ok(div) = data.kind.as_div_mut() {
                    div.background_color = val;
                }
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
                if let Ok(div) = data.kind.as_div_mut() {
                    div.corner_radius = val;
                }
            },
        );
        self
    }
}

impl std::fmt::Debug for DivStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DivStyle")
            .field("background_color", &self.background_color)
            .field("corner_radius", &self.corner_radius)
            .finish()
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
