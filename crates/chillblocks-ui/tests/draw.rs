use chillblocks_core::game::{Game, Status};
use chillblocks_ui::color::Color;
use chillblocks_ui::draw::draw;
use chillblocks_ui::geometry::Rect;
use chillblocks_ui::layout::{compute, ZERO_INSETS};
use chillblocks_ui::painter::{Painter, TextStyle};
use chillblocks_ui::view::View;

struct Recording {
    ops: Vec<String>,
}

impl Recording {
    fn new() -> Self {
        Self { ops: Vec::new() }
    }
}

impl Painter for Recording {
    fn begin(&mut self, width: f32, height: f32) {
        self.ops.push(format!("begin {width}x{height}"));
    }
    fn clear(&mut self, background: Color) {
        self.ops.push(format!("clear {}", background.css()));
    }
    fn fill_round_rect(&mut self, rect: &Rect, radius: f32, color: Color) {
        self.ops.push(format!(
            "fill {:.1},{:.1},{:.1},{:.1} r{:.2} {}",
            rect.x,
            rect.y,
            rect.w,
            rect.h,
            radius,
            color.css()
        ));
    }
    fn stroke_round_rect(&mut self, rect: &Rect, radius: f32, width: f32, color: Color) {
        self.ops.push(format!(
            "stroke {:.1},{:.1},{:.1},{:.1} r{:.2} w{:.2} {}",
            rect.x,
            rect.y,
            rect.w,
            rect.h,
            radius,
            width,
            color.css()
        ));
    }
    fn text(&mut self, x: f32, y: f32, content: &str, style: &TextStyle) {
        self.ops.push(format!(
            "text {:.1},{:.1} {} {}",
            x,
            y,
            content,
            style.color.css()
        ));
    }
}

fn alpha_of(op: &str) -> Option<f32> {
    let rgba = op.rfind("rgba(")?;
    let rest = &op[rgba + 5..];
    rest.split(')')
        .next()?
        .split(',')
        .next_back()?
        .trim()
        .parse()
        .ok()
}

fn render(game: &Game, view: &View, now: f64) -> Vec<String> {
    let layout = compute(390.0, 844.0, ZERO_INSETS);
    let mut recording = Recording::new();
    draw(&layout, game, view, &mut recording, now);
    recording.ops
}

#[test]
fn the_same_state_renders_identically() {
    let game = Game::new(7);
    let view = View::new();
    assert_eq!(render(&game, &view, 1000.0), render(&game, &view, 1000.0));
}

#[test]
fn alphas_never_leave_the_valid_range() {
    let mut game = Game::new(11);
    let view = View::new();
    game.place(0, 0, 0).unwrap();

    for now in [0.0, 50.0, 180.0, 250.0, 1000.0] {
        let ops = render(&game, &view, now);
        assert!(!ops.is_empty());
        for op in &ops {
            if let Some(alpha) = alpha_of(op) {
                assert!((0.0..=1.0).contains(&alpha), "alpha out of range in: {op}");
            }
        }
    }
}

#[test]
fn a_finished_animation_matches_reduced_motion() {
    let mut game = Game::new(13);
    game.place(0, 2, 3).unwrap();

    let mut settled = View::new();
    settled.appear[3 * 8 + 2] = 0.0;

    let mut calm = View::new();
    calm.motion_scale = 0.0;
    calm.appear[3 * 8 + 2] = 0.0;

    // Once the animation is complete both must render the same scene.
    assert_eq!(
        render(&game, &settled, 10_000.0),
        render(&game, &calm, 10_000.0)
    );
}

#[test]
fn a_clear_produces_fading_ghosts_that_disappear() {
    let mut game = Game::new(14);
    let mut view = View::new();
    for x in 0..8 {
        game.board.set(x, 0, 0);
    }
    game.board.clear_cell(7, 0);
    let placement = chillblocks_core::game::Placement {
        slot: 0,
        color: 0,
        cells: vec![chillblocks_core::game::CellPos { x: 7, y: 0 }],
        cleared_lines: vec![chillblocks_core::game::Line {
            orientation: chillblocks_core::game::Orientation::Row,
            index: 0,
        }],
        cleared_cells: (0..8)
            .map(|x| chillblocks_core::game::CellPos { x, y: 0 })
            .collect(),
        points: 11,
    };
    for x in 0..8 {
        game.board.clear_cell(x, 0);
    }
    view.record_placement(&placement, 0.0);

    assert_eq!(view.ghosts.len(), 8);
    let during = render(&game, &view, 100.0);
    view.prune(300.0);
    assert!(view.ghosts.is_empty());
    let after = render(&game, &view, 300.0);

    assert!(
        during.len() > after.len(),
        "ghosts add draws while they fade"
    );
    assert_eq!(view.ghosts.len(), 0);
}

#[test]
fn the_game_over_card_is_drawn_only_when_the_game_is_over() {
    let game = Game::new(17);
    let view = View::new();
    assert!(!render(&game, &view, 0.0)
        .iter()
        .any(|op| op.contains("No more moves")));

    let mut over = Game::new(17);
    over.status = Status::GameOver;
    assert!(render(&over, &view, 0.0)
        .iter()
        .any(|op| op.contains("No more moves")));
}

#[test]
fn the_scene_uses_only_the_allowed_primitives() {
    let game = Game::new(23);
    let view = View::new();
    for op in render(&game, &view, 0.0) {
        let kind = op.split(' ').next().unwrap();
        assert!(
            matches!(kind, "begin" | "clear" | "fill" | "stroke" | "text"),
            "unexpected draw primitive: {op}"
        );
    }
}

#[test]
fn nothing_in_the_scene_is_animated_past_the_motion_budget() {
    // The only easing is a bounded ease-out, so a frame past the budget must look
    // exactly like the final frame.
    let mut game = Game::new(29);
    let mut view = View::new();
    let placement = game.place(0, 1, 1).unwrap();
    view.record_placement(&placement, 0.0);

    assert_eq!(render(&game, &view, 301.0), render(&game, &view, 10_000.0));
}
