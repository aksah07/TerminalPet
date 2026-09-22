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
    /// Feedback from the most recent action (e.g. "Unni munches happily!").
    /// Replaced by the next action; there's no timer, so it stays until then.
    pub last_message: Option<String>,
    pub show_info: bool,
}

impl App {
    pub fn new(pet: Pet) -> Self {
        App {
            pet,
            running: true,
            last_message: None,
            show_info: false,
        }
    }

    pub fn quit(&mut self) {
        self.running = false;
    }
}

/// Runs one pet action, records a message for the UI, and saves the result.
///
/// `action` is a function pointer — `Pet::feed` has type `fn(&mut Pet) -> bool`,
/// the same signature `feed`, `play`, `sleep` and `pet` all share, so one
/// helper can drive any of them instead of repeating this block four times.
fn apply_action(
    app: &mut App,
    db: &Database,
    action: fn(&mut Pet) -> bool,
    template: &str,
) -> Result<()> {
    let leveled_up = action(&mut app.pet);
    let mut message = template.replace("{name}", &app.pet.name);
    if leveled_up {
        message.push_str(&format!(" Level up! Now level {}.", app.pet.level));
    }
    app.last_message = Some(message);
    db.save_pet(&app.pet)?;
    Ok(())
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

    let result = event_loop(&mut terminal, &mut app, db);

    // Restore the terminal before propagating any error from the loop, so a
    // failure never leaves the user's shell in raw/alternate-screen mode.
    restore_terminal(&mut terminal)?;
    db.save_pet(&app.pet)?;
    result
}

fn event_loop(terminal: &mut Term, app: &mut App, db: &Database) -> Result<()> {
    while app.running {
        terminal.draw(|f| ui::draw(f, app))?;

        let event = events::poll_event(Duration::from_millis(250))?;
        if let Some(AppEvent::Key(code, modifiers)) = event {
            let ctrl_c = modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('c');
            match code {
                KeyCode::Char('q' | 'Q') | KeyCode::Esc => app.quit(),
                _ if ctrl_c => app.quit(),
                KeyCode::Char('f' | 'F') => {
                    apply_action(app, db, Pet::feed, "{name} munches happily!")?
                }
                KeyCode::Char('p' | 'P') => {
                    apply_action(app, db, Pet::play, "{name} had fun playing!")?
                }
                KeyCode::Char('s' | 'S') => {
                    apply_action(app, db, Pet::sleep, "{name} wakes up feeling rested.")?
                }
                KeyCode::Char('a' | 'A') => {
                    apply_action(app, db, Pet::pet, "{name} purrs contentedly.")?
                }
                KeyCode::Char('i' | 'I') => app.show_info = !app.show_info,
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
