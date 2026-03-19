#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Copy, PartialEq)]
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
    pub const RED: Color = Color::rgba(1.0, 0.0, 0.0, 1.0);
    pub const GREEN: Color = Color::rgba(0.0, 1.0, 0.0, 1.0);
    pub const BLUE: Color = Color::rgba(0.0, 0.0, 1.0, 1.0);
    pub const YELLOW: Color = Color::rgba(1.0, 1.0, 0.0, 1.0);
    pub const CYAN: Color = Color::rgba(0.0, 1.0, 1.0, 1.0);
    pub const MAGENTA: Color = Color::rgba(1.0, 0.0, 1.0, 1.0);
    pub const ORANGE: Color = Color::rgba(1.0, 0.5, 0.0, 1.0);
    pub const PURPLE: Color = Color::rgba(0.5, 0.0, 1.0, 1.0);
    pub const GRAY: Color = Color::rgba(0.5, 0.5, 0.5, 1.0);
    pub const LIGHT_GRAY: Color = Color::rgba(0.75, 0.75, 0.75, 1.0);
    pub const DARK_GRAY: Color = Color::rgba(0.25, 0.25, 0.25, 1.0);

    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Color { r, g, b, a }
    }

    pub const fn hex(hex: u32) -> Self {
        let r = ((hex >> 16) & 0xff) as f32 / 255.0;
        let g = ((hex >> 8) & 0xff) as f32 / 255.0;
        let b = (hex & 0xff) as f32 / 255.0;
        Color { r, g, b, a: 1.0 }
    }

    pub const fn with_alpha(self, a: f32) -> Self {
        Color { a, ..self }
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Copy, PartialEq)]
pub struct Transform {
    pub matrix: [f32; 6],
}

impl Transform {
    pub const IDENTITY: Transform = Transform {
        matrix: [
            1.0, 0.0, // m11, m12
            0.0, 1.0, // m21, m22
            0.0, 0.0, // dx, dy
        ],
    };
}
