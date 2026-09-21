use chrono::{DateTime, Utc};

/// Represents the pet's state.
/// This struct holds all persistent data about Mochi.
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct Pet {
    pub name: String,
    pub happiness: u32,    // 0-100
    pub hunger: u32,       // 0-100 (0 = full, 100 = starving)
    pub energy: u32,       // 0-100
    pub health: u32,       // 0-100
    pub level: u32,
    pub xp: u32,
    pub created_at: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
}

#[allow(dead_code)]
impl Pet {
    /// Create a new pet with default stats.
    pub fn new(name: String) -> Self {
        let now = Utc::now();
        Pet {
            name,
            happiness: 50,
            hunger: 50,
            energy: 80,
            health: 100,
            level: 1,
            xp: 0,
            created_at: now,
            last_updated: now,
        }
    }

    // Action: Feed the pet
    pub fn feed(&mut self) {
        self.hunger = self.hunger.saturating_sub(30);
        self.happiness = (self.happiness + 5).min(100);
        self.last_updated = Utc::now();
    }

    // Action: Play with the pet
    pub fn play(&mut self) {
        self.happiness = (self.happiness + 20).min(100);
        self.energy = self.energy.saturating_sub(15);
        self.hunger = (self.hunger + 5).min(100);
        self.xp = self.xp + 10;
        self.last_updated = Utc::now();
    }

    // Action: Let the pet sleep
    pub fn sleep(&mut self) {
        self.energy = (self.energy + 40).min(100);
        self.hunger = (self.hunger + 10).min(100);
        self.last_updated = Utc::now();
    }

    // Action: Pet/interact with the pet
    pub fn pet(&mut self) {
        self.happiness = (self.happiness + 10).min(100);
        self.last_updated = Utc::now();
    }

    // Clamp all stats to valid ranges (0-100)
    pub fn clamp_stats(&mut self) {
        self.happiness = self.happiness.min(100);
        self.hunger = self.hunger.min(100);
        self.energy = self.energy.min(100);
        self.health = self.health.min(100);
    }

    // Get a status message based on pet's current state
    pub fn get_status_message(&self) -> String {
        if self.happiness > 70 {
            "I'm feeling great!".to_string()
        } else if self.happiness < 30 {
            "I'm sad...".to_string()
        } else if self.hunger > 80 {
            "I'm so hungry!".to_string()
        } else if self.energy < 20 {
            "I need sleep...".to_string()
        } else {
            "I'm doing okay.".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feed() {
        let mut pet = Pet::new("Mochi".to_string());
        pet.hunger = 100;
        pet.feed();
        assert_eq!(pet.hunger, 70);
        assert!(pet.happiness > 50);
    }

    #[test]
    fn test_play() {
        let mut pet = Pet::new("Mochi".to_string());
        let initial_energy = pet.energy;
        pet.play();
        assert!(pet.happiness > 50);
        assert!(pet.energy < initial_energy);
        assert!(pet.xp > 0);
    }

    #[test]
    fn test_stats_clamped() {
        let mut pet = Pet::new("Mochi".to_string());
        pet.happiness = 1000;
        pet.clamp_stats();
        assert_eq!(pet.happiness, 100);
    }
}
