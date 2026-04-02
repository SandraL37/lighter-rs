bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct InteractionState: u8 {
        const HOVER = 1 << 0;
        const ACTIVE = 1 << 1;
        const FOCUS = 1 << 2;
        const DISABLED = 1 << 3;
    }
}

impl InteractionState {
    pub fn set_flag(&mut self, flag: InteractionState, on: bool) {
        if on {
            self.insert(flag);
        } else {
            self.remove(flag);
        }
    }

    #[inline(always)]
    pub fn is_hovered(self) -> bool {
        self.contains(InteractionState::HOVER)
    }

    #[inline(always)]
    pub fn is_active(self) -> bool {
        self.contains(InteractionState::ACTIVE)
    }

    #[inline(always)]
    pub fn is_focused(self) -> bool {
        self.contains(InteractionState::FOCUS)
    }

    #[inline(always)]
    pub fn is_disabled(self) -> bool {
        self.contains(InteractionState::DISABLED)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum PatchValue<T> {
    #[default]
    Keep,
    Set(T),
}

#[derive(Debug, Clone, Default)]
pub struct StatePatches<P> {
    pub hover: Option<P>,
    pub active: Option<P>,
    pub focus: Option<P>,
    pub disabled: Option<P>,
}

pub(crate) fn select_patch<P>(state: InteractionState, patches: &StatePatches<P>) -> Option<&P> {
    if state.is_disabled() {
        if let Some(p) = &patches.disabled {
            return Some(p);
        }
    }
    if state.is_active() {
        if let Some(p) = &patches.active {
            return Some(p);
        }
    }
    if state.is_hovered() {
        if let Some(p) = &patches.hover {
            return Some(p);
        }
    }
    if state.is_focused() {
        if let Some(p) = &patches.focus {
            return Some(p);
        }
    }
    None
}
