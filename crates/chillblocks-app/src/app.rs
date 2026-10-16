//! The game host: owns the game state, turns input into moves, and drives the renderer.
//!
//! It talks only through the platform traits, so the same host runs in the browser today
//! and on a native backend later.

use chillblocks_core::game::{Game, PlaceError, Status};
use chillblocks_core::snapshot;
use chillblocks_platform::traits::{AudioSink, Clock, Store, Tone};
use chillblocks_ui::draw::draw;
use chillblocks_ui::input::{Command, PointerEvent, PointerKind, PointerPhase};
use chillblocks_ui::layout::{self, Insets, Layout, NEW_GAME_BUTTON, SOUND_BUTTON};
use chillblocks_ui::painter::Painter;
use chillblocks_ui::view::{Drag, View, TAP_THRESHOLD, TOUCH_LIFT_CELLS};

pub const STORE_KEY: &str = "chillblocks.v1";
pub const BEST_KEY: &str = "chillblocks.best";
pub const SETTINGS_KEY: &str = "chillblocks.settings";

/// A clear is the only thing that makes a sound during play, and only if the player opted in.
fn clear_tone() -> Tone {
    Tone::new(392.0, 0.05, 260.0)
}

/// Heard once when sound is switched on, so the player knows the audio device is live.
fn confirm_tone() -> Tone {
    Tone::new(294.0, 0.04, 180.0)
}

fn lift_for(kind: PointerKind, cell: f32) -> f32 {
    match kind {
        PointerKind::Touch => cell * TOUCH_LIFT_CELLS,
        _ => 0.0,
    }
}

pub struct App {
    pub game: Game,
    pub view: View,
    pub layout: Layout,
    painter: Box<dyn Painter>,
    store: Box<dyn Store>,
    clock: Box<dyn Clock>,
    audio: Box<dyn AudioSink>,
    insets: Insets,
}

impl App {
    pub fn new(
        painter: Box<dyn Painter>,
        store: Box<dyn Store>,
        clock: Box<dyn Clock>,
        mut audio: Box<dyn AudioSink>,
    ) -> App {
        let mut view = View::new();
        view.best = store
            .get(BEST_KEY)
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);
        view.sound_on = store.get(SETTINGS_KEY).as_deref() == Some("sound=on");
        audio.set_enabled(view.sound_on);

        let game = store
            .get(STORE_KEY)
            .and_then(|text| snapshot::decode(&text))
            .unwrap_or_else(|| Game::new(clock.now_ms() as u64));

