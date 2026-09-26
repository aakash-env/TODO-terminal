mod app;
mod input;
mod model;
mod storage;
mod theme;
mod ui;

use std::{
    io::{self, Stdout},
    path::PathBuf,
    time::{Duration, Instant},
};

use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, buffer::Buffer, Terminal};
use tachyonfx::EffectManager;

use crate::{
    app::{App, Filter, Sort},
    model::Task,
    storage::Storage,
    theme::{Config, Theme},
};

#[derive(Parser, Debug)]
#[command(name = "taskdeck", version, about = "A focused task dashboard for your terminal")]
struct Args {
    #[arg(long, value_enum)]
    filter: Option<Filter>,
    #[arg(long, value_enum)]
    sort: Option<Sort>,
    #[arg(long, value_name = "TITLE")]
    add: Option<String>,
    #[arg(long, value_name = "PATH")]
    db_path: Option<PathBuf>,
}

type AppTerminal = Terminal<CrosstermBackend<Stdout>>;

fn main() -> Result<()> {
    let args = Args::parse();
    let config = Config::load_or_create()?;
    let db_path = args.db_path.unwrap_or(Config::default_db_path()?);
    let mut storage = Storage::open(&db_path)?;

    if let Some(title) = args.add.filter(|title| !title.trim().is_empty()) {
        storage.insert(&Task::new(title.trim().to_owned()))?;
    }

    let tasks = storage.load_tasks()?;
    let mut app = App::new(
        tasks,
        args.filter.unwrap_or_default(),
        args.sort.unwrap_or_default(),
    );
    let theme = Theme::from_config(&config.theme);
    let mut terminal = setup_terminal()?;
    let result = run(&mut terminal, &mut app, &mut storage, &theme, &config);
    let restore_result = restore_terminal();

    result?;
    restore_result?;
    Ok(())
}

fn setup_terminal() -> io::Result<AppTerminal> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, crossterm::cursor::Hide)?;
    Terminal::new(CrosstermBackend::new(stdout))
}

fn restore_terminal() -> io::Result<()> {
    disable_raw_mode()?;
    execute!(
        io::stdout(),
        crossterm::cursor::Show,
        LeaveAlternateScreen
    )?;
    Ok(())
}

fn run(
    terminal: &mut AppTerminal,
    app: &mut App,
    storage: &mut Storage,
    theme: &Theme,
    config: &Config,
) -> Result<()> {
    let mut previous_frame: Option<Buffer> = None;
    let mut effects: EffectManager<&'static str> = EffectManager::default();
    let mut last_draw = Instant::now();

    while !app.should_quit {
        let poll_time = if effects.is_running() { 16 } else { 250 };
        if event::poll(Duration::from_millis(poll_time))? {
            if let Event::Key(key) = event::read()? {
                input::handle_key(app, storage, key, config)?;
            }
        }

        if app.dirty || effects.is_running() {
            let elapsed = last_draw.elapsed();
            last_draw = Instant::now();
            let transition = app.take_transition_pending();
            let old_frame = transition.then(|| previous_frame.clone()).flatten();
            let mut settled_snapshot = None;

            terminal.draw(|frame| {
                let table_area = ui::render(frame, app, theme, &config.keybindings);
                if let Some(old) = old_frame {
                    effects.add_unique_effect(
                        "table-transition",
                        ui::crossfade_effect(old, table_area),
                    );
                }
                let frame_area = frame.area();
                effects.process_effects(elapsed.into(), frame.buffer_mut(), frame_area);
                if !effects.is_running() {
                    settled_snapshot = Some(frame.buffer_mut().clone());
                }
            })?;

            if let Some(snapshot) = settled_snapshot {
                previous_frame = Some(snapshot);
            }
            app.dirty = false;
        }
    }

    Ok(())
}