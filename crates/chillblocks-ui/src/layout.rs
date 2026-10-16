use chillblocks_core::board::BOARD_SIZE;
use chillblocks_core::piece::Shape;

use crate::geometry::{FPoint, Rect};

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Insets {
    pub top: f32,
    pub bottom: f32,
    pub left: f32,
    pub right: f32,
}

pub const ZERO_INSETS: Insets = Insets {
    top: 0.0,
    bottom: 0.0,
    left: 0.0,
    right: 0.0,
};

pub const SLOT_COUNT: usize = 3;
pub const SOUND_BUTTON: usize = 0;
pub const NEW_GAME_BUTTON: usize = 1;
pub const BUTTON_COUNT: usize = 2;

/// Minimum touch target. Buttons are sized to at least this whenever the screen allows.
pub const MIN_TOUCH: f32 = 44.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Layout {
    pub width: f32,
    pub height: f32,
    pub insets: Insets,
    pub hud: Rect,
    pub board: Rect,
    pub tray: Rect,
    pub slots: [Rect; SLOT_COUNT],
    pub footer: Rect,
    pub buttons: [Rect; BUTTON_COUNT],
    pub cell: f32,
    pub slot_cell: f32,
    pub radius: f32,
}

impl Layout {
    pub fn cell_rect(&self, x: usize, y: usize) -> Rect {
        Rect::new(
            self.board.x + x as f32 * self.cell,
            self.board.y + y as f32 * self.cell,
            self.cell,
            self.cell,
        )
    }

    pub fn slot_at(&self, p: FPoint) -> Option<usize> {
        self.slots.iter().position(|slot| slot.contains(p))
    }

    pub fn cell_at(&self, p: FPoint) -> Option<(usize, usize)> {
        if !self.board.contains(p) {
            return None;
        }
        let x = ((p.x - self.board.x) / self.cell).floor();
        let y = ((p.y - self.board.y) / self.cell).floor();
        if x < 0.0 || y < 0.0 || x >= 8.0 || y >= 8.0 {
            return None;
        }
        Some((x as usize, y as usize))
    }

    pub fn button_at(&self, p: FPoint) -> Option<usize> {
        self.buttons.iter().position(|b| b.contains(p))
    }

    /// Which cell of the shape a press inside `slot` lands on. A press on an empty part of
    /// the bounding box falls back to the nearest filled cell, so the piece never hangs
    /// from a point outside itself.
    pub fn slot_cell_at(&self, slot: usize, p: FPoint, shape: &Shape) -> (usize, usize) {
        let slot_rect = self.slots[slot];
        let gx = ((p.x - slot_rect.x) / self.slot_cell)
            .floor()
            .clamp(0.0, shape.w as f32 - 1.0);
        let gy = ((p.y - slot_rect.y) / self.slot_cell)
            .floor()
            .clamp(0.0, shape.h as f32 - 1.0);
        let (gx, gy) = (gx as usize, gy as usize);
        if shape.mask[gy * shape.w as usize + gx] == 1 {
            return (gx, gy);
        }
        shape
            .cells()
            .map(|(dx, dy)| (dx as usize, dy as usize))
            .min_by_key(|&(dx, dy)| {
                let dx = dx as f32 - gx as f32;
                let dy = dy as f32 - gy as f32;
                (dx * dx + dy * dy) as i64
            })
            .unwrap_or((gx, gy))
    }

    /// Where a dragged piece lands, accounting for the touch lift and for which cell of
    /// the shape the finger is holding. Returns None when the grab puts the shape outside
    /// the board, which is the same as dragging off the board.
    pub fn drag_anchor(
        &self,
        p: FPoint,
        lift: f32,
        grab: (usize, usize),
        shape: &Shape,
    ) -> Option<(usize, usize)> {
        let cell = self.cell_at(FPoint {
            x: p.x,
            y: p.y - lift,
        })?;
        if cell.0 < grab.0 || cell.1 < grab.1 {
            return None;
        }
        let anchor = (cell.0 - grab.0, cell.1 - grab.1);
        if anchor.0 + shape.w as usize > BOARD_SIZE || anchor.1 + shape.h as usize > BOARD_SIZE {
            return None;
        }
        Some(anchor)
    }

    /// The end-of-game card, kept inside the surface on narrow screens.
    pub fn summary_card(&self) -> Rect {
        let x = (self.board.x - self.cell * 0.5).max(self.insets.left);
        let width = (self.board.w + self.cell).min(self.width - self.insets.left - x);
        Rect::new(
            x,
            self.board.y + self.board.h * 0.32,
            width,
            self.board.h * 0.36,
        )
    }
}

