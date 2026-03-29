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
        let r = (((hex >> 16) & 0xff) as f32) / 255.0;
        let g = (((hex >> 8) & 0xff) as f32) / 255.0;
        let b = ((hex & 0xff) as f32) / 255.0;
        Color { r, g, b, a: 1.0 }
    }

    pub const fn with_alpha(self, a: f32) -> Self {
        Color { a, ..self }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    pub matrix: [f32; 6],
}

impl Transform {
    pub const IDENTITY: Transform = Transform {
        matrix: [
            1.0,
            0.0, // m11, m12
            0.0,
            1.0, // m21, m22
            0.0,
            0.0, // dx, dy
        ],
    };

    pub fn scale(sx: f32, sy: f32) -> Transform {
        Transform {
            matrix: [
                sx,
                0.0, // m11, m12
                0.0,
                sy, // m21, m22
                0.0,
                0.0, // dx, dy
            ],
        }
    }

    pub fn translate(tx: f32, ty: f32) -> Transform {
        Transform {
            matrix: [
                1.0,
                0.0, // m11, m12
                0.0,
                1.0, // m21, m22
                tx,
                ty, // dx, dy
            ],
        }
    }

    pub fn rotate(angle: f32) -> Transform {
        let cos = angle.cos();
        let sin = angle.sin();
        Transform {
            matrix: [
                cos,
                -sin, // m11, m12
                sin,
                cos, // m21, m22
                0.0,
                0.0, // dx, dy
            ],
        }
    }
}

#[macro_export]
macro_rules! deg {
    ($x:expr) => {
        (($x as f32) * std::f32::consts::PI / 180.0)
    };
}

#[macro_export]
macro_rules! rad {
    ($x:expr) => {
        $x
    };
}

impl std::ops::Mul for Transform {
    type Output = Transform;

    fn mul(self, other: Transform) -> Transform {
        let a = self.matrix;
        let b = other.matrix;

        Transform {
            matrix: [
                a[0] * b[0] + a[1] * b[2],
                a[0] * b[1] + a[1] * b[3],
                a[2] * b[0] + a[3] * b[2],
                a[2] * b[1] + a[3] * b[3],
                a[4] * b[0] + a[5] * b[2] + b[4],
                a[4] * b[1] + a[5] * b[3] + b[5],
            ],
        }
    }
}
