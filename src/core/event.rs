pub mod hit_test;

use std::rc::Rc;

use windows::Win32::Foundation::HWND;

use crate::core::{
    arena::node::{EventHandlers, NodeId},
    layout::types::{point::Point, rect::Rect, size::Size},
    render::Dpi,
};

#[derive(Debug, Clone, Copy)]
pub enum EngineEvent {
    WindowCreated,
    WindowDestroyed,
    WindowResized {
        size: Size<usize>,
    },
    MouseMove {
        position: Point<f32>,
    },
    MouseDown {
        position: Point<f32>,
        button: MouseButton,
    },
    MouseUp {
        position: Point<f32>,
        button: MouseButton,
    },
    DpiChanged(HWND, Rect<i32>, Dpi),
    WindowFocusGained,
    WindowFocusLost,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

#[derive(Debug, Clone, Copy)]
pub enum Event {
    Click,
    MouseEnter,
    MouseLeave,
    MouseMove { position: Point<f32> },
}

pub trait MouseEvents: Sized {
    fn event_handlers(&mut self) -> &mut EventHandlers;

    fn on_click(mut self, handler: impl Fn(&mut EventContext) + 'static) -> Self {
        self.event_handlers().on_click = Some(Rc::new(handler));
        self
    }

    fn on_mouse_enter(mut self, handler: impl Fn(&mut EventContext) + 'static) -> Self {
        self.event_handlers().on_mouse_enter = Some(Rc::new(handler));
        self
    }

    fn on_mouse_leave(mut self, handler: impl Fn(&mut EventContext) + 'static) -> Self {
        self.event_handlers().on_mouse_leave = Some(Rc::new(handler));
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventPhase {
    Target,
    Bubble,
}

#[derive(Debug, Clone, Copy)]
pub struct EventContext {
    // TODO: Add event specific meta ~ position
    pub target: NodeId,
    pub current: NodeId,
    pub position: Option<Point<f32>>,
    pub phase: EventPhase,
    stop: bool,
}

impl EventContext {
    pub fn new(
        target: NodeId,
        current: NodeId,
        position: Option<Point<f32>>,
        phase: EventPhase,
    ) -> Self {
        Self {
            target,
            current,
            position,
            phase,
            stop: false,
        }
    }

    pub fn stop_propagation(&mut self) {
        self.stop = true
    }

    pub fn is_propagation_stopped(&self) -> bool {
        self.stop
    }
}
