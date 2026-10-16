/// Small, fast, deterministic PRNG (SplitMix64).
///
/// Deliberately plain: pieces are drawn uniformly from a fixed set. There is no
/// weighting, no streak logic and no near-miss generation, because those are the
/// mechanisms that turn a puzzle into a dopamine loop.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rng {
    state: u64,
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF03_6384_5471_D9F1);
        z = (z ^ (z >> 27)).wrapping_mul(0xD405_03F0_84E1_8051);
        z ^ (z >> 31)
    }

    /// Uniform value in `0..n`. `n` must be non-zero.
    pub fn below(&mut self, n: u64) -> u64 {
        if n == 0 {
            return 0;
        }
        self.next_u64() % n
    }

    pub fn state(&self) -> u64 {
        self.state
    }

    pub fn set_state(&mut self, state: u64) {
        self.state = state;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_sequence() {
        let mut a = Rng::new(42);
        let mut b = Rng::new(42);
        for _ in 0..1000 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn different_seeds_diverge() {
        let mut a = Rng::new(1);
        let mut b = Rng::new(2);
        assert_ne!(a.next_u64(), b.next_u64());
    }

    #[test]
    fn below_stays_in_range_and_covers_the_range() {
        let mut r = Rng::new(7);
        let mut seen = [false; 23];
        for _ in 0..10_000 {
            let v = r.below(23);
            assert!(v < 23);
            seen[v as usize] = true;
        }
        assert!(seen.iter().all(|s| *s));
    }

    #[test]
    fn state_round_trips() {
        let mut r = Rng::new(99);
        for _ in 0..50 {
            r.next_u64();
        }
        let saved = r.state();
        let mut copy = Rng::new(0);
        copy.set_state(saved);
        for _ in 0..20 {
            assert_eq!(r.next_u64(), copy.next_u64());
        }
    }
}
