pub mod trace;

use std::sync::Arc;

use windows::Win32::{Foundation::*, System::Threading::*, UI::WindowsAndMessaging::*};

#[cfg(debug_assertions)]
use crate::core::app::engine::trace::{trace_engine_event, trace_hover_change};

use crate::{
    core::{
        app::custom_messages,
        arena::{
            NodeArena,
            node::{InteractionState, NodeId, NodeKind},
            tree::TreeContext,
        },
        error::*,
        event::{EngineEvent, EventContext, EventPhase, MouseButton, hit_test::hit_test},
        layout::{
            AvailableSpace, LayoutContext,
            types::{point::Point, rect::Rect, size::Size},
        },
        reactive::{dirty::DirtyFlags, runtime::Runtime},
        render::{Dpi, RenderCommand, Renderer},
        style::Transform,
    },
    elements::Element,
};

pub struct Engine<R: Renderer> {
    arena: NodeArena,
    root: NodeId,
    renderer: R,
    size: Size<usize>,
    hover_path: Vec<NodeId>,
}

#[derive(Debug)]
pub struct HoverDelta {
    leaving: Vec<NodeId>,
    entering: Vec<NodeId>,
    old_leaf: Option<NodeId>,
    new_leaf: Option<NodeId>,
}

fn diff_hover_paths(old_path: &[NodeId], new_path: &[NodeId]) -> HoverDelta {
    let leaving = old_path
        .iter()
        .filter(|id| !new_path.contains(id))
        .copied()
        .collect();

    let entering = new_path
        .iter()
        .filter(|id| !old_path.contains(id))
        .copied()
        .collect();

    let old_leaf = old_path.last().copied();
    let new_leaf = new_path.last().copied();

    HoverDelta {
        leaving,
        entering,
        old_leaf,
        new_leaf,
    }
}

impl<R: Renderer> Engine<R> {
    pub fn new(renderer: R, root: Box<dyn Element>, size: Size<usize>) -> Result<Self> {
        let mut arena = NodeArena::new();
        let root = root.build(&mut arena, None)?;
        Ok(Engine {
            arena,
            root,
            renderer,
            size,
            hover_path: Vec::new(),
        })
    }

    fn handle_window_created(&mut self, frame: &mut bool) -> Result<()> {
        *frame = true;
        Ok(())
    }

    fn handle_window_resized(&mut self, new_size: Size<usize>, frame: &mut bool) -> Result<()> {
        let old_size = self.size;

        if !(old_size == new_size) {
            self.resize(new_size)?;
            *frame = true;
        }

        Ok(())
    }

    fn handle_mouse_move(&mut self, position: Point<f32>) -> Result<()> {
        let new_hover_path = hit_test(&self.arena, self.root, position);
        let hover_delta = diff_hover_paths(&self.hover_path, &new_hover_path);

        if new_hover_path != self.hover_path {
            #[cfg(debug_assertions)]
            if hover_delta.old_leaf != hover_delta.new_leaf {
                #[cfg(debug_assertions)]
                trace_hover_change(&hover_delta);
            }

            for node_id in hover_delta.leaving {
                self.set_state(node_id, InteractionState::HOVER, false)?;
                self.set_state(node_id, InteractionState::ACTIVE, false)?;
            }

            for node_id in hover_delta.entering {
                self.set_state(node_id, InteractionState::HOVER, true)?;
            }

            if hover_delta.old_leaf != hover_delta.new_leaf {
                if let Some(old) = hover_delta.old_leaf
                    && let Ok(data) = self.arena.get_data(old)
                    && let Some(cb) = &data.event_handlers.on_mouse_leave
                {
                    let mut ctx = EventContext::new(old, old, Some(position), EventPhase::Target);
                    cb(&mut ctx)
                }

                if let Some(new) = hover_delta.new_leaf
                    && let Ok(data) = self.arena.get_data(new)
                    && let Some(cb) = &data.event_handlers.on_mouse_enter
                {
                    let mut ctx = EventContext::new(new, new, Some(position), EventPhase::Target);
                    cb(&mut ctx)
                }
            }

            self.hover_path = new_hover_path;
        }

        Ok(())
    }

