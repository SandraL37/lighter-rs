bitflags::bitflags! {
    #[cfg_attr(feature = "debug", derive(Debug))]
    #[derive(Copy, Clone, PartialEq, Eq)]
    pub struct DirtyFlags: u8 {
        const LAYOUT = 1 << 0;
        const PAINT  = 1 << 1;
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Default)]
pub struct DirtyCounter {
    counts: [usize; 2],
}

impl DirtyCounter {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn increment(&mut self, flags: DirtyFlags) {
        let mut bits = flags.bits();

        while bits != 0 {
            let idx = bits.trailing_zeros() as usize;
            self.counts[idx] += 1;
            bits &= bits - 1;
        }
    }

    #[inline]
    pub fn decrement(&mut self, flags: DirtyFlags) {
        let mut bits = flags.bits();

        while bits != 0 {
            let idx = bits.trailing_zeros() as usize;

            debug_assert!(
                self.counts[idx] > 0,
                "DirtyCounter underflow on flag index {}",
                idx
            );

            self.counts[idx] -= 1;

            bits &= bits - 1;
        }
    }

    #[inline]
    pub fn is_any_dirty(&self, flags: DirtyFlags) -> bool {
        let mut bits = flags.bits();

        while bits != 0 {
            let idx = bits.trailing_zeros() as usize;

            if self.counts[idx] > 0 {
                return true;
            }

            bits &= bits - 1;
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dirty_counter() {
        let mut counter = DirtyCounter::new();

        counter.increment(DirtyFlags::LAYOUT);
        assert!(counter.is_any_dirty(DirtyFlags::LAYOUT));
        assert!(counter.is_any_dirty(DirtyFlags::LAYOUT | DirtyFlags::PAINT));
        assert!(!counter.is_any_dirty(DirtyFlags::PAINT));

        counter.decrement(DirtyFlags::LAYOUT);
        assert!(!counter.is_any_dirty(DirtyFlags::LAYOUT));
    }
}
