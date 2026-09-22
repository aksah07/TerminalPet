use chrono::Utc;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Gauge, Paragraph};
use ratatui::Frame;

use crate::app::App;
use crate::pet::{Pet, XP_PER_LEVEL};

const CAT_ART: &str = "\n/\\_/\\\n( o.o )\n > ^ <";

/// Draws one frame. This function owns no state — it just reads `app` and
/// issues render calls. Ratatui recomputes this layout from the current
/// terminal size every call, which is what makes resizing work for free.
pub fn draw(f: &mut Frame, app: &App) {
    let pet = &app.pet;
    let area = f.area();

    let outer = Block::bordered().title(format!(" {} ", pet.name.to_uppercase()));
    let inner = outer.inner(area);
    f.render_widget(outer, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // ascii art
            Constraint::Length(1), // spacer
            Constraint::Length(1), // happiness
            Constraint::Length(1), // hunger
            Constraint::Length(1), // energy
            Constraint::Length(1), // health
            Constraint::Length(1), // spacer
            Constraint::Length(1), // level / xp
            Constraint::Length(1), // spacer
            Constraint::Length(1), // message
            Constraint::Min(0),    // filler: absorbs any extra height
            Constraint::Length(1), // divider
            Constraint::Length(1), // controls
        ])
        .split(inner);

    f.render_widget(
        Paragraph::new(CAT_ART).alignment(Alignment::Center),
        rows[0],
    );

    stat_bar(f, rows[2], "Happiness", pet.happiness, Color::Magenta);
    stat_bar(f, rows[3], "Hunger", pet.hunger, Color::Yellow);
    stat_bar(f, rows[4], "Energy", pet.energy, Color::Cyan);
    stat_bar(f, rows[5], "Health", pet.health, Color::Green);

    level_bar(f, rows[7], pet);

    let message_text = match &app.last_message {
        Some(msg) => msg.clone(),
        None => format!("\"{}\"", crate::personality::message(pet)),
    };
    let message = Paragraph::new(message_text)
        .style(Style::default().add_modifier(Modifier::ITALIC))
        .alignment(Alignment::Center);
    f.render_widget(message, rows[9]);

    if app.show_info {
        f.render_widget(info_panel(pet), rows[10]);
    }

    f.render_widget(Paragraph::new("-".repeat(inner.width as usize)), rows[11]);
    f.render_widget(
        Paragraph::new("[F] Feed  [P] Play  [S] Sleep  [A] Pet  [I] Info  [Q] Quit")
            .alignment(Alignment::Center),
        rows[12],
    );
}

/// One "label [bar] value" row. Splitting the row into three columns ourselves
/// is simpler than fighting Gauge's built-in centered label for this layout.
fn stat_bar(f: &mut Frame, area: Rect, label: &str, value: u32, color: Color) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(11),
            Constraint::Min(0),
            Constraint::Length(5),
        ])
        .split(area);

    f.render_widget(Paragraph::new(label), cols[0]);
    f.render_widget(
        Gauge::default()
            .gauge_style(Style::default().fg(color))
            .ratio(value as f64 / 100.0)
            .label(""),
        cols[1],
    );
    f.render_widget(Paragraph::new(format!("{value:>3}")), cols[2]);
}

fn level_bar(f: &mut Frame, area: Rect, pet: &Pet) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(11),
            Constraint::Min(0),
            Constraint::Length(10),
        ])
        .split(area);

    f.render_widget(Paragraph::new(format!("Level {}", pet.level)), cols[0]);
    f.render_widget(
        Gauge::default()
            .gauge_style(Style::default().fg(Color::Blue))
            .ratio(pet.xp as f64 / XP_PER_LEVEL as f64)
            .label(""),
        cols[1],
    );
    f.render_widget(
        Paragraph::new(format!("{}/{}", pet.xp, XP_PER_LEVEL)),
        cols[2],
    );
}

/// The `[I]` panel: details that don't fit the always-on stat bars.
fn info_panel(pet: &Pet) -> Paragraph<'static> {
    let age = Utc::now() - pet.created_at;
    let text = format!(
        "Born {}  |  Alive for {} day(s), {} hour(s)  |  XP {}/{}",
        pet.created_at.format("%Y-%m-%d %H:%M"),
        age.num_days(),
        age.num_hours() % 24,
        pet.xp,
        XP_PER_LEVEL,
    );
    Paragraph::new(text)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center)
}
