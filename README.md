# Taskdeck

Taskdeck is a local-first terminal task dashboard for developers. The interface is designed for a wide terminal, with a compact column set on narrower windows. Filtering and sorting retain selection by task ID, and table changes are blended with tachyonfx instead of snapping between screens.

## Install

**macOS / Linux** (prebuilt, no Rust toolchain required):

```sh
curl -sSf https://github.com/aakash-env/TODO-terminal/releases/latest/download/taskdeck-installer.sh | sh
```

**Windows PowerShell** (prebuilt):

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/aakash-env/TODO-terminal/releases/latest/download/taskdeck-installer.ps1 | iex"
```

**Homebrew:**

```sh
brew install aakash-env/tap/taskdeck
```

**Rust toolchain fallback** (install directly from Git; crates.io publishing is not configured):

```sh
cargo install --git https://github.com/aakash-env/TODO-terminal.git
```

**Manual download:** choose the archive for your platform from the [GitHub Releases](https://github.com/aakash-env/TODO-terminal/releases) page.

Homebrew releases require the `aakash-env/homebrew-tap` repository and a `HOMEBREW_TAP_TOKEN` GitHub Actions secret with write access to it.

## Requirements

- Rust 1.88 or newer
- A terminal with Unicode block-character support

## Build and run

```powershell
cargo build --release
cargo run --release
cargo run -- --filter running --sort priority
cargo run -- --add "Prepare incident review"
cargo run -- --db-path .\scratch\tasks.db
```

The first launch creates `~/.config/taskdeck/config.toml` and `~/.config/taskdeck/tasks.db`. The database is seeded with a varied demo set only when it contains no tasks. `--db-path` selects a different SQLite file. A task passed with `--add` is inserted before the dashboard opens.

## Controls

| Key | Action |
| --- | --- |
| `j` / `k`, arrows | Move selection |
| `f` | Cycle all / running / pending / done |
| `o` | Cycle created / deadline / priority order |
| `/` | Search titles with a fuzzy subsequence match |
| `a` | Add a task inline |
| `space` / `enter` | Cycle task state |
| `d` | Confirm and delete selected task |
| `?` | Help |
| `esc` | Cancel input or clear search |
| `q` | Quit |

## Configuration

The generated TOML file exposes named color values as `#RRGGBB` strings and customizable single-character bindings. For example:

```toml
[theme]
accent = "#4cccb7"
urgent = "#ff5c68"
selected_background = "#1f3436"

[keybindings]
down = "j"
up = "k"
filter = "f"
sort = "o"
search = "/"
add = "a"
delete = "d"
cycle_state = " "
help = "?"
quit = "q"
```

All configuration colors fall back to named constants in `src/theme.rs` if a value is malformed.

## Architecture and decisions

- `app.rs` owns view state, selection, filters, sorting, search, and state transitions. `input.rs` maps key events to those updates.
- `ui.rs` owns layout and rendering. `storage.rs` owns SQLite schema and CRUD; `model.rs` defines stored task values. `theme.rs` owns palette constants and TOML configuration.
- SQLite is the source of truth. Demo records are inserted only into an empty database, so they never overwrite user tasks.
- ratatui `0.30.2` and tachyonfx `0.25.2` are pinned as a matched current release pair. A synchronous crossterm event loop redraws while dirty and ticks at animation cadence only while an effect is active.
- Table transitions use tachyonfx's buffer effect API to blend the previous settled table into the new render. Task state stays independent of transient animation buffers.