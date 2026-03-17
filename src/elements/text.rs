use crate::{
    core::{
        arena::{ NodeArena, node::{ EventHandlers, NodeData, NodeId, NodeKind, NodeProps, NodePropsExt } },
        error::*,
        event::MouseEvents,
        layout::{ LeafStyle, LeafStylePropsExt, NodeLayout },
        reactive::{ bind::{ DeferredBinding, bind_field }, dirty::DirtyFlags, signal::{MaybeSignal, Signal, signal} },
        style::Color,
    },
    elements::Element,
};
use std::sync::Arc;

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
    fn content(mut self, value: impl Into<MaybeSignal<Arc<str>>>) -> Self {
        bind_field(
            &mut self.text_props,
            &mut self.deferred_bindings,
            value,
            DirtyFlags::PAINT | DirtyFlags::LAYOUT,
            |props| &mut props.content,
            resolve_text_props
        );

        self
    }
}

fn resolve_text_props(data: &mut NodeData, _layout: &mut NodeLayout) -> &'static mut TextProps {
    match &mut data.kind {
        NodeKind::Text(props) => Arc::make_mut(&mut props).expect("multiple references to text props"),
        _ => unreachable!(),
    }
}

pub trait TextPropsExt: Sized {
    fn text_props_mut(&mut self) -> &mut TextProps;
    fn bindings_mut(&mut self) -> &mut Vec<DeferredBinding>;

    fn color(mut self, value: impl Into<MaybeSignal<Color>>) -> Self {
        bind_field(
            self.text_props_mut(),
            self.bindings_mut(),
            value,
            DirtyFlags::PAINT,
            |props| &mut props.color,
            resolve_text_props
        );

        self
    }

    fn font_family(mut self, value: impl IntoTextContent) -> Self {
        let value: MaybeSignal<Arc<str>> = value.into_text_content();
        bind_field(
            self.text_props_mut(),
            self.bindings_mut(),
            value,
            DirtyFlags::PAINT | DirtyFlags::LAYOUT,
            |props| &mut props.font_family,
            resolve_text_props
        );
        self
    }

    fn font_size(mut self, value: impl Into<MaybeSignal<f32>>) -> Self {
        bind_field(
            self.text_props_mut(),
            self.bindings_mut(),
            value,
            DirtyFlags::PAINT | DirtyFlags::LAYOUT,
            |props| &mut props.font_size,
            resolve_text_props
        );

        self
    }

    fn font_weight(mut self, value: impl Into<MaybeSignal<FontWeight>>) -> Self {
        bind_field(
            self.text_props_mut(),
            self.bindings_mut(),
            value,
            DirtyFlags::PAINT | DirtyFlags::LAYOUT,
            |props| &mut props.font_weight,
            resolve_text_props
        );

        self
    }
}

impl TextPropsExt for Text {
    fn text_props_mut(&mut self) -> &mut TextProps {
        &mut self.text_props
    }

    fn bindings_mut(&mut self) -> &mut Vec<DeferredBinding> {
        &mut self.deferred_bindings
    }
}

impl LeafStylePropsExt for Text {
    fn leaf_style_mut(&mut self) -> &mut LeafStyle {
        &mut self.layout_style
    }

    fn bindings_mut(&mut self) -> &mut Vec<DeferredBinding> {
        &mut self.deferred_bindings
    }
}

impl NodePropsExt for Text {
    fn node_props_mut(&mut self) -> &mut NodeProps {
        &mut self.node_props
    }

    fn bindings_mut(&mut self) -> &mut Vec<DeferredBinding> {
        &mut self.deferred_bindings
    }
}

impl Element for Text {
    fn build(self: Box<Self>, arena: &mut NodeArena, parent: Option<NodeId>) -> Result<NodeId> {
        let id = arena.create_node(
            NodeKind::Text(Arc::new(self.text_props)),
            self.node_props,
            parent,
            self.layout_style,
            self.event_handlers
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
        layout_style: LeafStyle::default(),
        text_props: TextProps::default(),
        deferred_bindings: Vec::new(),
        event_handlers: EventHandlers::default(),
    }).content(content.into_text_content())
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
