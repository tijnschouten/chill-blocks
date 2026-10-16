use wasm_bindgen::JsCast;
use web_sys::{AudioContext, CanvasRenderingContext2d, HtmlCanvasElement, Storage};

use chillblocks_ui::color::Color;
use chillblocks_ui::geometry::Rect;
use chillblocks_ui::painter::{Align, Baseline, Painter, TextStyle};

use crate::traits::{AudioSink, Clock, Store, Tone};

pub struct WebPainter {
    ctx: CanvasRenderingContext2d,
    dpr: f32,
    width: f32,
    height: f32,
}

impl WebPainter {
    pub fn new(canvas: &HtmlCanvasElement) -> Result<Self, String> {
        let context = canvas
            .get_context("2d")
            .map_err(|e| format!("canvas context: {e:?}"))?
            .ok_or_else(|| "no 2d context".to_string())?
            .dyn_into::<CanvasRenderingContext2d>()
            .map_err(|e| format!("2d context: {e:?}"))?;
        Ok(Self {
            ctx: context,
            dpr: 1.0,
            width: 0.0,
            height: 0.0,
        })
    }

    fn path_round_rect(&self, x: f64, y: f64, w: f64, h: f64, radius: f64) {
        let r = radius.min(w / 2.0).min(h / 2.0);
        let _ = self.ctx.begin_path();
        let _ = self.ctx.move_to(x + r, y);
        let _ = self.ctx.arc_to(x + w, y, x + w, y + r, r);
        let _ = self.ctx.arc_to(x + w, y + h, x + w - r, y + h, r);
        let _ = self.ctx.arc_to(x, y + h, x, y + h - r, r);
        let _ = self.ctx.arc_to(x, y, x + r, y, r);
        let _ = self.ctx.close_path();
    }
}

impl Painter for WebPainter {
    fn begin(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
        let dpr = self.dpr as f64;
        let _ = self.ctx.set_transform(dpr, 0.0, 0.0, dpr, 0.0, 0.0);
    }

    fn clear(&mut self, background: Color) {
        self.ctx.set_fill_style_str(&background.css());
        self.ctx
            .fill_rect(0.0, 0.0, self.width as f64, self.height as f64);
    }

    fn fill_round_rect(&mut self, rect: &Rect, radius: f32, color: Color) {
        self.path_round_rect(
            rect.x as f64,
            rect.y as f64,
            rect.w as f64,
            rect.h as f64,
            radius as f64,
        );
        self.ctx.set_fill_style_str(&color.css());
        self.ctx.fill();
    }

    fn stroke_round_rect(&mut self, rect: &Rect, radius: f32, width: f32, color: Color) {
        self.path_round_rect(
            rect.x as f64,
            rect.y as f64,
            rect.w as f64,
            rect.h as f64,
            radius as f64,
        );
        self.ctx.set_line_width(width as f64);
        self.ctx.set_stroke_style_str(&color.css());
        self.ctx.stroke();
    }

    fn text(&mut self, x: f32, y: f32, content: &str, style: &TextStyle) {
        self.ctx.set_font(&format!(
            "{} {}px {}",
            style.weight, style.size, style.family
        ));
        self.ctx.set_text_align(match style.align {
            Align::Left => "left",
            Align::Center => "center",
            Align::Right => "right",
        });
        self.ctx.set_text_baseline(match style.baseline {
            Baseline::Top => "top",
            Baseline::Middle => "middle",
            Baseline::Alphabetic => "alphabetic",
        });
        self.ctx.set_fill_style_str(&style.color.css());
        let _ = self.ctx.fill_text(content, x as f64, y as f64);
    }

    fn set_device_scale(&mut self, dpr: f32) {
        self.dpr = dpr.max(1.0);
    }
}

pub struct WebStore {
    storage: Option<Storage>,
}

impl WebStore {
    pub fn new() -> Self {
        let storage = web_sys::window()
            .and_then(|window| window.local_storage().ok())
            .flatten();
        Self { storage }
    }
}

impl Store for WebStore {
    fn get(&self, key: &str) -> Option<String> {
        self.storage
            .as_ref()
            .and_then(|storage| storage.get_item(key).ok().flatten())
    }

    fn set(&mut self, key: &str, value: &str) {
        if let Some(storage) = self.storage.as_ref() {
            let _ = storage.set_item(key, value);
        }
    }
}

pub struct WebClock;

impl Clock for WebClock {
    fn now_ms(&self) -> f64 {
        web_sys::window()
            .and_then(|window| window.performance())
            .map(|performance| performance.now())
            .unwrap_or(0.0)
    }
}

/// One soft sine tone, capped gain, off by default. There is deliberately no reward
/// sound: audio only marks a clear, and only if the player opted in.
pub struct WebAudio {
    context: Option<AudioContext>,
    enabled: bool,
}

impl WebAudio {
    pub fn new(enabled: bool) -> Self {
        Self {
            context: None,
            enabled,
        }
    }
}

impl AudioSink for WebAudio {
    fn wake(&mut self) -> bool {
        if !self.enabled {
            return false;
        }
        if self.context.is_none() {
            self.context = AudioContext::new().ok();
        }
        let context = match self.context.as_ref() {
            Some(context) => context,
            None => return false,
        };
        // A context created outside a user gesture starts suspended and never advances,
        // which is why scheduling a tone alone produces silence.
        let _ = context.resume();
        true
    }

    fn play(&mut self, tone: Tone) -> bool {
        if !self.wake() {
            return false;
        }
        let context = self.context.as_ref().unwrap();

        let oscillator = match context.create_oscillator() {
            Ok(oscillator) => oscillator,
            Err(_) => return false,
        };
        let gain = match context.create_gain() {
            Ok(gain) => gain,
            Err(_) => return false,
        };

        let start = context.current_time() + 0.05;
        let duration = (tone.duration_ms as f64 / 1000.0).clamp(0.05, 0.6);
        let attack = duration.min(0.03);

        oscillator.frequency().set_value(tone.freq);
        // OscillatorNode defaults to "sine", which is the only timbre the game uses.

        let param = gain.gain();
        let _ = param.set_value_at_time(0.0, start);
        let _ = param.linear_ramp_to_value_at_time(tone.gain, start + attack);
        let _ = param.linear_ramp_to_value_at_time(0.0, start + duration);

        if oscillator.connect_with_audio_node(&gain).is_err() {
            return false;
        }
        if gain
            .connect_with_audio_node(&context.destination())
            .is_err()
        {
            return false;
        }

        let _ = oscillator.start_with_when(start);
        let _ = oscillator.stop_with_when(start + duration + 0.05);
        true
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}
