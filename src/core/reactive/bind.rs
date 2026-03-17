use crate::core::{
    arena::node::{ NodeData, NodeId },
    layout::NodeLayout,
    reactive::{ dirty::DirtyFlags, runtime::{ PendingUpdate, Runtime }, signal::MaybeSignal },
};

pub struct DeferredBinding(pub Box<dyn FnOnce(NodeId)>);

impl std::fmt::Debug for DeferredBinding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("DeferredBinding").finish()
    }
}

pub fn bind_field<P: 'static, T: Clone + 'static>(
    props: &mut P,
    bindings: &mut Vec<DeferredBinding>,
    value: impl Into<MaybeSignal<T>>,
    flags: DirtyFlags,
    accessor: fn(&mut P) -> &mut T,
    resolve: fn(&mut NodeData, &mut NodeLayout) -> &'static mut P
) {
    match value.into() {
        MaybeSignal::Static(v) => {
            *accessor(props) = v;
        }
        MaybeSignal::Signal(sig) => {
            *accessor(props) = sig.get();
            bindings.push(
                DeferredBinding(
                    Box::new(move |node_id| {
                        sig.subscribe(move || {
                            let val = sig.get();
                            Runtime::push_update(PendingUpdate {
                                node_id,
                                flags,
                                apply: Box::new(move |data, layout| {
                                    *accessor(resolve(data, layout)) = val;
                                }),
                            });
                        });
                    })
                )
            );
        }
    }
}