    fn handle_mouse_down(&mut self, position: Point<f32>) -> Result<()> {
        let hit_path = hit_test(&self.arena, self.root, position);

        let Some(target) = hit_path.last().copied() else {
            return Ok(());
        };

        for (idx, node_id) in hit_path.iter().rev().copied().enumerate() {
            self.set_state(node_id, InteractionState::ACTIVE, true)?;

            if let Ok(data) = self.arena.get_data(node_id) {
                if let Some(cb) = &data.event_handlers.on_click {
                    let phase = if idx == 0 {
                        EventPhase::Target
                    } else {
                        EventPhase::Bubble
                    };
                    let mut ctx = EventContext::new(target, node_id, Some(position), phase);

                    cb(&mut ctx);

                    if ctx.is_propagation_stopped() {
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    fn handle_mouse_up(&mut self, position: Point<f32>) -> Result<()> {
        let hit_path = hit_test(&self.arena, self.root, position);

        for &node_id in hit_path.iter().rev() {
            self.set_state(node_id, InteractionState::ACTIVE, false)?;
        }

        Ok(())
    }

    fn handle_window_destroyed(&self) -> Result<()> {
        unsafe {
            PostThreadMessageW(
                GetCurrentThreadId(),
                custom_messages::WINDOWCLOSED,
                WPARAM(0),
                LPARAM(0),
            )?;
        }

        Ok(())
    }

    fn handle_dpi_changed(
        &mut self,
        hwnd: HWND,
        rect: Rect<i32>,
        dpi: Dpi,
        frame: &mut bool,
    ) -> Result<()> {
        self.renderer.set_dpi(dpi)?;

        unsafe {
            SetWindowPos(
                hwnd,
                None,
                rect.location.x,
                rect.location.y,
                rect.size.width,
                rect.size.height,
                SWP_NOZORDER | SWP_NOACTIVATE,
            )?
        };

        *frame = true;

        Ok(())
    }

    fn handle_focus_gained(&self) -> Result<()> {
        Ok(())
    }

    fn handle_window_focus_lost(&mut self, force_frame: &mut bool) -> Result<()> {
        let hovered = self.hover_path.clone();

        for node_id in hovered {
            self.set_state(node_id, InteractionState::ACTIVE, false)?;
        }

        *force_frame = true;
        Ok(())
    }

    pub fn dispatch_event(&mut self, event: EngineEvent) {
        let mut force_frame = false;

        #[cfg(debug_assertions)]
        trace_engine_event(&event);

        let mut result = || -> Result<()> {
            match event {
                EngineEvent::WindowCreated => self.handle_window_created(&mut force_frame),

                EngineEvent::WindowResized { size: new_size } => {
                    self.handle_window_resized(new_size, &mut force_frame)
                }

                EngineEvent::MouseMove { position } => self.handle_mouse_move(position),

                EngineEvent::MouseDown {
                    position,
                    button: MouseButton::Left,
                } => self.handle_mouse_down(position),

                EngineEvent::MouseUp {
                    position,
                    button: MouseButton::Left,
                } => self.handle_mouse_up(position),

                EngineEvent::WindowDestroyed => self.handle_window_destroyed(),

                EngineEvent::DpiChanged(hwnd, rect, dpi) => {
                    self.handle_dpi_changed(hwnd, rect, dpi, &mut force_frame)
                }

                EngineEvent::WindowFocusGained => self.handle_focus_gained(),

                EngineEvent::WindowFocusLost => self.handle_window_focus_lost(&mut force_frame),

                _ => Ok(()),
            }?;

            if Runtime::has_updates()
                || force_frame
                || self
                    .arena
                    .is_any_dirty(DirtyFlags::PAINT | DirtyFlags::LAYOUT)
            {
                self.frame()?;
            }

            Ok(())
        };

        // TODO: improve error handling in some way
        #[cfg(debug_assertions)]
        if let Err(error) = result() {
            eprintln!("{error:?}");
        }
        #[cfg(not(debug_assertions))]
        let _ = result();
    }

    fn set_state(&mut self, node_id: NodeId, state: InteractionState, on: bool) -> Result<()> {
        let data = self.arena.get_data_mut(node_id)?;
        let before = data.interaction_state;

        data.interaction_state.set_flag(state, on);

        if data.interaction_state != before {
            self.arena.mark_dirty(node_id, DirtyFlags::PAINT)?;
        }

        Ok(())
    }

    fn build_render_list(
        &mut self,
        layout_dirty: bool,
        paint_dirty: bool,
    ) -> Result<Vec<RenderCommand>> {
        let cursor = Point::xy(0.0, 0.0);
        let mut commands = Vec::new();
        let mut redrawn_nodes = Vec::new();

        self.arena.traverse(
            self.root,
            &mut (|node_id, node, layout, cursor: Point<f32>| -> Point<f32> {
                let cursor = Point::xy(
                    cursor.x + layout.computed.location.x,
                    cursor.y + layout.computed.location.y,
                );

                let unrounded_bounds = Rect::xywh(
                    cursor.x,
                    cursor.y,
                    layout.unrounded.size.width,
                    layout.unrounded.size.height,
                );

                if layout_dirty || paint_dirty {
                    commands.push(match &node.kind {
                        NodeKind::Div(props) => RenderCommand::Rect {
                            bounds: unrounded_bounds,
                            color: props.background_color,
                            corner_radius: props.corner_radius,
                            opacity: node.style.opacity,
                            transform: node.style.transform.unwrap_or(Transform::IDENTITY),
                            z_index: node.style.z_index,
                        },
                        NodeKind::Text(props) => RenderCommand::Text {
                            bounds: unrounded_bounds,
                            props: Arc::clone(props), // TODO: is there a better way
                            opacity: node.style.opacity,
                            transform: node.style.transform.unwrap_or(Transform::IDENTITY),
                            z_index: node.style.z_index,
                        },
                    });
                    redrawn_nodes.push(node_id);
                }

                cursor
            }),
            cursor,
        );

        for node_id in redrawn_nodes {
            self.arena.mark_clean(node_id, DirtyFlags::PAINT)?;
        }

        Ok(commands)
    }

    pub fn frame(&mut self) -> Result<()> {
        let pending = Runtime::drain_updates();

        // let mut changed_nodes = vec![];

        for update in pending {
            if let Ok((data, layout)) = self.arena.get_data_layout_mut(update.node_id) {
                // changed_nodes.push(update.node_id);
                (update.apply)(data, layout);
            }

            let _ = self.arena.mark_dirty(update.node_id, update.flags);
        }

        let is_layout_dirty = self.arena.is_any_dirty(DirtyFlags::LAYOUT);
        let is_paint_dirty = self.arena.is_any_dirty(DirtyFlags::PAINT);

        if is_layout_dirty {
            let mut layout_context = LayoutContext {
                root: self.root,
                arena: &mut self.arena,
                renderer: &mut self.renderer,
            };

            layout_context.compute_layout(Size::wh(
                AvailableSpace::Definite(self.size.width as f32),
                AvailableSpace::Definite(self.size.height as f32),
            ));
        }

        if is_layout_dirty || is_paint_dirty {
            let mut commands = self.build_render_list(is_layout_dirty, is_paint_dirty)?;

            //  O(nlogn) sort is not ideal TODO: optimize
            // There are lots of better solution to sorting, find the best one
            commands.sort_by(|a, b| a.z_index().cmp(&b.z_index()));

            self.renderer.render(&commands)?;
        }
        Ok(())
    }

    pub fn resize(&mut self, size: Size<usize>) -> Result<()> {
        self.size = size;
        self.arena.mark_dirty(self.root, DirtyFlags::LAYOUT)?;
        self.renderer.resize(size)?;
        Ok(())
    }

    pub fn renderer(&mut self) -> &mut R {
        &mut self.renderer
    }

    pub fn tree(&self) -> TreeContext<'_> {
        TreeContext {
            arena: &self.arena,
            root: self.root,
        }
    }

    pub fn get_size(&self) -> Size<usize> {
        self.size
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::Cell, panic::AssertUnwindSafe, rc::Rc, time::Instant};

    use super::*;
    use crate::{
        core::{
            event::MouseEvents,
            layout::{
                LeafStyleBuilder,
                types::{
                    dimension::{percent, px},
                    point::Point,
                    size::Size,
                },
            },
            render::{Dpi, RenderCommand, Renderer},
            style::Color,
        },
        elements::{
            div::{ChildrenExt, div, style::DivStyleBuilder},
            text::style::TextStyle,
        },
    };

    struct TestRenderer {
        size: Size<usize>,
        dpi: Dpi,
        render_calls: usize,
    }

    impl TestRenderer {
        fn new(size: Size<usize>) -> Self {
            Self {
                size,
                dpi: Dpi::uniform(96.0),
                render_calls: 0,
            }
        }
    }

    impl Renderer for TestRenderer {
        fn render(&mut self, _commands: &[RenderCommand]) -> Result<()> {
            self.render_calls += 1;
            Ok(())
        }

        fn resize(&mut self, size: Size<usize>) -> Result<()> {
            self.size = size;
            Ok(())
        }

        fn measure_text(
            &mut self,
            _text_props: &TextStyle,
            _available_size: Size<AvailableSpace>,
        ) -> Result<Size<f32>> {
            Ok(Size::wh(0.0, 0.0))
        }

        fn set_dpi(&mut self, dpi: Dpi) -> Result<()> {
            self.dpi = dpi;
            Ok(())
        }
    }

    #[test]
    fn interaction_sequence_does_not_panic() {
        let root = div().size(percent(1.0)).bg(Color::GREEN);

        let mut engine = Engine::new(
            TestRenderer::new(Size::wh(400, 300)),
            Box::new(root),
            Size::wh(400, 300),
        )
        .expect("engine init");

        let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
            engine.dispatch_event(EngineEvent::WindowCreated);
            engine.dispatch_event(EngineEvent::MouseMove {
                position: Point::xy(10.0, 10.0),
            });
            engine.dispatch_event(EngineEvent::MouseDown {
                position: Point::xy(10.0, 10.0),
                button: MouseButton::Left,
            });
            engine.dispatch_event(EngineEvent::MouseUp {
                position: Point::xy(10.0, 10.0),
                button: MouseButton::Left,
            });
            engine.dispatch_event(EngineEvent::MouseMove {
                position: Point::xy(9999.0, 9999.0),
            });
        }));

        assert!(result.is_ok(), "interaction pipeline panicked");
    }

    #[test]
    fn hover_enter_leave_baseline_sequence() {
        let enter_count = Rc::new(Cell::new(0));
        let leave_count = Rc::new(Cell::new(0));

        let enter_count_cb = Rc::clone(&enter_count);
        let leave_count_cb = Rc::clone(&leave_count);

        let root = div()
            .size(percent(1.0))
            .bg(Color::GREEN)
            .on_mouse_enter(move |_| enter_count_cb.set(enter_count_cb.get() + 1))
            .on_mouse_leave(move |_| leave_count_cb.set(leave_count_cb.get() + 1));

        let mut engine = Engine::new(
            TestRenderer::new(Size::wh(400, 300)),
            Box::new(root),
            Size::wh(400, 300),
        )
        .expect("engine init");

        engine.dispatch_event(EngineEvent::WindowCreated);

        engine.dispatch_event(EngineEvent::MouseMove {
            position: Point::xy(10.0, 10.0),
        });
        engine.dispatch_event(EngineEvent::MouseMove {
            position: Point::xy(20.0, 20.0),
        });
        engine.dispatch_event(EngineEvent::MouseMove {
            position: Point::xy(9999.0, 9999.0),
        });

        assert_eq!(enter_count.get(), 1, "enter should fire once");
        assert_eq!(leave_count.get(), 1, "leave should fire once");
    }

    #[test]
    #[ignore = "manual baseline perf probe"]
    /// Got: 3.1043ms - 3.9207ms - 3.3785ms - 3.7557ms ~ 3.5ms
    fn baseline_mousemove_perf_probe() {
        let root = div()
            .size(percent(1.0))
            .bg(Color::GREEN)
            .child(div().size(px(100.0)));

        let mut engine = Engine::new(
            TestRenderer::new(Size::wh(400, 300)),
            Box::new(root),
            Size::wh(400, 300),
        )
        .expect("engine init");

        engine.dispatch_event(EngineEvent::WindowCreated);

        let start = Instant::now();
        for i in 0..5000usize {
            let x = (i % 400) as f32;
            let y = ((i * 7) % 300) as f32;
            engine.dispatch_event(EngineEvent::MouseMove {
                position: Point::xy(x, y),
            });
        }
        let elapsed = start.elapsed();

        eprintln!("[phase0][perf] 5000 mouse moves in {:?}", elapsed);
    }
}
