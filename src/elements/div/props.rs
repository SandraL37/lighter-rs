use crate::{
    core::{
        arena::node::{NodeStyle, NodeStyleBuilder},
        layout::{ContainerStyleBuilder, LayoutStyle, LeafStyleBuilder},
        reactive::bind::{DeferredBinding, HasDeferredBindings},
    },
    elements::div::{
        DivStyle, DivStyleBuilder,
        style::{DivStylePatch, DivStylePatcher},
    },
};

#[derive(Debug, Default)]
pub struct DivBuildProps {
    pub(crate) node: NodeStyle,
    pub(crate) layout: LayoutStyle,
    pub(crate) div: DivStyle,

    pub(crate) bindings: Vec<DeferredBinding>,
}

impl HasDeferredBindings for DivBuildProps {
    type Style = DivBuildProps;

    fn bindings(&mut self) -> &mut Vec<DeferredBinding> {
        &mut self.bindings
    }

    fn style(&mut self) -> &mut Self::Style {
        self
    }
}

impl LeafStyleBuilder for DivBuildProps {
    fn layout_style(style: &mut Self::Style) -> &mut LayoutStyle {
        &mut style.layout
    }
}

impl ContainerStyleBuilder for DivBuildProps {}

impl NodeStyleBuilder for DivBuildProps {
    fn node_style(style: &mut Self::Style) -> &mut NodeStyle {
        &mut style.node
    }
}

impl DivStyleBuilder for DivBuildProps {
    fn div_style(style: &mut Self::Style) -> &mut DivStyle {
        &mut style.div
    }
}

#[derive(Debug, Default)]
pub struct DivPatchProps {
    pub(crate) div: DivStylePatch,
}

impl DivStylePatcher for DivPatchProps {
    fn div_style(&mut self) -> &mut DivStylePatch {
        &mut self.div
    }
}
