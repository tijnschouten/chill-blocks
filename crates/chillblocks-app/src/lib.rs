pub mod app;

pub use app::{App, BEST_KEY, SETTINGS_KEY, STORE_KEY};

#[cfg(target_arch = "wasm32")]
mod wasm;
