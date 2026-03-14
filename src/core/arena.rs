pub mod node;
pub mod tree;

use crate::core::{
    arena::node::{EventHandlers, NodeData, NodeId, NodeKind, NodeProps},
    error::*,
    layout::{LayoutKind, NodeLayout},
    reactive::dirty::{DirtyCounter, DirtyFlags},
};

#[derive(Debug)]
pub struct NodeArena {
    data: slotmap::SlotMap<NodeId, NodeData>,
    layout: slotmap::SecondaryMap<NodeId, NodeLayout>,
    children: slotmap::SecondaryMap<NodeId, Vec<NodeId>>,
    parent: slotmap::SecondaryMap<NodeId, Option<NodeId>>,
    dirty_counter: DirtyCounter,
}

impl NodeArena {
    pub fn new() -> Self {
        Self {
            data: slotmap::SlotMap::with_key(),
            layout: slotmap::SecondaryMap::new(),
            children: slotmap::SecondaryMap::new(),
            parent: slotmap::SecondaryMap::new(),
            dirty_counter: DirtyCounter::new(),
        }
    }

    pub fn get_data(&self, node_id: NodeId) -> Result<&NodeData> {
        self.data.get(node_id).ok_or(Error::NodeNotFound(node_id))
    }

    pub fn get_data_mut(&mut self, node_id: NodeId) -> Result<&mut NodeData> {
        self.data
            .get_mut(node_id)
            .ok_or(Error::NodeNotFound(node_id))
    }

    pub fn get_parent(&self, node_id: NodeId) -> Result<Option<NodeId>> {
        self.parent
            .get(node_id)
            .ok_or(Error::NodeNotFound(node_id))
            .copied()
    }

    pub fn get_children(&self, node_id: NodeId) -> Result<&Vec<NodeId>> {
        self.children
            .get(node_id)
            .ok_or(Error::NodeNotFound(node_id))
    }

    pub fn get_layout(&self, node_id: NodeId) -> Result<&NodeLayout> {
        self.layout.get(node_id).ok_or(Error::NodeNotFound(node_id))
    }

    pub fn get_layout_mut(&mut self, node_id: NodeId) -> Result<&mut NodeLayout> {
        self.layout
            .get_mut(node_id)
            .ok_or(Error::NodeNotFound(node_id))
    }

    pub fn get_data_layout_mut(
        &mut self,
        node_id: NodeId,
    ) -> Result<(&mut NodeData, &mut NodeLayout)> {
        let data = self
            .data
            .get_mut(node_id)
            .ok_or(Error::NodeNotFound(node_id))?;
        let layout = self
            .layout
            .get_mut(node_id)
            .ok_or(Error::NodeNotFound(node_id))?;
        Ok((data, layout))
    }

    pub fn create_node(
        &mut self,
        kind: NodeKind,
        props: NodeProps,
        parent: Option<NodeId>,
        layout_style: impl Into<LayoutKind>,
        event_handlers: EventHandlers,
    ) -> Result<NodeId> {
        let id = self.data.insert(NodeData {
            kind,
            props,
            dirty: DirtyFlags::all(),
            event_handlers,
        });

        self.dirty_counter.increment(DirtyFlags::all());

        self.parent.insert(id, parent);
        self.children.insert(id, Vec::new());

        if let Some(parent_id) = parent {
            self.children
                .get_mut(parent_id)
                .ok_or(Error::NodeNotFound(parent_id))?
                .push(id);
        }

        self.layout.insert(id, NodeLayout::new(layout_style.into()));

        Ok(id)
    }

    pub fn delete_node(&mut self, node_id: NodeId) -> Result<()> {
        if let Some(&Some(parent_id)) = self.parent.get(node_id) {
            if let Some(siblings) = self.children.get_mut(parent_id) {
                siblings.retain(|&id| id != node_id);
            }
        }

        if let Some(node_data) = self.data.get(node_id) {
            if !node_data.dirty.is_empty() {
                self.dirty_counter.decrement(node_data.dirty);
            }
        }

        self.data
            .remove(node_id)
            .ok_or(Error::NodeNotFound(node_id))?;
        self.layout.remove(node_id);
        self.children.remove(node_id);
        self.parent.remove(node_id);

        Ok(())
    }

