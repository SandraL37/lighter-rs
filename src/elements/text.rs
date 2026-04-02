use std::sync::Arc;

use crate::{
    core::{
        arena::{
            NodeArena,
            node::{ EventHandlers, NodeId, NodeKind, NodeStyle, NodeStyleBuilder },
        },
        error::*,
        event::MouseEvents,
        layout::{ LayoutStyle, LeafStyleBuilder },
        reactive::{
            bind::{ DeferredBinding, HasDeferredBindings },
            dirty::DirtyFlags,
            signal::{ MaybeSignal, Signal, signal },
        },
        style::Color,
    },
    elements::Element,
};

#[derive(Debug)]
pub struct Text {
    props: TextProps,
    event_handlers: EventHandlers,
}

#[derive(Debug, Default)]
pub struct TextProps {
    node: NodeStyle,
    layout: LayoutStyle,
    text: TextStyle,
    bindings: Vec<DeferredBinding>,
}

impl HasDeferredBindings for TextProps {
    type Style = TextProps;

    fn bindings(&mut self) -> &mut Vec<DeferredBinding> {
        &mut self.bindings
    }

    fn style(&mut self) -> &mut Self::Style {
        self
    }
}

impl LeafStyleBuilder for TextProps {
    fn layout_style(style: &mut Self::Style) -> &mut LayoutStyle {
        &mut style.layout
    }
}

impl NodeStyleBuilder for TextProps {
    fn node_style(style: &mut Self::Style) -> &mut NodeStyle {
        &mut style.node
    }
}

impl HasDeferredBindings for Text {
    type Style = TextProps;

    fn bindings(&mut self) -> &mut Vec<DeferredBinding> {
        &mut self.props.bindings
    }

    fn style(&mut self) -> &mut Self::Style {
        &mut self.props
    }
}

impl LeafStyleBuilder for Text {
    fn layout_style(style: &mut Self::Style) -> &mut LayoutStyle {
        &mut style.layout
    }
}

impl NodeStyleBuilder for Text {
    fn node_style(style: &mut Self::Style) -> &mut NodeStyle {
        &mut style.node
    }
}

#[derive(Debug, Clone)]
pub struct TextStyle {
    pub content: Arc<str>,
    pub color: Color,
    pub font_size: f32,
    pub font_family: Arc<str>,
    pub font_weight: FontWeight,
}

impl Default for TextStyle {
    fn default() -> Self {
        TextStyle {
            content: Arc::from(""),
            color: Color::BLACK,
            font_size: 16.0,
            font_family: Arc::from("Segoe UI"),
            font_weight: FontWeight::NORMAL,
        }
    }
}

pub trait TextPropsExt: HasDeferredBindings + Sized {
    fn text_props(style: &mut Self::Style) -> &mut TextStyle;

    fn content(mut self, value: impl IntoTextContent) -> Self {
        self.bind(
            |style| &mut Self::text_props(style).content,
            value.into_text_content(),
            DirtyFlags::PAINT | DirtyFlags::LAYOUT,
            |data, _, val| {
                data.kind.as_text_mut().content = val;
            }
        );
        self
    }

    fn color(mut self, value: impl Into<MaybeSignal<Color>>) -> Self {
        self.bind(
            |style| &mut Self::text_props(style).color,
            value,
            DirtyFlags::PAINT,
            |data, _, val| {
                data.kind.as_text_mut().color = val;
            }
        );
        self
    }

    fn font_family(mut self, value: impl IntoTextContent) -> Self {
        self.bind(
            |style| &mut Self::text_props(style).font_family,
            value.into_text_content(),
            DirtyFlags::PAINT | DirtyFlags::LAYOUT,
            |data, _, val| {
                data.kind.as_text_mut().font_family = val;
            }
        );
        self
    }

    fn font_size(mut self, value: impl Into<MaybeSignal<f32>>) -> Self {
        self.bind(
            |style| &mut Self::text_props(style).font_size,
            value,
            DirtyFlags::PAINT | DirtyFlags::LAYOUT,
            |data, _, val| {
                data.kind.as_text_mut().font_size = val;
            }
        );
        self
    }

    fn font_weight(mut self, value: impl Into<MaybeSignal<FontWeight>>) -> Self {
        self.bind(
            |style| &mut Self::text_props(style).font_weight,
            value,
            DirtyFlags::PAINT | DirtyFlags::LAYOUT,
            |data, _, val| {
                data.kind.as_text_mut().font_weight = val;
            }
        );
        self
    }
}

impl TextPropsExt for TextProps {
    fn text_props(style: &mut Self::Style) -> &mut TextStyle {
        &mut style.text
    }
}

impl TextPropsExt for Text {
    fn text_props(style: &mut Self::Style) -> &mut TextStyle {
        &mut style.text
    }
}

impl Element for Text {
    fn build(self: Box<Self>, arena: &mut NodeArena, parent: Option<NodeId>) -> Result<NodeId> {
        let Text { props, event_handlers } = *self;

        let TextProps { node, layout, text, bindings } = props;

        let id = arena.create_node(
            NodeKind::Text(Arc::new(text)),
            node,
            parent,
            layout,
            event_handlers
        )?;

        for binding in bindings {
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
        props: TextProps::default(),
        event_handlers: EventHandlers::default(),
    }).content(content)
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
