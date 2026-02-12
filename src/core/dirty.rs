bitflags::bitflags! {
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub struct DirtyFlags: u8 {
        const PROPS = 1 << 0;
        const LAYOUT = 1 << 1;
        const PAINT = 1 << 2;
        const ALL = Self::PROPS.bits() | Self::LAYOUT.bits() | Self::PAINT.bits();
    }
}
