/// A piece shape is a fixed polyomino in a bounding box, stored row-major.
///
/// Shapes are never rotated at runtime: every orientation is pre-enumerated as its own
/// entry. That keeps placement trivial and keeps the game readable for a child.
pub const MAX_SHAPE_CELLS: usize = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shape {
    pub w: u8,
    pub h: u8,
    /// Row-major occupancy, `mask[y * w + x]`. Only the first `w * h` entries are used.
    pub mask: [u8; MAX_SHAPE_CELLS],
}

const fn shape(w: u8, h: u8, mask: [u8; MAX_SHAPE_CELLS]) -> Shape {
    Shape { w, h, mask }
}

/// The full set. Uniform draw over this table — no weighting.
pub const SHAPES: &[Shape] = &[
    // Bars.
    shape(1, 1, [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
    shape(2, 1, [1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
    shape(1, 2, [1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
    shape(3, 1, [1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
    shape(1, 3, [1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
    shape(4, 1, [1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
    shape(1, 4, [1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
    // Square.
    shape(2, 2, [1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
    // Corner trominoes.
    shape(2, 2, [1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
    shape(2, 2, [1, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
    shape(2, 2, [1, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
    shape(2, 2, [0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
    // L tetrominoes.
    shape(2, 3, [1, 0, 1, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
    shape(3, 2, [0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
    shape(2, 3, [1, 1, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
    shape(3, 2, [1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
    // T tetrominoes.
    shape(3, 2, [0, 1, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
    shape(2, 3, [0, 1, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
    shape(3, 2, [1, 1, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
    shape(2, 3, [1, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
    // S tetrominoes.
    shape(3, 2, [0, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
    shape(2, 4, [1, 0, 1, 0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0]),
    // Z tetrominoes.
    shape(3, 2, [1, 1, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
    shape(2, 3, [0, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
    // Plus.
    shape(3, 3, [0, 1, 0, 1, 1, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0]),
];

pub const SHAPE_COUNT: usize = SHAPES.len();

impl Shape {
    /// Cell offsets relative to the placement anchor (top-left of the bounding box).
    pub fn cells(&self) -> impl Iterator<Item = (u8, u8)> + '_ {
        (0..self.h as usize).flat_map(move |y| {
            (0..self.w as usize)
                .filter(move |&x| self.mask[y * self.w as usize + x] == 1)
                .map(move |x| (x as u8, y as u8))
        })
    }

    pub fn cell_count(&self) -> usize {
        self.mask[..self.w as usize * self.h as usize]
            .iter()
            .filter(|&&v| v == 1)
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_shape_has_at_least_one_cell() {
        for s in SHAPES {
            assert!(s.cell_count() >= 1, "empty shape in table");
            assert!(s.cell_count() <= MAX_SHAPE_CELLS);
        }
    }

    #[test]
    fn masks_only_use_the_bounding_box() {
        for s in SHAPES {
            let used = s.w as usize * s.h as usize;
            for &v in &s.mask[used..] {
                assert_eq!(v, 0, "mask set outside its bounding box");
            }
        }
    }

    /// The anchor is the top-left of the bounding box, so a shape with an empty edge row or
    /// column would place and highlight offset from the cells the player can see.
    #[test]
    fn every_shape_is_tightly_bounded() {
        for s in SHAPES {
            let cells = s.cells().collect::<Vec<_>>();
            let min_x = cells.iter().map(|(x, _)| *x).min().unwrap();
            let max_x = cells.iter().map(|(x, _)| *x).max().unwrap();
            let min_y = cells.iter().map(|(_, y)| *y).min().unwrap();
            let max_y = cells.iter().map(|(_, y)| *y).max().unwrap();
            assert_eq!(min_x, 0, "shape {s:?} has an empty left column");
            assert_eq!(min_y, 0, "shape {s:?} has an empty top row");
            assert_eq!(max_x + 1, s.w, "shape {s:?} has an empty right column");
            assert_eq!(max_y + 1, s.h, "shape {s:?} has an empty bottom row");
        }
    }

    #[test]
    fn cell_count_matches_the_iterator() {
        for s in SHAPES {
            assert_eq!(s.cells().count(), s.cell_count());
        }
    }

    #[test]
    fn shapes_are_unique() {
        let mut seen: Vec<Shape> = Vec::new();
        for s in SHAPES {
            if seen.contains(s) {
                panic!("duplicate shape: {:?}", s);
            }
            seen.push(*s);
        }
    }

    #[test]
    fn table_is_the_expected_size() {
        assert_eq!(SHAPE_COUNT, 25);
    }
}
