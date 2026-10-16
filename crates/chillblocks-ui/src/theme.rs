use crate::color::Color;
use crate::painter::{Align, Baseline, TextStyle};

/// Muted, low-saturation palette. One accent, no saturated red anywhere: an invalid
/// placement is a dim outline, not an alarm.
#[derive(Clone, Debug, PartialEq)]
pub struct Theme {
    pub background: Color,
    pub panel: Color,
    pub board: Color,
    pub grid: Color,
    pub text: Color,
    pub text_muted: Color,
    pub accent: Color,
    pub palette: [Color; 6],
    pub font: &'static str,
}

pub const fn theme() -> Theme {
    Theme {
        background: Color::rgb(242, 239, 232),
        panel: Color::rgb(233, 228, 218),
        board: Color::rgb(229, 224, 213),
        grid: Color::rgb(217, 211, 199),
        text: Color::rgb(75, 74, 68),
        text_muted: Color::rgb(139, 136, 128),
        accent: Color::rgb(124, 140, 121),
        palette: [
            Color::rgb(143, 163, 148), // sage
            Color::rgb(201, 168, 143), // clay
            Color::rgb(142, 154, 168), // slate
            Color::rgb(211, 196, 162), // sand
            Color::rgb(169, 155, 176), // mauve
            Color::rgb(127, 168, 163), // teal
        ],
        font: "system-ui, -apple-system, 'Segoe UI', sans-serif",
    }
}

impl Theme {
    pub fn style(
        &self,
        size: f32,
        color: Color,
        weight: u32,
        align: Align,
        baseline: Baseline,
    ) -> TextStyle {
        TextStyle {
            size,
            family: self.font,
            color,
            weight,
            align,
            baseline,
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        theme()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The chill contract: nothing in the palette is loud.
    #[test]
    fn palette_is_low_saturation() {
        let t = theme();
        for color in t.palette.iter().chain([t.accent].iter()) {
            let max = color.r.max(color.g).max(color.b) as f32;
            let min = color.r.min(color.g).min(color.b) as f32;
            let saturation = if max == 0.0 { 0.0 } else { (max - min) / max };
            assert!(
                saturation < 0.35,
                "{color:?} is too saturated for a calm palette"
            );
        }
    }

    #[test]
    fn there_is_no_alarm_red() {
        let t = theme();
        for color in [
            t.background,
            t.panel,
            t.board,
            t.grid,
            t.text,
            t.text_muted,
            t.accent,
        ]
        .iter()
        .chain(t.palette.iter())
        {
            assert!(
                color.r as f32 <= color.g as f32 + 60.0,
                "{color:?} reads as alarm red"
            );
        }
    }
}
