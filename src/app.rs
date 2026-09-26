use clap::ValueEnum;

use crate::model::{Health, Priority, State, Task};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
pub enum Filter {
    #[default]
    All,
    Running,
    Pending,
    Done,
}

impl Filter {
    pub fn label(self) -> &'static str {
        match self {
            Self::All => "ALL",
            Self::Running => "RUNNING",
            Self::Pending => "PENDING",
            Self::Done => "DONE",
        }
    }

    fn next(self) -> Self {
        match self {
            Self::All => Self::Running,
            Self::Running => Self::Pending,
            Self::Pending => Self::Done,
            Self::Done => Self::All,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
pub enum Sort {
    #[default]
    Created,
    Deadline,
    Priority,
}

impl Sort {
    pub fn label(self) -> &'static str {
        match self {
            Self::Created => "CREATED",
            Self::Deadline => "DEADLINE",
            Self::Priority => "PRIORITY",
        }
    }

    fn next(self) -> Self {
        match self {
            Self::Created => Self::Deadline,
            Self::Deadline => Self::Priority,
            Self::Priority => Self::Created,
        }
    }
}

#[derive(Clone, Debug)]
pub enum InputMode {
    Normal,
    Search { draft: String, original: String },
    Add { draft: String },
    ConfirmDelete,
    Help,
}

pub struct App {
    pub tasks: Vec<Task>,
    pub filter: Filter,
    pub sort: Sort,
    pub selected_id: Option<String>,
    pub search_query: String,
    pub mode: InputMode,
    pub status: String,
    pub should_quit: bool,
    pub dirty: bool,
    transition_pending: bool,
}

impl App {
    pub fn new(tasks: Vec<Task>, filter: Filter, sort: Sort) -> Self {
        let mut app = Self {
            tasks,
            filter,
            sort,
            selected_id: None,
            search_query: String::new(),
            mode: InputMode::Normal,
            status: "Ready".to_owned(),
            should_quit: false,
            dirty: true,
            transition_pending: false,
        };
        app.keep_selection_visible();
        app
    }

    pub fn visible_tasks(&self) -> Vec<&Task> {
        let search = match &self.mode {
            InputMode::Search { draft, .. } => draft.as_str(),
            _ => self.search_query.as_str(),
        };
        let mut visible: Vec<&Task> = self
            .tasks
            .iter()
            .filter(|task| match self.filter {
                Filter::All => true,
                Filter::Running => task.state == State::Running,
                Filter::Pending => task.state == State::Pending,
                Filter::Done => task.state == State::Done,
            })
            .filter(|task| fuzzy_match(&task.title, search))
            .collect();

        match self.sort {
            Sort::Created => visible.sort_by(|left, right| right.created_at.cmp(&left.created_at)),
            Sort::Deadline => visible.sort_by(|left, right| match (&left.deadline, &right.deadline) {
                (Some(left), Some(right)) => left.cmp(right),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => left.created_at.cmp(&right.created_at),
            }),
            Sort::Priority => visible.sort_by_key(|task| match task.priority {
                Priority::Urgent => 0,
                Priority::High => 1,
                Priority::Medium => 2,
                Priority::Low => 3,
            }),
        }
        visible
    }

    pub fn selected_index(&self) -> Option<usize> {
        let visible = self.visible_tasks();
        visible
            .iter()
            .position(|task| Some(task.id.as_str()) == self.selected_id.as_deref())
            .or_else(|| (!visible.is_empty()).then_some(0))
    }

    pub fn move_selection(&mut self, offset: isize) {
        let visible = self.visible_tasks();
        if visible.is_empty() {
            self.selected_id = None;
            self.dirty = true;
            return;
        }
        let current = visible
            .iter()
            .position(|task| Some(task.id.as_str()) == self.selected_id.as_deref())
            .unwrap_or(0) as isize;
        let next = (current + offset).rem_euclid(visible.len() as isize) as usize;
        self.selected_id = Some(visible[next].id.clone());
        self.dirty = true;
    }

    pub fn cycle_filter(&mut self) {
        self.filter = self.filter.next();
        self.after_visible_set_change();
        self.status = format!("Filter: {}", self.filter.label());
    }

    pub fn cycle_sort(&mut self) {
        self.sort = self.sort.next();
        self.after_visible_set_change();
        self.status = format!("Sort: {}", self.sort.label());
    }

    pub fn cycle_selected_state(&mut self) -> Option<Task> {
        let id = self.selected_id.as_deref()?;
        let task = self.tasks.iter_mut().find(|task| task.id == id)?;
        task.state = task.state.next();
        match task.state {
            State::Done => {
                task.progress_pct = 100;
                task.health = Health::Done;
                task.effort_left = std::time::Duration::ZERO;
            }
            State::Running => {
                if task.progress_pct == 100 {
                    task.progress_pct = 0;
                }
                task.health = Health::OnTrack;
            }
            State::Pending => {
                if task.health == Health::Done {
                    task.health = Health::Unknown;
                }
            }
        }
        let updated = task.clone();
        self.transition_pending = true;
        self.dirty = true;
        Some(updated)
    }

    pub fn selected_task(&self) -> Option<&Task> {
        let id = self.selected_id.as_deref()?;
        self.tasks.iter().find(|task| task.id == id)
    }

    pub fn delete_selected(&mut self) -> Option<String> {
        let id = self.selected_id.clone()?;
        let index = self.tasks.iter().position(|task| task.id == id)?;
        self.tasks.remove(index);
        self.selected_id = self.visible_tasks().first().map(|task| task.id.clone());
        self.transition_pending = true;
        self.dirty = true;
        Some(id)
    }

    pub fn insert_task(&mut self, task: Task) {
        self.selected_id = Some(task.id.clone());
        self.status = format!("Added: {}", task.title);
        self.tasks.push(task);
        self.transition_pending = true;
        self.dirty = true;
        self.keep_selection_visible();
    }

    pub fn commit_search(&mut self, query: String) {
        self.search_query = query;
        self.mode = InputMode::Normal;
        self.status = if self.search_query.is_empty() {
            "Search cleared".to_owned()
        } else {
            format!("Search: {}", self.search_query)
        };
        self.transition_pending = true;
        self.dirty = true;
        self.keep_selection_visible();
    }

    pub fn clear_search(&mut self) {
        self.search_query.clear();
        self.mode = InputMode::Normal;
        self.status = "Search cleared".to_owned();
        self.after_visible_set_change();
    }

    pub fn keep_selection_visible(&mut self) {
        let visible = self.visible_tasks();
        if !visible
            .iter()
            .any(|task| Some(task.id.as_str()) == self.selected_id.as_deref())
        {
            self.selected_id = visible.first().map(|task| task.id.clone());
        }
    }

    pub fn take_transition_pending(&mut self) -> bool {
        std::mem::take(&mut self.transition_pending)
    }

    fn after_visible_set_change(&mut self) {
        self.transition_pending = true;
        self.dirty = true;
        self.keep_selection_visible();
    }
}

fn fuzzy_match(title: &str, query: &str) -> bool {
    if query.is_empty() {
        return true;
    }
    let mut query_chars = query.chars().flat_map(char::to_lowercase);
    let Some(mut wanted) = query_chars.next() else {
        return true;
    };
    for character in title.chars().flat_map(char::to_lowercase) {
        if character == wanted {
            if let Some(next) = query_chars.next() {
                wanted = next;
            } else {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fuzzy_search_matches_subsequences_case_insensitively() {
        assert!(fuzzy_match("Ship terminal dashboard", "s t d"));
        assert!(fuzzy_match("Ship terminal dashboard", "TERM"));
        assert!(!fuzzy_match("Ship terminal dashboard", "xyz"));
    }

    #[test]
    fn selected_task_survives_sort_changes() {
        let tasks = Task::seeded();
        let selected = tasks[2].id.clone();
        let mut app = App::new(tasks, Filter::All, Sort::Created);
        app.selected_id = Some(selected.clone());
        app.cycle_sort();
        assert_eq!(app.selected_task().map(|task| task.id.as_str()), Some(selected.as_str()));
    }
}