use crate::{
    core::{
        arena::NodeArena,
        error::*,
        layout::{
            AlignItems, ContainerStyle, DefiniteDimension, Dimension, FlexDirection, FlexWrap,
            JustifyContent, Margin, Padding,
        },
        node::{NodeId, NodeKind, NodeProps, NodePropsExt},
        style::Color,
    },
    elements::{Element, ElementBuild, ElementKind},
};

#[derive(Debug)]
pub struct Div {
    node_props: NodeProps,
    props: DivProps,
    layout_style: ContainerStyle,
    children: Vec<ElementKind>,
}

impl Div {
    pub fn w(mut self, width: Dimension) -> Self {
        self.layout_style.size.width = width;
        self
    }

    pub fn h(mut self, height: Dimension) -> Self {
        self.layout_style.size.height = height;
        self
    }

    pub fn size(mut self, size: Dimension) -> Self {
        self.layout_style.size.width = size;
        self.layout_style.size.height = size;
        self
    }

    pub fn max_w(mut self, max_width: Dimension) -> Self {
        self.layout_style.max_size.width = max_width;
        self
    }

    pub fn max_h(mut self, max_height: Dimension) -> Self {
        self.layout_style.max_size.height = max_height;
        self
    }

    pub fn max_size(mut self, max_width: Dimension, max_height: Dimension) -> Self {
        self.layout_style.max_size.width = max_width;
        self.layout_style.max_size.height = max_height;
        self
    }

    pub fn p(mut self, padding: Padding) -> Self {
        self.layout_style.padding = padding;
        self
    }

    pub fn m(mut self, margin: Margin) -> Self {
        self.layout_style.margin = margin;
        self
    }

    pub fn align(mut self, align_items: AlignItems) -> Self {
        self.layout_style.align_items = align_items;
        self
    }

    pub fn justify(mut self, justify_content: JustifyContent) -> Self {
        self.layout_style.justify_content = justify_content;
        self
    }

    pub fn gap_x(mut self, gap: DefiniteDimension) -> Self {
        self.layout_style.gap.width = gap;
        self
    }

    pub fn gap_y(mut self, gap: DefiniteDimension) -> Self {
        self.layout_style.gap.height = gap;
        self
    }

    pub fn gap(mut self, gap: DefiniteDimension) -> Self {
        self.layout_style.gap.width = gap;
        self.layout_style.gap.height = gap;
        self
    }

    pub fn flex_direction(mut self, direction: FlexDirection) -> Self {
        self.layout_style.flex_direction = direction;
        self
    }

    pub fn flex_wrap(mut self, wrap: FlexWrap) -> Self {
        self.layout_style.flex_wrap = wrap;
        self
    }

    pub fn bg(mut self, color: Color) -> Self {
        self.props.background_color = color;
        self
    }

    pub fn child(mut self, child: impl Element) -> Self {
        self.children.push(child.into());
        self
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DivProps {
    pub background_color: Color,
}

impl Default for DivProps {
    fn default() -> Self {
        DivProps {
            background_color: Color::TRANSPARENT,
        }
    }
}

impl Into<ElementKind> for Div {
    fn into(self) -> ElementKind {
        ElementKind::Div(self)
    }
}

impl ElementBuild for Div {
    fn build(self, ctx: &mut NodeArena, parent: Option<NodeId>) -> Result<NodeId> {
        let id = ctx.create_node(
            NodeKind::Div(self.props),
            self.node_props,
            parent,
            self.layout_style,
        )?;

        for child in self.children {
            child.build(ctx, Some(id))?;
        }

        Ok(id)
    }
}

impl Element for Div {}

impl NodePropsExt for Div {
    fn props_mut(&mut self) -> &mut NodeProps {
        &mut self.node_props
    }
}

pub fn div() -> Div {
    Div {
        node_props: NodeProps::default(),
        props: DivProps::default(),
        children: Vec::new(),
        layout_style: ContainerStyle::default(),
    }
}
