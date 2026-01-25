use crate::types::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Point { x, y }
    }
}

impl From<foundation::Point> for Point {
    fn from(point: foundation::Point) -> Self {
        Point {
            x: point.x as f32,
            y: point.y as f32,
        }
    }
}

impl Into<windows_numerics::Vector2> for Point {
    fn into(self) -> windows_numerics::Vector2 {
        windows_numerics::Vector2 {
            X: self.x,
            Y: self.y,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Size {
    pub fn new(width: f32, height: f32) -> Self {
        Size { width, height }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

impl Rect {
    pub fn xywh(x: f32, y: f32, width: f32, height: f32) -> Self {
        Rect {
            left: x,
            top: y,
            right: x + width,
            bottom: y + height,
        }
    }

    pub fn pp(start: Point, end: Point) -> Self {
        Rect {
            left: start.x,
            top: start.y,
            right: end.x,
            bottom: end.y,
        }
    }

    pub fn ps(start: Point, size: Size) -> Self {
        Rect {
            left: start.x,
            top: start.y,
            right: start.x + size.width,
            bottom: start.y + size.height,
        }
    }

    pub fn win32(&self) -> foundation::Rect {
        foundation::Rect {
            left: self.left as i32,
            top: self.top as i32,
            right: self.right as i32,
            bottom: self.bottom as i32,
        }
    }

    pub fn d2d(&self) -> d2d::common::RectF {
        d2d::common::RectF {
            left: self.left,
            top: self.top,
            right: self.right,
            bottom: self.bottom,
        }
    }
}

impl Into<foundation::Rect> for Rect {
    fn into(self) -> foundation::Rect {
        foundation::Rect {
            left: self.left as i32,
            top: self.top as i32,
            right: self.right as i32,
            bottom: self.bottom as i32,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Color { r, g, b, a }
    }

    pub fn d2d(&self) -> d2d::common::ColorF {
        d2d::common::ColorF {
            r: self.r,
            g: self.g,
            b: self.b,
            a: self.a,
        }
    }
}

impl Into<d2d::common::ColorF> for Color {
    fn into(self) -> d2d::common::ColorF {
        d2d::common::ColorF {
            r: self.r,
            g: self.g,
            b: self.b,
            a: self.a,
        }
    }
}
