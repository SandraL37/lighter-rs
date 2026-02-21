use crate::{
    core::{
        arena::NodeArena,
        cx::Cx,
        error::*,
        layout::{AvailableSpace, Size, compute_layout},
        node::NodeId,
    },
    elements::Element,
};

#[derive(Debug)]
pub struct Tree {
    arena: NodeArena,
    root: NodeId,
}

impl Tree {
    pub fn build(root: impl Element + 'static) -> Result<(Self, Cx)> {
        let mut arena = NodeArena::new();
        let mut cx = Cx::new();
        let root = Box::new(root).build(&mut arena, &mut cx, None)?;
        Ok((Self { arena, root }, cx))
    }

    pub fn compute_layout(&mut self, available_space: Size<AvailableSpace>) {
        compute_layout(&mut self.arena, self.root, available_space);
    }

    pub(crate) fn arena(&self) -> &NodeArena {
        &self.arena
    }

    pub(crate) fn arena_mut(&mut self) -> &mut NodeArena {
        &mut self.arena
    }

    pub fn print(&self) {
        taffy::print_tree(&self.arena, self.root.into());
    }

    pub fn root(&self) -> NodeId {
        self.root
    }
}
