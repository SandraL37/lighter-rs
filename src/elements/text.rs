use crate::{
    core::{
        arena::NodeArena,
        error::*,
        layout::LeafStyle,
        node::{NodeId, NodeKind, NodeProps, NodePropsExt},
        style::Color,
    },
    elements::{Element, ElementBuild, ElementKind},
};

#[derive(Debug)]
pub struct Text {
    node_props: NodeProps,
    props: TextProps,
    layout_style: LeafStyle,
}

impl Text {
    pub fn color(mut self, color: Color) -> Self {
        self.props.color = color;
        self
    }

    pub fn font_size(mut self, size: f32) -> Self {
        self.props.font_size = size;
        self
    }
}

#[derive(Debug, Clone)]
pub struct TextProps {
    pub content: String,
    pub color: Color,
    pub font_size: f32,
}

impl TextProps {
    pub fn content(&self) -> &str {
        &self.content
    }
}

impl Default for TextProps {
    fn default() -> Self {
        TextProps {
            content: String::new(),
            color: Color::BLACK,
            font_size: 12.0,
        }
    }
}

impl Into<ElementKind> for Text {
    fn into(self) -> ElementKind {
        ElementKind::Text(self)
    }
}

impl ElementBuild for Text {
    fn build(self, ctx: &mut NodeArena, parent: Option<NodeId>) -> Result<NodeId> {
        ctx.create_node(
            NodeKind::Text(self.props),
            self.node_props,
            parent,
            self.layout_style,
        )
    }
}

impl Element for Text {}

impl NodePropsExt for Text {
    fn props_mut(&mut self) -> &mut NodeProps {
        &mut self.node_props
    }
}

pub fn text<S: Into<String>>(content: S) -> Text {
    Text {
        node_props: NodeProps::default(),
        props: TextProps {
            content: content.into(),
            ..Default::default()
        },
        layout_style: LeafStyle::default(),
    }
}
