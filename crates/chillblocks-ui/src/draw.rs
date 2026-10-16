use chillblocks_core::board::BOARD_SIZE;
use chillblocks_core::game::{Game, Status};
use chillblocks_core::piece::Shape;

use crate::color::Color;
use crate::geometry::{FPoint, Rect};
use crate::input::PointerKind;
use crate::layout::{Layout, SOUND_BUTTON};
use crate::motion::{progress, FADE_IN_MS, FADE_OUT_MS, LIFT_MS};
use crate::painter::{Align, Baseline, Painter};
use crate::theme::Theme;
use crate::view::{Drag, View, TOUCH_LIFT_CELLS};

fn inset(rect: &Rect, by: f32) -> Rect {
    Rect::new(
        rect.x + by,
        rect.y + by,
        rect.w - by * 2.0,
        rect.h - by * 2.0,
    )
}

fn draw_cells(
    painter: &mut dyn Painter,
    origin: FPoint,
    shape: &Shape,
    size: f32,
    color: Color,
    radius: f32,
    alpha: f32,
) {
    for (dx, dy) in shape.cells() {
        let cell = Rect::new(
            origin.x + dx as f32 * size,
            origin.y + dy as f32 * size,
            size,
            size,
        );
        painter.fill_round_rect(&inset(&cell, size * 0.08), radius, color.with_alpha(alpha));
    }
}

fn draw_hud(layout: &Layout, game: &Game, view: &View, theme: &Theme, painter: &mut dyn Painter) {
    let title_size = (layout.hud.h * 0.34).clamp(15.0, 22.0);
    painter.text(
        layout.hud.x,
        layout.hud.y + title_size,
        "ChillBlocks",
        &theme.style(
            title_size,
            theme.text,
            600,
            Align::Left,
            Baseline::Alphabetic,
        ),
    );

    let score_size = title_size * 1.15;
    painter.text(
        layout.hud.x + layout.hud.w,
        layout.hud.y + score_size,
        &game.score.to_string(),
        &theme.style(
            score_size,
            theme.text,
            600,
            Align::Right,
            Baseline::Alphabetic,
        ),
    );

    let sub_size = title_size * 0.72;
    painter.text(
        layout.hud.x,
        layout.hud.y + title_size + sub_size + 4.0,
        &format!("best {} · {} lines", view.best, game.lines_cleared),
        &theme.style(
            sub_size,
            theme.text_muted,
            400,
            Align::Left,
            Baseline::Alphabetic,
        ),
    );
}

fn draw_board(
    layout: &Layout,
    game: &Game,
    view: &View,
    theme: &Theme,
    painter: &mut dyn Painter,
    now: f64,
) {
    let frame = layout.cell * 0.14;
    painter.fill_round_rect(
        &Rect::new(
            layout.board.x - frame,
            layout.board.y - frame,
            layout.board.w + frame * 2.0,
            layout.board.h + frame * 2.0,
        ),
        layout.radius * 1.6,
        theme.panel,
    );

    for y in 0..BOARD_SIZE {
        for x in 0..BOARD_SIZE {
            let inner = inset(&layout.cell_rect(x, y), layout.cell * 0.08);
            painter.fill_round_rect(&inner, layout.radius, theme.board);

            if let Some(color) = game.board.color_at(x, y) {
                let alpha = progress(
                    view.appear[y * BOARD_SIZE + x],
                    now,
                    FADE_IN_MS,
                    view.motion_scale,
                );
                painter.fill_round_rect(
                    &inner,
                    layout.radius,
                    theme.palette[color as usize].with_alpha(alpha as f32),
                );
            }
        }
    }

    // Cells that have just cleared keep fading after the board has already removed them.
    for ghost in &view.ghosts {
        let inner = inset(&layout.cell_rect(ghost.x, ghost.y), layout.cell * 0.08);
        let alpha = 1.0 - progress(ghost.started, now, FADE_OUT_MS, view.motion_scale);
        painter.fill_round_rect(
            &inner,
            layout.radius,
            theme.palette[ghost.color as usize].with_alpha(alpha as f32),
        );
    }

    if view.selected.is_some() && view.cursor_set && game.status == Status::Playing {
        let rect = layout.cell_rect(view.cursor.0, view.cursor.1);
        painter.stroke_round_rect(&rect, layout.radius, 1.5, theme.accent);
    }
}

