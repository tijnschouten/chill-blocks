use std::collections::HashMap;

use chillblocks_app::App;
use chillblocks_core::board::BOARD_SIZE;
use chillblocks_core::game::{Game, Status};
use chillblocks_core::snapshot;
use chillblocks_platform::traits::{Clock, NullAudio, Store};
use chillblocks_ui::color::Color;
use chillblocks_ui::geometry::{FPoint, Rect};
use chillblocks_ui::input::{Command, PointerEvent, PointerKind, PointerPhase};
use chillblocks_ui::layout::ZERO_INSETS;
use chillblocks_ui::painter::{Painter, TextStyle};

struct TestStore(HashMap<String, String>);

impl Store for TestStore {
    fn get(&self, key: &str) -> Option<String> {
        self.0.get(key).cloned()
    }
    fn set(&mut self, key: &str, value: &str) {
        self.0.insert(key.to_string(), value.to_string());
    }
}

struct TestClock(f64);

impl Clock for TestClock {
    fn now_ms(&self) -> f64 {
        self.0
    }
}

struct TestPainter;

impl Painter for TestPainter {
    fn begin(&mut self, _w: f32, _h: f32) {}
    fn clear(&mut self, _c: Color) {}
    fn fill_round_rect(&mut self, _r: &Rect, _radius: f32, _c: Color) {}
    fn stroke_round_rect(&mut self, _r: &Rect, _radius: f32, _w: f32, _c: Color) {}
    fn line(&mut self, _from: FPoint, _to: FPoint, _w: f32, _c: Color) {}
    fn text(&mut self, _x: f32, _y: f32, _s: &str, _style: &TextStyle) {}
}

fn app() -> App {
    let mut app = App::new(
        Box::new(TestPainter),
        Box::new(TestStore(HashMap::new())),
        Box::new(TestClock(1000.0)),
        Box::new(NullAudio),
    );
    app.resize(390.0, 844.0, 1.0, ZERO_INSETS);
    app
}

fn event(phase: PointerPhase, point: FPoint) -> PointerEvent {
    PointerEvent {
        id: 1,
        phase,
        kind: PointerKind::Mouse,
        point,
        time_ms: 1000.0,
    }
}

fn first_fit(app: &App, slot: usize) -> (usize, usize) {
    for y in 0..BOARD_SIZE {
        for x in 0..BOARD_SIZE {
            if app.game.can_place(slot, x, y) {
                return (x, y);
            }
        }
    }
    panic!("no fit for slot {slot}");
}

fn cell_center(app: &App, x: usize, y: usize) -> FPoint {
    let rect = app.layout.cell_rect(x, y);
    FPoint {
        x: rect.x + rect.w / 2.0,
        y: rect.y + rect.h / 2.0,
    }
}

#[test]
fn a_tap_on_a_tray_slot_selects_that_piece() {
    let mut app = app();
    let slot_center = app.layout.slots[0].center();
    app.handle_pointer(&event(PointerPhase::Down, slot_center));
    app.handle_pointer(&event(PointerPhase::Up, slot_center));
    assert_eq!(app.view.selected, Some(0));
    // A tap does not place anything.
    assert_eq!(app.game.pieces_placed, 0);
}

#[test]
fn a_drag_from_the_tray_places_the_piece() {
    let mut app = app();
    let slot_center = app.layout.slots[0].center();
    let piece = app.game.tray[0].unwrap();
    let shape = app.game.shape(piece);
    let grab = app.layout.slot_cell_at(0, slot_center, shape);
    let (x, y) = first_fit(&app, 0);

    // The piece follows the finger by the cell it was grabbed by, so the pointer sits one
    // grab-cell away from where the piece lands.
    let pointer = cell_center(&app, x + grab.0, y + grab.1);
    app.handle_pointer(&event(PointerPhase::Down, slot_center));
    app.handle_pointer(&event(PointerPhase::Move, pointer));
    app.handle_pointer(&event(PointerPhase::Up, pointer));

    assert_eq!(app.game.pieces_placed, 1);
    assert_eq!(app.game.tray[0], None);
    assert_eq!(app.view.selected, None);
    assert_eq!(app.view.drag, None);
}

