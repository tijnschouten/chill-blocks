use crate::piece::Shape;

pub const BOARD_SIZE: usize = 8;
pub const CELL_COUNT: usize = BOARD_SIZE * BOARD_SIZE;

/// Cell value is a palette index + 1; 0 means empty.
pub const EMPTY: u8 = 0;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Board {
    cells: [u8; CELL_COUNT],
}

impl Board {
    pub fn new() -> Self {
        Self {
            cells: [EMPTY; CELL_COUNT],
        }
    }

    pub fn cells(&self) -> &[u8; CELL_COUNT] {
        &self.cells
    }

    pub fn index(&self, x: usize, y: usize) -> usize {
        y * BOARD_SIZE + x
    }

    pub fn is_empty(&self, x: usize, y: usize) -> bool {
        self.cells[self.index(x, y)] == EMPTY
    }

    pub fn color_at(&self, x: usize, y: usize) -> Option<u8> {
        let v = self.cells[self.index(x, y)];
        if v == EMPTY {
            None
        } else {
            Some(v - 1)
        }
    }

    pub fn set(&mut self, x: usize, y: usize, color: u8) {
        self.cells[self.index(x, y)] = color + 1;
    }

    pub fn clear_cell(&mut self, x: usize, y: usize) {
        self.cells[self.index(x, y)] = EMPTY;
    }

    pub fn filled_count(&self) -> usize {
        self.cells.iter().filter(|&&v| v != EMPTY).count()
    }

    /// Whether `shape` fits entirely inside the board with its anchor at `(ax, ay)`.
    pub fn can_fit(&self, shape: &Shape, ax: usize, ay: usize) -> bool {
        if ax + shape.w as usize > BOARD_SIZE || ay + shape.h as usize > BOARD_SIZE {
            return false;
        }
        shape
            .cells()
            .all(|(dx, dy)| self.is_empty(ax + dx as usize, ay + dy as usize))
    }

    /// Rows and columns that are completely filled.
    pub fn full_lines(&self) -> Vec<crate::game::Line> {
        let mut lines = Vec::new();
        for y in 0..BOARD_SIZE {
            if (0..BOARD_SIZE).all(|x| !self.is_empty(x, y)) {
                lines.push(crate::game::Line {
                    orientation: crate::game::Orientation::Row,
                    index: y,
                });
            }
        }
        for x in 0..BOARD_SIZE {
            if (0..BOARD_SIZE).all(|y| !self.is_empty(x, y)) {
                lines.push(crate::game::Line {
                    orientation: crate::game::Orientation::Column,
                    index: x,
                });
            }
        }
        lines
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::piece::SHAPES;

    #[test]
    fn empty_board() {
        let b = Board::new();
        assert_eq!(b.filled_count(), 0);
        assert!(b.full_lines().is_empty());
        for s in SHAPES {
            assert!(b.can_fit(s, 0, 0));
        }
    }

    #[test]
    fn shape_does_not_fit_off_the_board() {
        let b = Board::new();
        let quad = SHAPES.iter().find(|s| s.w == 4 && s.h == 1).unwrap();
        assert!(b.can_fit(quad, 4, 0));
        assert!(!b.can_fit(quad, 5, 0));
    }

    #[test]
    fn shape_does_not_fit_on_an_occupied_cell() {
        let mut b = Board::new();
        b.set(3, 2, 0);
        let square = SHAPES.iter().find(|s| s.w == 2 && s.h == 2).unwrap();
        assert!(!b.can_fit(square, 3, 2));
        assert!(!b.can_fit(square, 2, 1));
        assert!(b.can_fit(square, 4, 2));
    }

    #[test]
    fn detects_full_row_and_column() {
        let mut b = Board::new();
        for i in 0..BOARD_SIZE {
            b.set(i, 3, 0);
        }
        for i in 0..BOARD_SIZE {
            b.set(6, i, 0);
        }
        let lines = b.full_lines();
        assert_eq!(lines.len(), 2);
        assert!(lines
            .iter()
            .any(|l| l.orientation == crate::game::Orientation::Row && l.index == 3));
        assert!(lines
            .iter()
            .any(|l| l.orientation == crate::game::Orientation::Column && l.index == 6));
    }

    #[test]
    fn a_gap_prevents_a_clear() {
        let mut b = Board::new();
        for i in 0..BOARD_SIZE {
            b.set(i, 1, 0);
        }
        b.clear_cell(4, 1);
        assert!(b.full_lines().is_empty());
    }
}
