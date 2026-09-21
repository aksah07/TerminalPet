use crate::pet::Pet;
use anyhow::Result;

/// Initialize the database and return a Pet.
/// Phase 3: This will create SQLite tables and load/save pet data.
#[allow(dead_code)]
pub fn init() -> Result<Pet> {
    // For now, just create a new pet
    Ok(Pet::new("Unni"))
}

/// Save pet to database.
/// Phase 3: Will persist to SQLite.
#[allow(dead_code)]
pub fn save_pet(_pet: &Pet) -> Result<()> {
    Ok(())
}

/// Load pet from database.
/// Phase 3: Will read from SQLite.
#[allow(dead_code)]
pub fn load_pet() -> Result<Option<Pet>> {
    Ok(None)
}
