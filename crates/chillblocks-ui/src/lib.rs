//! Renderer-agnostic scene: theme, layout, view state and draw.
//!
//! Nothing here knows about canvas, wgpu or any platform. It emits draw primitives
//! through the `Painter` trait, so the same scene renders identically on the browser
//! backend today and on a native backend later.

pub mod color;
pub mod draw;
pub mod geometry;
pub mod input;
pub mod layout;
pub mod motion;
pub mod painter;
pub mod theme;
pub mod view;

pub use painter::Painter;
pub use view::View;
