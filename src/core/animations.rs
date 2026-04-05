pub trait Lerpable {
    fn lerp(self, other: Self, t: f32) -> Self;
}

#[cfg(test)]
mod tests {
    use crate::core::style::Color;

    use super::*;

    #[test]
    fn lerp_colors() {
        let blue = Color::BLUE;
        let green = Color::GREEN;

        for t in 1..=100 {
            let lerped = blue.lerp(green, t as f32 / 100.0);

            assert!(lerped.r >= 0.0 && lerped.r <= 1.0);
            assert!(lerped.g >= 0.0 && lerped.g <= 1.0);
            assert!(lerped.b >= 0.0 && lerped.b <= 1.0);
            assert!(lerped.a >= 0.0 && lerped.a <= 1.0);
        }
    }
}
