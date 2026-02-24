use crate::core::{
    cx::{Cx, UpdateQueue},
    dirty::DirtyFlags,
    error::*,
    layout::{AvailableSpace, LayoutContext, Point, Rect, Size, compute_layout},
    node::NodeKind,
    render::{RenderCommand, Renderer},
    style::Transform,
    tree::Tree,
};

pub struct Engine<R: Renderer> {
    tree: Tree,
    renderer: R,
    size: Size<usize>,
    updates: UpdateQueue,
}

impl<R: Renderer> Engine<R> {
    pub fn new(tree: Tree, renderer: R, cx: Cx) -> Self {
        let size = renderer.get_size();
        Engine {
            tree,
            renderer,
            size,
            updates: cx.updates,
        }
    }

    fn build_render_list(
        &mut self,
        layout_dirty: bool,
        paint_dirty: bool,
    ) -> Result<Vec<RenderCommand>> {
        let cursor = Point::new(0.0, 0.0);
        let mut commands = Vec::new();
        let mut redrawn_nodes = Vec::new();

        self.tree.arena().traverse(
            self.tree.root(),
            &mut |node_id, node, layout, cursor: Point<f32>| -> Point<f32> {
                let cursor = Point::new(
                    cursor.x + layout.computed.location.x,
                    cursor.y + layout.computed.location.y,
                );

                let bounds = Rect::xywh(
                    cursor.x,
                    cursor.y,
                    layout.computed.size.width,
                    layout.computed.size.height,
                );

                // TODO: BUG! partial rerenders are broken because of alpha
                // TODO: Check if the only fix is adding damage tracking
                // if layout_dirty || node.dirty.contains(DirtyFlags::PAINT) {
                if layout_dirty || paint_dirty {
                    commands.push(match &node.kind {
                        NodeKind::Div(props) => RenderCommand::Rect {
                            bounds,
                            color: props.background_color,
                            opacity: node.props.opacity,
                            transform: node.props.transform.unwrap_or(Transform::IDENTITY),
                            z_index: node.props.z_index,
                        },
                        NodeKind::Text(props) => RenderCommand::Text {
                            bounds,
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
            self.tree
                .arena_mut()
                .mark_clean(node_id, DirtyFlags::PAINT)?;
        }

        Ok(commands)
    }

    pub fn frame(&mut self) -> Result<()> {
        let pending = self.updates.borrow_mut().drain(..).collect::<Vec<_>>();

        for update in pending {
            if let Ok((data, layout)) = self.tree.arena_mut().get_data_layout_mut(update.node_id) {
                (update.apply)(data, layout);
            }

            let _ = self
                .tree
                .arena_mut()
                .mark_dirty(update.node_id, update.flags);
        }

        let is_layout_dirty = self.tree.arena().is_any_dirty(DirtyFlags::LAYOUT);
        let is_paint_dirty = self.tree.arena().is_any_dirty(DirtyFlags::PAINT);

        if is_layout_dirty {
            let root = self.tree.root(); // TODO: Consider how to manage tree and arena, should engine own arena or tree?

            let mut layout_context = LayoutContext {
                arena: self.tree.arena_mut(),
                renderer: &mut self.renderer,
            };

            compute_layout(
                &mut layout_context,
                root,
                Size::wh(
                    AvailableSpace::Definite(self.size.width as f32),
                    AvailableSpace::Definite(self.size.height as f32),
                ),
            );
        }

        if is_layout_dirty || is_paint_dirty {
            let mut commands = self.build_render_list(is_layout_dirty, is_paint_dirty)?;

            commands.sort_by(|a, b| a.z_index().cmp(&b.z_index()));

            self.renderer.render(&commands)?;
        }

        Ok(())
    }

    pub fn renderer(&mut self) -> &mut R {
        &mut self.renderer
    }

    pub fn tree(&self) -> &Tree {
        &self.tree
    }
}
