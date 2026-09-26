use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

use crate::{
    app::{App, InputMode},
    model::Task,
    storage::Storage,
    theme::{Config, KeyBindings},
};

pub fn handle_key(app: &mut App, storage: &mut Storage, key: KeyEvent, config: &Config) -> Result<()> {
    if key.kind != KeyEventKind::Press {
        return Ok(());
    }
    if key.code == KeyCode::Char('c') && key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) {
        app.should_quit = true;
        return Ok(());
    }
    let bindings = config.keybindings.clone();

    match app.mode.clone() {
        InputMode::Search { mut draft, original } => match key.code {
            KeyCode::Esc => {
                app.mode = InputMode::Normal;
                app.status = if original.is_empty() { "Search cancelled" } else { "Search restored" }.to_owned();
                app.dirty = true;
            }
            KeyCode::Enter => app.commit_search(draft),
            KeyCode::Backspace => {
                draft.pop();
                app.mode = InputMode::Search { draft, original };
                app.dirty = true;
            }
            KeyCode::Char(_character) if !key.modifiers.is_empty() => {}
            KeyCode::Char(character) => {
                draft.push(character);
                app.mode = InputMode::Search { draft, original };
                app.dirty = true;
            }
            _ => {}
        },
        InputMode::Add { mut draft } => match key.code {
            KeyCode::Esc => {
                app.mode = InputMode::Normal;
                app.status = "Add cancelled".to_owned();
                app.dirty = true;
            }
            KeyCode::Enter => {
                let title = draft.trim();
                if title.is_empty() {
                    app.status = "Task title cannot be empty".to_owned();
                    app.dirty = true;
                } else {
                    let task = Task::new(title.to_owned());
                    storage.insert(&task)?;
                    app.insert_task(task);
                    app.mode = InputMode::Normal;
                }
            }
            KeyCode::Backspace => {
                draft.pop();
                app.mode = InputMode::Add { draft };
                app.dirty = true;
            }
            KeyCode::Char(character) if key.modifiers.is_empty() => {
                draft.push(character);
                app.mode = InputMode::Add { draft };
                app.dirty = true;
            }
            _ => {}
        },
        InputMode::ConfirmDelete => match key.code {
            KeyCode::Char('y') | KeyCode::Enter => {
                if let Some(id) = app.delete_selected() {
                    storage.delete(&id)?;
                    app.status = "Task deleted".to_owned();
                }
                app.mode = InputMode::Normal;
            }
            KeyCode::Char('n') | KeyCode::Esc => {
                app.mode = InputMode::Normal;
                app.status = "Delete cancelled".to_owned();
                app.dirty = true;
            }
            _ => {}
        },
        InputMode::Help => {
            if matches!(key.code, KeyCode::Esc | KeyCode::Enter | KeyCode::Char('?')) {
                app.mode = InputMode::Normal;
                app.dirty = true;
            }
        }
        InputMode::Normal => handle_normal(app, storage, key, &bindings)?,
    }
    Ok(())
}

fn handle_normal(app: &mut App, storage: &mut Storage, key: KeyEvent, bindings: &KeyBindings) -> Result<()> {
    match key.code {
        KeyCode::Esc if !app.search_query.is_empty() => app.clear_search(),
        KeyCode::Up if key.modifiers.is_empty() => app.move_selection(-1),
        KeyCode::Down if key.modifiers.is_empty() => app.move_selection(1),
        KeyCode::Enter => persist_state_cycle(app, storage)?,
        KeyCode::Char('?') if bindings.help == "?" => {
            app.mode = InputMode::Help;
            app.dirty = true;
        }
        KeyCode::Char(character) if key.modifiers.is_empty() && character.to_string() == bindings.cycle_state => {
            persist_state_cycle(app, storage)?;
        }
        KeyCode::Char(character) if key.modifiers.is_empty() => {
            let value = character.to_string();
            if value == bindings.quit {
                app.should_quit = true;
            } else if value == bindings.down {
                app.move_selection(1);
            } else if value == bindings.up {
                app.move_selection(-1);
            } else if value == bindings.filter {
                app.cycle_filter();
            } else if value == bindings.sort {
                app.cycle_sort();
            } else if value == bindings.search {
                app.mode = InputMode::Search {
                    draft: app.search_query.clone(),
                    original: app.search_query.clone(),
                };
                app.dirty = true;
            } else if value == bindings.add {
                app.mode = InputMode::Add { draft: String::new() };
                app.dirty = true;
            } else if value == bindings.delete {
                if app.selected_task().is_some() {
                    app.mode = InputMode::ConfirmDelete;
                    app.dirty = true;
                }
            } else if value == bindings.help {
                app.mode = InputMode::Help;
                app.dirty = true;
            }
        }
        _ => {}
    }
    Ok(())
}

fn persist_state_cycle(app: &mut App, storage: &mut Storage) -> Result<()> {
    if let Some(task) = app.cycle_selected_state() {
        storage.update(&task)?;
        app.status = format!("{} -> {}", task.title, task.state.as_str().to_uppercase());
        app.keep_selection_visible();
    }
    Ok(())
}