//! ChillBlocks game logic.
//!
//! This crate is deliberately dependency-free and platform-agnostic: it knows nothing
//! about rendering, input, storage or time. Everything is deterministic, so a game can
//! be reproduced from its seed and tested headlessly. The same code will back the
//! future native iOS/Android builds.

pub mod board;
pub mod game;
pub mod piece;
pub mod rng;
pub mod snapshot;

pub use board::Board;
pub use game::{CellPos, Game, Line, Orientation, Piece, PlaceError, Placement, Status};
pub use piece::Shape;
pub use rng::Rng;
