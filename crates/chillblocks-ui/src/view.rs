use chillblocks_core::board::BOARD_SIZE;
use chillblocks_core::game::Placement;

use crate::geometry::FPoint;
use crate::input::PointerKind;
use crate::motion::{FADE_IN_MS, FADE_OUT_MS};

pub const BOARD_CELLS: usize = BOARD_SIZE * BOARD_SIZE;

/// A cell that has been cleared but is still fading out.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ghost {
    pub x: usize,
    pub y: usize,
    pub color: u8,
    pub started: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Drag {
    pub slot: usize,
    pub pointer: FPoint,
    pub origin: FPoint,
    pub started: f64,
    pub moved: bool,
    pub kind: PointerKind,
    /// Cell of the shape the finger caught. The piece is drawn so this cell sits under
    /// the pointer, inside the board and outside it alike.
    pub grab: (usize, usize),
    pub target: Option<(usize, usize)>,
}

/// Everything the renderer needs beyond the game state: what is being dragged, what is
/// fading, and the personal best. No gameplay decisions live here.
#[derive(Clone, Debug, PartialEq)]
pub struct View {
    pub drag: Option<Drag>,
    pub selected: Option<usize>,
    pub cursor: (usize, usize),
    pub ghosts: Vec<Ghost>,
    pub appear: [f64; BOARD_CELLS],
    pub best: u32,
    pub sound_on: bool,
    pub motion_scale: f64,
    pub dirty: bool,
}

/// A tap is a press that barely moved; anything longer is a drag.
pub const TAP_THRESHOLD: f32 = 14.0;

/// On touch the piece is drawn above the finger, otherwise the finger covers it.
pub const TOUCH_LIFT_CELLS: f32 = 1.4;

impl View {
    pub fn new() -> Self {
        Self {
            drag: None,
            selected: None,
            cursor: (0, 0),
            ghosts: Vec::new(),
            appear: [0.0; BOARD_CELLS],
            best: 0,
            sound_on: false,
            motion_scale: 1.0,
            dirty: true,
        }
    }

    pub fn reset(&mut self) {
        self.drag = None;
        self.selected = None;
        self.cursor = (0, 0);
        self.ghosts.clear();
        self.appear = [0.0; BOARD_CELLS];
        self.dirty = true;
    }

    pub fn record_placement(&mut self, placement: &Placement, now: f64) {
        for &cell in &placement.cells {
            self.appear[cell.y * BOARD_SIZE + cell.x] = now;
        }
        for &cell in &placement.cleared_cells {
            // A cleared cell is no longer a board cell: it becomes a fading ghost.
            self.appear[cell.y * BOARD_SIZE + cell.x] = 0.0;
            self.ghosts.push(Ghost {
                x: cell.x,
                y: cell.y,
                color: placement.color,
                started: now,
            });
        }
        self.dirty = true;
    }

    pub fn prune(&mut self, now: f64) {
        self.ghosts.retain(|g| now - g.started < FADE_OUT_MS);
    }

    pub fn animations_active(&self, now: f64) -> bool {
        if !self.ghosts.is_empty() {
            return true;
        }
        self.appear.iter().any(|&t| t > 0.0 && now - t < FADE_IN_MS)
    }
}

impl Default for View {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chillblocks_core::game::{CellPos, Line, Orientation};

    fn cell(x: usize, y: usize) -> CellPos {
        CellPos { x, y }
    }

    #[test]
    fn ghosts_expire_after_the_fade() {
        let mut view = View::new();
        view.ghosts.push(Ghost {
            x: 1,
            y: 2,
            color: 0,
            started: 100.0,
        });
        view.prune(100.0 + FADE_OUT_MS - 1.0);
        assert_eq!(view.ghosts.len(), 1);
        view.prune(100.0 + FADE_OUT_MS + 1.0);
        assert!(view.ghosts.is_empty());
    }

    #[test]
    fn a_clear_turns_placed_cells_into_ghosts() {
        let mut view = View::new();
        let placement = Placement {
            slot: 0,
            color: 3,
            cells: vec![cell(3, 0)],
            cleared_lines: vec![Line {
                orientation: Orientation::Row,
                index: 0,
            }],
            cleared_cells: vec![cell(3, 0)],
            points: 11,
        };
        view.record_placement(&placement, 500.0);
        assert_eq!(
            view.appear[3], 0.0,
            "a cleared cell must not also be an appearing cell"
        );
        assert_eq!(view.ghosts.len(), 1);
        assert_eq!(view.ghosts[0].color, 3);
    }

    #[test]
    fn a_plain_placement_keeps_the_cell_on_the_board() {
        let mut view = View::new();
        let placement = Placement {
            slot: 0,
            color: 1,
            cells: vec![cell(2, 4)],
            cleared_lines: Vec::new(),
            cleared_cells: Vec::new(),
            points: 1,
        };
        view.record_placement(&placement, 500.0);
        assert!(view.ghosts.is_empty());
        assert_eq!(view.appear[4 * BOARD_SIZE + 2], 500.0);
        assert!(view.animations_active(500.0));
        assert!(!view.animations_active(500.0 + FADE_IN_MS + 1.0));
    }

    #[test]
    fn tap_threshold_is_small_enough_for_a_tap_and_large_enough_for_a_drag() {
        let threshold = TAP_THRESHOLD;
        assert!(threshold < 20.0);
    }

    #[test]
    fn touch_lift_is_enough_to_clear_a_fingertip() {
        let lift = TOUCH_LIFT_CELLS;
        assert!(lift >= 1.0);
    }
}
