// Removed once the CLI and UI actually call everything in this file.
#![allow(dead_code)]

use chrono::{DateTime, Utc};

pub const MAX_STAT: u32 = 100;
pub const XP_PER_LEVEL: u32 = 100;

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
#[derive(Clone, Debug)]
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
    fn no_level_up_reports_false() {
        let mut pet = Pet::new("Unni");
        assert!(!pet.pet());
        assert_eq!(pet.level, 1);
    }
}
