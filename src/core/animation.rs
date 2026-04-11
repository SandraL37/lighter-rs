use std::time::Instant;

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

#[inline]
fn linear(t: f32) -> f32 {
    t
}

#[inline]
fn ease_in(t: f32) -> f32 {
    t * t
}

#[inline]
fn ease_out(t: f32) -> f32 {
    1.0 - (1.0 - t) * (1.0 - t)
}

#[inline]
fn ease_in_out(t: f32) -> f32 {
    if t < 0.5 { 2.0 * t * t } else { 1.0 - (-2.0 * t + 2.0).powi(2) / 2.0 }
}

impl Easing {
    pub const LINEAR: Self = Self(linear);
    pub const EASE_IN: Self = Self(ease_in);
    pub const EASE_OUT: Self = Self(ease_out);
    pub const EASE_IN_OUT: Self = Self(ease_in_out);
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

#[derive(Clone, Copy, Debug)]
pub struct TransitionRun<T: Interpolate> {
    pub from: T,
    pub to: T,
    pub started_at: Instant,
    pub spec: TransitionDir,
}

impl<T: Interpolate> TransitionRun<T> {
    pub fn progress(self, now: Instant) -> f32 {
        let duration_ms = self.spec.duration.0.max(1) as f32;
        let elapsed_ms = now.duration_since(self.started_at).as_secs_f32() * 1000.0;
        (elapsed_ms / duration_ms).clamp(0.0, 1.0)
    }

    pub fn sample(self, now: Instant) -> T {
        let t = (self.spec.easing.0)(self.progress(now));
        T::interpolate(self.from, self.to, t)
    }

    pub fn is_finished(self, now: Instant) -> bool {
        self.progress(now) >= 1.0
    }
}

pub trait Interpolate: Copy {
    fn interpolate(from: Self, to: Self, t: f32) -> Self;
}

impl Interpolate for f32 {
    fn interpolate(from: Self, to: Self, t: f32) -> Self {
        let t = t.clamp(0.0, 1.0);

        let delta = to - from;
        from + delta * t
    }
}

impl Interpolate for Color {
    fn interpolate(from: Self, to: Self, t: f32) -> Self {
        let t = t.clamp(0.0, 1.0);

        let delta = to - from;
        from +
            Self {
                r: delta.r * t,
                g: delta.g * t,
                b: delta.b * t,
                a: delta.a * t,
            }
    }
}
