use crate::core::{
    arena::NodeArena,
    node::{Node, NodeId},
};

bitflags::bitflags! {
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub struct DirtyFlags: u8 {
        const LAYOUT = 1 << 0;
        const PAINT = 1 << 1;
        const ALL = Self::LAYOUT.bits() | Self::PAINT.bits();
    }
}

impl Node {
    pub(crate) fn mark_dirty(&mut self, flags: DirtyFlags) {
        self.dirty.insert(flags);

        match flags {
            DirtyFlags::LAYOUT => {
                self.layout.cache.clear();
            }
            DirtyFlags::PAINT => {}
            _ => {}
        }
    }

    pub(crate) fn mark_clean(&mut self, flags: DirtyFlags) {
        self.dirty.remove(flags);
    }

    pub(crate) fn has_dirty_flags(&self, flags: DirtyFlags) -> bool {
        self.dirty.contains(flags)
    }
}

impl NodeArena {
    pub(crate) fn mark_dirty(&mut self, node_id: NodeId, flags: DirtyFlags) {
        let Ok(node) = self.get_node_mut(node_id) else {
            return;
        };

        if node.has_dirty_flags(flags) {
            return;
        }

        node.mark_dirty(flags);

        if flags.contains(DirtyFlags::LAYOUT) {
            if let Some(parent_id) = node.parent {
                self.mark_dirty(parent_id, DirtyFlags::LAYOUT);
            }
        }
    }
}
