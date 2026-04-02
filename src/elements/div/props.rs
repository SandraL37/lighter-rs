use crate::{
    core::{
        arena::node::{NodeStyle, NodeStyleBuilder},
        layout::{ContainerStyleBuilder, LayoutStyle, LeafStyleBuilder},
        reactive::bind::{DeferredBinding, HasDeferredBindings},
    },
    elements::div::{DivStyle, DivStyleBuilder},
};

#[derive(Debug, Default)]
pub struct DivProps {
    pub(crate) node: NodeStyle,
    pub(crate) layout: LayoutStyle,
    pub(crate) div: DivStyle,

    pub(crate) bindings: Vec<DeferredBinding>,
}

impl HasDeferredBindings for DivProps {
    type Style = DivProps;

    fn bindings(&mut self) -> &mut Vec<DeferredBinding> {
        &mut self.bindings
    }

    fn style(&mut self) -> &mut Self::Style {
        self
    }
}

impl LeafStyleBuilder for DivProps {
    fn layout_style(style: &mut Self::Style) -> &mut LayoutStyle {
        &mut style.layout
    }
}

impl ContainerStyleBuilder for DivProps {}

impl NodeStyleBuilder for DivProps {
    fn node_style(style: &mut Self::Style) -> &mut NodeStyle {
        &mut style.node
    }
}

impl DivStyleBuilder for DivProps {
    fn div_style(style: &mut Self::Style) -> &mut DivStyle {
        &mut style.div
    }
}
