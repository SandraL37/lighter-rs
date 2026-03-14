use crate::{
    core::{
        arena::{
            NodeArena,
            node::{EventHandlers, NodeId, NodeKind, NodeProps, NodePropsExt},
        },
        error::*,
        event::MouseEvents,
        layout::{LeafStyle, LeafStylePropsExt},
        reactive::{
            cx::{Cx, DeferredBinding, ReactivePropsExt},
            dirty::DirtyFlags,
            signal::{Reactive, ReadSignal},
        },
        style::Color,
    },
    elements::Element,
};
use std::sync::Arc;

#[derive(Debug)]
pub struct Text {
    node_props: NodeProps,
    layout_style: LeafStyle,
    text_props: TextProps,
    deferred_bindings: Vec<DeferredBinding>,
    event_handlers: EventHandlers,
}

#[derive(Debug, Clone)]
pub struct TextProps {
    pub content: Arc<str>,
    pub color: Color,
    pub font_size: f32,
    pub font_family: Arc<str>,
    pub font_weight: FontWeight,
}

impl Default for TextProps {
    fn default() -> Self {
        TextProps {
            content: Arc::from(""),
            color: Color::BLACK,
            font_size: 16.0,
            font_family: Arc::from("Segoe UI"),
            font_weight: FontWeight::NORMAL,
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
            DirtyFlags::PAINT | DirtyFlags::LAYOUT,
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

    fn font_family(mut self, value: impl IntoTextContent) -> Self {
        let value: Reactive<Arc<str>> = value.into_text_content();
        self.bind(
            value,
            &mut |this, v| {
                this.text_props_mut().font_family = v;
            },
            DirtyFlags::PAINT | DirtyFlags::LAYOUT,
            |data, _, v| {
                data.kind.as_text_mut().font_family = v;
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

    fn font_weight(mut self, value: impl Into<Reactive<FontWeight>>) -> Self {
        self.bind(
            value,
            &mut |this, v| {
                this.text_props_mut().font_weight = v;
            },
            DirtyFlags::PAINT | DirtyFlags::LAYOUT,
            |data, _, v| {
                data.kind.as_text_mut().font_weight = v;
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
            NodeKind::Text(Arc::new(self.text_props)),
            self.node_props,
            parent,
            self.layout_style,
            self.event_handlers,
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
        event_handlers: EventHandlers::default(),
    }
    .content(content.into_text_content())
}

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
pub struct FontWeight(pub u16);

impl FontWeight {
    pub const THIN: FontWeight = FontWeight(100);
    pub const EXTRA_LIGHT: FontWeight = FontWeight(200);
    pub const LIGHT: FontWeight = FontWeight(300);
    pub const REGULAR: FontWeight = FontWeight(400);
    pub const NORMAL: FontWeight = FontWeight::REGULAR;
    pub const MEDIUM: FontWeight = FontWeight(500);
    pub const SEMI_BOLD: FontWeight = FontWeight(600);
    pub const BOLD: FontWeight = FontWeight(700);
    pub const EXTRA_BOLD: FontWeight = FontWeight(800);
    pub const BLACK: FontWeight = FontWeight(900);
    pub const EXTRA_BLACK: FontWeight = FontWeight(950);

    pub fn new(raw: u16) -> FontWeight {
        let raw = raw.clamp(1, 1000);
        FontWeight(raw)
    }
}

impl MouseEvents for Text {
    fn event_handlers(&mut self) -> &mut EventHandlers {
        &mut self.event_handlers
    }
}
