use std::ops::Add;

use crate::core::layout::types::{point::Point, size::Size};

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Copy, PartialEq)]
pub struct Rect<T> {
    pub location: Point<T>,
    pub size: Size<T>,
}

impl<T: Copy> Rect<T> {
    pub fn new(location: Point<T>, size: Size<T>) -> Self {
        Self { location, size }
    }

    pub fn xywh(x: T, y: T, width: T, height: T) -> Self {
        Self {
            location: Point::xy(x, y),
            size: Size::wh(width, height),
        }
    }

    pub fn includes(&self, point: Point<T>) -> bool
    where
        T: PartialOrd + Add<Output = T>,
    {
        let max_x = self.location.x + self.size.width;
        let max_y = self.location.y + self.size.height;

        point.x >= self.location.x
            && point.y >= self.location.y
            && point.x < max_x
            && point.y < max_y
    }
}
