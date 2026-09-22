# Unni — a terminal pet

A small digital pet that lives in your Linux terminal. It's a Rust +
Ratatui + SQLite learning project — small on purpose, so every part of
it can be understood, not just run.

```
┌ 🐈 UNNI ─────────────────────────────────────────────────────────────┐
│                                 /\_/\                                │
│                                ( o.o )                               │
│                                 > ^ <                                │
│                                                                      │
│Happiness  ██████████████████████████████████████                 70 │
│Hunger     ██████████████████████████████████████                 70 │
│Energy     ███████████████████████████████████████████            80 │
│Health     ██████████████████████████████████████████████████████100 │
│                                                                      │
│Level 1                                                     0/100     │
│                                                                      │
│                           "I'm doing okay."                          │
│──────────────────────────────────────────────────────────────────────│
│      [F] Feed  [P] Play  [S] Sleep  [A] Pet  [I] Info  [Q] Quit      │
└──────────────────────────────────────────────────────────────────────┘
```

## Installation

Requires Rust (install via [rustup](https://rustup.rs) if you don't
have it).

```bash
git clone https://github.com/aksah07/TerminalPet.git
cd TerminalPet
cargo build --release
```

To run `pet` from anywhere, put the built binary on your `PATH`:

```bash
cargo install --path .
```

## Usage

```
pet               Open the interactive terminal UI
pet status        Print stats without opening the UI
pet feed          Feed Unni directly
pet play          Play with Unni directly
pet reset         Reset Unni, with a y/n confirmation
pet help          Show available commands
```

Inside the interactive UI: `F` feed, `P` play, `S` sleep, `A` pet/pat,
`I` toggle an info panel, `Q` or `Ctrl+C` quit.

Unni's data lives at `~/.local/share/unni/pet.db` (or
`$XDG_DATA_HOME/unni/pet.db` if that's set). Closing the app doesn't
pause Unni — elapsed real time is applied the next time you open it,
in fixed 30-minute steps (see `Pet::apply_elapsed` in `src/pet.rs`).

## Architecture

The project is one crate, split into modules by responsibility:

| Module | Responsibility |
|---|---|
| [`main.rs`](src/main.rs) | Entry point; hands off to `cli` |
| [`cli.rs`](src/cli.rs) | Parses `pet <command>`, dispatches to the right handler |
| [`pet.rs`](src/pet.rs) | The `Pet` struct: stats, actions (`feed`/`play`/`sleep`/`pet`), and the time-catch-up logic |
| [`database.rs`](src/database.rs) | SQLite schema, load/save, the "one pet, one row" table |
| [`personality.rs`](src/personality.rs) | Picks a mood message: deterministic category, randomized line |
| [`app.rs`](src/app.rs) | The interactive UI's state and event loop; terminal setup/teardown |
| [`ui.rs`](src/ui.rs) | Pure rendering: turns `App` state into a Ratatui frame |
| [`events.rs`](src/events.rs) | Polls Crossterm for keyboard input with a timeout |

**Data flow for the interactive UI:**

```
Database::load_current()   (load saved pet, apply elapsed time, save)
        │
        ▼
   App::new(pet)
        │
        ▼
┌──────────────────────────┐
│  loop while app.running: │
│    ui::draw(app)          │◄── reads App, never mutates it
│    events::poll_event()   │
│    if action key: mutate  │──► Pet::feed/play/sleep/pet(),
│      app.pet, save to db  │    Database::save_pet()
└──────────────────────────┘
```

The UI module never touches the database, and the database module
never imports Ratatui — each module only knows about the layer below
it.

### Why SQLite for one pet?

A single pet only needs one row, so a flat file (JSON, TOML) would
have worked too. SQLite was the point of the exercise: real
`CREATE TABLE` / `INSERT ... ON CONFLICT` / `CHECK` constraints, and
practice with `rusqlite`'s query API, rather than a bigger app design.
The `CHECK (happiness BETWEEN 0 AND 100)` constraints mean even a
hand-edited database can't produce a stat out of range — the database
enforces the same invariant the Rust code does.

## Development

```bash
cargo fmt      # format
cargo check    # fast type-check
cargo test     # unit tests (25, covering pet actions, time catch-up,
               # database round-trips, and personality selection)
cargo clippy   # lint
cargo run      # launch the interactive UI
```

## What I learned

This project was built in eight incremental phases — CLI skeleton,
`Pet` struct and actions, SQLite persistence, time-based catch-up, the
Ratatui layout, wiring keyboard input to actions, a personality
system, and a final polish pass. Each phase compiled, tested, and ran
before the next one started. The result is a working pet, but more
importantly it's a working map of how `Result`/`?`, ownership and
borrowing, SQLite schemas, and a real-time event loop fit together —
see the "Things I should understand" list below for the specific
concepts worth being able to explain.
