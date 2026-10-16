#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FPoint {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    pub fn center(&self) -> FPoint {
        FPoint {
            x: self.x + self.w / 2.0,
            y: self.y + self.h / 2.0,
        }
    }

    pub fn contains(&self, p: FPoint) -> bool {
        p.x >= self.x && p.x <= self.x + self.w && p.y >= self.y && p.y <= self.y + self.h
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contains_works_on_edges() {
        let r = Rect::new(10.0, 10.0, 20.0, 20.0);
        assert!(r.contains(FPoint { x: 10.0, y: 10.0 }));
        assert!(r.contains(FPoint { x: 30.0, y: 30.0 }));
        assert!(r.contains(FPoint { x: 20.0, y: 20.0 }));
        assert!(!r.contains(FPoint { x: 9.9, y: 20.0 }));
        assert!(!r.contains(FPoint { x: 20.0, y: 30.1 }));
    }

    #[test]
    fn center_is_the_middle() {
        assert_eq!(
            Rect::new(0.0, 0.0, 10.0, 20.0).center(),
            FPoint { x: 5.0, y: 10.0 }
        );
    }
}
