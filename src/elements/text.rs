use std::sync::Arc;

use crate::{
    core::{
        arena::NodeArena,
        cx::{Cx, DeferredBinding, ReactivePropsExt},
        dirty::DirtyFlags,
        error::*,
        layout::{LeafStyle, LeafStylePropsExt},
        node::{NodeId, NodeKind, NodeProps, NodePropsExt},
        signal::{Reactive, ReadSignal},
        style::Color,
    },
    elements::Element,
};

#[derive(Debug)]
pub struct Text {
    node_props: NodeProps,
    layout_style: LeafStyle,
    text_props: TextProps,
    deferred_bindings: Vec<DeferredBinding>,
}

#[derive(Debug, Clone)]
pub struct TextProps {
    pub content: Arc<str>,
    pub color: Color,
    pub font_size: f32,
}

impl Default for TextProps {
    fn default() -> Self {
        TextProps {
            content: Arc::from(""),
            color: Color::BLACK,
            font_size: 12.0,
        }
    }
}

impl Text {
    fn content(mut self, value: impl Into<Reactive<Arc<str>>>) -> Self {
        self.bind(
            value,
            &mut |this, v| {
                this.text_props_mut().content = v;
            },
            DirtyFlags::PAINT,
            |data, _, v| {
                data.kind.as_text_mut().content = v;
            },
        );

        self
    }
}

pub trait TextPropsExt: ReactivePropsExt + LeafStylePropsExt + NodePropsExt {
    fn text_props_mut(&mut self) -> &mut TextProps;

    fn color(mut self, value: impl Into<Reactive<Color>>) -> Self {
        self.bind(
            value,
            &mut |this, v| {
                this.text_props_mut().color = v;
            },
            DirtyFlags::PAINT,
            |data, _, v| {
                data.kind.as_text_mut().color = v;
            },
        );

        self
    }

    fn font_size(mut self, value: impl Into<Reactive<f32>>) -> Self {
        self.bind(
            value,
            &mut |this, v| {
                this.text_props_mut().font_size = v;
            },
            DirtyFlags::PAINT | DirtyFlags::LAYOUT,
            |data, _, v| {
                data.kind.as_text_mut().font_size = v;
            },
        );

        self
    }
}

impl TextPropsExt for Text {
    fn text_props_mut(&mut self) -> &mut TextProps {
        &mut self.text_props
    }
}

impl ReactivePropsExt for Text {
    fn deferred_bindings(&mut self) -> &mut Vec<DeferredBinding> {
        &mut self.deferred_bindings
    }
}

impl LeafStylePropsExt for Text {
    fn leaf_style_mut(&mut self) -> &mut LeafStyle {
        &mut self.layout_style
    }
}

impl NodePropsExt for Text {
    fn node_props_mut(&mut self) -> &mut NodeProps {
        &mut self.node_props
    }
}

impl Element for Text {
    fn build(
        self: Box<Self>,
        arena: &mut NodeArena,
        cx: &mut Cx,
        parent: Option<NodeId>,
    ) -> Result<NodeId> {
        let id = arena.create_node(
            NodeKind::Text(self.text_props),
            self.node_props,
            parent,
            self.layout_style,
        )?;

        for binding in self.deferred_bindings {
            (binding.0)(id, cx);
        }

        Ok(id)
    }
}

pub trait IntoTextContent {
    fn into_text_content(self) -> Reactive<Arc<str>>;
}

impl<T> IntoTextContent for T
where
    T: Into<Arc<str>> + Clone,
{
    fn into_text_content(self) -> Reactive<Arc<str>> {
        Reactive::from(self).map(|v: T| v.into())
    }
}

impl<T> IntoTextContent for ReadSignal<T>
where
    T: Into<Arc<str>> + Clone,
{
    fn into_text_content(self) -> Reactive<Arc<str>> {
        Reactive::Dynamic(self).map(|v: T| v.into())
    }
}

pub fn text(content: impl IntoTextContent) -> Text {
    Text {
        node_props: NodeProps::default(),
        layout_style: LeafStyle::default(),
        text_props: TextProps::default(),
        deferred_bindings: Vec::new(),
    }
    .content(content.into_text_content())
}
