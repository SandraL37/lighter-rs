#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const TRANSPARENT: Color = Color::rgba(0.0, 0.0, 0.0, 0.0);
    pub const BLACK: Color = Color::rgba(0.0, 0.0, 0.0, 1.0);
    pub const WHITE: Color = Color::rgba(1.0, 1.0, 1.0, 1.0);

    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Color { r, g, b, a }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    pub matrix: [[f32; 4]; 4],
}

impl Transform {
    pub const IDENTITY: Transform = Transform {
        matrix: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
    };
}
