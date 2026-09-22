use rand::seq::SliceRandom;

use crate::pet::Pet;

// A level of 5+ means roughly 30-50 actions taken, so it's a real milestone
// without taking too long to reach.
const HIGH_LEVEL_THRESHOLD: u32 = 5;

const HUNGRY: &[&str] = &["I could really use some food...", "Do you have snacks?"];
const TIRED: &[&str] = &["...sleep...", "I'm exhausted."];
const HIGH_LEVEL: &[&str] = &["Look how strong I've become!"];
const HAPPY: &[&str] = &["Today is a good day.", "Can we play?", "I'm feeling great!"];
const NEUTRAL: &[&str] = &["I'm doing okay.", "Just another day."];

/// Picks a message for the pet's current mood.
///
/// Which POOL is used is decided by simple, fixed rules (deterministic, in
/// priority order: hungry beats tired beats a level milestone beats being
/// happy, with a neutral fallback). Which MESSAGE from that pool is picked is
/// randomized, so the same mood doesn't always print the exact same line.
pub fn message(pet: &Pet) -> &'static str {
    let pool = if pet.hunger < 20 {
        HUNGRY
    } else if pet.energy < 20 {
        TIRED
    } else if pet.level >= HIGH_LEVEL_THRESHOLD {
        HIGH_LEVEL
    } else if pet.happiness > 70 {
        HAPPY
    } else {
        NEUTRAL
    };

    // Every pool above is a non-empty const, so choose() can't return None.
    pool.choose(&mut rand::thread_rng()).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hungry_outranks_everything_else() {
        let mut pet = Pet::new("Unni");
        pet.hunger = 5;
        pet.energy = 5;
        pet.level = 10;
        pet.happiness = 90;
        assert!(HUNGRY.contains(&message(&pet)));
    }

    #[test]
    fn tired_outranks_level_and_happiness() {
        let mut pet = Pet::new("Unni");
        pet.hunger = 50;
        pet.energy = 5;
        pet.level = 10;
        pet.happiness = 90;
        assert!(TIRED.contains(&message(&pet)));
    }

    #[test]
    fn high_level_shown_when_well_fed_and_rested() {
        let mut pet = Pet::new("Unni");
        pet.hunger = 50;
        pet.energy = 50;
        pet.level = HIGH_LEVEL_THRESHOLD;
        assert!(HIGH_LEVEL.contains(&message(&pet)));
    }

    #[test]
    fn happy_shown_when_happiness_is_high() {
        let mut pet = Pet::new("Unni");
        pet.hunger = 50;
        pet.energy = 50;
        pet.level = 1;
        pet.happiness = 90;
        assert!(HAPPY.contains(&message(&pet)));
    }

    #[test]
    fn neutral_is_the_fallback() {
        let mut pet = Pet::new("Unni");
        pet.hunger = 50;
        pet.energy = 50;
        pet.level = 1;
        pet.happiness = 50;
        assert!(NEUTRAL.contains(&message(&pet)));
    }
}
