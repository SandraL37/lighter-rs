use crate::{
    core::{
        arena::node::{NodeStyle, NodeStyleBuilder},
        layout::{LayoutStyle, LeafStyleBuilder},
        reactive::bind::{DeferredBinding, HasDeferredBindings},
    },
    elements::text::{TextStyle, style::TextStyleBuilder},
};

#[derive(Debug, Default)]
pub struct TextBuildProps {
    pub(crate) node: NodeStyle,
    pub(crate) layout: LayoutStyle,
    pub(crate) text: TextStyle,
    pub(crate) bindings: Vec<DeferredBinding>,
}

impl HasDeferredBindings for TextBuildProps {
    type Style = TextBuildProps;

    fn bindings(&mut self) -> &mut Vec<DeferredBinding> {
        &mut self.bindings
    }

    fn style(&mut self) -> &mut Self::Style {
        self
    }
}

impl LeafStyleBuilder for TextBuildProps {
    fn layout_style(style: &mut Self::Style) -> &mut LayoutStyle {
        &mut style.layout
    }
}

impl NodeStyleBuilder for TextBuildProps {
    fn node_style(style: &mut Self::Style) -> &mut NodeStyle {
        &mut style.node
    }
}

impl TextStyleBuilder for TextBuildProps {
    fn text_style(style: &mut Self::Style) -> &mut TextStyle {
        &mut style.text
    }
}
