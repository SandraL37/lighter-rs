use std::collections::HashMap;

use crate::{
    core::{
        dirty::DirtyFlags,
        error::*,
        node::{Node, NodeId, NodeKey, NodeKind, NodeProps},
    },
    prelude::{LayoutKind, NodeLayout},
};

/// The NodeArena owns the nodes.
/// Handles changes to the nodes.
#[derive(Debug)]
pub struct NodeArena {
    nodes: slotmap::SlotMap<NodeId, Node>,
    node_keys: HashMap<NodeKey, NodeId>,
}

impl NodeArena {
    pub fn new() -> Self {
        Self {
            nodes: slotmap::SlotMap::with_key(),
            node_keys: HashMap::new(),
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

    pub fn get_node_id_by_key(&self, key: NodeKey) -> Result<NodeId> {
        self.node_keys
            .get(&key)
            .copied()
            .ok_or(Error::NodeNotFoundByKey(key))
    }

    pub fn get_node_by_key(&self, key: NodeKey) -> Result<&Node> {
        self.node_keys
            .get(&key)
            .and_then(|id| self.nodes.get(*id))
            .ok_or(Error::NodeNotFoundByKey(key))
    }

    pub fn get_node_mut_by_key(&mut self, key: NodeKey) -> Result<&mut Node> {
        self.node_keys
            .get(&key)
            .and_then(|id| self.nodes.get_mut(*id))
            .ok_or(Error::NodeNotFoundByKey(key))
    }

    pub fn create_node(
        &mut self,
        kind: NodeKind,
        props: NodeProps,
        parent: Option<NodeId>,
        layout_style: impl Into<LayoutKind>,
    ) -> Result<NodeId> {
        let key = props.key;

        let id = self.nodes.insert(Node {
            kind,
            props,
            children: Vec::new(),
            parent,
            dirty: DirtyFlags::ALL,
            layout: NodeLayout::new(layout_style.into()),
        });

        if let Some(key) = key {
            self.node_keys.insert(key, id);
        }

        if let Some(parent_id) = parent {
            self.get_node_mut(parent_id)?.children.push(id);
            self.mark_dirty(parent_id, DirtyFlags::LAYOUT);
        }

        Ok(id)
    }
}
