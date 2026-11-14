//! Rule dice. Every roll that decides a game rule goes through `Dice`, and
//! every draw is logged. The generator inside is private, so no rule roll can
//! skip the log. Visual effects and building things (maps, faces, names) do
//! not use `Dice` directly: effects use their own unlogged `Rng`, and builders
//! get a child generator from `Dice::generator`.
use crate::data::Rng;

/// One logged rule roll.
#[derive(Clone)]
pub struct Roll {
    /// Who acted: a unit, a craft, a base, or "campaign".
    pub who: String,
    /// A short fixed label for the kind of roll, such as "shot" or "damage".
    pub what: &'static str,
    /// For a percent check, the chance out of 100. None for other rolls.
    pub odds: Option<i32>,
    /// The number that came up.
    pub roll: i32,
    /// For a percent check, whether it passed. None for other rolls.
    pub passed: Option<bool>,
}

/// One splitmix64 step. Turns any number into a well-spread 64-bit number.
fn mix64(x: u64) -> u64 {
    let mut z = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Mix a 64-bit seed down to the generator's 32-bit state.
fn seed32(seed: u64) -> u64 {
    let z = mix64(seed);
    ((z ^ (z >> 32)) as u32) as u64
}

pub struct Dice {
    rng: Rng,
    log: Vec<Roll>,
}

impl Dice {
    pub fn from_seed(seed: u64) -> Self {
        Self { rng: Rng::new(seed32(seed)), log: Vec::new() }
    }

    /// A battle's own dice, from the campaign seed and the battle's number
    /// (1, 2, 3...). Same inputs, same battle.
    pub fn for_battle(campaign_seed: u64, battle_number: u32) -> Self {
        let b = mix64((battle_number as u64).wrapping_mul(0xD1B5_4A32_D192_ED03));
        Self::from_seed(mix64(campaign_seed) ^ b)
    }

    fn push(&mut self, who: &str, what: &'static str, odds: Option<i32>, roll: i32, passed: Option<bool>) {
        self.log.push(Roll { who: who.to_string(), what, odds, roll, passed });
    }

    /// Percent check. Rolls 0..=99 and passes when roll < odds. So odds 100
    /// (or more) ALWAYS passes and odds 0 (or less) NEVER passes.
    pub fn check(&mut self, who: &str, what: &'static str, odds: i32) -> bool {
        self.check_roll(who, what, odds).0
    }

    /// Same as `check`, but also returns the roll (the shot code needs it to size a miss).
    pub fn check_roll(&mut self, who: &str, what: &'static str, odds: i32) -> (bool, i32) {
        let roll = self.rng.rand(99);
        let passed = roll < odds;
        self.push(who, what, Some(odds), roll, Some(passed));
        (passed, roll)
    }

    /// Logged uniform integer in lo..=hi. If hi <= lo it returns lo (still logged).
    pub fn range(&mut self, who: &str, what: &'static str, lo: i32, hi: i32) -> i32 {
        let v = self.rng.range(lo, hi);
        self.push(who, what, None, v, None);
        v
    }

    /// Logged uniform index in 0..len (for picking from a list). len must be > 0.
    pub fn index(&mut self, who: &str, what: &'static str, len: usize) -> usize {
        assert!(len > 0, "Dice::index needs len > 0");
        let v = self.rng.rand(len as i32 - 1);
        self.push(who, what, None, v, None);
        v as usize
    }

    /// A child generator for building things (a map, a face, a name, weather).
    /// Draws ONE logged value and seeds a plain Rng from it. The child's own
    /// draws are not logged. This keeps generators from shifting the rule rolls.
    pub fn generator(&mut self, what: &'static str) -> Rng {
        // Two 15-bit draws give a 30-bit value to seed from.
        let hi = self.rng.next_u32() as u64;
        let lo = self.rng.next_u32() as u64;
        let v = (hi << 15) | lo;
        self.push("generator", what, None, v as i32, None);
        Rng::new(seed32(v))
    }

    pub fn log(&self) -> &[Roll] { &self.log }

    pub fn take_log(&mut self) -> Vec<Roll> { std::mem::take(&mut self.log) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn odds_100_always_passes() {
        let mut d = Dice::from_seed(7);
        let n = (0..10_000).filter(|_| d.check("t", "x", 100)).count();
        assert_eq!(n, 10_000);
    }

    #[test]
    fn odds_0_never_passes() {
        let mut d = Dice::from_seed(7);
        let n = (0..10_000).filter(|_| d.check("t", "x", 0)).count();
        assert_eq!(n, 0);
    }

    #[test]
    fn same_seed_same_sequence() {
        let mut a = Dice::from_seed(42);
        let mut b = Dice::from_seed(42);
        for _ in 0..200 {
            assert_eq!(a.range("t", "x", 0, 1000), b.range("t", "x", 0, 1000));
        }
    }

    #[test]
    fn battles_differ() {
        let mut a = Dice::for_battle(99, 1);
        let mut b = Dice::for_battle(99, 2);
        let sa: Vec<i32> = (0..20).map(|_| a.range("t", "x", 0, 1000)).collect();
        let sb: Vec<i32> = (0..20).map(|_| b.range("t", "x", 0, 1000)).collect();
        assert_ne!(sa, sb);
        let mut c = Dice::for_battle(99, 1);
        let sc: Vec<i32> = (0..20).map(|_| c.range("t", "x", 0, 1000)).collect();
        assert_eq!(sa, sc);
    }

    #[test]
    fn seeds_one_and_two_differ() {
        let mut a = Dice::from_seed(1);
        let mut b = Dice::from_seed(2);
        let sa: Vec<i32> = (0..20).map(|_| a.range("t", "x", 0, 1000)).collect();
        let sb: Vec<i32> = (0..20).map(|_| b.range("t", "x", 0, 1000)).collect();
        assert_ne!(sa, sb);
    }

    #[test]
    fn every_call_adds_one_entry() {
        let mut d = Dice::from_seed(5);
        d.check("t", "a", 50);
        assert_eq!(d.log().len(), 1);
        d.check_roll("t", "b", 50);
        assert_eq!(d.log().len(), 2);
        d.range("t", "c", 0, 9);
        assert_eq!(d.log().len(), 3);
        d.index("t", "d", 5);
        assert_eq!(d.log().len(), 4);
        let _ = d.generator("e");
        assert_eq!(d.log().len(), 5);
        let taken = d.take_log();
        assert_eq!(taken.len(), 5);
        assert!(d.log().is_empty());
    }

    #[test]
    fn check_covers_0_to_99() {
        let mut d = Dice::from_seed(3);
        let mut seen = [false; 100];
        for _ in 0..20_000 { let (_, r) = d.check_roll("t", "x", 50); assert!((0..=99).contains(&r)); seen[r as usize] = true; }
        assert!(seen.iter().all(|s| *s));
    }
}
