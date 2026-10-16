/// Motion budget. Enforced in code, not by convention: nothing in the scene may
/// animate longer than `MAX_MS`, and the only easing allowed is a plain ease-out.
/// No bounce, no overshoot, no particles, no shake.
pub const MAX_MS: f64 = 300.0;
pub const FADE_IN_MS: f64 = 180.0;
pub const FADE_OUT_MS: f64 = 250.0;
pub const LIFT_MS: f64 = 150.0;

/// `motion_scale` is the hook for a future reduced-motion setting: 0.0 means no
/// animation at all, 1.0 is the calm default.
pub fn ease_out(t: f64, motion_scale: f64) -> f64 {
    if motion_scale <= 0.0 {
        return 1.0;
    }
    let t = (t / motion_scale).clamp(0.0, 1.0);
    1.0 - (1.0 - t).powi(3)
}

/// Progress of an animation started at `started`, in `0..=1`.
pub fn progress(started: f64, now: f64, duration: f64, motion_scale: f64) -> f64 {
    if duration <= 0.0 || motion_scale <= 0.0 {
        return 1.0;
    }
    ease_out((now - started) / duration, motion_scale)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn durations_respect_the_budget() {
        let budget = MAX_MS;
        assert!(FADE_IN_MS <= budget);
        assert!(FADE_OUT_MS <= budget);
        assert!(LIFT_MS <= budget);
    }

    #[test]
    fn easing_is_monotonic_and_bounded() {
        let mut last = -1.0;
        for i in 0..=100 {
            let t = i as f64 / 100.0;
            let v = ease_out(t, 1.0);
            assert!((0.0..=1.0).contains(&v));
            assert!(v >= last);
            last = v;
        }
        assert_eq!(ease_out(0.0, 1.0), 0.0);
        assert_eq!(ease_out(1.0, 1.0), 1.0);
    }

    #[test]
    fn no_overshoot() {
        // A calm ease-out must never exceed its target.
        for i in 0..=200 {
            let t = i as f64 / 100.0;
            assert!(ease_out(t, 1.0) <= 1.0);
        }
    }

    #[test]
    fn reduced_motion_jumps_straight_to_the_end() {
        assert_eq!(progress(0.0, 10.0, FADE_IN_MS, 0.0), 1.0);
    }

    #[test]
    fn progress_reaches_one_after_the_duration() {
        assert_eq!(progress(0.0, FADE_OUT_MS, FADE_OUT_MS, 1.0), 1.0);
        assert_eq!(progress(0.0, FADE_OUT_MS * 3.0, FADE_OUT_MS, 1.0), 1.0);
    }
}
