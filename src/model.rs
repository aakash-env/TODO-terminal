use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration as ChronoDuration, Local};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum State {
    Running,
    Pending,
    Done,
}

impl State {
    pub fn next(self) -> Self {
        match self {
            Self::Running => Self::Pending,
            Self::Pending => Self::Done,
            Self::Done => Self::Running,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Pending => "pending",
            Self::Done => "done",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "running" => Ok(Self::Running),
            "pending" => Ok(Self::Pending),
            "done" => Ok(Self::Done),
            _ => Err(anyhow!("unknown task state: {value}")),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Priority {
    Urgent,
    High,
    Medium,
    Low,
}

impl Priority {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Urgent => "urgent",
            Self::High => "high",
            Self::Medium => "medium",
            Self::Low => "low",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "urgent" => Ok(Self::Urgent),
            "high" => Ok(Self::High),
            "medium" => Ok(Self::Medium),
            "low" => Ok(Self::Low),
            _ => Err(anyhow!("unknown task priority: {value}")),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Health {
    OnTrack,
    Tight,
    Unknown,
    Done,
}

impl Health {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::OnTrack => "on_track",
            Self::Tight => "tight",
            Self::Unknown => "unknown",
            Self::Done => "done",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "on_track" => Ok(Self::OnTrack),
            "tight" => Ok(Self::Tight),
            "unknown" => Ok(Self::Unknown),
            "done" => Ok(Self::Done),
            _ => Err(anyhow!("unknown task health: {value}")),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub state: State,
    pub time_tracked: Duration,
    pub effort_left: Duration,
    pub deadline: Option<DateTime<Local>>,
    pub progress_pct: u8,
    pub priority: Priority,
    pub health: Health,
    pub created_at: DateTime<Local>,
}

impl Task {
    pub fn new(title: String) -> Self {
        let mut hasher = DefaultHasher::new();
        title.hash(&mut hasher);
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
            .hash(&mut hasher);
        let id = format!("{:010x}", hasher.finish() & 0xffffffffff);

        Self {
            id,
            title,
            state: State::Pending,
            time_tracked: Duration::ZERO,
            effort_left: Duration::from_secs(3600),
            deadline: None,
            progress_pct: 0,
            priority: Priority::Medium,
            health: Health::Unknown,
            created_at: Local::now(),
        }
    }

    pub fn seeded() -> Vec<Self> {
        let now = Local::now();
        let definitions = [
            ("Ship terminal dashboard v1", State::Running, 2_670, 1_800, 3, 91, Priority::Urgent, Health::Tight),
            ("Review auth token rotation", State::Running, 5_220, 3_600, 9, 68, Priority::High, Health::OnTrack),
            ("Build release pipeline cache", State::Running, 7_800, 10_800, 30, 54, Priority::Medium, Health::OnTrack),
            ("Document database recovery", State::Pending, 1_800, 5_400, 52, 37, Priority::Medium, Health::Unknown),
            ("Audit dependency licenses", State::Pending, 0, 7_200, 78, 12, Priority::Low, Health::Unknown),
            ("Tune query plan for reports", State::Running, 3_900, 2_700, 18, 76, Priority::High, Health::Tight),
            ("Move worker to regional pool", State::Done, 10_800, 0, -48, 100, Priority::Low, Health::Done),
            ("Refresh onboarding checklist", State::Pending, 600, 1_800, 120, 23, Priority::Low, Health::OnTrack),
        ];

        definitions
            .into_iter()
            .enumerate()
            .map(|(index, (title, state, tracked, effort, deadline_hours, progress, priority, health))| {
                let mut task = Self::new(title.to_owned());
                task.state = state;
                task.time_tracked = Duration::from_secs(tracked);
                task.effort_left = Duration::from_secs(effort);
                task.deadline = Some(now + ChronoDuration::hours(deadline_hours));
                task.progress_pct = progress;
                task.priority = priority;
                task.health = health;
                task.created_at = now - ChronoDuration::days(index as i64 + 1);
                task
            })
            .collect()
    }
}