fn draw_drag(
    layout: &Layout,
    game: &Game,
    view: &View,
    theme: &Theme,
    painter: &mut dyn Painter,
    drag: &Drag,
    now: f64,
) {
    let Some(piece) = game.tray[drag.slot] else {
        return;
    };
    let shape = game.shape(piece);
    let color = theme.palette[piece.color as usize];

    // The piece is drawn at board size with the cell it was grabbed by under the pointer,
    // inside the board and outside it alike. Crossing the board edge only changes whether
    // it snaps to a cell, not how big it is or where it sits relative to the finger.
    let lift = if drag.kind == PointerKind::Touch {
        layout.cell * TOUCH_LIFT_CELLS
    } else {
        0.0
    };
    // Picking up grows the piece from its tray size to board size over the lift budget, so
    // the change is a short ease-out rather than a jump.
    let size = layout.slot_cell
        + (layout.cell - layout.slot_cell)
            * progress(drag.started, now, LIFT_MS, view.motion_scale) as f32;
    let origin = match drag.target {
        Some((x, y)) => FPoint {
            x: layout.board.x + x as f32 * layout.cell,
            y: layout.board.y + y as f32 * layout.cell,
        },
        None => FPoint {
            x: drag.pointer.x - drag.grab.0 as f32 * size,
            y: drag.pointer.y - lift - drag.grab.1 as f32 * size,
        },
    };
    // Cells that would land on an already occupied square keep the existing block visible:
    // a light fill with the outline on the whole cell, so the clash reads as a ring around
    // the block that is already there rather than a colour change on the piece.
    let mut held_blocked = false;
    for (dx, dy) in shape.cells() {
        let cell = Rect::new(
            origin.x + dx as f32 * size,
            origin.y + dy as f32 * size,
            size,
            size,
        );
        let blocked = drag.target.is_some_and(|(x, y)| {
            game.board
                .color_at(x + dx as usize, y + dy as usize)
                .is_some()
        });
        if blocked {
            if usize::from(dx) == drag.grab.0 && usize::from(dy) == drag.grab.1 {
                held_blocked = true;
            }
            let inner = inset(&cell, size * 0.08);
            painter.fill_round_rect(&inner, layout.radius, theme.text_muted.with_alpha(0.22));
            painter.stroke_round_rect(&cell, layout.radius, 2.0, theme.text.with_alpha(0.75));
        } else {
            painter.fill_round_rect(
                &inset(&cell, size * 0.08),
                layout.radius,
                color.with_alpha(0.7),
            );
        }
    }

    // The held cell is a little stronger, so the anchor you are moving is visible. When the
    // held cell is the clashing one, the conflict matters more than the anchor.
    if !held_blocked {
        let held = Rect::new(
            origin.x + drag.grab.0 as f32 * size,
            origin.y + drag.grab.1 as f32 * size,
            size,
            size,
        );
        painter.fill_round_rect(
            &inset(&held, size * 0.08),
            layout.radius,
            color.with_alpha(0.9),
        );
    }

    if let Some((x, y)) = drag.target {
        let bounds = Rect::new(
            layout.board.x + x as f32 * layout.cell,
            layout.board.y + y as f32 * layout.cell,
            shape.w as f32 * layout.cell,
            shape.h as f32 * layout.cell,
        );
        let fits = game.board.can_fit(shape, x, y);
        // A valid target gets an accent outline; an invalid one is a thicker muted one,
        // never alarm red.
        painter.stroke_round_rect(
            &bounds,
            layout.radius,
            if fits { 1.5 } else { 2.5 },
            if fits { theme.accent } else { theme.text_muted },
        );
    }
}

