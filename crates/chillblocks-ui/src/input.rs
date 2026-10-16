use crate::geometry::FPoint;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PointerPhase {
    Down,
    Move,
    Up,
    Cancel,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PointerKind {
    Mouse,
    Touch,
    Pen,
}

/// One input model for mouse and touch. Coordinates are in CSS pixels relative to the
/// render surface.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointerEvent {
    pub id: i32,
    pub phase: PointerPhase,
    pub kind: PointerKind,
    pub point: FPoint,
    pub time_ms: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    SelectSlot(usize),
    MoveCursor(isize, isize),
    Place,
    NewGame,
    ToggleSound,
}