        App {
            game,
            view,
            layout: layout::compute(800.0, 600.0, Insets::default()),
            painter,
            store,
            clock,
            audio,
            insets: Insets::default(),
        }
    }

    pub fn resize(&mut self, width: f32, height: f32, dpr: f32, insets: Insets) {
        self.insets = insets;
        self.layout = layout::compute(width, height, insets);
        self.painter.set_device_scale(dpr);
        self.view.dirty = true;
    }

    pub fn handle_pointer(&mut self, event: &PointerEvent) {
        let now = self.clock.now_ms();

        // A press is the only thing a browser accepts as a user gesture, so this is where
        // the audio device has to be opened.
        if event.phase == PointerPhase::Down {
            self.audio.wake();
        }

        match event.phase {
            PointerPhase::Down => {
                if let Some(button) = self.layout.button_at(event.point) {
                    self.press_button(button);
                    return;
                }

                if let Some(slot) = self.layout.slot_at(event.point) {
                    if let Some(piece) = self.game.tray[slot] {
                        if self.game.status == Status::Playing {
                            let shape = self.game.shape(piece);
                            let lift = lift_for(event.kind, self.layout.cell);
                            let grab = self.layout.slot_cell_at(slot, event.point, shape);
                            let target = self.layout.drag_anchor(event.point, lift, grab, shape);
                            self.view.drag = Some(Drag {
                                slot,
                                pointer: event.point,
                                origin: event.point,
                                started: now,
                                moved: false,
                                kind: event.kind,
                                grab,
                                target,
                            });
                            self.view.selected = Some(slot);
                            self.view.dirty = true;
                        }
                    }
                    return;
                }

                if let (Some(slot), Some((x, y))) =
                    (self.view.selected, self.layout.cell_at(event.point))
                {
                    self.try_place(slot, x, y);
                }
            }

            PointerPhase::Move => {
                if let Some(drag) = &mut self.view.drag {
                    drag.pointer = event.point;
                    let dx = event.point.x - drag.origin.x;
                    let dy = event.point.y - drag.origin.y;
                    if dx * dx + dy * dy > TAP_THRESHOLD * TAP_THRESHOLD {
                        drag.moved = true;
                    }
                    if let Some(piece) = self.game.tray[drag.slot] {
                        let shape = self.game.shape(piece);
                        let lift = lift_for(drag.kind, self.layout.cell);
                        drag.target = self.layout.drag_anchor(event.point, lift, drag.grab, shape);
                    }
                    self.view.dirty = true;
                } else if self.view.selected.is_some() {
                    if let Some(cell) = self.layout.cell_at(event.point) {
                        if cell != self.view.cursor {
                            self.view.cursor = cell;
                            self.view.dirty = true;
                        }
                    }
                }
            }

            PointerPhase::Up | PointerPhase::Cancel => {
                if let Some(drag) = self.view.drag.take() {
                    if drag.moved {
                        match drag.target {
                            Some((x, y)) => self.try_place(drag.slot, x, y),
                            None => self.view.selected = None,
                        }
                    }
                    // A tap on the tray leaves the piece selected for tap-to-place.
                    self.view.dirty = true;
                }
            }
        }
    }

    pub fn handle_command(&mut self, command: Command) {
        match command {
            Command::SelectSlot(slot) => {
                if self.game.tray[slot].is_some() {
                    self.view.selected = Some(slot);
                    self.view.dirty = true;
                }
            }
            Command::MoveCursor(dx, dy) => {
                let (x, y) = self.view.cursor;
                let nx = (x as isize + dx).clamp(0, 7) as usize;
                let ny = (y as isize + dy).clamp(0, 7) as usize;
                if (nx, ny) != (x, y) {
                    self.view.cursor = (nx, ny);
                    self.view.dirty = true;
                }
            }
            Command::Place => {
                if let Some(slot) = self.view.selected {
                    self.try_place(slot, self.view.cursor.0, self.view.cursor.1);
                }
            }
            Command::NewGame => self.new_game(),
            Command::ToggleSound => {
                self.audio.set_enabled(!self.audio.is_enabled());
                self.view.sound_on = self.audio.is_enabled();
                // Pressing the sound button is a user gesture, so this is where the audio
                // device can be opened. The tone also tells the player it is really working.
                if self.view.sound_on && self.audio.wake() {
                    self.audio.play(confirm_tone());
                }
                self.store.set(
                    SETTINGS_KEY,
                    if self.view.sound_on {
                        "sound=on"
                    } else {
                        "sound=off"
                    },
                );
                self.view.dirty = true;
            }
        }
    }

    fn press_button(&mut self, button: usize) {
        match button {
            SOUND_BUTTON => self.handle_command(Command::ToggleSound),
            NEW_GAME_BUTTON => self.handle_command(Command::NewGame),
            _ => {}
        }
    }

    fn try_place(&mut self, slot: usize, x: usize, y: usize) {
        match self.game.place(slot, x, y) {
            Ok(placement) => {
                self.view.record_placement(&placement, self.clock.now_ms());
                if !placement.cleared_lines.is_empty() && self.audio.is_enabled() {
                    self.audio.play(clear_tone());
                }
                if self.game.score > self.view.best {
                    self.view.best = self.game.score;
                    self.store.set(BEST_KEY, &self.view.best.to_string());
                }
                self.view.selected = None;
                self.save();
                self.view.dirty = true;
            }
            Err(PlaceError::DoesNotFit) => {
                // Deliberately silent: a rejected placement is not a punishment.
                self.view.dirty = true;
            }
            Err(_) => {}
        }
    }

    pub fn new_game(&mut self) {
        self.game = Game::new(self.clock.now_ms() as u64);
        self.view.reset();
        self.save();
    }

    pub fn save(&mut self) {
        self.store.set(STORE_KEY, &snapshot::encode(&self.game));
    }

    pub fn saved(&self) -> Option<String> {
        self.store.get(STORE_KEY)
    }

    pub fn audio_is_enabled(&self) -> bool {
        self.audio.is_enabled()
    }

    pub fn frame(&mut self) {
        let now = self.clock.now_ms();
        self.view.prune(now);
        if self.view.dirty || self.view.animations_active(now) {
            draw(
                &self.layout,
                &self.game,
                &self.view,
                &mut *self.painter,
                now,
            );
            self.view.dirty = false;
        }
    }
}
