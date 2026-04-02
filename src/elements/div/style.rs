use crate::core::{
    interaction::{InteractionState, StatePatches, select_patch},
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

impl Default for DivStyle {
    fn default() -> Self {
        DivStyle {
            background_color: Color::TRANSPARENT,
            corner_radius: 0.0,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DivStylePatch {
    pub background_color: Option<Color>,
    pub corner_radius: Option<f32>,
}

impl DivStyle {
    pub fn resolve_with_state(
        &self,
        state: InteractionState,
        patches: &StatePatches<DivStylePatch>,
    ) -> Self {
        let mut out = self.clone();
        if let Some(p) = select_patch(state, patches) {
            if let Some(v) = p.background_color {
                out.background_color = v;
            }
            if let Some(v) = p.corner_radius {
                out.corner_radius = v;
            }
        }
        out
    }
}

pub fn div_patch_dirty_flags(p: &DivStylePatch) -> DirtyFlags {
    let mut flags = DirtyFlags::empty();
    if p.background_color.is_some() {
        flags |= DirtyFlags::PAINT;
    }
    if p.corner_radius.is_some() {
        flags |= DirtyFlags::PAINT;
    }
    flags
}
