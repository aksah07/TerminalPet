mod cli;
mod pet;
mod database;
mod app;
mod ui;
mod events;

use anyhow::Result;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    cli::handle_cli(&args)
}
