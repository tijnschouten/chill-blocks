/// A single soft tone. Gain is capped so audio can never become a reward signal.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tone {
    pub freq: f32,
    pub gain: f32,
    pub duration_ms: f32,
}

/// Raised from 0.06 because that was inaudible on a phone speaker. Still a quiet single
/// sine, and still the only sound the game can make.
pub const MAX_GAIN: f32 = 0.12;

impl Tone {
    pub fn new(freq: f32, gain: f32, duration_ms: f32) -> Self {
        Self {
            freq,
            gain: gain.min(MAX_GAIN),
            duration_ms,
        }
    }
}

pub trait Store {
    fn get(&self, key: &str) -> Option<String>;
    fn set(&mut self, key: &str, value: &str);
}

pub trait Clock {
    fn now_ms(&self) -> f64;
}

pub trait AudioSink {
    /// Returns false when audio is unavailable or disabled.
    fn play(&mut self, tone: Tone) -> bool;
    fn is_enabled(&self) -> bool;
    fn set_enabled(&mut self, enabled: bool);
    /// Open the audio device from a user gesture. Browsers create an audio context
    /// suspended until it is touched by one, so this has to happen on a real press.
    fn wake(&mut self) -> bool;
}

/// Inert implementations so the app can be built and tested on any target.
pub struct NullStore;

impl Store for NullStore {
    fn get(&self, _key: &str) -> Option<String> {
        None
    }
    fn set(&mut self, _key: &str, _value: &str) {}
}

pub struct NullClock;

impl Clock for NullClock {
    fn now_ms(&self) -> f64 {
        0.0
    }
}

pub struct NullAudio;

impl AudioSink for NullAudio {
    fn play(&mut self, _tone: Tone) -> bool {
        false
    }
    fn is_enabled(&self) -> bool {
        false
    }
    fn set_enabled(&mut self, _enabled: bool) {}
    fn wake(&mut self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gain_is_capped() {
        assert_eq!(Tone::new(440.0, 0.5, 200.0).gain, MAX_GAIN);
        assert_eq!(Tone::new(440.0, 0.02, 200.0).gain, 0.02);
    }

    #[test]
    fn null_implementations_are_inert() {
        let mut store = NullStore;
        store.set("k", "v");
        assert_eq!(store.get("k"), None);
        assert_eq!(NullClock.now_ms(), 0.0);
        assert!(!NullAudio.play(Tone::new(440.0, 0.05, 100.0)));
        assert!(!NullAudio.is_enabled());
    }
}
