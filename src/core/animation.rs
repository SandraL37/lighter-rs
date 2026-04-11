use crate::core::style::Color;

#[derive(Debug, Clone, Copy)]
pub struct Duration(pub u32);

impl Duration {
    pub fn from_millis(duration: u32) -> Self {
        Self(duration)
    }

    pub fn from_secs(duration: u32) -> Self {
        Self(duration * 1000)
    }
}

pub trait DurationExt {
    fn ms(self) -> Duration;
    fn s(self) -> Duration;
}

impl DurationExt for u32 {
    fn ms(self) -> Duration {
        Duration::from_millis(self)
    }

    fn s(self) -> Duration {
        Duration::from_secs(self)
    }
}

#[derive(Clone, Copy)]
pub struct Easing(pub fn(f32) -> f32);

impl Easing {
    pub const LINEAR: Self = Self(|t| t);
    pub const EASE_IN: Self = Self(|t| t * t);
    pub const EASE_OUT: Self = Self(|t| 1.0 - (1.0 - t) * (1.0 - t));
}

impl std::fmt::Debug for Easing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Easing").finish()
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct TransitionSpec {
    pub enter: Option<TransitionDir>,
    pub exit: Option<TransitionDir>,
}

#[derive(Clone, Copy, Debug)]
pub struct TransitionDir {
    pub duration: Duration,
    pub easing: Easing,
}

pub trait Interpolate: Copy {
    fn interpolate(from: Self, to: Self, t: f32) -> Self;
}

impl Interpolate for f32 {
    fn interpolate(from: Self, to: Self, t: f32) -> Self {
        let delta = to - from;
        from + delta * t
    }
}

impl Interpolate for Color {
    fn interpolate(from: Self, to: Self, t: f32) -> Self {
        let delta = to - from;

        from + Self {
            r: delta.r * t,
            g: delta.g * t,
            b: delta.b * t,
            a: delta.a * t,
        }
    }
}
