use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::{Context, Result};
use chrono::{DateTime, Local};
use rusqlite::{params, Connection};

use crate::model::{Health, Priority, State, Task};

pub struct Storage {
    connection: Connection,
}

impl Storage {
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent().filter(|parent| !parent.as_os_str().is_empty()) {
            fs::create_dir_all(parent)
                .with_context(|| format!("creating database directory {}", parent.display()))?;
        }

        let connection = Connection::open(path)
            .with_context(|| format!("opening database {}", path.display()))?;
        connection.execute_batch(
            "PRAGMA foreign_keys = ON;
             CREATE TABLE IF NOT EXISTS tasks (
                 id TEXT PRIMARY KEY NOT NULL,
                 title TEXT NOT NULL,
                 state TEXT NOT NULL,
                 tracked_seconds INTEGER NOT NULL,
                 effort_seconds INTEGER NOT NULL,
                 deadline TEXT,
                 progress_pct INTEGER NOT NULL,
                 priority TEXT NOT NULL,
                 health TEXT NOT NULL,
                 created_at TEXT NOT NULL
             );",
        )?;

        let storage = Self { connection };
        storage.seed_if_empty()?;
        Ok(storage)
    }

    fn seed_if_empty(&self) -> Result<()> {
        let count: i64 = self
            .connection
            .query_row("SELECT COUNT(*) FROM tasks", [], |row| row.get(0))?;
        if count == 0 {
            for task in Task::seeded() {
                self.insert(&task)?;
            }
        }
        Ok(())
    }

    pub fn load_tasks(&self) -> Result<Vec<Task>> {
        let mut statement = self.connection.prepare(
            "SELECT id, title, state, tracked_seconds, effort_seconds, deadline,
                    progress_pct, priority, health, created_at
             FROM tasks",
        )?;
        let records = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, Option<String>>(5)?,
                    row.get::<_, u8>(6)?,
                    row.get::<_, String>(7)?,
                    row.get::<_, String>(8)?,
                    row.get::<_, String>(9)?,
                ))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        records
            .into_iter()
            .map(|(id, title, state, tracked, effort, deadline, progress, priority, health, created)| {
                let created_at = DateTime::parse_from_rfc3339(&created)?.with_timezone(&Local);
                let deadline = deadline
                    .map(|value| DateTime::parse_from_rfc3339(&value).map(|date| date.with_timezone(&Local)))
                    .transpose()?;
                Ok(Task {
                    id,
                    title,
                    state: State::parse(&state)?,
                    time_tracked: Duration::from_secs(tracked.max(0) as u64),
                    effort_left: Duration::from_secs(effort.max(0) as u64),
                    deadline,
                    progress_pct: progress.min(100),
                    priority: Priority::parse(&priority)?,
                    health: Health::parse(&health)?,
                    created_at,
                })
            })
            .collect()
    }

    pub fn insert(&self, task: &Task) -> Result<()> {
        self.connection.execute(
            "INSERT INTO tasks
             (id, title, state, tracked_seconds, effort_seconds, deadline,
              progress_pct, priority, health, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                task.id,
                task.title,
                task.state.as_str(),
                task.time_tracked.as_secs() as i64,
                task.effort_left.as_secs() as i64,
                task.deadline.as_ref().map(DateTime::to_rfc3339),
                task.progress_pct,
                task.priority.as_str(),
                task.health.as_str(),
                task.created_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn update(&self, task: &Task) -> Result<()> {
        self.connection.execute(
            "UPDATE tasks SET title = ?2, state = ?3, tracked_seconds = ?4,
             effort_seconds = ?5, deadline = ?6, progress_pct = ?7,
             priority = ?8, health = ?9, created_at = ?10 WHERE id = ?1",
            params![
                task.id,
                task.title,
                task.state.as_str(),
                task.time_tracked.as_secs() as i64,
                task.effort_left.as_secs() as i64,
                task.deadline.as_ref().map(DateTime::to_rfc3339),
                task.progress_pct,
                task.priority.as_str(),
                task.health.as_str(),
                task.created_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn delete(&self, id: &str) -> Result<()> {
        self.connection
            .execute("DELETE FROM tasks WHERE id = ?1", params![id])?;
        Ok(())
    }
}

pub fn default_database_path() -> Result<PathBuf> {
    let home = dirs::home_dir().context("could not locate the home directory")?;
    Ok(home.join(".config").join("taskdeck").join("tasks.db"))
}