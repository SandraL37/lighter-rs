use std::{ rc::Rc, sync::Arc };

use crate::{
    core::{
        layout::NodeLayout, reactive::{ bind::{ DeferredBinding, bind_field }, dirty::DirtyFlags, signal::MaybeSignal }, style::Transform
    },
    elements::{ div::DivProps, text::TextProps },
};

slotmap::new_key_type! {
    pub struct NodeId;
}

#[derive(Debug)]
pub struct NodeData {
    pub kind: NodeKind,
    pub props: NodeProps,
    pub dirty: DirtyFlags,
    pub event_handlers: EventHandlers,
}

#[derive(Debug, Clone)]
pub enum NodeKind {
    Div(Arc<DivProps>),
    Text(Arc<TextProps>),
}

impl NodeKind {
    pub fn as_div_mut(&mut self) -> &mut DivProps {
        if let NodeKind::Div(props) = self {
            Arc::make_mut(props)
        } else {
            unreachable!("Not a div"); // TODO: check correctness
        }
    }

    pub fn as_text_mut(&mut self) -> &mut TextProps {
        if let NodeKind::Text(props) = self {
            Arc::make_mut(props)
        } else {
            unreachable!("Not a text");
        }
    }
}

//  TODO: bench this
#[derive(Debug)]
pub struct NodeProps {
    pub opacity: f32,
    pub z_index: i32,
    pub transform: Option<Transform>,
}

impl Default for NodeProps {
    fn default() -> Self {
        NodeProps {
            opacity: 1.0,
            z_index: 0,
            transform: None, // TODO: benchmark if Box<Transform> is faster
        }
    }
}

fn resolve_node_props(data: &mut NodeData, _layout: &mut NodeLayout) -> &'static mut NodeProps {
    &mut data.props
}

pub trait NodePropsExt: Sized {
    fn node_props_mut(&mut self) -> &mut NodeProps;
    fn bindings_mut(&mut self) -> &mut Vec<DeferredBinding>;

    fn opacity(mut self, value: impl Into<MaybeSignal<f32>>) -> Self {
        bind_field(
            self.node_props_mut(),
            self.bindings_mut(),
            value,
            DirtyFlags::PAINT,
            |props| &mut props.opacity,
            resolve_node_props
        );
        self
    }

    fn z(mut self, value: impl Into<MaybeSignal<i32>>) -> Self {
        bind_field(
            self.node_props_mut(),
            self.bindings_mut(),
            value,
            DirtyFlags::PAINT,
            |props| &mut props.z_index,
            resolve_node_props
        );
        self
    }

    fn transform(mut self, value: impl Into<MaybeSignal<Transform>>) -> Self {
        let value = value.into().map(|transform| Some(transform));
        
        bind_field(
            self.node_props_mut(),
            self.bindings_mut(),
            value,
            DirtyFlags::PAINT,
            |props| &mut props.transform,
            resolve_node_props
        );
        self
    }
}

pub type EventCallback = Rc<dyn Fn()>;

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
