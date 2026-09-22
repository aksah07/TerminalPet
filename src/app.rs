use std::io::{self, Stdout};
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{KeyCode, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::database::Database;
use crate::events::{self, AppEvent};
use crate::pet::Pet;
use crate::ui;

/// The state the UI reads from each frame.
pub struct App {
    pub pet: Pet,
    pub running: bool,
}

impl App {
    pub fn new(pet: Pet) -> Self {
        App { pet, running: true }
    }

    pub fn quit(&mut self) {
        self.running = false;
    }
}

// A type alias: writing `Term` instead of this full generic type everywhere.
type Term = Terminal<CrosstermBackend<Stdout>>;

/// Entry point for the interactive UI. Sets the terminal into "TUI mode",
/// runs the event loop, and always puts the terminal back afterwards — even
/// if the loop returns an error.
pub fn run(db: &Database, pet: Pet) -> Result<()> {
    install_panic_hook();
    let mut terminal = setup_terminal()?;
    let mut app = App::new(pet);

    let result = event_loop(&mut terminal, &mut app);

    // Restore the terminal before propagating any error from the loop, so a
    // failure never leaves the user's shell in raw/alternate-screen mode.
    restore_terminal(&mut terminal)?;
    db.save_pet(&app.pet)?;
    result
}

fn event_loop(terminal: &mut Term, app: &mut App) -> Result<()> {
    while app.running {
        terminal.draw(|f| ui::draw(f, app))?;

        let event = events::poll_event(Duration::from_millis(250))?;
        if let Some(AppEvent::Key(code, modifiers)) = event {
            let ctrl_c = modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('c');
            match code {
                KeyCode::Char('q' | 'Q') | KeyCode::Esc => app.quit(),
                _ if ctrl_c => app.quit(),
                // Phase 6 adds F / P / S / A / I here, calling app.pet's actions.
                _ => {}
            }
        }
    }
    Ok(())
}

fn setup_terminal() -> Result<Term> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    Ok(Terminal::new(CrosstermBackend::new(stdout))?)
}

fn restore_terminal(terminal: &mut Term) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

/// If the app panics while the terminal is in raw/alternate-screen mode, the
/// user's shell is left broken until they type `reset` blind. This hook
/// restores the terminal first, then hands off to Rust's normal panic report.
fn install_panic_hook() {
    let original = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        original(panic_info);
    }));
}
