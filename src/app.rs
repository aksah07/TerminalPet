use crate::pet::Pet;

/// The main application state.
/// Phase 5: Will run the event loop and manage TUI state.
#[allow(dead_code)]
pub struct App {
    pub pet: Pet,
    pub running: bool,
}

impl App {
    #[allow(dead_code)]
    pub fn new(pet: Pet) -> Self {
        App { pet, running: true }
    }

    #[allow(dead_code)]
    pub fn quit(&mut self) {
        self.running = false;
    }
}
