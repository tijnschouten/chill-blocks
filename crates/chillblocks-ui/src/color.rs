#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: f32,
}

impl Color {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    pub fn with_alpha(&self, a: f32) -> Self {
        Self {
            r: self.r,
            g: self.g,
            b: self.b,
            a: a.clamp(0.0, 1.0),
        }
    }

    pub fn css(&self) -> String {
        format!("rgba({}, {}, {}, {})", self.r, self.g, self.b, self.a)
    }

    /// Blend `other` over `self` by `t`, used for soft cross-fades.
    pub fn mix(&self, other: &Color, t: f32) -> Color {
        let t = t.clamp(0.0, 1.0);
        let lerp = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * t) as u8;
        Color {
            r: lerp(self.r, other.r),
            g: lerp(self.g, other.g),
            b: lerp(self.b, other.b),
            a: self.a + (other.a - self.a) * t,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alpha_is_clamped() {
        assert_eq!(Color::rgb(1, 2, 3).with_alpha(2.0).a, 1.0);
        assert_eq!(Color::rgb(1, 2, 3).with_alpha(-1.0).a, 0.0);
    }

    #[test]
    fn mix_endpoints_are_exact() {
        let a = Color::rgb(0, 100, 200);
        let b = Color::rgb(10, 110, 210);
        assert_eq!(a.mix(&b, 0.0), a);
        assert_eq!(a.mix(&b, 1.0), b);
        assert_eq!(a.mix(&b, 0.5), Color::rgb(5, 105, 205));
    }

    #[test]
    fn css_is_readable() {
        assert_eq!(Color::rgb(10, 20, 30).css(), "rgba(10, 20, 30, 1)");
    }
}
