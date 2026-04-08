use crate::core::{
    reactive::{bind::HasDeferredBindings, dirty::DirtyFlags},
    state::{IntoStyleProp, Property},
    style::Color,
};

#[derive(Clone, Debug)]
pub struct DivStyle {
    pub background_color: Property<Color>,
    pub corner_radius: Property<f32>,
}

impl Default for DivStyle {
    fn default() -> Self {
        DivStyle {
            background_color: Property::base(Color::TRANSPARENT),
            corner_radius: Property::base(0.0),
        }
    }
}

pub trait DivStyleBuilder: HasDeferredBindings + Sized {
    fn div_style(style: &mut Self::Style) -> &mut DivStyle;

    fn bg(mut self, color: impl IntoStyleProp<Color>) -> Self {
        let color = color.into_style_prop();

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

    fn rounded(mut self, radius: impl IntoStyleProp<f32>) -> Self {
        let radius = radius.into_style_prop();

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
