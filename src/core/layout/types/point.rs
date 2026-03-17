#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point<T> {
    pub x: T,
    pub y: T,
}

impl<T: Clone> Point<T> {
    pub fn xy(x: T, y: T) -> Self {
        Self { x, y }
    }
}
