use std::{cell::RefCell, rc::Rc};

struct SignalInner<T> {
    pub(crate) value: T,
    pub(crate) subscribers: Vec<Box<dyn Fn(&T) + 'static>>,
}

pub struct Signal<T>(Rc<RefCell<SignalInner<T>>>);

impl<T: Clone> Signal<T> {
    pub fn new(value: T) -> Self {
        Signal(Rc::new(RefCell::new(SignalInner {
            value,
            subscribers: Vec::new(),
        })))
    }

    pub fn get(&self) -> T {
        self.0.borrow().value.clone()
    }

    pub fn set(&self, value: T) {
        let mut inner = self.0.borrow_mut();
        inner.value = value;

        for sub in &inner.subscribers {
            sub(&inner.value);
        }
    }

    pub fn read(&self) -> ReadSignal<T> {
        ReadSignal(Rc::clone(&self.0))
    }
}

impl<T> Clone for Signal<T> {
    fn clone(&self) -> Self {
        Signal(Rc::clone(&self.0))
    }
}

pub struct ReadSignal<T>(Rc<RefCell<SignalInner<T>>>);

impl<T: Clone> ReadSignal<T> {
    pub fn get(&self) -> T {
        self.0.borrow().value.clone()
    }

    pub fn map<F, U>(self, f: F) -> ReadSignal<U>
    // TODO: this is a horrible escape hatch, needs to be fixed
    // TODO: check if map needs to be deleted, it is used by layout and text but i think it could be managed better
    where
        F: Fn(T) -> U + 'static,
        U: Clone + 'static,
    {
        let mapped_signal = ReadSignal(Rc::new(RefCell::new(SignalInner {
            value: f(self.get()),
            subscribers: Vec::new(),
        })));
        let mapped_clone = mapped_signal.clone();
        self.subscribe(move |val| {
            let new_val = f(val.clone());
            let mut inner = mapped_clone.0.borrow_mut();
            inner.value = new_val;
            let new_val_ref = &inner.value;
            for sub in &inner.subscribers {
                sub(new_val_ref);
            }
        });
        mapped_signal
    }
}

impl<T> ReadSignal<T> {
    pub(crate) fn subscribe(&self, f: impl Fn(&T) + 'static) {
        self.0.borrow_mut().subscribers.push(Box::new(f));
    }
}

impl<T> Clone for ReadSignal<T> {
    fn clone(&self) -> Self {
        ReadSignal(Rc::clone(&self.0))
    }
}

pub enum Reactive<T> {
    Static(T),
    Dynamic(ReadSignal<T>),
}

impl<T: std::fmt::Debug + Clone> std::fmt::Debug for Reactive<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Reactive::Static(v) => write!(f, "Static({:?})", v),
            Reactive::Dynamic(sig) => write!(f, "Dynamic({:?})", sig.get()),
        }
    }
}

impl<T: Clone> Reactive<T> {
    pub fn get(&self) -> T {
        match self {
            Reactive::Static(v) => v.clone(),
            Reactive::Dynamic(sig) => sig.get(),
        }
    }

    pub fn map<F, U>(self, f: F) -> Reactive<U>
    where
        F: Fn(T) -> U + 'static,
        U: Clone + 'static,
    {
        match self {
            Reactive::Static(v) => Reactive::Static(f(v)),
            Reactive::Dynamic(sig) => Reactive::Dynamic(sig.map(f)),
        }
    }
}

impl<T> From<T> for Reactive<T> {
    fn from(val: T) -> Self {
        Reactive::Static(val)
    }
}

impl<T> From<ReadSignal<T>> for Reactive<T> {
    fn from(sig: ReadSignal<T>) -> Self {
        Reactive::Dynamic(sig)
    }
}

pub fn signal<T: Clone>(value: T) -> Signal<T> {
    Signal::new(value)
}
