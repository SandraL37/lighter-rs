use std::{ any::Any, cell::RefCell, rc::Rc };

use slotmap::SlotMap;

use crate::core::{
    arena::node::{ NodeData, NodeId },
    layout::NodeLayout,
    reactive::{ dirty::DirtyFlags },
};

slotmap::new_key_type! {
    pub struct SignalId;
}

pub struct PendingUpdate {
    pub node_id: NodeId,
    pub flags: DirtyFlags,
    pub apply: Box<dyn FnOnce(&mut NodeData, &mut NodeLayout)>,
}

type Subscriber = Rc<dyn Fn()>;

pub struct SignalData {
    value: Box<dyn Any>,
    subscribers: Vec<Subscriber>,
}

pub struct Runtime {
    signals: slotmap::SlotMap<SignalId, SignalData>,
    pending_updates: Vec<PendingUpdate>,
    tracking_stack: Vec<Vec<SignalId>>,
}

impl Runtime {
    pub fn create_signal<T: 'static>(value: T) -> SignalId {
        RT.with(|rt| {
            rt.borrow_mut().signals.insert(SignalData {
                value: Box::new(value),
                subscribers: Vec::new(),
            })
        })
    }

    pub fn get<T: Clone + 'static>(id: SignalId) -> T {
        RT.with(|rt| {
            let mut rt = rt.borrow_mut();

            if let Some(deps) = rt.tracking_stack.last_mut() {
                if !deps.contains(&id) {
                    deps.push(id);
                }
            }
            rt.signals[id].value.downcast_ref::<T>().expect("signal type mismatch").clone()
        })
    }

    pub fn set<T: 'static>(id: SignalId, value: T) {
        let subs = RT.with(|rt| {
            let mut rt = rt.borrow_mut();

            let data = &mut rt.signals[id];
            data.value = Box::new(value);
            data.subscribers.clone()
        });

        for sub in subs {
            sub();
        }
    }

    pub fn update<T: 'static>(id: SignalId, f: impl FnOnce(&mut T)) {
        let subs = RT.with(|rt| {
            let mut rt = rt.borrow_mut();

            let data = &mut rt.signals[id];
            f(data.value.downcast_mut::<T>().expect("signal type mismatch"));
            data.subscribers.clone()
        });

        for sub in subs {
            sub();
        }
    }

    pub fn subscribe(id: SignalId, f: Subscriber) {
        RT.with(|rt| {
            let mut rt = rt.borrow_mut();
            rt.signals[id].subscribers.push(f);
        });
    }

    pub fn push_update(update: PendingUpdate) {
        RT.with(|rt| {
            rt.borrow_mut().pending_updates.push(update);
        })
    }

    pub fn drain_updates() -> Vec<PendingUpdate> {
        RT.with(|rt| {
            rt.borrow_mut().pending_updates.drain(..).collect()
        })
    }

    pub fn has_updates() -> bool {
        RT.with(|rt| !rt.borrow().pending_updates.is_empty())
    }

    pub fn track<T>(f: impl FnOnce() -> T) -> (T, Vec<SignalId>) {
        RT.with(|rt| {
            rt.borrow_mut().tracking_stack.push(Vec::new());
        });

        let value = f();

        let deps = RT.with(|rt| {
            rt.borrow_mut().tracking_stack.pop().unwrap()
        });

        (value, deps)
    }
}

thread_local! {
    static RT: RefCell<Runtime> = RefCell::new(Runtime {
        signals: SlotMap::with_key(),
        pending_updates: Vec::new(),
        tracking_stack: Vec::new(),
    });
}
