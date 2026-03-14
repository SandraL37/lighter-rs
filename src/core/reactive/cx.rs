use std::{cell::RefCell, rc::Rc};

use crate::core::{
    arena::node::{NodeData, NodeId},
    layout::NodeLayout,
    reactive::{
        dirty::DirtyFlags,
        signal::{Reactive, ReadSignal},
    },
};

pub struct PendingUpdate {
    pub node_id: NodeId,
    pub flags: DirtyFlags,
    pub apply: Box<dyn FnOnce(&mut NodeData, &mut NodeLayout)>,
}

pub type UpdateQueue = Rc<RefCell<Vec<PendingUpdate>>>;

pub struct DeferredBinding(pub Box<dyn FnOnce(NodeId, &Cx)>);

impl std::fmt::Debug for DeferredBinding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DeferredBinding").finish()
    }
}

pub struct Cx {
    pub(crate) updates: UpdateQueue,
}

impl Cx {
    pub fn new() -> Self {
        Cx {
            updates: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub fn updates(&self) -> UpdateQueue {
        Rc::clone(&self.updates)
    }

    pub fn bind<T: Clone + 'static>(
        &self,
        read_signal: ReadSignal<T>,
        node_id: NodeId,
        flags: DirtyFlags,
        setter: impl Fn(&mut NodeData, &mut NodeLayout, T) + Clone + 'static,
    ) {
        let q = self.updates();
        read_signal.subscribe(move |val| {
            let val = val.clone();
            let setter = setter.clone();
            q.borrow_mut().push(PendingUpdate {
                node_id,
                flags,
                apply: Box::new(move |data, layout| setter(data, layout, val)),
            });
        });
    }
}

impl Default for Cx {
    fn default() -> Self {
        Self::new()
    }
}

pub trait ReactivePropsExt: Sized {
    fn deferred_bindings(&mut self) -> &mut Vec<DeferredBinding>;
    fn bind<T>(
        &mut self,
        value: impl Into<Reactive<T>>,
        static_callback: &mut (impl FnMut(&mut Self, T) + 'static),
        dirty_flags: DirtyFlags,
        reactive_callback: impl Fn(&mut NodeData, &mut NodeLayout, T) + Clone + 'static,
    ) where
        T: Clone + 'static,
    {
        match value.into() {
            Reactive::Static(v) => static_callback(self, v),
            Reactive::Dynamic(read_signal) => {
                static_callback(self, read_signal.get());
                self.deferred_bindings()
                    .push(DeferredBinding(Box::new(move |node_id, cx| {
                        cx.bind(read_signal, node_id, dirty_flags, reactive_callback);
                    })));
            }
        }
    }
}
