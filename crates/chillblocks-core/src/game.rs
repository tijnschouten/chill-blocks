use crate::board::Board;
use crate::piece::{Shape, SHAPES};
use crate::rng::Rng;

pub const TRAY_SIZE: usize = 3;
pub const PALETTE_SIZE: usize = 6;

/// Deliberately flat scoring: no streaks, no multipliers, no combo chains.
/// A bigger clear is worth more only because it is worth more lines.
pub const POINTS_PER_CELL: u32 = 1;
pub const POINTS_PER_LINE: u32 = 10;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CellPos {
    pub x: usize,
    pub y: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Orientation {
    Row,
    Column,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Line {
    pub orientation: Orientation,
    pub index: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Piece {
    pub shape: usize,
    pub color: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Playing,
    GameOver,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlaceError {
    SlotEmpty,
    DoesNotFit,
    GameOver,
}

/// What a placement changed, so the renderer can animate exactly that.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Placement {
    pub slot: usize,
    pub color: u8,
    pub cells: Vec<CellPos>,
    pub cleared_lines: Vec<Line>,
    pub cleared_cells: Vec<CellPos>,
    pub points: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Game {
    pub board: Board,
    pub tray: [Option<Piece>; TRAY_SIZE],
    pub rng: Rng,
    pub score: u32,
    pub lines_cleared: u32,
    pub pieces_placed: u32,
    pub status: Status,
}

impl Game {
    pub fn new(seed: u64) -> Self {
        let mut game = Game {
            board: Board::new(),
            tray: [None; TRAY_SIZE],
            rng: Rng::new(seed),
            score: 0,
            lines_cleared: 0,
            pieces_placed: 0,
            status: Status::Playing,
        };
        for slot in 0..TRAY_SIZE {
            game.tray[slot] = Some(game.draw());
        }
        game
    }

    pub fn shape(&self, piece: Piece) -> &Shape {
        &SHAPES[piece.shape]
    }

    /// Uniform draw over the fixed shape table. No weighting, no streak logic.
    fn draw(&mut self) -> Piece {
        Piece {
            shape: self.rng.below(SHAPES.len() as u64) as usize,
            color: self.rng.below(PALETTE_SIZE as u64) as u8,
        }
    }

    pub fn can_place(&self, slot: usize, x: usize, y: usize) -> bool {
        match self.tray[slot] {
            Some(piece) => self.board.can_fit(self.shape(piece), x, y),
            None => false,
        }
    }

    /// Whether any remaining tray piece fits anywhere.
    pub fn has_any_move(&self) -> bool {
        if self.status != Status::Playing {
            return false;
        }
        self.tray.iter().flatten().any(|piece| {
            let shape = self.shape(*piece);
            (0..crate::board::BOARD_SIZE)
                .any(|y| (0..crate::board::BOARD_SIZE).any(|x| self.board.can_fit(shape, x, y)))
        })
    }

    pub fn place(&mut self, slot: usize, x: usize, y: usize) -> Result<Placement, PlaceError> {
        if self.status == Status::GameOver {
            return Err(PlaceError::GameOver);
        }
        let piece = self.tray[slot].ok_or(PlaceError::SlotEmpty)?;
        let shape = self.shape(piece);
        if !self.board.can_fit(shape, x, y) {
            return Err(PlaceError::DoesNotFit);
        }

        let mut placement = Placement {
            slot,
            color: piece.color,
            cells: shape
                .cells()
                .map(|(dx, dy)| CellPos {
                    x: x + dx as usize,
                    y: y + dy as usize,
                })
                .collect(),
            ..Default::default()
        };

        for &cell in &placement.cells {
            self.board.set(cell.x, cell.y, piece.color);
        }
        self.tray[slot] = None;
        self.pieces_placed += 1;

        let lines = self.board.full_lines();
        for &line in &lines {
            for i in 0..crate::board::BOARD_SIZE {
                let (cx, cy) = match line.orientation {
                    Orientation::Row => (i, line.index),
                    Orientation::Column => (line.index, i),
                };
                placement.cleared_cells.push(CellPos { x: cx, y: cy });
            }
            self.lines_cleared += 1;
        }

        // Flat scoring: one point per placed cell, one bonus per cleared line.
        placement.points =
            placement.cells.len() as u32 * POINTS_PER_CELL + lines.len() as u32 * POINTS_PER_LINE;
        self.score += placement.points;
        placement.cleared_lines = lines;

        for &cell in &placement.cleared_cells {
            self.board.clear_cell(cell.x, cell.y);
        }

        if self.tray.iter().all(|slot| slot.is_none()) {
            for slot in 0..TRAY_SIZE {
                self.tray[slot] = Some(self.draw());
            }
        }

        if !self.has_any_move() {
            self.status = Status::GameOver;
        }

        Ok(placement)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::BOARD_SIZE;

    const DOT: usize = 0;
    const BAR_4: usize = 5; // horizontal 4x1

    fn force_tray(game: &mut Game, shapes: [usize; TRAY_SIZE]) {
        for (slot, shape) in shapes.iter().enumerate() {
            game.tray[slot] = Some(Piece {
                shape: *shape,
                color: 0,
            });
        }
    }

    #[test]
    fn placement_without_a_clear_scores_one_per_cell() {
        let mut game = Game::new(1);
        force_tray(&mut game, [DOT, DOT, DOT]);
        let placement = game.place(0, 0, 0).unwrap();
        assert_eq!(placement.points, 1);
        assert_eq!(game.score, 1);
    }

    #[test]
    fn rejects_used_slots_and_out_of_bounds() {
        let mut game = Game::new(2);
        force_tray(&mut game, [DOT, DOT, DOT]);
        game.place(0, 0, 0).unwrap();
        assert_eq!(game.place(0, 5, 5), Err(PlaceError::SlotEmpty));
    }

    #[test]
    fn rejects_overlap_and_out_of_bounds() {
        let mut game = Game::new(2);
        force_tray(&mut game, [BAR_4, DOT, DOT]);
        game.board.set(2, 0, 0);
        assert_eq!(game.place(0, 0, 0), Err(PlaceError::DoesNotFit));
        assert_eq!(game.place(0, 5, 0), Err(PlaceError::DoesNotFit));
    }

    #[test]
    fn fills_a_row_and_clears_it() {
        let mut game = Game::new(3);
        force_tray(&mut game, [DOT, DOT, DOT]);
        for x in 0..BOARD_SIZE - 1 {
            game.board.set(x, 0, 0);
        }
        let placement = game.place(0, BOARD_SIZE - 1, 0).unwrap();
        assert_eq!(placement.cleared_lines.len(), 1);
        assert_eq!(placement.cleared_cells.len(), BOARD_SIZE);
        assert_eq!(placement.points, 1 + POINTS_PER_LINE);
        assert_eq!(game.score, 1 + POINTS_PER_LINE);
        assert_eq!(game.lines_cleared, 1);
        assert_eq!(game.board.filled_count(), 0);
    }

    #[test]
    fn a_cross_clears_both_lines_and_scores_both() {
        let mut game = Game::new(4);
        force_tray(&mut game, [DOT, DOT, DOT]);
        for i in 0..BOARD_SIZE {
            game.board.set(i, 4, 0);
            game.board.set(4, i, 0);
        }
        game.board.clear_cell(4, 4);
        let placement = game.place(0, 4, 4).unwrap();
        assert_eq!(placement.cleared_lines.len(), 2);
        assert_eq!(game.score, 1 + 2 * POINTS_PER_LINE);
        assert_eq!(game.lines_cleared, 2);
    }

    #[test]
    fn tray_refills_when_it_becomes_empty() {
        let mut game = Game::new(6);
        force_tray(&mut game, [DOT, DOT, DOT]);
        for slot in 0..TRAY_SIZE {
            game.place(slot, slot, 3).unwrap();
        }
        assert!(game.tray.iter().all(|slot| slot.is_some()));
    }

    #[test]
    fn tray_does_not_refill_while_a_slot_is_still_filled() {
        let mut game = Game::new(7);
        force_tray(&mut game, [DOT, DOT, DOT]);
        game.place(0, 0, 0).unwrap();
        assert!(game.tray[0].is_none());
        assert!(game.tray[1].is_some());
    }

    #[test]
    fn game_over_is_detected_after_a_placement() {
        let mut game = Game::new(8);
        // Checkerboard: every hole is isolated, so only single-cell pieces fit.
        for y in 0..BOARD_SIZE {
            for x in 0..BOARD_SIZE {
                if (x + y) % 2 == 1 {
                    game.board.set(x, y, 0);
                }
            }
        }
        force_tray(&mut game, [DOT, BAR_4, BAR_4]);
        assert!(game.can_place(0, 0, 0));
        game.place(0, 0, 0).unwrap();
        assert_eq!(game.status, Status::GameOver);
        assert_eq!(game.place(1, 0, 1), Err(PlaceError::GameOver));
    }

    #[test]
    fn a_fresh_game_is_never_over() {
        let game = Game::new(9);
        assert_eq!(game.status, Status::Playing);
        assert!(game.has_any_move());
    }

    #[test]
    fn same_seed_produces_the_same_game() {
        for seed in 0..20 {
            assert_eq!(Game::new(seed), Game::new(seed));
        }
    }
}
