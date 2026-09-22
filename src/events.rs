use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};

/// The small set of things the app loop reacts to. A real project might grow
/// a `Tick` variant here too; we don't need one yet.
pub enum AppEvent {
    Key(KeyCode, KeyModifiers),
}

/// Wait up to `timeout` for a key press. `Ok(None)` means nothing happened in
/// that time — that's what lets the app loop wake up and redraw periodically
/// instead of blocking forever on one key.
pub fn poll_event(timeout: Duration) -> Result<Option<AppEvent>> {
    if event::poll(timeout)? {
        if let Event::Key(key) = event::read()? {
            // Some terminals report both a press and a release; only act on press,
            // or a key held down could fire the same action twice.
            if key.kind == KeyEventKind::Press {
                return Ok(Some(AppEvent::Key(key.code, key.modifiers)));
            }
        }
    }
    Ok(None)
}