    pub fn delete_subtree(&mut self, node_id: NodeId) -> Result<()> {
        let children: Vec<NodeId> = self
            .children
            .get(node_id)
            .map(|v| v.clone())
            .unwrap_or_default();

        for child_id in children {
            self.delete_subtree(child_id)?;
        }
        self.delete_node(node_id)
    }

    pub fn traverse<T: Copy>(
        &self,
        node_id: NodeId,
        callback: &mut impl FnMut(NodeId, &NodeData, &NodeLayout, T) -> T,
        data: T,
    ) {
        fn traverse_inner<T: Copy>(
            data: &slotmap::SlotMap<NodeId, NodeData>,
            children: &slotmap::SecondaryMap<NodeId, Vec<NodeId>>,
            layout: &slotmap::SecondaryMap<NodeId, NodeLayout>,
            node_id: NodeId,
            callback: &mut impl FnMut(NodeId, &NodeData, &NodeLayout, T) -> T,
            acc: T,
        ) {
            let result = callback(node_id, &data[node_id], &layout[node_id], acc);
            for &child_id in &children[node_id] {
                traverse_inner(data, children, layout, child_id, callback, result);
            }
        }

        traverse_inner(
            &self.data,
            &self.children,
            &self.layout,
            node_id,
            callback,
            data,
        );
    }

    pub fn traverse_mut<T: Copy>(
        &mut self,
        node_id: NodeId,
        callback: &mut impl FnMut(NodeId, &mut NodeData, &mut NodeLayout, T) -> T,
        data: T,
    ) {
        fn traverse_inner_mut<T: Copy>(
            data: &mut slotmap::SlotMap<NodeId, NodeData>,
            children: &slotmap::SecondaryMap<NodeId, Vec<NodeId>>,
            layout: &mut slotmap::SecondaryMap<NodeId, NodeLayout>,
            node_id: NodeId,
            callback: &mut impl FnMut(NodeId, &mut NodeData, &mut NodeLayout, T) -> T,
            acc: T,
        ) {
            let result = callback(node_id, &mut data[node_id], &mut layout[node_id], acc);
            for &child_id in &children[node_id] {
                traverse_inner_mut(data, children, layout, child_id, callback, result);
            }
        }

        traverse_inner_mut(
            &mut self.data,
            &self.children,
            &mut self.layout,
            node_id,
            callback,
            data,
        );
    }

    pub fn mark_dirty(&mut self, node_id: NodeId, flags: DirtyFlags) -> Result<()> {
        let node_data = self
            .data
            .get_mut(node_id)
            .ok_or(Error::NodeNotFound(node_id))?;

        let new_dirty = node_data.dirty | flags;

        if new_dirty == node_data.dirty {
            return Ok(());
        }

        let newly_set = flags & !node_data.dirty;
        node_data.dirty = new_dirty;

        self.dirty_counter.increment(newly_set);

        if flags.contains(DirtyFlags::LAYOUT) {
            if let Ok(layout) = self
                .layout
                .get_mut(node_id)
                .ok_or(Error::NodeNotFound(node_id))
            {
                layout.cache.clear();
            }

            let parent = self
                .parent
                .get(node_id)
                .ok_or(Error::NodeNotFound(node_id))?;

            if let Some(parent_id) = parent {
                self.mark_dirty(*parent_id, flags)?;
            }
        }

        Ok(())
    }

    pub fn mark_clean(&mut self, node_id: NodeId, flags: DirtyFlags) -> Result<()> {
        let node_data = self
            .data
            .get_mut(node_id)
            .ok_or(Error::NodeNotFound(node_id))?;

        let new_dirty = node_data.dirty - flags;

        if new_dirty == node_data.dirty {
            return Ok(());
        }

        node_data.dirty = new_dirty;

        self.dirty_counter.decrement(flags);

        Ok(())
    }

    pub fn is_any_dirty(&self, flags: DirtyFlags) -> bool {
        self.dirty_counter.is_any_dirty(flags)
    }
}
