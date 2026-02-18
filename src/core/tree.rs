use crate::{
    core::{
        arena::NodeArena,
        dirty::DirtyFlags,
        error::*,
        layout::{AvailableSpace, Point, Rect, Size, compute_layout},
        node::{Node, NodeId, NodeKey, NodeKind},
        render::RenderCommand,
    },
    elements::Element,
};

#[derive(Debug)]
pub struct Tree {
    arena: NodeArena,
    root: NodeId,
}

impl Tree {
    pub fn build(root: impl Element) -> Result<Self> {
        let mut arena = NodeArena::new();
        let root = root.build(&mut arena, None)?;
        Ok(Self { arena, root })
    }

    pub fn print(&self) {
        taffy::print_tree(&self.arena, self.root.into());
    }

    pub fn get_node(&self, node_id: NodeId) -> Result<&Node> {
        self.arena.get_node(node_id)
    }

    pub fn get_node_mut(&mut self, node_id: NodeId) -> Result<&mut Node> {
        self.arena.get_node_mut(node_id)
    }

    pub fn get_node_by_key(&self, key: NodeKey) -> Result<&Node> {
        self.arena.get_node_by_key(key)
    }

    pub fn get_node_mut_by_key(&mut self, key: NodeKey) -> Result<&mut Node> {
        self.arena.get_node_mut_by_key(key)
    }

    pub fn traverse_recursive<T: Copy>(
        &mut self,
        node_id: NodeId,
        callback: &mut impl FnMut(&mut Node, T) -> T,
        data: T,
    ) {
        let node = self.get_node_mut(node_id).unwrap();
        let result = callback(node, data);

        // TODO: too long
        let children: Vec<NodeId> = node.children.iter().copied().collect();
        for child_id in children {
            self.traverse_recursive(child_id, callback, result);
        }
    }

    pub fn build_render_list(&mut self) -> Vec<RenderCommand> {
        let mut commands = Vec::new();
        let cursor = Point::new(0.0, 0.0);

        self.traverse_recursive(
            self.root,
            &mut |node, cursor: Point<f32>| {
                let layout = node.layout.computed;

                let cursor = Point::new(cursor.x + layout.location.x, cursor.y + layout.location.y);

                let bounds = Rect::xywh(cursor.x, cursor.y, layout.size.width, layout.size.height);

                commands.push(match &node.kind {
                    NodeKind::Div(props) => RenderCommand::Rect {
                        bounds,
                        color: props.background_color,
                        opacity: node.props.opacity,
                        transform: node.props.transform,
                        z_index: node.props.z_index,
                    },
                    NodeKind::Text(props) => RenderCommand::Text {
                        bounds,
                        content: props.content.clone(),
                        color: props.color,
                        font_size: props.font_size,
                        opacity: node.props.opacity,
                        transform: node.props.transform,
                        z_index: node.props.z_index,
                    },
                });

                cursor
            },
            cursor,
        );

        // TODO: Sorting can be slow, consider avoiding it if possible
        commands.sort_by(|a, b| a.z_index().cmp(&b.z_index()));

        commands
    }

    pub fn compute_layout(&mut self, available_space: Size<AvailableSpace>) {
        compute_layout(&mut self.arena, self.root, available_space.into());
    }

    pub fn mutate(
        &mut self,
        node_id: NodeId,
        flags: DirtyFlags,
        f: impl FnOnce(&mut Node),
    ) -> Result<()> {
        let node = self.arena.get_node_mut(node_id)?;
        f(node);
        self.arena.mark_dirty(node_id, flags);
        Ok(())
    }

    pub fn mutate_by_key(
        &mut self,
        key: NodeKey,
        flags: DirtyFlags,
        f: impl FnOnce(&mut Node),
    ) -> Result<()> {
        let node_id = self.arena.get_node_id_by_key(key)?;
        self.mutate(node_id, flags, f)
    }

    pub fn root(&self) -> NodeId {
        self.root
    }
}
