use std::io::{self, Write};

use anyhow::{bail, Result};

use crate::app;
use crate::database::Database;
use crate::personality;
use crate::pet::{Pet, DEFAULT_NAME, XP_PER_LEVEL};

pub fn handle_cli(args: &[String]) -> Result<()> {
    // args[0] is the program name; args.get(1) is Some(command) or None.
    match args.get(1).map(String::as_str) {
        None => {
            let db = Database::open()?;
            let pet = db.load_current()?;
            app::run(&db, pet)
        }
        Some("help") => {
            show_help();
            Ok(())
        }
        Some("status") => cmd_status(),
        Some("feed") => cmd_feed(),
        Some("play") => cmd_play(),
        Some("reset") => cmd_reset(),
        Some(other) => bail!("unknown command `{other}` (try `pet help`)"),
    }
}

fn show_help() {
    println!("Digital Pet - {DEFAULT_NAME}");
    println!();
    println!("Usage:");
    println!("  pet               Open interactive terminal UI");
    println!("  pet status        Show pet status");
    println!("  pet feed          Feed {DEFAULT_NAME}");
    println!("  pet play          Play with {DEFAULT_NAME}");
    println!("  pet reset         Reset pet (with confirmation)");
    println!("  pet help          Show this help");
}

fn print_status(pet: &Pet) {
    println!(
        "{} - level {} ({}/{} xp)",
        pet.name, pet.level, pet.xp, XP_PER_LEVEL
    );
    println!("  Happiness {:>3}", pet.happiness);
    println!("  Hunger    {:>3}", pet.hunger);
    println!("  Energy    {:>3}", pet.energy);
    println!("  Health    {:>3}", pet.health);
    println!("  \"{}\"", personality::message(pet));
}

fn cmd_status() -> Result<()> {
    let db = Database::open()?;
    let pet = db.load_current()?;
    print_status(&pet);
    Ok(())
}

fn cmd_feed() -> Result<()> {
    let db = Database::open()?;
    let mut pet = db.load_current()?;
    let leveled_up = pet.feed();
    db.save_pet(&pet)?;
    println!("{} munches happily.", pet.name);
    if leveled_up {
        println!("Level up! {} is now level {}.", pet.name, pet.level);
    }
    print_status(&pet);
    Ok(())
}

fn cmd_play() -> Result<()> {
    let db = Database::open()?;
    let mut pet = db.load_current()?;
    let leveled_up = pet.play();
    db.save_pet(&pet)?;
    println!("{} plays and gets a little tired.", pet.name);
    if leveled_up {
        println!("Level up! {} is now level {}.", pet.name, pet.level);
    }
    print_status(&pet);
    Ok(())
}

fn cmd_reset() -> Result<()> {
    print!("Reset {DEFAULT_NAME}? All progress will be lost. [y/N] ");
    // print! doesn't end the line, so stdout may wait before showing it; flush forces it out.
    io::stdout().flush()?;

    let mut answer = String::new();
    io::stdin().read_line(&mut answer)?;

    if !matches!(answer.trim().to_lowercase().as_str(), "y" | "yes") {
        println!("Cancelled. Nothing was changed.");
        return Ok(());
    }

    let db = Database::open()?;
    db.delete_pet()?;
    let pet = db.load_or_create()?;
    println!("{} has been reset to a fresh start.", pet.name);
    Ok(())
}