#[test]
fn tap_to_place_works_after_a_selection() {
    let mut app = app();
    let slot_center = app.layout.slots[0].center();
    app.handle_pointer(&event(PointerPhase::Down, slot_center));
    app.handle_pointer(&event(PointerPhase::Up, slot_center));

    let (x, y) = first_fit(&app, 0);
    app.handle_pointer(&event(PointerPhase::Down, cell_center(&app, x, y)));

    assert_eq!(app.game.pieces_placed, 1);
    assert_eq!(app.view.selected, None);
}

#[test]
fn a_placement_that_does_not_fit_is_rejected_without_changing_the_game() {
    let mut app = app();
    let before = app.game.clone();
    let slot_center = app.layout.slots[0].center();
    let piece = app.game.tray[0].unwrap();
    let grab = app
        .layout
        .slot_cell_at(0, slot_center, app.game.shape(piece));
    let cells: Vec<(usize, usize)> = app
        .game
        .shape(piece)
        .cells()
        .map(|(dx, dy)| (dx as usize, dy as usize))
        .collect();

    // Holding the piece by its grab cell puts the anchor on (0, 0).
    let pointer = cell_center(&app, grab.0, grab.1);
    app.handle_pointer(&event(PointerPhase::Down, slot_center));
    app.handle_pointer(&event(PointerPhase::Move, pointer));

    // Occupy the target so the placement fails.
    for (x, y) in cells {
        app.game.board.set(x, y, 0);
    }

    let score_before = app.game.score;
    app.handle_pointer(&event(PointerPhase::Up, pointer));
    assert_eq!(app.game.score, score_before);
    assert_eq!(app.game.pieces_placed, before.pieces_placed);
}

#[test]
fn a_selection_does_not_highlight_a_cell_until_the_player_points_at_one() {
    let mut app = app();
    let slot_center = app.layout.slots[0].center();
    app.handle_pointer(&event(PointerPhase::Down, slot_center));
    app.handle_pointer(&event(PointerPhase::Up, slot_center));

    assert_eq!(app.view.selected, Some(0));
    assert!(!app.view.cursor_set, "a selection alone has no target");

    app.handle_pointer(&event(PointerPhase::Move, cell_center(&app, 3, 4)));
    assert!(app.view.cursor_set);
    assert_eq!(app.view.cursor, (3, 4));
}

#[test]
fn selecting_another_piece_does_not_carry_over_the_previous_pieces_cursor() {
    let mut app = app();
    let slot_center = app.layout.slots[0].center();
    app.handle_pointer(&event(PointerPhase::Down, slot_center));
    app.handle_pointer(&event(PointerPhase::Up, slot_center));
    app.handle_pointer(&event(PointerPhase::Move, cell_center(&app, 5, 7)));
    assert!(app.view.cursor_set);

    let other = app.layout.slots[1].center();
    app.handle_pointer(&event(PointerPhase::Down, other));
    app.handle_pointer(&event(PointerPhase::Up, other));

    assert_eq!(app.view.selected, Some(1));
    assert!(
        !app.view.cursor_set,
        "the highlight belongs to the piece in hand, not to where the last one went"
    );
}

#[test]
fn moving_off_the_board_clears_the_cursor_highlight() {
    let mut app = app();
    let slot_center = app.layout.slots[0].center();
    app.handle_pointer(&event(PointerPhase::Down, slot_center));
    app.handle_pointer(&event(PointerPhase::Up, slot_center));
    app.handle_pointer(&event(PointerPhase::Move, cell_center(&app, 3, 4)));
    assert!(app.view.cursor_set);

    app.handle_pointer(&event(PointerPhase::Move, app.layout.slots[1].center()));
    assert!(
        !app.view.cursor_set,
        "the highlight follows the pointer and does not linger on the board"
    );
}

#[test]
fn the_piece_keeps_its_grab_cell_under_the_pointer() {
    let mut app = app();
    let slot_center = app.layout.slots[0].center();
    let piece = app.game.tray[0].unwrap();
    let shape = app.game.shape(piece);
    let grab = app.layout.slot_cell_at(0, slot_center, shape);
    assert_eq!(
        shape.mask[grab.1 * shape.w as usize + grab.0],
        1,
        "the grab has to be a cell of the piece, not empty space"
    );

    app.handle_pointer(&event(PointerPhase::Down, slot_center));
    let cell = (3, 4);
    app.handle_pointer(&event(
        PointerPhase::Move,
        cell_center(&app, cell.0, cell.1),
    ));

    let drag = app.view.drag.unwrap();
    assert_eq!(drag.grab, grab);
    let target = drag.target.unwrap();
    assert_eq!((target.0 + grab.0, target.1 + grab.1), cell);
}

