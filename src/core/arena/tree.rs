use crate::core::arena::{NodeArena, node::NodeId};

// TODO: check if this needs to be deleted

#[derive(Debug)]
pub struct TreeContext<'a> {
    pub arena: &'a NodeArena,
    pub root: NodeId,
}

impl<'a> TreeContext<'a> {
    pub fn arena(&self) -> &NodeArena {
        &self.arena
    }

    pub fn root(&self) -> NodeId {
        self.root
    }
}
