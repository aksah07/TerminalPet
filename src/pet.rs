// Removed once the CLI and UI actually call everything in this file.
#![allow(dead_code)]

use chrono::{DateTime, Duration, Utc};

// Time passes in fixed-size steps ("ticks"). Only whole ticks are consumed, so
// leftover minutes are never lost. MAX_TICKS stops a months-long absence from
// looping forever; stats hit their limits long before that anyway.
const TICK_MINUTES: i64 = 30;
const MAX_TICKS: i64 = 200;

pub const MAX_STAT: u32 = 100;
pub const XP_PER_LEVEL: u32 = 100;
pub const DEFAULT_NAME: &str = "Unni";

/// Add a (possibly negative) amount to a stat and keep it within 0..=100.
///
/// Rust concept: `u32` cannot go below 0, so `stat - 30` would panic in debug
/// builds when stat is 10. We convert to `i32` (a signed number), do the math,
/// `clamp` it into range, then convert back. `as` is Rust's explicit cast.
fn adjust(stat: u32, delta: i32) -> u32 {
    (stat as i32 + delta).clamp(0, MAX_STAT as i32) as u32
}

/// All persistent data about the pet. Every stat is 0..=100 and higher is better
/// (so `hunger` really means "fullness": 100 = well fed, 0 = starving).
#[derive(Clone, Debug, PartialEq)]
pub struct Pet {
    pub name: String,
    pub happiness: u32,
    pub hunger: u32,
    pub energy: u32,
    pub health: u32,
    pub level: u32,
    pub xp: u32,
    pub created_at: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
}

impl Pet {
    pub fn new(name: &str) -> Self {
        let now = Utc::now();
        Pet {
            name: name.to_string(),
            happiness: 70,
            hunger: 70,
            energy: 80,
            health: 100,
            level: 1,
            xp: 0,
            created_at: now,
            last_updated: now,
        }
    }

    // Actions take `&mut self`: a mutable borrow, meaning "I may change this pet".
    // They return `true` if the pet levelled up, so the UI can announce it later.
    // They deliberately do NOT touch `last_updated`; Phase 4 owns time.

    pub fn feed(&mut self) -> bool {
        self.hunger = adjust(self.hunger, 30);
        self.happiness = adjust(self.happiness, 5);
        self.add_xp(5)
    }

    pub fn play(&mut self) -> bool {
        self.happiness = adjust(self.happiness, 15);
        self.energy = adjust(self.energy, -15);
        self.hunger = adjust(self.hunger, -8);
        self.add_xp(15)
    }

    pub fn sleep(&mut self) -> bool {
        self.energy = adjust(self.energy, 40);
        self.hunger = adjust(self.hunger, -10);
        self.add_xp(5)
    }

    pub fn pet(&mut self) -> bool {
        self.happiness = adjust(self.happiness, 8);
        self.add_xp(3)
    }

    /// Catch up on the time that passed since `last_updated`.
    /// `now` is a parameter (not `Utc::now()` inside) so tests can pick any time.
    pub fn apply_elapsed(&mut self, now: DateTime<Utc>) {
        // Clock moved backwards (timezone/NTP glitch): just resync, change nothing.
        if now < self.last_updated {
            self.last_updated = now;
            return;
        }

        // Integer division throws away the remainder: 50 minutes is 1 tick.
        let ticks = (now - self.last_updated).num_minutes() / TICK_MINUTES;
        for _ in 0..ticks.min(MAX_TICKS) {
            self.tick();
        }
        // Advance by whole ticks only, so the leftover minutes count next time.
        self.last_updated += Duration::minutes(ticks * TICK_MINUTES);
    }

    /// One 30-minute step of the world.
    fn tick(&mut self) {
        self.hunger = adjust(self.hunger, -3);
        self.energy = adjust(self.energy, 2);
        self.happiness = adjust(self.happiness, -1);

        if self.hunger == 0 {
            self.health = adjust(self.health, -5);
            self.happiness = adjust(self.happiness, -2);
        } else if self.hunger >= 40 {
            self.health = adjust(self.health, 1);
        }
    }