fn draw_tray(
    layout: &Layout,
    game: &Game,
    view: &View,
    theme: &Theme,
    painter: &mut dyn Painter,
    now: f64,
) {
    for (slot, rect) in layout.slots.iter().enumerate() {
        painter.fill_round_rect(rect, layout.radius * 1.6, theme.panel);

        let Some(piece) = game.tray[slot] else {
            continue;
        };
        if view.drag.as_ref().map(|d| d.slot) == Some(slot) {
            continue;
        }

        let shape = game.shape(piece);
        let lift = if view.selected == Some(slot) {
            (progress(0.0, now, LIFT_MS, view.motion_scale) * 4.0) as f32
        } else {
            0.0
        };
        let size = layout.slot_cell;
        let origin = FPoint {
            x: rect.x + (rect.w - shape.w as f32 * size) / 2.0,
            y: rect.y + (rect.h - shape.h as f32 * size) / 2.0 - lift,
        };
        draw_cells(
            painter,
            origin,
            shape,
            size,
            theme.palette[piece.color as usize],
            layout.radius,
            1.0,
        );

        if view.selected == Some(slot) {
            painter.stroke_round_rect(rect, layout.radius * 1.6, 1.5, theme.accent);
        }
    }
}

fn draw_footer(layout: &Layout, view: &View, theme: &Theme, painter: &mut dyn Painter) {
    // The button says what pressing it does, not what the current state is: a label reading
    // "Sound off" while sound is off reads as an action, and that is what people expect.
    let labels = [
        if view.sound_on {
            "Turn sound off"
        } else {
            "Turn sound on"
        },
        "New game",
    ];
    let size = (layout.footer.h * 0.36).clamp(13.0, 18.0);
    for (i, rect) in layout.buttons.iter().enumerate() {
        painter.fill_round_rect(rect, layout.radius * 1.4, theme.panel);
        if i == SOUND_BUTTON && view.sound_on {
            painter.stroke_round_rect(rect, layout.radius * 1.4, 1.5, theme.accent);
        }
        painter.text(
            rect.x + rect.w / 2.0,
            rect.y + rect.h / 2.0,
            labels[i],
            &theme.style(size, theme.text, 500, Align::Center, Baseline::Middle),
        );
    }
}

/// Rough glyph-width estimate, used only to shrink text so it stays inside its container.
/// Backends measure differently, but this keeps the scene readable on small screens.
fn fit_size(text: &str, size: f32, max_width: f32) -> f32 {
    let glyphs = text.chars().count() as f32;
    if glyphs == 0.0 {
        return size;
    }
    let needed = max_width / (glyphs * 0.55);
    size.min(needed).max(10.0)
}

/// Calm end-of-game card: a summary, no celebration.
fn draw_summary(
    layout: &Layout,
    game: &Game,
    view: &View,
    theme: &Theme,
    painter: &mut dyn Painter,
) {
    let card = layout.summary_card();
    painter.fill_round_rect(&card, layout.radius * 2.0, theme.panel.with_alpha(0.94));

    let title = "No more moves";
    let body = format!("score {} · best {}", game.score, view.best);

    let title_size = fit_size(title, (card.h * 0.26).clamp(14.0, 22.0), card.w - 16.0);
    let body_size = fit_size(&body, title_size * 0.82, card.w - 16.0);

    painter.text(
        card.x + card.w / 2.0,
        card.y + title_size + 10.0,
        title,
        &theme.style(
            title_size,
            theme.text,
            600,
            Align::Center,
            Baseline::Alphabetic,
        ),
    );

    painter.text(
        card.x + card.w / 2.0,
        card.y + title_size + body_size + 22.0,
        &body,
        &theme.style(
            body_size,
            theme.text_muted,
            400,
            Align::Center,
            Baseline::Alphabetic,
        ),
    );
}

