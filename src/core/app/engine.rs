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
        event::{EngineEvent, MouseButton, hit_test::hit_test},
        layout::{
            AvailableSpace, LayoutContext,
            types::{point::Point, rect::Rect, size::Size},
        },
        reactive::{dirty::DirtyFlags, runtime::Runtime},
        render::{RenderCommand, Renderer},
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

    pub fn dispatch_event(&mut self, event: EngineEvent) {
        let mut force_frame = false;

        #[cfg(debug_assertions)]
        trace_engine_event(&event);

        let mut result = || -> Result<()> {
            match event {
                EngineEvent::WindowCreated => {
                    force_frame = true;
                }
                EngineEvent::WindowResized { size: new_size } => {
                    let old_size = self.size;

                    if !(old_size == new_size) {
                        self.resize(new_size)?;
                        self.frame()?;
                    }
                }
                EngineEvent::MouseMove { position } => {
                    let new_hover_path = hit_test(&self.arena, self.root, position);
                    let old_hover_leaf = self.hover_path.last().copied();
                    let new_hover_leaf = new_hover_path.last().copied();

                    if new_hover_path != self.hover_path {
                        let old_hover_path = self.hover_path.clone();

                        let leaving: Vec<NodeId> = old_hover_path
                            .iter()
                            .copied()
                            .filter(|id| !new_hover_path.contains(id))
                            .collect();

                        let entering: Vec<NodeId> = new_hover_path
                            .iter()
                            .copied()
                            .filter(|id| !old_hover_path.contains(id))
                            .collect();

                        for node_id in leaving {
                            self.set_state(node_id, InteractionState::HOVER, false)?;
                            self.set_state(node_id, InteractionState::ACTIVE, false)?;
                        }

                        for node_id in entering {
                            self.set_state(node_id, InteractionState::HOVER, true)?;
                        }

                        if old_hover_leaf != new_hover_leaf {
                            #[cfg(debug_assertions)]
                            trace_hover_change(
                                old_hover_leaf,
                                new_hover_leaf,
                                old_hover_path.len(),
                                new_hover_path.len(),
                            );

                            if let Some(old) = old_hover_leaf
                                && let Ok(data) = self.arena.get_data(old)
                            {
                                if let Some(cb) = &data.event_handlers.on_mouse_leave {
                                    cb();
                                }

                                // if let Some(hover_style) = &data.
                            }

                            if let Some(new) = new_hover_leaf
                                && let Ok(data) = self.arena.get_data(new)
                                && let Some(cb) = &data.event_handlers.on_mouse_enter
                            {
                                cb();
                            }
                        }

                        self.hover_path = new_hover_path;
                    }
                }
                EngineEvent::MouseDown {
                    position,
                    button: MouseButton::Left,
                } => {
                    let hit_path = hit_test(&self.arena, self.root, position);

                    for &node_id in hit_path.iter().rev() {
                        self.set_state(node_id, InteractionState::ACTIVE, true)?;

                        if let Ok(data) = self.arena.get_data(node_id) {
                            if let Some(cb) = &data.event_handlers.on_click {
                                cb();
                            }
                        }
                    }
                }
                EngineEvent::MouseUp {
                    position,
                    button: MouseButton::Left,
                } => {
                    let hit_path = hit_test(&self.arena, self.root, position);

                    for &node_id in hit_path.iter().rev() {
                        self.set_state(node_id, InteractionState::ACTIVE, false)?;
                    }
                }
                EngineEvent::WindowDestroyed => unsafe {
                    PostThreadMessageW(
                        GetCurrentThreadId(),
                        custom_messages::WINDOWCLOSED,
                        WPARAM(0),
                        LPARAM(0),
                    )?;
                },
                EngineEvent::DpiChanged(hwnd, rect, dpi) => {
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

                    force_frame = true;
                }
                _ => {}
            }

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

        if on {
            data.interaction_state.insert(state);
        } else {
            data.interaction_state.remove(state);
        }

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
                            opacity: node.props.opacity,
                            transform: node.props.transform.unwrap_or(Transform::IDENTITY),
                            z_index: node.props.z_index,
                        },
                        NodeKind::Text(props) => RenderCommand::Text {
                            bounds: unrounded_bounds,
                            props: Arc::clone(props), // TODO: is there a better way
                            opacity: node.props.opacity,
                            transform: node.props.transform.unwrap_or(Transform::IDENTITY),
                            z_index: node.props.z_index,
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
        elements::div::{ChildrenExt, div, style::DivStyleBuilder},
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
            _text_props: &crate::elements::text::TextStyle,
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
            .on_mouse_enter(move || enter_count_cb.set(enter_count_cb.get() + 1))
            .on_mouse_leave(move || leave_count_cb.set(leave_count_cb.get() + 1));

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
