use crate::color::Color;
use crate::geometry::Rect;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Align {
    Left,
    Center,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Baseline {
    Top,
    Middle,
    Alphabetic,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextStyle {
    pub size: f32,
    pub family: &'static str,
    pub color: Color,
    pub weight: u32,
    pub align: Align,
    pub baseline: Baseline,
}

impl TextStyle {
    pub fn new(size: f32, color: Color) -> Self {
        Self {
            size,
            family: "system-ui",
            color,
            weight: 400,
            align: Align::Left,
            baseline: Baseline::Alphabetic,
        }
    }
}

/// The only thing a renderer has to implement. Keeping this small is what lets the
/// browser backend and a future native backend share one scene.
pub trait Painter {
    /// Logical size in CSS pixels; the backend applies its own device scaling.
    fn begin(&mut self, width: f32, height: f32);
    fn clear(&mut self, background: Color);
    fn fill_round_rect(&mut self, rect: &Rect, radius: f32, color: Color);
    fn stroke_round_rect(&mut self, rect: &Rect, radius: f32, width: f32, color: Color);
    fn text(&mut self, x: f32, y: f32, content: &str, style: &TextStyle);

    /// Backends that need to know the device pixel ratio can override this.
    fn set_device_scale(&mut self, _dpr: f32) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Recording(Vec<String>);

    impl Painter for Recording {
        fn begin(&mut self, width: f32, height: f32) {
            self.0.push(format!("begin {width}x{height}"));
        }
        fn clear(&mut self, background: Color) {
            self.0.push(format!("clear {}", background.css()));
        }
        fn fill_round_rect(&mut self, rect: &Rect, radius: f32, color: Color) {
            self.0
                .push(format!("fill {} r{radius} {}", rect.x, color.css()));
        }
        fn stroke_round_rect(&mut self, _rect: &Rect, radius: f32, width: f32, color: Color) {
            self.0
                .push(format!("stroke r{radius} w{width} {}", color.css()));
        }
        fn text(&mut self, x: f32, y: f32, content: &str, style: &TextStyle) {
            self.0
                .push(format!("text {x} {y} {content} {}", style.color.css()));
        }
    }

    #[test]
    fn painter_is_object_safe() {
        let mut recording = Recording(Vec::new());
        let painter: &mut dyn Painter = &mut recording;
        painter.begin(100.0, 100.0);
        painter.clear(Color::rgb(1, 2, 3));
        painter.fill_round_rect(&Rect::new(0.0, 0.0, 10.0, 10.0), 2.0, Color::rgb(4, 5, 6));
        painter.stroke_round_rect(
            &Rect::new(0.0, 0.0, 10.0, 10.0),
            2.0,
            1.0,
            Color::rgb(4, 5, 6),
        );
        painter.text(1.0, 2.0, "hi", &TextStyle::new(12.0, Color::rgb(7, 8, 9)));
        assert_eq!(recording.0.len(), 5);
    }
}
