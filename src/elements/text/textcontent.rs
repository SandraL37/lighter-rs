use std::sync::Arc;

use crate::core::reactive::signal::{MaybeSignal, Signal, signal};

pub trait IntoTextContent {
    fn into_text_content(self) -> MaybeSignal<Arc<str>>;
}

impl<T: Into<Arc<str>>> IntoTextContent for T {
    fn into_text_content(self) -> MaybeSignal<Arc<str>> {
        MaybeSignal::Static(self.into())
    }
}

impl<T: std::fmt::Display + Clone + 'static> IntoTextContent for Signal<T> {
    fn into_text_content(self) -> MaybeSignal<Arc<str>> {
        let text_sig = signal::<Arc<str>>(Arc::from(self.get().to_string().as_str()));
        self.subscribe(move || {
            text_sig.set(Arc::from(self.get().to_string().as_str()));
        });
        MaybeSignal::Signal(text_sig)
    }
}
