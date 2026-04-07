use crate::core::interaction::InteractionState;

#[derive(Debug, Clone)]
pub struct Patch<T> {
    pub hover: Option<T>,
    pub active: Option<T>,
    pub focus: Option<T>,
    pub disabled: Option<T>,
}

impl<T> Default for Patch<T> {
    fn default() -> Self {
        Self {
            hover: None,
            active: None,
            focus: None,
            disabled: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Prop<T> {
    pub base: T,
    pub patch: Patch<T>,
}

impl<T> Prop<T> {
    pub fn new(base: T) -> Self {
        Prop {
            base,
            patch: Patch::default(),
        }
    }

    #[inline]
    pub fn resolve(&self, state: InteractionState) -> &T {
        if state.is_disabled() {
            if let Some(ref v) = self.patch.disabled {
                return v;
            }
        }
        if state.is_active() {
            if let Some(ref v) = self.patch.active {
                return v;
            }
        }
        if state.is_focused() {
            if let Some(ref v) = self.patch.focus {
                return v;
            }
        }
        if state.is_hovered() {
            if let Some(ref v) = self.patch.hover {
                return v;
            }
        }
        &self.base
    }

    #[inline]
    pub fn set_base(&mut self, value: T) {
        self.base = value;
    }

    #[inline]
    pub fn set_hover(&mut self, value: Option<T>) {
        if let Some(value) = value {
            self.patch.hover = Some(value);
        }
    }

    #[inline]
    pub fn set_active(&mut self, value: Option<T>) {
        if let Some(value) = value {
            self.patch.active = Some(value);
        }
    }

    #[inline]
    pub fn set_focus(&mut self, value: Option<T>) {
        if let Some(value) = value {
            self.patch.focus = Some(value);
        }
    }

    #[inline]
    pub fn set_disabled(&mut self, value: Option<T>) {
        if let Some(value) = value {
            self.patch.disabled = Some(value);
        }
    }
}

impl<T: Copy> Prop<T> {
    #[inline]
    pub fn get(&self, state: InteractionState) -> T {
        *self.resolve(state)
    }
}