pub fn compute(width: f32, height: f32, insets: Insets) -> Layout {
    let pad = (width.min(height) * 0.045).clamp(14.0, 28.0);
    let hud_h = (height * 0.08).clamp(44.0, 72.0);
    let footer_h = (height * 0.06).clamp(40.0, 56.0);
    let gap = pad * 0.9;

    let content_x = insets.left + pad;
    let content_y = insets.top + pad;
    let content_w = (width - insets.left - insets.right - pad * 2.0).max(0.0);
    let usable_h =
        (height - insets.top - insets.bottom - pad * 2.0 - hud_h - footer_h - gap * 3.0).max(0.0);

    let cell_by_height = usable_h / (8.0 + 2.6);
    let cell_by_width = content_w / 8.0;
    let cell = cell_by_height.min(cell_by_width).clamp(14.0, 64.0);

    // A slot is four slot cells wide, but the row of slots must still fit inside the
    // content area on a narrow phone.
    let max_slot_w: f32 = (content_w - gap * (SLOT_COUNT as f32 - 1.0)) / SLOT_COUNT as f32;
    let slot_w = (cell * 0.65 * 4.0).min(max_slot_w);
    let slot_cell = slot_w / 4.0;
    let slot_h = slot_w;

    let board_w = cell * 8.0;
    let board_x = content_x + (content_w - board_w) / 2.0;
    let board_y = content_y + hud_h + gap;

    let tray_y = board_y + board_w + gap;
    let total_slots = slot_w * SLOT_COUNT as f32 + gap * (SLOT_COUNT as f32 - 1.0);
    let slots_start = content_x + (content_w - total_slots) / 2.0;
    let slots = [0, 1, 2].map(|i| {
        Rect::new(
            slots_start + i as f32 * (slot_w + gap),
            tray_y,
            slot_w,
            slot_h,
        )
    });

    let button_h = footer_h.max(MIN_TOUCH);
    let button_w = (content_w / BUTTON_COUNT as f32).clamp(MIN_TOUCH, 220.0);
    let buttons = [0, 1].map(|i| {
        Rect::new(
            content_x + i as f32 * button_w,
            content_y + hud_h + gap + board_w + gap + slot_h + gap,
            button_w,
            button_h,
        )
    });

    Layout {
        width,
        height,
        insets,
        hud: Rect::new(content_x, content_y, content_w, hud_h),
        board: Rect::new(board_x, board_y, board_w, board_w),
        tray: Rect::new(content_x, tray_y, content_w, slot_h),
        slots,
        footer: Rect::new(content_x, buttons[0].y, content_w, button_h),
        buttons,
        cell,
        slot_cell,
        radius: (cell * 0.18).clamp(3.0, 12.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chillblocks_core::piece::SHAPES;

    #[test]
    fn everything_stays_inside_the_surface() {
        for (w, h) in [
            (390.0, 844.0),
            (768.0, 1024.0),
            (1440.0, 900.0),
            (320.0, 480.0),
        ] {
            let l = compute(w, h, ZERO_INSETS);
            for rect in [l.hud, l.board, l.tray, l.footer]
                .into_iter()
                .chain(l.slots)
                .chain(l.buttons)
            {
                assert!(
                    rect.x >= 0.0 && rect.y >= 0.0,
                    "{w}x{h}: {rect:?} starts off-surface"
                );
                assert!(
                    rect.x + rect.w <= w + 0.001,
                    "{w}x{h}: {rect:?} overflows width"
                );
                assert!(
                    rect.y + rect.h <= h + 0.001,
                    "{w}x{h}: {rect:?} overflows height"
                );
            }
        }
    }

    #[test]
    fn safe_area_insets_are_respected() {
        let insets = Insets {
            top: 47.0,
            bottom: 34.0,
            left: 0.0,
            right: 0.0,
        };
        let l = compute(390.0, 844.0, insets);
        assert!(l.hud.y >= insets.top);
        assert!(l.footer.y + l.footer.h <= 844.0 - insets.bottom + 0.001);
    }

    #[test]
    fn the_summary_card_stays_inside_the_surface() {
        for (w, h) in [
            (390.0f32, 844.0f32),
            (320.0f32, 480.0f32),
            (1440.0f32, 900.0f32),
        ] {
            let l = compute(w, h, ZERO_INSETS);
            let card = l.summary_card();
            assert!(card.x >= 0.0, "{w}x{h}: card starts off-surface: {card:?}");
            assert!(
                card.x + card.w <= w + 0.001,
                "{w}x{h}: card overflows: {card:?}"
            );
            assert!(
                card.y + card.h <= h + 0.001,
                "{w}x{h}: card overflows: {card:?}"
            );
        }
    }

    #[test]
    fn slots_stay_inside_the_content_padding() {
        for (w, h) in [
            (390.0f32, 844.0f32),
            (320.0f32, 480.0f32),
            (1440.0f32, 900.0f32),
        ] {
            let pad: f32 = (w.min(h) * 0.045).clamp(14.0, 28.0);
            let l = compute(w, h, ZERO_INSETS);
            for slot in l.slots {
                assert!(
                    slot.x >= pad - 0.001,
                    "{w}x{h}: slot {slot:?} breaks the padding"
                );
                assert!(
                    slot.x + slot.w <= w - pad + 0.001,
                    "{w}x{h}: slot {slot:?} overflows"
                );
            }
        }
    }

    #[test]
    fn cell_at_rounds_to_the_right_cell() {
        let l = compute(390.0, 844.0, ZERO_INSETS);
        assert_eq!(
            l.cell_at(FPoint {
                x: l.board.x,
                y: l.board.y
            }),
            Some((0, 0))
        );
        assert_eq!(
            l.cell_at(FPoint {
                x: l.board.x + l.cell * 7.5,
                y: l.board.y + l.cell * 3.2
            }),
            Some((7, 3))
        );
        assert_eq!(
            l.cell_at(FPoint {
                x: l.board.x - 1.0,
                y: l.board.y
            }),
            None
        );
        // The far corner belongs to no cell: it is the edge of cell (7, 7).
        assert_eq!(
            l.cell_at(FPoint {
                x: l.board.x + l.board.w - 0.1,
                y: l.board.y + l.board.h - 0.1
            }),
            Some((7, 7))
        );
        assert_eq!(
            l.cell_at(FPoint {
                x: l.board.x + l.board.w,
                y: l.board.y + l.board.h
            }),
            None
        );
    }

    #[test]
    fn touch_lift_moves_the_target_up_by_the_expected_number_of_cells() {
        let l = compute(390.0, 844.0, ZERO_INSETS);
        let lift = l.cell * 1.4;
        let p = FPoint {
            x: l.board.x + l.cell * 3.5,
            y: l.board.y + l.cell * 5.5,
        };
        assert_eq!(l.cell_at(p), Some((3, 5)));
        assert_eq!(l.drag_anchor(p, lift, (0, 0), &SHAPES[0]), Some((3, 4)));
    }

    #[test]
    fn the_grab_cell_shifts_the_anchor_back_by_the_same_amount() {
        let l = compute(390.0, 844.0, ZERO_INSETS);
        // A 4x1 bar held by its right end lands four cells to the left of the finger.
        let bar = SHAPES.iter().find(|s| s.w == 4 && s.h == 1).unwrap();
        let p = FPoint {
            x: l.board.x + l.cell * 5.5,
            y: l.board.y + l.cell * 3.5,
        };
        assert_eq!(l.drag_anchor(p, 0.0, (3, 0), bar), Some((2, 3)));

        // A grab that would push the piece off the top edge is the same as dragging off
        // the board, so the piece is drawn un-snapped rather than half outside.
        assert_eq!(l.drag_anchor(p, 0.0, (3, 0), bar).map(|a| a.1), Some(3));
        assert_eq!(
            l.drag_anchor(
                FPoint {
                    x: l.board.x + l.cell * 1.5,
                    y: l.board.y + l.cell * 3.5,
                },
                0.0,
                (3, 0),
                bar
            ),
            None
        );
    }

    #[test]
    fn a_press_on_empty_space_grabs_the_nearest_cell_of_the_piece() {
        let l = compute(390.0, 844.0, ZERO_INSETS);
        // An L whose filled cells are (0,0) and (0,1); pressing the empty top-right corner
        // should grab (0,1) or (0,0), never the empty cell.
        let corner = SHAPES
            .iter()
            .find(|s| s.w == 2 && s.h == 2 && s.mask[0] == 1 && s.mask[1] == 0)
            .unwrap();
        let grab = l.slot_cell_at(
            0,
            FPoint {
                x: l.slots[0].x + l.slot_cell * 1.5,
                y: l.slots[0].y + l.slot_cell * 0.5,
            },
            corner,
        );
        assert_eq!(corner.mask[grab.1 * corner.w as usize + grab.0], 1);
    }

    #[test]
    fn buttons_meet_the_minimum_touch_target_when_there_is_room() {
        let l = compute(390.0, 844.0, ZERO_INSETS);
        for b in l.buttons {
            assert!(b.h >= MIN_TOUCH - 0.001);
            assert!(b.w >= MIN_TOUCH - 0.001);
        }
    }

    #[test]
    fn slots_are_ordered_left_to_right_and_do_not_overlap() {
        let l = compute(390.0, 844.0, ZERO_INSETS);
        for i in 1..SLOT_COUNT {
            assert!(l.slots[i].x > l.slots[i - 1].x + l.slots[i - 1].w - 0.001);
        }
    }

    #[test]
    fn hit_testing_finds_the_slot_and_button_under_a_point() {
        let l = compute(390.0, 844.0, ZERO_INSETS);
        assert_eq!(l.slot_at(l.slots[1].center()), Some(1));
        assert_eq!(l.slot_at(FPoint { x: 0.0, y: 0.0 }), None);
        assert_eq!(
            l.button_at(l.buttons[NEW_GAME_BUTTON].center()),
            Some(NEW_GAME_BUTTON)
        );
        assert_eq!(l.button_at(FPoint { x: 0.0, y: 0.0 }), None);
    }
}
