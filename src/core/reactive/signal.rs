use std::{marker::PhantomData, rc::Rc};

use crate::core::reactive::runtime::{ Runtime, SignalId };

pub struct Signal<T: 'static> {
    pub(crate) id: SignalId,
    _marker: PhantomData<T>,
}

impl<T: 'static> Clone for Signal<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl <T: 'static> Copy for Signal<T> {}

impl<T: Clone + 'static> Signal<T> {
    pub fn get(self) -> T {
        Runtime::get::<T>(self.id)
    }

    pub fn set(self, value: T) {
        Runtime::set::<T>(self.id, value);
    }

    pub fn update(self, f: impl FnOnce(&mut T)) {
        Runtime::update::<T>(self.id, f);
    }

    pub fn derive(f: impl Fn() -> T + 'static) -> Signal<T> {
        let (initial, deps) = Runtime::track(|| f());
        let derived = signal(initial);

        if !deps.is_empty() {
            let f = Rc::new(f);
            for dep in deps {
                let f = f.clone();
                Runtime::subscribe(dep, Rc::new(move || {
                    derived.set(f());
                }));
            }
        }
        
        derived
    }

    pub(crate) fn subscribe(self, f: impl Fn() + 'static) {
        Runtime::subscribe(self.id, Rc::new(f));
    }
}

pub fn signal<T: Clone + 'static>(value: T) -> Signal<T> {
    Signal {
        id: Runtime::create_signal(value),
        _marker: PhantomData,
    }
}

pub fn derived<T: Clone + 'static>(f: impl Fn() -> T + 'static) -> Signal<T> {
    Signal::derive(f)
}

pub enum MaybeSignal<T: 'static> {
    Static(T),
    Signal(Signal<T>),
}

impl<T: Clone + 'static> MaybeSignal<T> {
    pub fn get(&self) -> T {
        match self {
            MaybeSignal::Static(v) => v.clone(),
            MaybeSignal::Signal(s) => s.get(),
        }
    }
}

impl<T: 'static> From<T> for MaybeSignal<T> {
    fn from(value: T) -> Self {
        MaybeSignal::Static(value)
    }
}

impl<T: 'static> From<Signal<T>> for MaybeSignal<T> {
    fn from(signal: Signal<T>) -> Self {
        MaybeSignal::Signal(signal)
    }
}

impl<T: Clone + 'static> Clone for MaybeSignal<T> {
    fn clone(&self) -> Self {
        match self {
            MaybeSignal::Static(v) => MaybeSignal::Static(v.clone()),
            MaybeSignal::Signal(s) => MaybeSignal::Signal(*s),
        }
    }
}

impl<T: Clone + 'static> MaybeSignal<T> {
    /// Maps the value through `f`. For static values this is free; for signals
    /// it creates a derived signal (one extra reactive hop).
    pub fn map<U: Clone + 'static>(self, f: impl Fn(T) -> U + 'static) -> MaybeSignal<U> {
        match self {
            MaybeSignal::Static(v) => MaybeSignal::Static(f(v)),
            MaybeSignal::Signal(sig) => MaybeSignal::Signal(Signal::derive(move || f(sig.get()))),
        }
    }
}