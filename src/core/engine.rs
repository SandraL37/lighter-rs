use crate::{
    core::{
        arena::NodeArena,
        cx::{Cx, UpdateQueue},
        dirty::DirtyFlags,
        error::*,
        layout::{AvailableSpace, LayoutContext, Point, Rect, Size},
        node::{NodeId, NodeKind},
        render::{RenderCommand, Renderer},
        style::Transform,
        tree::TreeContext,
    },
    elements::Element,
};

pub struct Engine<R: Renderer> {
    arena: NodeArena,
    root: NodeId,
    renderer: R,
    size: Size<usize>,
    updates: UpdateQueue,
}

impl<R: Renderer> Engine<R> {
    pub fn new(renderer: R, root: Box<dyn Element>, size: Size<usize>) -> Result<Self> {
        let mut arena = NodeArena::new();
        let mut cx = Cx::new();
        let root = root.build(&mut arena, &mut cx, None)?;
        Ok(Engine {
            arena,
            root,
            renderer,
            size,
            updates: cx.updates,
        })
    }

    fn build_render_list(
        &mut self,
        layout_dirty: bool,
        paint_dirty: bool,
    ) -> Result<Vec<RenderCommand>> {
        let cursor = Point::new(0.0, 0.0);
        let mut commands = Vec::new();
        let mut redrawn_nodes = Vec::new();

        self.arena.traverse(
            self.root,
            &mut |node_id, node, layout, cursor: Point<f32>| -> Point<f32> {
                let cursor = Point::new(
                    cursor.x + layout.computed.location.x,
                    cursor.y + layout.computed.location.y,
                );

                // let bounds = Rect::xywh(
                //     cursor.x,
                //     cursor.y,
                //     layout.computed.size.width,
                //     layout.computed.size.height,
                // );

                let unrounded_bounds = Rect::xywh(
                    // TODO: avoid calculating both rounded and unrounded
                    cursor.x,
                    cursor.y,
                    layout.unrounded.size.width,
                    layout.unrounded.size.height,
                );

                // TODO: BUG! partial rerenders are broken because of alpha
                // TODO: Check if the only fix is adding damage tracking
                // if layout_dirty || node.dirty.contains(DirtyFlags::PAINT) {
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
                            props: props.clone(), // TODO: is there a better way
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
        let pending = self.updates.borrow_mut().drain(..).collect::<Vec<_>>();

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
            commands.sort_by(|a, b| a.z_index().cmp(&b.z_index()));

            self.renderer.render(&commands)?;
        }

        Ok(())
    }

    pub fn resize(&mut self, size: Size<usize>) -> Result<()> {
        self.size = size;
        self.arena.mark_dirty(self.root, DirtyFlags::LAYOUT)?;

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