#[test]
fn a_release_outside_the_board_does_not_place() {
    let mut app = app();
    let slot_center = app.layout.slots[0].center();
    app.handle_pointer(&event(PointerPhase::Down, slot_center));
    app.handle_pointer(&event(PointerPhase::Move, FPoint { x: 5.0, y: 5.0 }));
    app.handle_pointer(&event(PointerPhase::Up, FPoint { x: 5.0, y: 5.0 }));
    assert_eq!(app.game.pieces_placed, 0);
    assert_eq!(app.view.drag, None);
}

#[test]
fn buttons_are_hit_and_do_the_right_thing() {
    let mut app = app();
    let sound = app.layout.buttons[0].center();
    app.handle_pointer(&event(PointerPhase::Down, sound));
    assert!(!app.view.sound_on);

    app.handle_command(Command::ToggleSound);
    // Null audio cannot be enabled, so the view must follow the sink, not the intent.
    assert_eq!(app.view.sound_on, app.audio_is_enabled());

    let new_game = app.layout.buttons[1].center();
    app.handle_pointer(&event(PointerPhase::Down, new_game));
    assert_eq!(app.game.pieces_placed, 0);
    assert_eq!(app.game.status, Status::Playing);
}

#[test]
fn keyboard_navigation_selects_moves_and_places() {
    let mut app = app();
    app.handle_command(Command::SelectSlot(1));
    assert_eq!(app.view.selected, Some(1));

    app.handle_command(Command::MoveCursor(3, 4));
    assert_eq!(app.view.cursor, (3, 4));

    if app.game.can_place(1, 3, 4) {
        app.handle_command(Command::Place);
        assert_eq!(app.game.pieces_placed, 1);
    }
}

#[test]
fn the_game_is_saved_and_resumed() {
    let mut app = app();
    let (x, y) = first_fit(&app, 0);
    app.handle_command(Command::SelectSlot(0));
    app.handle_command(Command::MoveCursor(x as isize, y as isize));
    app.handle_command(Command::Place);

    let saved = snapshot::decode(&app.saved().expect("saved snapshot")).expect("saved snapshot");
    assert_eq!(saved, app.game);

    // A fresh app reading the same store resumes the same game.
    let store = TestStore(HashMap::from_iter([
        (
            chillblocks_app::STORE_KEY.to_string(),
            snapshot::encode(&app.game),
        ),
        (
            chillblocks_app::BEST_KEY.to_string(),
            app.game.score.to_string(),
        ),
    ]));
    let resumed = App::new(
        Box::new(TestPainter),
        Box::new(store),
        Box::new(TestClock(2000.0)),
        Box::new(NullAudio),
    );
    assert_eq!(resumed.game, app.game);
    assert_eq!(resumed.view.best, app.game.score);
}

#[test]
fn a_corrupt_save_starts_a_new_game_instead_of_crashing() {
    let store = TestStore(HashMap::from_iter([(
        chillblocks_app::STORE_KEY.to_string(),
        "not a snapshot".to_string(),
    )]));
    let app = App::new(
        Box::new(TestPainter),
        Box::new(store),
        Box::new(TestClock(3000.0)),
        Box::new(NullAudio),
    );
    assert_eq!(app.game, Game::new(3000));
}

#[test]
fn personal_best_only_ever_grows() {
    let mut app = app();
    app.handle_command(Command::SelectSlot(0));
    let (x, y) = first_fit(&app, 0);
    app.handle_command(Command::MoveCursor(x as isize, y as isize));
    app.handle_command(Command::Place);
    let first = app.view.best;
    assert!(first > 0);

    app.handle_command(Command::NewGame);
    assert_eq!(app.view.best, first, "best survives a new game");
}

#[test]
fn audio_is_off_by_default() {
    let app = app();
    assert!(!app.view.sound_on);
    assert!(!app.audio_is_enabled());
}
