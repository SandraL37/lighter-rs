use crate::core::{
    dirty::DirtyFlags,
    error::*,
    layout::{LayoutStyle, NodeLayout},
    node::{Node, NodeId, NodeKind, NodeProps},
};

#[derive(Debug)]
pub struct Tree {
    arena: NodeArena,
}

impl Tree {
    pub fn build_context(&mut self) -> BuildCtx<'_> {
        BuildCtx::new(&mut self.arena)
    }
}

#[derive(Debug)]
pub struct NodeArena {
    pub nodes: slotmap::SlotMap<NodeId, Node>,
}

impl NodeArena {
    pub fn mark_dirty(&mut self, node_id: NodeId, flags: DirtyFlags) {
        let Some(node) = self.nodes.get_mut(node_id) else {
            return;
        };

        let new_flags = node.dirty | flags;

        if new_flags == node.dirty {
            return;
        }

        node.dirty = new_flags;

        if flags.contains(DirtyFlags::LAYOUT) {
            if let Some(parent) = node.parent {
                self.mark_dirty(parent, DirtyFlags::LAYOUT);
            }
        }
    }

    pub fn get_node(&self, node_id: NodeId) -> Result<&Node> {
        self.nodes.get(node_id).ok_or(Error::NodeNotFound(node_id))
    }

    pub fn get_node_mut(&mut self, node_id: NodeId) -> Result<&mut Node> {
        self.nodes
            .get_mut(node_id)
            .ok_or(Error::NodeNotFound(node_id))
    }
}

#[derive(Debug)]
pub struct BuildCtx<'a> {
    pub arena: &'a mut NodeArena,
}

impl<'a> BuildCtx<'a> {
    pub fn new(arena: &'a mut NodeArena) -> Self {
        Self { arena }
    }

    pub fn create_node(
        &mut self,
        kind: NodeKind,
        props: NodeProps,
        parent: Option<NodeId>,
        layout_style: impl Into<LayoutStyle>,
    ) -> Result<NodeId> {
        let id = self.arena.nodes.insert(Node {
            kind,
            props,
            children: Vec::new(),
            parent,
            dirty: DirtyFlags::ALL,
            layout: NodeLayout::new(layout_style.into()),
        });

        if let Some(parent_id) = parent {
            self.arena.get_node_mut(parent_id)?.children.push(id);

            self.arena.mark_dirty(parent_id, DirtyFlags::LAYOUT);
        }

        Ok(id)
    }
}
