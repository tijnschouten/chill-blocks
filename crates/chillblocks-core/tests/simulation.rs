//! Headless simulation harness: plays many games with a simple greedy player and
//! checks the rules hold. Runs on any target, including wasm, with no platform code.

use chillblocks_core::board::BOARD_SIZE;
use chillblocks_core::game::{Game, Orientation, Piece, PlaceError, Status, TRAY_SIZE};
use chillblocks_core::piece::SHAPES;
use chillblocks_core::snapshot;

const LIMIT: usize = 200_000;

fn cells() -> impl Iterator<Item = (usize, usize)> {
    (0..BOARD_SIZE).flat_map(|y| (0..BOARD_SIZE).map(move |x| (x, y)))
}

/// Pick the best placement for a slot: prefer one that clears lines, otherwise any fit.
fn best_move(game: &Game, slot: usize) -> Option<(usize, usize)> {
    let piece = game.tray[slot]?;
    let shape = game.shape(piece);

    let mut fallback = None;
    let mut best: Option<((usize, usize), usize)> = None;

    for (x, y) in cells() {
        if !game.board.can_fit(shape, x, y) {
            continue;
        }
        let mut probe = game.board.clone();
        for (dx, dy) in shape.cells() {
            probe.set(x + dx as usize, y + dy as usize, piece.color);
        }
        let lines = probe.full_lines().len();
        if lines > 0 && best.map(|(_, l)| lines > l).unwrap_or(true) {
            best = Some(((x, y), lines));
        } else if fallback.is_none() {
            fallback = Some((x, y));
        }
    }

    best.map(|(pos, _)| pos).or(fallback)
}

/// A player is not stuck until *none* of the three tray pieces fits, which is exactly
/// the rule the core enforces. One unplayable piece is not game over.
fn next_move(game: &Game) -> Option<(usize, usize, usize)> {
    for slot in 0..TRAY_SIZE {
        if let Some((x, y)) = best_move(game, slot) {
            return Some((slot, x, y));
        }
    }
    None
}

fn play(seed: u64) -> Game {
    let mut game = Game::new(seed);
    let mut guard = 0;
    while game.status == Status::Playing && guard < LIMIT {
        guard += 1;
        let (slot, x, y) = match next_move(&game) {
            Some(move_) => move_,
            None => break,
        };
        game.place(slot, x, y).expect("placement should succeed");
    }
    game
}

#[test]
fn games_play_to_completion_without_breaking_invariants() {
    let mut total_placements = 0;
    let mut games_with_clears = 0;

    for seed in 0..200 {
        let mut game = Game::new(seed);
        let mut guard = 0;

        while game.status == Status::Playing && guard < LIMIT {
            guard += 1;
            let (slot, x, y) = match next_move(&game) {
                Some(move_) => move_,
                None => break,
            };

            let score_before = game.score;
            let placement = game.place(slot, x, y).expect("placement should succeed");

            assert_eq!(placement.slot, slot);
            assert!(!placement.cells.is_empty());
            assert!(
                game.board.full_lines().is_empty(),
                "board left with a full line after a placement"
            );
            for &cell in &placement.cleared_cells {
                assert!(
                    game.board.is_empty(cell.x, cell.y),
                    "cleared cell still filled"
                );
            }
            // Flat scoring: placed cells plus one bonus per cleared line, nothing else.
            assert_eq!(
                game.score,
                score_before
                    + placement.cells.len() as u32
                    + placement.cleared_lines.len() as u32 * 10
            );

            total_placements += 1;
            if !placement.cleared_lines.is_empty() {
                games_with_clears += 1;
            }
        }

        assert_eq!(game.status, Status::GameOver, "game did not finish");
        assert!(game.pieces_placed > 0);
    }

    assert!(total_placements > 200);
    assert!(
        games_with_clears > 100,
        "clears are not happening: {games_with_clears}"
    );
}

#[test]
fn the_same_seed_replays_identically() {
    for seed in [1, 7, 42, 1234, 99_999] {
        assert_eq!(play(seed), play(seed));
    }
}

#[test]
fn snapshot_round_trip_survives_a_full_game() {
    for seed in 0..50 {
        let game = play(seed);
        let decoded = snapshot::decode(&snapshot::encode(&game)).expect("round trip");
        assert_eq!(decoded, game);
    }
}

#[test]
fn one_unplayable_piece_is_not_game_over() {
    let mut game = Game::new(40);
    // Checkerboard: every hole is isolated, so only single-cell pieces fit.
    for (x, y) in cells() {
        if (x + y) % 2 == 1 {
            game.board.set(x, y, 0);
        }
    }
    let dot = 0;
    let bar = SHAPES.iter().position(|s| s.w == 4 && s.h == 1).unwrap();
    game.tray[0] = Some(Piece {
        shape: bar,
        color: 0,
    });
    game.tray[1] = Some(Piece {
        shape: bar,
        color: 0,
    });
    game.tray[2] = Some(Piece {
        shape: dot,
        color: 0,
    });

    assert!(
        !game.can_place(0, 0, 0),
        "a 4x1 bar cannot fit a single hole"
    );
    assert!(game.can_place(2, 0, 0));
    assert!(game.has_any_move());
    assert_eq!(game.status, Status::Playing);
}

#[test]
fn orientation_is_used_consistently() {
    let mut game = Game::new(32);
    for x in 0..BOARD_SIZE {
        game.board.set(x, 2, 0);
    }
    let lines = game.board.full_lines();
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0].orientation, Orientation::Row);
    assert_eq!(lines[0].index, 2);
}

#[test]
fn placing_into_a_finished_game_is_rejected() {
    let mut game = Game::new(33);
    for (x, y) in cells() {
        if (x + y) % 2 == 1 {
            game.board.set(x, y, 0);
        }
    }
    let bar = SHAPES.iter().position(|s| s.w == 4 && s.h == 1).unwrap();
    for slot in 0..TRAY_SIZE {
        game.tray[slot] = Some(Piece {
            shape: bar,
            color: 0,
        });
    }
    game.status = Status::GameOver;
    assert_eq!(game.place(0, 0, 0), Err(PlaceError::GameOver));
}
