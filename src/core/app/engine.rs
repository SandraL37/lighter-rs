use std::{collections::HashMap, sync::Arc, time::Instant};

use windows::Win32::{ Foundation::*, System::Threading::*, UI::WindowsAndMessaging::* };

use crate::{
    core::{
        app::custom_messages,
        arena::{ NodeArena, node::{ InteractionState, NodeId, NodeKind }, tree::TreeContext },
        error::*,
        event::{ EngineEvent, MouseButton, hit_test::hit_test },
        layout::{ AvailableSpace, LayoutContext, types::{ point::Point, rect::Rect, size::Size } },
        reactive::{ dirty::DirtyFlags, runtime::Runtime },
        render::{ RenderCommand, Renderer },
        style::Transform,
    },
    elements::{
        div::{DivAnimationState, DivResolvedVisual},
        Element,
    },
};

pub struct Engine<R: Renderer> {
    arena: NodeArena,
    root: NodeId,
    renderer: R,
    size: Size<usize>,
    hovered_path: Vec<NodeId>,
    div_animations: HashMap<NodeId, DivAnimationState>,
    div_last_visual: HashMap<NodeId, DivResolvedVisual>,
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
            hovered_path: Vec::new(),
            div_animations: HashMap::new(),
            div_last_visual: HashMap::new(),
        })
    }

    pub fn dispatch_event(&mut self, event: EngineEvent) {
        let mut force_frame = false;

        let mut result = || -> Result<()> {
            match event {
                EngineEvent::WindowCreated => {
                    force_frame = true;
                }
                EngineEvent::Tick => {
                    if !self.div_animations.is_empty() {
                        for (id, _) in self.div_animations.iter() {
                            let _ = self.arena.mark_dirty(*id, DirtyFlags::PAINT);
                        }
                        force_frame = true;
                    }
                }
                EngineEvent::WindowResized { size: new_size } => {
                    let old_size = self.size;

                    if !(old_size == new_size) && new_size.width > 0 && new_size.height > 0 {
                        self.resize(new_size)?;
                        force_frame = true;
                    }
                }
                EngineEvent::MouseMove { position } => {
                    let new_hover_path = hit_test(&self.arena, self.root, position);
                    let old_hover_leaf = self.hovered_path.last().copied();
                    let new_hover_leaf = new_hover_path.last().copied();

                    if new_hover_path != self.hovered_path {
                        let old_hover_path = self.hovered_path.clone();

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

                        for id in leaving {
                            self.set_state(id, InteractionState::HOVER, false)?;
                        }

                        for id in entering {
                            self.set_state(id, InteractionState::HOVER, true)?;
                        }

                        if old_hover_leaf != new_hover_leaf {
                            if let Some(old) = old_hover_leaf {
                                if let Ok(data) = self.arena.get_data(old) {
                                    if let Some(cb) = &data.event_handlers.on_mouse_leave {
                                        cb();
                                    }
                                }
                            }
                            if let Some(new) = new_hover_leaf {
                                if let Ok(data) = self.arena.get_data(new) {
                                    if let Some(cb) = &data.event_handlers.on_mouse_enter {
                                        cb();
                                    }
                                }
                            }
                        }

                        self.hovered_path = new_hover_path;
                    }
                }
                EngineEvent::MouseDown { position, button: MouseButton::Left } => {
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
                EngineEvent::MouseUp { button: MouseButton::Left, .. } => {
                    let mut active_nodes = Vec::new();

                    self.arena.traverse(
                        self.root,
                        &mut (|node_id, data, _layout, ()| {
                            if data.interaction.contains(InteractionState::ACTIVE) {
                                active_nodes.push(node_id);
                            }
                        }),
                        ()
                    );

                    for node_id in active_nodes {
                        self.set_state(node_id, InteractionState::ACTIVE, false)?;
                    }
                }
                EngineEvent::WindowDestroyed => unsafe {
                    PostThreadMessageW(
                        GetCurrentThreadId(),
                        custom_messages::WINDOWCLOSED,
                        WPARAM(0),
                        LPARAM(0)
                    )?;
                }
                EngineEvent::DpiChanged(_rect, dpi) => {
                    self.renderer.set_dpi(dpi)?;
                    // self.arena.mark_dirty(self.root, DirtyFlags::LAYOUT)?;
                    force_frame = true;
                }
                _ => {}
            }

            if
                Runtime::has_updates() ||
                force_frame ||
                self.arena.is_any_dirty(DirtyFlags::PAINT | DirtyFlags::LAYOUT)
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

    fn build_render_list(
        &mut self,
        layout_dirty: bool,
        paint_dirty: bool
    ) -> Result<Vec<RenderCommand>> {
        #[derive(Clone, Copy)]
        struct RenderAccumulator {
            cursor: Point<f32>,
            opacity: f32,
            transform: Transform,
        }

        let root_acc = RenderAccumulator {
            cursor: Point::xy(0.0, 0.0),
            opacity: 1.0,
            transform: Transform::IDENTITY,
        };

        let mut commands = Vec::new();
        let mut redrawn_nodes = Vec::new();
        let mut div_animations = std::mem::take(&mut self.div_animations);
        let mut div_last_visual = std::mem::take(&mut self.div_last_visual);

        self.arena.traverse(
            self.root,
            &mut (|node_id, node, layout, acc: RenderAccumulator| -> RenderAccumulator {
                let cursor = Point::xy(
                    acc.cursor.x + layout.computed.location.x,
                    acc.cursor.y + layout.computed.location.y
                );
                let mut local_opacity = node.props.opacity;
                let mut local_transform = node.props.transform.unwrap_or(Transform::IDENTITY);
                let mut rect_color = None;
                let mut rect_corner_radius = 0.0;

                if let NodeKind::Div(props) = &node.kind {
                    let target =
                        props.resolve_visual(node.interaction, node.props.opacity, node.props.transform);
                    let now = Instant::now();

                    if let Some(transition) = props.transition {
                        let existing = div_animations.get(&node_id).cloned();
                        match existing {
                            Some(anim) if !anim.finished(now) => {
                                if visual_changed(anim.to, target) {
                                    div_animations.insert(node_id, DivAnimationState {
                                        from: anim.at(now),
                                        to: target,
                                        started: now,
                                        transition,
                                    });
                                }
                            }
                            Some(anim) => {
                                div_animations.remove(&node_id);
                                if visual_changed(anim.to, target) {
                                    div_animations.insert(node_id, DivAnimationState {
                                        from: anim.to,
                                        to: target,
                                        started: now,
                                        transition,
                                    });
                                }
                            }
                            None => {
                                let start_from = div_last_visual.get(&node_id).copied().unwrap_or(target);
                                if visual_changed(start_from, target) {
                                    div_animations.insert(node_id, DivAnimationState {
                                        from: start_from,
                                        to: target,
                                        started: now,
                                        transition,
                                    });
                                }
                            }
                        }
                    } else {
                        div_animations.remove(&node_id);
                    }

                    let resolved = div_animations
                        .get(&node_id)
                        .map(|a| a.at(now))
                        .unwrap_or(target);
                    if div_animations.get(&node_id).is_some_and(|a| a.finished(now)) {
                        div_animations.remove(&node_id);
                    }
                    div_last_visual.insert(node_id, resolved);

                    local_opacity = resolved.opacity;
                    local_transform = resolved.transform;
                    rect_color = Some(resolved.background_color);
                    rect_corner_radius = resolved.corner_radius;
                }

                let origin_x = cursor.x + layout.unrounded.size.width * 0.5;
                let origin_y = cursor.y + layout.unrounded.size.height * 0.5;
                let local_transform =
                    Transform::translate(-origin_x, -origin_y) *
                    local_transform *
                    Transform::translate(origin_x, origin_y);
                let world_transform = acc.transform * local_transform;
                let world_opacity = acc.opacity * local_opacity;

                let unrounded_bounds = Rect::xywh(
                    cursor.x,
                    cursor.y,
                    layout.unrounded.size.width,
                    layout.unrounded.size.height
                );

                if layout_dirty || paint_dirty {
                    commands.push(match &node.kind {
                        NodeKind::Div(props) =>
                            RenderCommand::Rect {
                                bounds: unrounded_bounds,
                                color: rect_color.unwrap_or(props.background_color),
                                corner_radius: rect_corner_radius,
                                opacity: world_opacity,
                                transform: world_transform,
                                z_index: node.props.z_index,
                            },
                        NodeKind::Text(props) =>
                            RenderCommand::Text {
                                bounds: unrounded_bounds,
                                props: Arc::clone(props), // TODO: is there a better way
                                opacity: world_opacity,
                                transform: world_transform,
                                z_index: node.props.z_index,
                            },
                    });
                    redrawn_nodes.push(node_id);
                }

                RenderAccumulator {
                    cursor,
                    opacity: world_opacity,
                    transform: world_transform,
                }
            }),
            root_acc
        );

        self.div_animations = div_animations;
        self.div_last_visual = div_last_visual;

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

            layout_context.compute_layout(
                Size::wh(
                    AvailableSpace::Definite(self.size.width as f32),
                    AvailableSpace::Definite(self.size.height as f32)
                )
            );
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

    fn set_state(&mut self, id: NodeId, flag: InteractionState, on: bool) -> Result<()> {
        let data = self.arena.get_data_mut(id)?;
        let before = data.interaction;
        if on {
            data.interaction.insert(flag);
        } else {
            data.interaction.remove(flag);
        }
        if data.interaction != before {
            self.arena.mark_dirty(id, DirtyFlags::PAINT)?; // TODO: check if i need to dirtyflag also layout
        }
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

    pub fn has_active_animations(&self) -> bool {
        !self.div_animations.is_empty()
    }
}

fn visual_changed(a: DivResolvedVisual, b: DivResolvedVisual) -> bool {
    a.background_color != b.background_color ||
        (a.corner_radius - b.corner_radius).abs() > f32::EPSILON ||
        (a.opacity - b.opacity).abs() > f32::EPSILON ||
        a.transform != b.transform
}