pub fn draw(layout: &Layout, game: &Game, view: &View, painter: &mut dyn Painter, now: f64) {
    let theme = crate::theme::theme();
    painter.begin(layout.width, layout.height);
    painter.clear(theme.background);
    draw_hud(layout, game, view, &theme, painter);
    draw_board(layout, game, view, &theme, painter, now);
    draw_tray(layout, game, view, &theme, painter, now);
    draw_footer(layout, view, &theme, painter);
    // Last, so a piece in hand is never covered by the tray slot it came from.
    if let Some(drag) = &view.drag {
        draw_drag(layout, game, view, &theme, painter, drag, now);
    }
    if game.status == Status::GameOver {
        draw_summary(layout, game, view, &theme, painter);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::{compute, ZERO_INSETS};
    use crate::painter::TextStyle;
    use crate::theme::theme;
    use chillblocks_core::game::Game;

    struct Recording {
        fills: Vec<(Rect, Color)>,
    }

    impl Painter for Recording {
        fn begin(&mut self, _width: f32, _height: f32) {}
        fn clear(&mut self, _background: Color) {}
        fn fill_round_rect(&mut self, rect: &Rect, _radius: f32, color: Color) {
            self.fills.push((*rect, color));
        }
        fn stroke_round_rect(&mut self, _rect: &Rect, _radius: f32, _width: f32, _color: Color) {}
        fn text(&mut self, _x: f32, _y: f32, _content: &str, _style: &TextStyle) {}
    }

    #[test]
    fn a_piece_in_hand_is_drawn_on_top_of_the_slot_it_came_from() {
        let layout = compute(390.0, 844.0, ZERO_INSETS);
        let game = Game::new(7);
        let piece = game.tray[0].unwrap();
        let slot = layout.slots[0];
        let grab = layout.slot_cell_at(0, slot.center(), game.shape(piece));

        let mut view = View::new();
        view.drag = Some(Drag {
            slot: 0,
            pointer: slot.center(),
            origin: slot.center(),
            started: 0.0,
            moved: false,
            kind: PointerKind::Mouse,
            grab,
            target: None,
        });

        let mut recording = Recording { fills: Vec::new() };
        draw(&layout, &game, &view, &mut recording, 0.0);

        let panel = recording
            .fills
            .iter()
            .position(|(rect, _)| {
                (rect.x - slot.x).abs() < 0.001 && (rect.y - slot.y).abs() < 0.001
            })
            .expect("the tray slot panel is drawn");
        let piece_color = theme().palette[piece.color as usize];
        let held = recording
            .fills
            .iter()
            .position(|(_, color)| {
                (color.r, color.g, color.b) == (piece_color.r, piece_color.g, piece_color.b)
            })
            .expect("the dragged piece is drawn");

        assert!(
            held > panel,
            "the piece in hand must be drawn after the slot panel, not under it"
        );
    }

    #[test]
    fn a_blocked_cell_is_drawn_muted() {
        let layout = compute(390.0, 844.0, ZERO_INSETS);
        let mut game = Game::new(7);
        let piece = game.tray[0].unwrap();
        let shape = game.shape(piece);
        let slot = layout.slots[0];
        let grab = layout.slot_cell_at(0, slot.center(), shape);

        // Occupy a square the piece actually covers when its anchor is (0, 0).
        let (dx, dy) = game.shape(piece).cells().next().unwrap();
        game.board.set(dx as usize, dy as usize, piece.color);

        let mut view = View::new();
        view.drag = Some(Drag {
            slot: 0,
            pointer: slot.center(),
            origin: slot.center(),
            started: 0.0,
            moved: false,
            kind: PointerKind::Mouse,
            grab,
            target: Some((0, 0)),
        });

        let mut recording = Recording { fills: Vec::new() };
        draw(&layout, &game, &view, &mut recording, 0.0);

        let muted = theme().text_muted;
        assert!(
            recording
                .fills
                .iter()
                .any(|(_, color)| (color.r, color.g, color.b) == (muted.r, muted.g, muted.b)),
            "a cell that clashes with an occupied square is drawn muted"
        );
        assert!(
            recording
                .fills
                .iter()
                .any(
                    |(_, color)| (color.r, color.g, color.b) == (muted.r, muted.g, muted.b)
                        && color.a <= 0.3
                ),
            "the clashing cell has to stay translucent so the block underneath shows through"
        );
    }

    #[test]
    fn text_is_never_larger_than_requested() {
        assert_eq!(fit_size("short", 20.0, 1000.0), 20.0);
    }

    #[test]
    fn long_text_is_shrunk_to_fit_its_container() {
        let text = "score 170 · best 192";
        let size = fit_size(text, 20.0, 120.0);
        assert!(size < 20.0);
        assert!(size >= 10.0);
        // The estimated width must fit the container.
        assert!(size * text.chars().count() as f32 * 0.55 <= 120.0 + 0.001);
    }

    #[test]
    fn empty_text_does_not_divide_by_zero() {
        assert_eq!(fit_size("", 20.0, 100.0), 20.0);
    }
}
