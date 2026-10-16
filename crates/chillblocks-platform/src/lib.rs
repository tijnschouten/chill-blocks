//! Platform boundary. Everything the game needs from the outside world is a trait:
//! a renderer, storage, a clock, audio. The browser implements them today; native
//! iOS/Android implementations later slot in here without touching game logic.

pub mod traits;

#[cfg(target_arch = "wasm32")]
pub mod web;

pub use traits::{AudioSink, Clock, Store, Tone};
