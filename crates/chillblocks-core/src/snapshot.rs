//! Compact, human-readable snapshot for persistence.
//!
//! Plain text so a saved game can be inspected and so the core crate needs no
//! serialization dependency. Format:
//!
//! `v1|score|lines|placed|status|rng_state|board|tray`
//!
//! `board` is 64 characters, `.` for empty and a base-36 digit for palette index + 1.
//! `tray` is three `shape.color` entries separated by `;`, or `-` for an empty slot.

use crate::board::{BOARD_SIZE, CELL_COUNT};
use crate::game::{Game, Piece, Status, PALETTE_SIZE, TRAY_SIZE};
use crate::piece::SHAPES;

pub const VERSION: &str = "v1";

fn base36_char(value: u64) -> char {
    const DIGITS: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";
    DIGITS[value as usize % DIGITS.len()] as char
}

fn base36_value(c: char) -> Option<u64> {
    match c {
        '0'..='9' => Some(c as u64 - '0' as u64),
        'a'..='z' => Some(c as u64 - 'a' as u64 + 10),
        _ => None,
    }
}

pub fn encode(game: &Game) -> String {
    let board: String = game
        .board
        .cells()
        .iter()
        .map(|&v| {
            if v == 0 {
                '.'
            } else {
                base36_char(v as u64 - 1)
            }
        })
        .collect();

    let tray = game
        .tray
        .iter()
        .map(|slot| match slot {
            Some(piece) => format!(
                "{}.{}",
                base36_char(piece.shape as u64),
                base36_char(piece.color as u64)
            ),
            None => "-".to_string(),
        })
        .collect::<Vec<_>>()
        .join(";");

    format!(
        "{}|{}|{}|{}|{}|{}|{}|{}",
        VERSION,
        game.score,
        game.lines_cleared,
        game.pieces_placed,
        match game.status {
            Status::Playing => "play",
            Status::GameOver => "over",
        },
        game.rng.state(),
        board,
        tray
    )
}

pub fn decode(text: &str) -> Option<Game> {
    let parts: Vec<&str> = text.split('|').collect();
    if parts.len() != 8 || parts[0] != VERSION {
        return None;
    }

    let score = parts[1].parse().ok()?;
    let lines_cleared = parts[2].parse().ok()?;
    let pieces_placed = parts[3].parse().ok()?;
    let status = match parts[4] {
        "play" => Status::Playing,
        "over" => Status::GameOver,
        _ => return None,
    };
    let rng_state = parts[5].parse().ok()?;

    if parts[6].chars().count() != CELL_COUNT {
        return None;
    }

    let tray: Vec<Option<Piece>> = parts[7]
        .split(';')
        .map(|entry| {
            if entry == "-" {
                return None;
            }
            let mut fields = entry.split('.');
            let shape = base36_value(fields.next()?.chars().next()?)? as usize;
            let color = base36_value(fields.next()?.chars().next()?)? as u8;
            if shape >= SHAPES.len() || color as usize >= PALETTE_SIZE {
                return None;
            }
            Some(Piece { shape, color })
        })
        .collect();

    if tray.len() != TRAY_SIZE {
        return None;
    }

    let mut game = Game::new(rng_state);
    game.rng.set_state(rng_state);
    game.score = score;
    game.lines_cleared = lines_cleared;
    game.pieces_placed = pieces_placed;
    game.status = status;

    for (i, c) in parts[6].chars().enumerate() {
        if c == '.' {
            continue;
        }
        let color = base36_value(c)? as u8;
        game.board.set(i % BOARD_SIZE, i / BOARD_SIZE, color);
    }

    for (slot, piece) in tray.into_iter().enumerate() {
        game.tray[slot] = piece;
    }

    // A decoded game must agree with the rules, otherwise the save is corrupt.
    // A live game never leaves a full line on the board.
    if !game.board.full_lines().is_empty() {
        return None;
    }
    match (game.status, game.has_any_move()) {
        (Status::Playing, false) | (Status::GameOver, true) => return None,
        _ => {}
    }

    Some(game)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_a_fresh_game() {
        let game = Game::new(11);
        let text = encode(&game);
        let decoded = decode(&text).expect("decode");
        assert_eq!(decoded, game);
    }

    #[test]
    fn round_trips_a_game_in_progress() {
        let mut game = Game::new(12);
        for _ in 0..25 {
            if game.status != Status::Playing {
                break;
            }
            let slot = game.tray.iter().position(|s| s.is_some()).unwrap();
            let piece = game.tray[slot].unwrap();
            let placed = (0..BOARD_SIZE)
                .flat_map(|y| (0..BOARD_SIZE).map(move |x| (x, y)))
                .find(|(x, y)| game.board.can_fit(game.shape(piece), *x, *y));
            match placed {
                Some((x, y)) => {
                    game.place(slot, x, y).unwrap();
                }
                None => break,
            }
        }
        let decoded = decode(&encode(&game)).expect("decode");
        assert_eq!(decoded, game);
    }

    #[test]
    fn rejects_garbage() {
        assert!(decode("").is_none());
        assert!(decode("v2|0|0|0|play|0|....|0.0;0.0;0.0").is_none());
        assert!(decode("v1|0|0|0|play|0|....|0.0;0.0;0.0").is_none());
        assert!(decode(
            "v1|0|0|0|bogus|0|........................................................|0.0;0.0;0.0"
        )
        .is_none());
        assert!(decode(
            "v1|0|0|0|play|0|........................................................|9.0;0.0;0.0"
        )
        .is_none());
    }

    #[test]
    fn rejects_a_board_with_a_full_line() {
        let mut game = Game::new(13);
        for x in 0..BOARD_SIZE {
            game.board.set(x, 0, 0);
        }
        assert!(decode(&encode(&game)).is_none());
    }

    #[test]
    fn rejects_a_save_that_disagrees_with_the_rules() {
        let mut game = Game::new(14);
        // Checkerboard: nothing but a single-cell piece can fit.
        for y in 0..BOARD_SIZE {
            for x in 0..BOARD_SIZE {
                if (x + y) % 2 == 1 {
                    game.board.set(x, y, 0);
                }
            }
        }
        let bar = SHAPES.iter().position(|s| s.w == 4 && s.h == 1).unwrap();
        for slot in 0..TRAY_SIZE {
            game.tray[slot] = Some(Piece {
                shape: bar,
                color: 0,
            });
        }
        assert!(!game.has_any_move());
        // Marked as playable but no move exists: corrupt.
        assert!(decode(&encode(&game)).is_none());
    }
}
