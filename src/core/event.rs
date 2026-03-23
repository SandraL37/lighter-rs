pub mod hit_test;

use std::rc::Rc;

use crate::core::{
    arena::node::EventHandlers,
    layout::types::{point::Point, size::Size},
};

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Copy)]
pub enum EngineEvent {
    WindowCreated,
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
    WindowDestroyed,
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Copy)]
pub enum Event {
    Click,
    MouseEnter,
    MouseLeave,
    MouseMove { position: Point<f32> },
}

pub trait MouseEvents: Sized {
    fn event_handlers(&mut self) -> &mut EventHandlers;

    fn on_click(mut self, handler: impl Fn() + 'static) -> Self {
        self.event_handlers().on_click = Some(Rc::new(handler));
        self
    }

    fn on_mouse_enter(mut self, handler: impl Fn() + 'static) -> Self {
        self.event_handlers().on_mouse_enter = Some(Rc::new(handler));
        self
    }

    fn on_mouse_leave(mut self, handler: impl Fn() + 'static) -> Self {
        self.event_handlers().on_mouse_leave = Some(Rc::new(handler));
        self
    }
}
