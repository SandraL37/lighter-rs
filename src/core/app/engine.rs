use std::sync::Arc;

use crate::{
    core::{
        arena::{
            NodeArena,
            node::{NodeId, NodeKind},
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
    hovered: Option<NodeId>,
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
            hovered: None,
        })
    }

    pub fn dispatch_event(&mut self, event: EngineEvent) -> Result<()> {
        match event {
            EngineEvent::WindowCreated => self.frame()?,
            EngineEvent::WindowResized { size: new_size } => {
                let old_size = self.size;

                if !(old_size == new_size) {
                    self.resize(new_size)?;
                    self.frame()?;
                }
            }
            EngineEvent::MouseMove { position } => {
                let hit_path = hit_test(&self.arena, self.root, position);
                let new_hover = hit_path.last().copied();

                if new_hover != self.hovered {
                    if let Some(old) = self.hovered {
                        if let Ok(data) = self.arena.get_data(old) {
                            if let Some(cb) = &data.event_handlers.on_mouse_leave {
                                cb();
                            }
                        }
                    }
                    if let Some(new) = new_hover {
                        if let Ok(data) = self.arena.get_data(new) {
                            if let Some(cb) = &data.event_handlers.on_mouse_enter {
                                cb();
                            }
                        }
                    }
                    self.hovered = new_hover;
                }
            }
            EngineEvent::MouseDown {
                position,
                button: MouseButton::Left,
            } => {
                let hit_path = hit_test(&self.arena, self.root, position);
                for &node_id in hit_path.iter().rev() {
                    if let Ok(data) = self.arena.get_data(node_id) {
                        if let Some(cb) = &data.event_handlers.on_click {
                            cb();
                        }
                    }
                }
            }
            _ => {}
        }

        if Runtime::has_updates() {
            self.frame()?;
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
            &mut |node_id, node, layout, cursor: Point<f32>| -> Point<f32> {
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
            },
            cursor,
        );

        for node_id in redrawn_nodes {
            self.arena.mark_clean(node_id, DirtyFlags::PAINT)?;
        }

        Ok(commands)
    }

    pub fn frame(&mut self) -> Result<()> {
        println!("\rrendered frame");

        let pending = Runtime::drain_updates();

        for update in pending {
            if let Ok((data, layout)) = self.arena.get_data_layout_mut(update.node_id) {
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
