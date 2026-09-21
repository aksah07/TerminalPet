use anyhow::Result;

pub fn handle_cli(args: &[String]) -> Result<()> {
    if args.len() == 1 {
        // No arguments: run interactive UI
        println!("🐾 Starting Unni's terminal...");
        println!("(Interactive UI coming in Phase 5!)");
        return Ok(());
    }

    let command = &args[1];
    match command.as_str() {
        "help" => show_help(),
        "status" => cmd_status(),
        "feed" => cmd_feed(),
        "play" => cmd_play(),
        "reset" => cmd_reset(),
        _ => {
            eprintln!("Unknown command: {}", command);
            let _ = show_help();
            Ok(())
        }
    }
}

fn show_help() -> Result<()> {
    println!("🐈 Digital Pet - Unni");
    println!();
    println!("Usage:");
    println!("  pet               Open interactive terminal UI");
    println!("  pet status        Show pet status");
    println!("  pet feed          Feed Unni");
    println!("  pet play          Play with Unni");
    println!("  pet reset         Reset pet (with confirmation)");
    println!("  pet help          Show this help");
    Ok(())
}

fn cmd_status() -> Result<()> {
    println!("📊 Pet Status (coming after Phase 3)");
    Ok(())
}

fn cmd_feed() -> Result<()> {
    println!("🍖 Feeding Unni... (coming after Phase 2)");
    Ok(())
}

fn cmd_play() -> Result<()> {
    println!("🎮 Playing with Unni... (coming after Phase 2)");
    Ok(())
}

fn cmd_reset() -> Result<()> {
    println!("⚠️  Reset Unni? This cannot be undone. (y/n)");
    println!("(Coming after Phase 3)");
    Ok(())
}
