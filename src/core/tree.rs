use crate::{
    core::{arena::NodeArena, cx::Cx, error::*, node::NodeId},
    elements::Element,
};

// TODO: is it useful to have a tree struct like this? Should it own the node arena or should it be ephemeral like TreeContext<'a>
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

    pub(crate) fn arena(&self) -> &NodeArena {
        &self.arena
    }

    pub(crate) fn arena_mut(&mut self) -> &mut NodeArena {
        &mut self.arena
    }

    pub fn root(&self) -> NodeId {
        self.root
    }
}
