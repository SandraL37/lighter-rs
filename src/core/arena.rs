use crate::{
    core::{
        dirty::DirtyFlags,
        error::*,
        node::{Node, NodeId, NodeKind, NodeProps},
    },
    prelude::{LayoutKind, NodeLayout},
};

/// The NodeArena owns the nodes.
/// Handles changes to the nodes.
#[derive(Debug)]
pub struct NodeArena {
    nodes: slotmap::SlotMap<NodeId, Node>,
}

impl NodeArena {
    pub fn new() -> Self {
        Self {
            nodes: slotmap::SlotMap::with_key(),
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

    pub fn create_node(
        &mut self,
        kind: NodeKind,
        props: NodeProps,
        parent: Option<NodeId>,
        layout_style: impl Into<LayoutKind>,
    ) -> Result<NodeId> {
        let id = self.nodes.insert(Node {
            kind,
            props,
            children: Vec::new(),
            parent,
            dirty: DirtyFlags::ALL,
            layout: NodeLayout::new(layout_style.into()),
        });

        if let Some(parent) = parent {
            self.get_node_mut(parent)?.children.push(id);
        }

        Ok(id)
    }
}
