//! Watcom `rand`/`srand` and the game's `random_n` wrapper (docs/re/rules-01-rng-calendar.md §1).

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rng {
    pub s: u32,
}

impl Default for Rng {
    /// Watcom's initial state is 1.
    fn default() -> Self {
        Self { s: 1 }
    }
}

impl Rng {
    pub fn new(seed: u32) -> Self {
        Self { s: seed }
    }

    pub fn srand(&mut self, x: u32) {
        self.s = x;
    }

    pub fn rand(&mut self) -> u32 {
        self.s = self.s.wrapping_mul(1103515245).wrapping_add(12345);
        (self.s >> 16) & 0x7FFF
    }

    /// `rand() % (n mod 65536)`. The original divides by zero for n = 0 (mod 65536); we return 0
    /// without advancing the state (rules-01 §1 asks for 0; the original advances, then faults, rules-39 §10).
    pub fn n(&mut self, n: u32) -> u32 {
        let m = n & 0xFFFF;
        if m == 0 {
            return 0;
        }
        self.rand() % m
    }

    /// Convenience for signed call sites (`rand(151 - pay)` etc.): the argument is truncated to 16 bits.
    pub fn ni(&mut self, n: i32) -> i32 {
        self.n(n as u32) as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// rules-01 §1: LCG 1103515245/12345, output bits 16..30, initial state 1.
    #[test]
    fn rules01_s1_watcom_sequence() {
        let mut r = Rng::default();
        // First Watcom rand() outputs from state 1 (same LCG as the ANSI C sample implementation).
        assert_eq!([r.rand(), r.rand(), r.rand()], [16838, 5758, 10113]);
        r.srand(1);
        assert_eq!(r.rand(), 16838);
    }

    #[test]
    fn rules01_s1_random_n_truncates_to_16_bits() {
        let mut a = Rng::new(7);
        let mut b = Rng::new(7);
        assert_eq!(a.n(65536 + 10), b.n(10));
        assert_eq!(a.n(0), 0);
        assert_eq!(a.n(65536), 0);
        assert_eq!(a, b, "n = 0 does not advance");
        for _ in 0..1000 {
            assert!(a.n(6) < 6);
        }
    }
}
