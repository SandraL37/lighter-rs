use crate::{
    core::{
        arena::NodeArena,
        error::*,
        layout::{AvailableSpace, Point, Rect, Size, compute_layout},
        node::{Node, NodeId, NodeKind},
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

    pub fn traverse_recursive<T: Copy>(
        &self,
        node_id: NodeId,
        callback: &mut impl FnMut(&Node, T) -> T,
        data: T,
    ) {
        let node = self.get_node(node_id).unwrap();
        let result = callback(node, data);
        for child_id in node.children.iter() {
            self.traverse_recursive(*child_id, callback, result);
        }
    }

    pub fn build_render_list(&self) -> Vec<RenderCommand> {
        let mut commands = Vec::new();
        let cursor = Point::new(0.0, 0.0);

        self.traverse_recursive(
            self.root,
            &mut |node, cursor: Point<f32>| {
                let layout = node.layout.computed;

                let cursor = Point::new(cursor.x + layout.location.x, cursor.y + layout.location.y);

                let bounds = Rect::xywh(cursor.x, cursor.y, layout.size.width, layout.size.height);

                match &node.kind {
                    NodeKind::Div(props) => {
                        commands.push(RenderCommand::Rect {
                            bounds,
                            color: props.background_color,
                            opacity: node.props.opacity,
                            transform: node.props.transform,
                            z_index: node.props.z_index,
                        });
                    }
                    NodeKind::Text(props) => commands.push(RenderCommand::Text {
                        bounds,
                        content: props.content.clone(),
                        color: props.color,
                        font_size: props.font_size,
                        opacity: node.props.opacity,
                        transform: node.props.transform,
                        z_index: node.props.z_index,
                    }),
                }

                cursor
            },
            cursor,
        );

        commands
    }

    pub fn compute_layout(&mut self, available_space: Size<AvailableSpace>) {
        compute_layout(&mut self.arena, self.root, available_space.into());
    }
}
