#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Size<T> {
    pub width: T,
    pub height: T,
}

impl<T: Copy> Size<T> {
    pub const fn wh(width: T, height: T) -> Self {
        Self { width, height }
    }

    pub fn uniform(value: T) -> Self {
        Self {
            width: value,
            height: value,
        }
    }
}
