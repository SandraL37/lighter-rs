use std::{rc::Rc, sync::Arc};

use crate::{
    core::{
        error::*,
        event::EventContext,
        interaction::InteractionState,
        reactive::{bind::HasDeferredBindings, dirty::DirtyFlags, signal::MaybeSignal},
        style::Transform,
    },
    elements::{div::style::DivStyle, text::style::TextStyle},
};

slotmap::new_key_type! {
    pub struct NodeId;
}

#[derive(Debug)]
pub struct NodeData {
    pub kind: NodeKind,

    pub style: NodeStyle,

    pub dirty: DirtyFlags,

    pub interaction_state: InteractionState,
    pub event_handlers: EventHandlers,
}

#[derive(Debug, Clone)]
pub enum NodeKind {
    Div(Arc<DivStyle>),
    Text(Arc<TextStyle>),
}

impl NodeKind {
    #[cold]
    fn kind_name(&self) -> &'static str {
        match self {
            NodeKind::Div(_) => "Div",
            NodeKind::Text(_) => "Text",
        }
    }

    pub fn as_div_mut(&mut self) -> Result<&mut DivStyle> {
        match self {
            NodeKind::Div(props) => Ok(Arc::make_mut(props)),
            _ => Err(Error::NodeKindMismatch {
                expected: "Div",
                found: self.kind_name(),
            }),
        }
    }

    pub fn as_text_mut(&mut self) -> Result<&mut TextStyle> {
        match self {
            NodeKind::Text(props) => Ok(Arc::make_mut(props)),
            _ => Err(Error::NodeKindMismatch {
                expected: "Text",
                found: self.kind_name(),
            }),
        }
    }
}

//  TODO: bench this
#[derive(Debug, Clone)]
pub struct NodeStyle {
    pub opacity: f32,
    pub z_index: i32,
    pub transform: Option<Transform>,
}

impl Default for NodeStyle {
    fn default() -> Self {
        NodeStyle {
            opacity: 1.0,
            z_index: 0,
            transform: None, // TODO: benchmark if Box<Transform> is faster
        }
    }
}

pub trait NodeStyleBuilder: HasDeferredBindings + Sized {
    fn node_style(style: &mut Self::Style) -> &mut NodeStyle;

    fn opacity(mut self, value: impl Into<MaybeSignal<f32>>) -> Self {
        self.bind(
            |style| &mut Self::node_style(style).opacity,
            value,
            DirtyFlags::PAINT,
            |data, _, val| {
                data.style.opacity = val;
            },
        );
        self
    }

    fn z(mut self, value: impl Into<MaybeSignal<i32>>) -> Self {
        self.bind(
            |style| &mut Self::node_style(style).z_index,
            value,
            DirtyFlags::PAINT,
            |data, _, val| {
                data.style.z_index = val;
            },
        );
        self
    }

    fn transform(mut self, value: impl Into<MaybeSignal<Transform>>) -> Self {
        let value = value.into().map(Some);
        self.bind(
            |style| &mut Self::node_style(style).transform,
            value,
            DirtyFlags::PAINT,
            |data, _, val| {
                data.style.transform = val;
            },
        );
        self
    }
}

pub type EventCallback = Rc<dyn Fn(&mut EventContext)>;

pub struct EventHandlers {
    pub on_click: Option<EventCallback>,
    pub on_mouse_enter: Option<EventCallback>,
    pub on_mouse_leave: Option<EventCallback>,
    pub on_mouse_over: Option<EventCallback>,
    pub on_mouse_out: Option<EventCallback>,
}

impl Default for EventHandlers {
    fn default() -> Self {
        Self {
            on_click: None,
            on_mouse_enter: None,
            on_mouse_leave: None,
            on_mouse_over: None,
            on_mouse_out: None,
        }
    }
}

impl std::fmt::Debug for EventHandlers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventHandlers")
            .field("on_click", &self.on_click.is_some())
            .field("on_mouse_enter", &self.on_mouse_enter.is_some())
            .field("on_mouse_leave", &self.on_mouse_leave.is_some())
            .field("on_mouse_over", &self.on_mouse_over.is_some())
            .field("on_mouse_out", &self.on_mouse_out.is_some())
            .finish()
    }
}
