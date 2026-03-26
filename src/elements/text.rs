use crate::{
    core::{
        arena::{
            NodeArena,
            node::{EventHandlers, NodeId, NodeKind, NodeProps, NodePropsExt},
        },
        error::*,
        event::MouseEvents,
        layout::{LayoutStyle, LeafStylePropsExt},
        reactive::{
            bind::{DeferredBinding, bind_field},
            dirty::DirtyFlags,
            signal::{MaybeSignal, Signal, signal},
        },
        style::Color,
    },
    elements::Element,
};
use std::sync::Arc;

#[derive(Debug)]
pub struct Text {
    node_props: NodeProps,
    layout_style: LayoutStyle,
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

pub trait TextPropsExt: Sized {
    fn text_ctx(&mut self) -> (&mut TextProps, &mut Vec<DeferredBinding>);

    fn content(mut self, value: impl IntoTextContent) -> Self {
        let (props, bindings) = self.text_ctx();
        bind_field(
            &mut props.content,
            bindings,
            value.into_text_content(),
            DirtyFlags::PAINT | DirtyFlags::LAYOUT,
            |data, _, val| data.kind.as_text_mut().content = val,
        );
        self
    }

    fn color(mut self, value: impl Into<MaybeSignal<Color>>) -> Self {
        let (props, bindings) = self.text_ctx();
        bind_field(
            &mut props.color,
            bindings,
            value,
            DirtyFlags::PAINT,
            |data, _, val| data.kind.as_text_mut().color = val,
        );
        self
    }

    fn font_family(mut self, value: impl IntoTextContent) -> Self {
        let (props, bindings) = self.text_ctx();
        bind_field(
            &mut props.font_family,
            bindings,
            value.into_text_content(),
            DirtyFlags::PAINT | DirtyFlags::LAYOUT,
            |data, _, val| data.kind.as_text_mut().font_family = val,
        );
        self
    }

    fn font_size(mut self, value: impl Into<MaybeSignal<f32>>) -> Self {
        let (props, bindings) = self.text_ctx();
        bind_field(
            &mut props.font_size,
            bindings,
            value,
            DirtyFlags::PAINT | DirtyFlags::LAYOUT,
            |data, _, val| data.kind.as_text_mut().font_size = val,
        );
        self
    }

    fn font_weight(mut self, value: impl Into<MaybeSignal<FontWeight>>) -> Self {
        let (props, bindings) = self.text_ctx();
        bind_field(
            &mut props.font_weight,
            bindings,
            value,
            DirtyFlags::PAINT | DirtyFlags::LAYOUT,
            |data, _, val| data.kind.as_text_mut().font_weight = val,
        );
        self
    }
}

impl TextPropsExt for Text {
    fn text_ctx(&mut self) -> (&mut TextProps, &mut Vec<DeferredBinding>) {
        (&mut self.text_props, &mut self.deferred_bindings)
    }
}

impl LeafStylePropsExt for Text {
    fn ctx(&mut self) -> (&mut LayoutStyle, &mut Vec<DeferredBinding>) {
        (&mut self.layout_style, &mut self.deferred_bindings)
    }
}

impl NodePropsExt for Text {
    fn node_ctx(&mut self) -> (&mut NodeProps, &mut Vec<DeferredBinding>) {
        (&mut self.node_props, &mut self.deferred_bindings)
    }
}

impl Element for Text {
    fn build(self: Box<Self>, arena: &mut NodeArena, parent: Option<NodeId>) -> Result<NodeId> {
        let id = arena.create_node(
            NodeKind::Text(Arc::new(self.text_props)),
            self.node_props,
            parent,
            self.layout_style,
            self.event_handlers,
        )?;

        for binding in self.deferred_bindings {
            (binding.0)(id);
        }

        Ok(id)
    }
}

pub trait IntoTextContent {
    fn into_text_content(self) -> MaybeSignal<Arc<str>>;
}

impl<T: Into<Arc<str>>> IntoTextContent for T {
    fn into_text_content(self) -> MaybeSignal<Arc<str>> {
        MaybeSignal::Static(self.into())
    }
}

impl<T: std::fmt::Display + Clone + 'static> IntoTextContent for Signal<T> {
    fn into_text_content(self) -> MaybeSignal<Arc<str>> {
        let text_sig = signal::<Arc<str>>(Arc::from(self.get().to_string().as_str()));
        self.subscribe(move || {
            text_sig.set(Arc::from(self.get().to_string().as_str()));
        });
        MaybeSignal::Signal(text_sig)
    }
}

pub fn text(content: impl IntoTextContent) -> Text {
    (Text {
        node_props: NodeProps::default(),
        layout_style: LayoutStyle::default(),
        text_props: TextProps::default(),
        deferred_bindings: Vec::new(),
        event_handlers: EventHandlers::default(),
    })
    .content(content)
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
