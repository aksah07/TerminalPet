mod app;
mod cli;
mod database;
mod events;
mod personality;
mod pet;
mod ui;

use anyhow::Result;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    cli::handle_cli(&args)
}
