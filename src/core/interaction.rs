use crate::core::state::ComputedState;

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct InteractionState: u8 {
        const HOVER    = 1 << 0;
        const ACTIVE   = 1 << 1;
        const FOCUS    = 1 << 2;
        const DISABLED = 1 << 3;
    }
}

impl InteractionState {
    #[inline]
    pub fn set_flag(&mut self, flag: InteractionState, on: bool) {
        if on {
            self.insert(flag);
        } else {
            self.remove(flag);
        }
    }

    #[inline]
    pub fn is_hovered(self) -> bool {
        self.contains(Self::HOVER)
    }
    #[inline]
    pub fn is_active(self) -> bool {
        self.contains(Self::ACTIVE)
    }
    #[inline]
    pub fn is_focused(self) -> bool {
        self.contains(Self::FOCUS)
    }
    #[inline]
    pub fn is_disabled(self) -> bool {
        self.contains(Self::DISABLED)
    }
}

impl From<InteractionState> for ComputedState {
    fn from(value: InteractionState) -> Self {
        if value.is_active() {
            ComputedState::Active
        } else if value.is_hovered() {
            ComputedState::Hover
        } else {
            ComputedState::Base
        }
    }
}
