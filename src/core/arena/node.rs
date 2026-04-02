use std::{rc::Rc, sync::Arc};

use crate::{
    core::{
        error::*,
        event::EventContext,
        interaction::{InteractionState, PatchValue, StatePatches, select_patch},
        reactive::{bind::HasDeferredBindings, dirty::DirtyFlags, signal::MaybeSignal},
        style::Transform,
    },
    elements::{
        div::style::{DivStyle, DivStylePatch},
        text::style::{TextStyle, TextStylePatch},
    },
};

slotmap::new_key_type! {
    pub struct NodeId;
}

// TODO: Check coherence.
#[derive(Debug, Clone, Copy)]
pub struct NodeRuntimeMeta {
    pub focusable: bool,
    pub pointer_events: bool,
    pub transition_profile_id: Option<u16>,
}

impl Default for NodeRuntimeMeta {
    fn default() -> Self {
        Self {
            focusable: false,
            pointer_events: true,
            transition_profile_id: None,
        }
    }
}

#[derive(Debug)]
pub struct NodeData {
    // Element-specific visual payload (DivStyle/TextStyle)
    pub kind: NodeKind,
    // Shared node visual style (opacity/z/transform)
    pub style: NodeStyle,
    // Dirty flags for render/layout invalidation.
    pub dirty: DirtyFlags,
    // Interaction bits (hover/active/focus/disabled)
    pub interaction_state: InteractionState,
    // Runtime metadata used by focus/transition/event routing.
    pub runtime_meta: NodeRuntimeMeta,
    pub state_styles: NodeStateStyles,
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
}

impl Default for EventHandlers {
    fn default() -> Self {
        Self {
            on_click: None,
            on_mouse_enter: None,
            on_mouse_leave: None,
        }
    }
}

impl std::fmt::Debug for EventHandlers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventHandlers")
            .field("on_click", &self.on_click.is_some())
            .field("on_mouse_enter", &self.on_mouse_enter.is_some())
            .field("on_mouse_leave", &self.on_mouse_leave.is_some())
            .finish()
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct NodeStylePatch {
    pub opacity: PatchValue<f32>,
    pub z_index: PatchValue<i32>,
    pub transform: PatchValue<Option<Transform>>,
}

impl NodeStyle {
    pub fn resolve_with_state(
        &self,
        state: InteractionState,
        patches: &StatePatches<NodeStylePatch>,
    ) -> Self {
        let mut out = self.clone(); // TODO: find better solutions

        if let Some(p) = select_patch(state, patches) {
            if let PatchValue::Set(v) = p.opacity {
                out.opacity = v;
            }
            if let PatchValue::Set(v) = p.z_index {
                out.z_index = v;
            }
            if let PatchValue::Set(v) = p.transform {
                out.transform = v;
            }
        }

        out
    }
}

#[derive(Debug, Clone, Default)]
pub struct NodeStateStyles {
    pub node: StatePatches<NodeStylePatch>,
    pub div: StatePatches<DivStylePatch>,
    pub text: StatePatches<TextStylePatch>,
}
