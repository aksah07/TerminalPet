use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use rusqlite::{params, Connection, OptionalExtension};

use crate::pet::{Pet, DEFAULT_NAME};

// The whole schema: one table with (at most) one row. `id = 1` is enforced by a
// CHECK constraint, so the database itself refuses a second pet. The other CHECKs
// mean even a hand-edited database can't hold an out-of-range stat.
// Timestamps are stored as readable text (RFC 3339, e.g. 2026-09-22T10:00:00+00:00).
const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS pet (
    id           INTEGER PRIMARY KEY CHECK (id = 1),
    name         TEXT    NOT NULL,
    happiness    INTEGER NOT NULL CHECK (happiness BETWEEN 0 AND 100),
    hunger       INTEGER NOT NULL CHECK (hunger    BETWEEN 0 AND 100),
    energy       INTEGER NOT NULL CHECK (energy    BETWEEN 0 AND 100),
    health       INTEGER NOT NULL CHECK (health    BETWEEN 0 AND 100),
    level        INTEGER NOT NULL CHECK (level >= 1),
    xp           INTEGER NOT NULL CHECK (xp >= 0),
    created_at   TEXT    NOT NULL,
    last_updated TEXT    NOT NULL
);";

/// Owns the SQLite connection. The connection is closed automatically when a
/// `Database` goes out of scope (Rust calls this "dropping" a value).
pub struct Database {
    conn: Connection,
}

impl Database {
    /// Open the real database file, creating the folder and file on first launch.
    pub fn open() -> Result<Self> {
        let path = default_path()?;
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)
                .with_context(|| format!("could not create folder {}", dir.display()))?;
        }
        Self::open_at(&path)
    }

    fn open_at(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)
            .with_context(|| format!("could not open database at {}", path.display()))?;
        Self::with_connection(conn)
    }

    fn with_connection(conn: Connection) -> Result<Self> {
        // IF NOT EXISTS makes this safe to run on every launch.
        conn.execute_batch(SCHEMA)?;
        Ok(Database { conn })
    }

    /// Returns `None` if no pet has been saved yet. `Option` is Rust's way of
    /// saying "there might be nothing here", instead of using null.
    pub fn load_pet(&self) -> Result<Option<Pet>> {
        let pet = self
            .conn
            .query_row(
                "SELECT name, happiness, hunger, energy, health, level, xp,
                        created_at, last_updated
                 FROM pet WHERE id = 1",
                [],
                // A closure (an inline function) that turns one SQL row into a Pet.
                // `row.get(n)?` reads column n; the field type tells rusqlite what to convert to.
                |row| {
                    Ok(Pet {
                        name: row.get(0)?,
                        happiness: row.get(1)?,
                        hunger: row.get(2)?,
                        energy: row.get(3)?,
                        health: row.get(4)?,
                        level: row.get(5)?,
                        xp: row.get(6)?,
                        created_at: row.get(7)?,
                        last_updated: row.get(8)?,
                    })
                },
            )
            // query_row errors if there are zero rows; .optional() turns that into Ok(None).
            .optional()?;
        Ok(pet)
    }

    /// Insert the pet, or update it if it already exists (an "upsert").
    /// `?1`, `?2`... are placeholders filled from `params!`; never build SQL by
    /// gluing strings together, because placeholders prevent SQL injection.
    pub fn save_pet(&self, pet: &Pet) -> Result<()> {
        self.conn.execute(
            "INSERT INTO pet (id, name, happiness, hunger, energy, health, level, xp,
                              created_at, last_updated)
             VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(id) DO UPDATE SET
                 name = excluded.name,
                 happiness = excluded.happiness,
                 hunger = excluded.hunger,
                 energy = excluded.energy,
                 health = excluded.health,
                 level = excluded.level,
                 xp = excluded.xp,
                 last_updated = excluded.last_updated",
            params![
                pet.name,
                pet.happiness,
                pet.hunger,
                pet.energy,
                pet.health,
                pet.level,
                pet.xp,
                pet.created_at,
                pet.last_updated,
            ],
        )?;
        Ok(())
    }

    /// Load the saved pet, or hatch a brand-new one on the very first launch.
    pub fn load_or_create(&self) -> Result<Pet> {
        match self.load_pet()? {
            Some(pet) => Ok(pet),
            None => {
                let pet = Pet::new(DEFAULT_NAME);
                self.save_pet(&pet)?;
                Ok(pet)
            }
        }
    }

    pub fn delete_pet(&self) -> Result<()> {
        self.conn.execute("DELETE FROM pet", [])?;
        Ok(())
    }
}

/// Follows the Linux convention: $XDG_DATA_HOME, or ~/.local/share if unset.
/// Result: ~/.local/share/unni/pet.db
fn default_path() -> Result<PathBuf> {
    let base = match std::env::var_os("XDG_DATA_HOME") {
        Some(dir) => PathBuf::from(dir),
        None => {
            let home = std::env::var_os("HOME").context("HOME is not set")?;
            PathBuf::from(home).join(".local").join("share")
        }
    };
    Ok(base.join("unni").join("pet.db"))
}

#[cfg(test)]
mod tests {
    use super::*;

    // An in-memory database is thrown away after each test, so tests never
    // touch your real pet.
    fn test_db() -> Database {
        Database::with_connection(Connection::open_in_memory().unwrap()).unwrap()
    }

    #[test]
    fn empty_database_has_no_pet() {
        assert_eq!(test_db().load_pet().unwrap(), None);
    }

    #[test]
    fn save_then_load_round_trips() {
        let db = test_db();
        let mut pet = Pet::new("Unni");
        pet.feed();
        pet.play();
        db.save_pet(&pet).unwrap();
        assert_eq!(db.load_pet().unwrap(), Some(pet));
    }

    #[test]
    fn saving_twice_updates_instead_of_duplicating() {
        let db = test_db();
        let mut pet = Pet::new("Unni");
        db.save_pet(&pet).unwrap();
        pet.pet();
        db.save_pet(&pet).unwrap();
        let count: i64 = db
            .conn
            .query_row("SELECT COUNT(*) FROM pet", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1);
        assert_eq!(db.load_pet().unwrap(), Some(pet));
    }

    #[test]
    fn load_or_create_makes_a_pet_once() {
        let db = test_db();
        let first = db.load_or_create().unwrap();
        let second = db.load_or_create().unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn delete_removes_the_pet() {
        let db = test_db();
        db.load_or_create().unwrap();
        db.delete_pet().unwrap();
        assert_eq!(db.load_pet().unwrap(), None);
    }

    #[test]
    fn database_rejects_out_of_range_stats() {
        let db = test_db();
        let mut pet = Pet::new("Unni");
        pet.happiness = 500;
        assert!(db.save_pet(&pet).is_err());
    }
}
