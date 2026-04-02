use std::rc::Rc;

use crate::core::{
    arena::node::{NodeData, NodeId},
    layout::NodeLayout,
    reactive::{
        dirty::DirtyFlags,
        runtime::{PendingUpdate, Runtime},
        signal::MaybeSignal,
    },
};

pub struct DeferredBinding(pub Box<dyn FnOnce(NodeId)>);

impl std::fmt::Debug for DeferredBinding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("DeferredBinding").finish()
    }
}

pub trait HasDeferredBindings {
    type Style;

    fn bindings(&mut self) -> &mut Vec<DeferredBinding>;
    fn style(&mut self) -> &mut Self::Style;

    fn bind<T: Clone + 'static>(
        &mut self,
        current: impl Fn(&mut Self::Style) -> &mut T,
        value: impl Into<MaybeSignal<T>>,
        flags: DirtyFlags,
        apply: impl Fn(&mut NodeData, &mut NodeLayout, T) + 'static,
    ) {
        match value.into() {
            MaybeSignal::Static(v) => {
                *current(self.style()) = v;
            }
            MaybeSignal::Signal(sig) => {
                {
                    let current_field = current(self.style());
                    *current_field = sig.get();
                }

                let apply = Rc::new(apply);
                self.bindings()
                    .push(DeferredBinding(Box::new(move |node_id| {
                        sig.subscribe(move || {
                            let val = sig.get();
                            let apply = apply.clone();
                            Runtime::push_update(PendingUpdate {
                                node_id,
                                flags,
                                apply: Box::new(move |data, layout| apply(data, layout, val)),
                            });
                        });
                    })));
            }
        }
    }
}