    /// Add XP, rolling over into levels. Returns true if at least one level was gained.
    fn add_xp(&mut self, amount: u32) -> bool {
        self.xp += amount;
        let mut leveled_up = false;
        while self.xp >= XP_PER_LEVEL {
            self.xp -= XP_PER_LEVEL;
            self.level += 1;
            leveled_up = true;
        }
        leveled_up
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn feed_raises_hunger_and_happiness() {
        let mut pet = Pet::new("Unni");
        pet.hunger = 40;
        pet.happiness = 50;
        pet.feed();
        assert_eq!(pet.hunger, 70);
        assert_eq!(pet.happiness, 55);
    }

    #[test]
    fn play_trades_energy_and_food_for_happiness() {
        let mut pet = Pet::new("Unni");
        let before = pet.clone();
        pet.play();
        assert!(pet.happiness > before.happiness);
        assert!(pet.energy < before.energy);
        assert!(pet.hunger < before.hunger);
    }

    #[test]
    fn sleep_restores_energy() {
        let mut pet = Pet::new("Unni");
        pet.energy = 10;
        pet.sleep();
        assert_eq!(pet.energy, 50);
    }

    #[test]
    fn petting_raises_happiness() {
        let mut pet = Pet::new("Unni");
        pet.happiness = 50;
        pet.pet();
        assert_eq!(pet.happiness, 58);
    }

    #[test]
    fn stats_never_exceed_100() {
        let mut pet = Pet::new("Unni");
        pet.hunger = 95;
        pet.happiness = 99;
        pet.feed();
        assert_eq!(pet.hunger, 100);
        assert_eq!(pet.happiness, 100);
    }

    #[test]
    fn stats_never_go_below_0() {
        let mut pet = Pet::new("Unni");
        pet.energy = 5;
        pet.hunger = 3;
        pet.play();
        assert_eq!(pet.energy, 0);
        assert_eq!(pet.hunger, 0);
    }

    #[test]
    fn xp_rolls_over_into_levels() {
        let mut pet = Pet::new("Unni");
        pet.xp = 95;
        let leveled = pet.feed(); // +5 xp -> exactly 100
        assert!(leveled);
        assert_eq!(pet.level, 2);
        assert_eq!(pet.xp, 0);
    }

    #[test]
    fn three_hours_away() {
        let mut pet = Pet::new("Unni");
        let start = pet.last_updated;
        pet.apply_elapsed(start + Duration::hours(3)); // 6 ticks
        assert_eq!(pet.hunger, 70 - 18);
        assert_eq!(pet.energy, 80 + 12);
        assert_eq!(pet.happiness, 70 - 6);
        assert_eq!(pet.health, 100);
        assert_eq!(pet.last_updated, start + Duration::hours(3));
    }

    #[test]
    fn less_than_one_tick_changes_nothing_and_keeps_the_time() {
        let mut pet = Pet::new("Unni");
        let before = pet.clone();
        pet.apply_elapsed(before.last_updated + Duration::minutes(29));
        assert_eq!(pet, before); // last_updated untouched, so the 29 minutes aren't lost
    }

    #[test]
    fn leftover_minutes_are_kept() {
        let mut pet = Pet::new("Unni");
        let start = pet.last_updated;
        pet.apply_elapsed(start + Duration::minutes(50)); // 1 tick + 20 spare minutes
        assert_eq!(pet.hunger, 67);
        assert_eq!(pet.last_updated, start + Duration::minutes(30));
    }

    #[test]
    fn starving_hurts_health() {
        let mut pet = Pet::new("Unni");
        pet.hunger = 5;
        let start = pet.last_updated;
        pet.apply_elapsed(start + Duration::hours(3));
        assert_eq!(pet.hunger, 0);
        assert_eq!(pet.health, 75); // starving for ticks 2..=6, -5 each
    }

    #[test]
    fn very_long_absence_stays_in_range() {
        let mut pet = Pet::new("Unni");
        let start = pet.last_updated;
        pet.apply_elapsed(start + Duration::days(365));
        assert_eq!(pet.hunger, 0);
        assert_eq!(pet.energy, 100);
        assert_eq!(pet.health, 0);
        assert_eq!(pet.last_updated, start + Duration::days(365));
    }

    #[test]
    fn clock_going_backwards_is_harmless() {
        let mut pet = Pet::new("Unni");
        let before = pet.clone();
        let earlier = pet.last_updated - Duration::hours(2);
        pet.apply_elapsed(earlier);
        assert_eq!(pet.hunger, before.hunger);
        assert_eq!(pet.last_updated, earlier);
    }

    #[test]
    fn no_level_up_reports_false() {
        let mut pet = Pet::new("Unni");
        assert!(!pet.pet());
        assert_eq!(pet.level, 1);
    }
}
