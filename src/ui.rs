use chrono::{DateTime, Local};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table, TableState},
    Frame,
};
use tachyonfx::{fx, Effect, Interpolation};

use crate::{
    app::{App, InputMode},
    model::{Health, Priority, State, Task},
    theme::{KeyBindings, Theme},
};

pub fn render(frame: &mut Frame<'_>, app: &App, theme: &Theme, bindings: &KeyBindings) -> Rect {
    frame.render_widget(Block::default().style(Style::default().bg(theme.background)), frame.area());
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(5), Constraint::Min(7), Constraint::Length(3)])
        .split(frame.area());

    render_header(frame, layout[0], app, theme);
    let table_area = render_table(frame, layout[1], app, theme);
    render_footer(frame, layout[2], app, theme, bindings);
    render_overlay(frame, app, theme);
    table_area
}

fn render_header(frame: &mut Frame<'_>, area: Rect, app: &App, theme: &Theme) {
    let counts = app.tasks.iter().fold([0usize; 3], |mut counts, task| {
        match task.state {
            State::Running => counts[0] += 1,
            State::Pending => counts[1] += 1,
            State::Done => counts[2] += 1,
        }
        counts
    });
    let visible = app.visible_tasks().len();
    let top = Line::from(vec![
        Span::styled(" TASKDECK ", Style::default().fg(theme.background).bg(theme.accent).add_modifier(Modifier::BOLD)),
        Span::raw("   "),
        Span::styled("FIELD NOTES / TASK OPERATIONS", Style::default().fg(theme.muted)),
        Span::raw("   "),
        Span::styled(format!("{visible:02} VISIBLE"), Style::default().fg(theme.foreground).add_modifier(Modifier::BOLD)),
        Span::raw(format!("  /  {} TOTAL", app.tasks.len())),
        Span::raw("     "),
        Span::styled(format!("{} RUNNING", counts[0]), Style::default().fg(theme.accent)),
        Span::raw(format!("   {} QUEUED", counts[1])),
        Span::raw(format!("   {} DONE", counts[2])),
    ]);
    let second = Line::from(vec![
        Span::styled("VIEW ", Style::default().fg(theme.muted)),
        Span::styled(app.filter.label(), Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        Span::raw("     "),
        Span::styled("ORDER ", Style::default().fg(theme.muted)),
        Span::styled(app.sort.label(), Style::default().fg(theme.foreground)),
        Span::raw("     "),
        Span::styled("LOCAL STORE ", Style::default().fg(theme.muted)),
        Span::styled("● CONNECTED", Style::default().fg(theme.low)),
    ]);
    let block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(theme.border))
        .style(Style::default().bg(theme.background));
    frame.render_widget(Paragraph::new(vec![top, second]).block(block), area);
}

fn render_table(frame: &mut Frame<'_>, area: Rect, app: &App, theme: &Theme) -> Rect {
    let compact = area.width < 132;
    let tasks = app.visible_tasks();
    let rows = tasks.iter().map(|task| task_row(task, compact, theme));
    let header_cells = if compact {
        vec!["ID", "STATE", "TASK", "DUE IN", "PROGRESS", "PRIORITY", "HEALTH"]
    } else {
        vec!["ID", "STATE", "TASK", "TRACKED", "EFFORT LEFT", "DEADLINE", "DUE IN", "PROGRESS", "PRIORITY", "HEALTH"]
    };
    let header = Row::new(header_cells.into_iter().map(Cell::from))
        .style(Style::default().fg(theme.muted).bg(theme.panel_alt).add_modifier(Modifier::BOLD))
        .height(1);
    let widths = if compact {
        vec![
            Constraint::Length(9),
            Constraint::Length(9),
            Constraint::Min(24),
            Constraint::Length(10),
            Constraint::Length(16),
            Constraint::Length(10),
            Constraint::Length(11),
        ]
    } else {
        vec![
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Min(25),
            Constraint::Length(11),
            Constraint::Length(12),
            Constraint::Length(12),
            Constraint::Length(10),
            Constraint::Length(16),
            Constraint::Length(11),
            Constraint::Length(12),
        ]
    };
    let table = Table::new(rows, widths)
        .header(header)
        .row_highlight_style(Style::default().bg(theme.selected_background).fg(theme.foreground).add_modifier(Modifier::BOLD))
        .highlight_symbol("› ")
        .column_spacing(1)
        .block(
            Block::default()
                .title(Line::from(vec![
                    Span::styled("  WORK QUEUE  ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
                    Span::styled("  /  SELECTED ROW", Style::default().fg(theme.muted)),
                ]))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border))
                .style(Style::default().bg(theme.panel)),
        );
    let mut state = TableState::default();
    state.select(app.selected_index());
    frame.render_stateful_widget(table, area, &mut state);
    area
}

fn task_row(task: &Task, compact: bool, theme: &Theme) -> Row<'static> {
    let id = Cell::from(Span::styled(task.id.clone(), Style::default().fg(theme.muted)));
    let state = Cell::from(Line::from(vec![
        Span::styled("● ", Style::default().fg(state_color(task.state, theme))),
        Span::styled(state_label(task.state), Style::default().fg(theme.foreground)),
    ]));
    let title = Cell::from(Line::from(vec![
        Span::styled(task.title.clone(), Style::default().fg(theme.foreground)),
    ]));
    let due = Cell::from(Span::styled(due_in(task.deadline.as_ref()), Style::default().fg(due_color(task, theme))));
    let progress = progress_cell(task.progress_pct, theme);
    let priority = Cell::from(Span::styled(
        format!("◆ {}", priority_label(task.priority)),
        Style::default().fg(priority_color(task.priority, theme)).add_modifier(Modifier::BOLD),
    ));
    let health = Cell::from(Span::styled(
        health_label(task.health),
        Style::default().fg(health_color(task.health, theme)),
    ));

    let cells = if compact {
        vec![id, state, title, due, progress, priority, health]
    } else {
        vec![
            id,
            state,
            title,
            Cell::from(Span::styled(format_duration(task.time_tracked), Style::default().fg(theme.foreground))),
            Cell::from(Span::styled(format_duration(task.effort_left), Style::default().fg(theme.muted))),
            Cell::from(Span::styled(
                task.deadline.as_ref().map(|value| value.format("%b %d").to_string()).unwrap_or_else(|| "—".to_owned()),
                Style::default().fg(theme.foreground),
            )),
            due,
            progress,
            priority,
            health,
        ]
    };
    Row::new(cells).height(1)
}

fn progress_cell(progress: u8, theme: &Theme) -> Cell<'static> {
    let filled = (progress.min(100) as usize * 8 / 100).min(8);
    Cell::from(Line::from(vec![
        Span::styled("█".repeat(filled), Style::default().fg(theme.accent)),
        Span::styled("░".repeat(8 - filled), Style::default().fg(theme.border)),
        Span::styled(format!(" {:>3}%", progress.min(100)), Style::default().fg(theme.foreground)),
    ]))
}

fn render_footer(frame: &mut Frame<'_>, area: Rect, app: &App, theme: &Theme, bindings: &KeyBindings) {
    let mode_line = match &app.mode {
        InputMode::Search { draft, .. } => format!("SEARCH  > {draft}▏"),
        InputMode::Add { draft } => format!("NEW TASK  > {draft}▏"),
        InputMode::ConfirmDelete => "DELETE SELECTED TASK?  [y] confirm  [n] cancel".to_owned(),
        InputMode::Help => "HELP".to_owned(),
        InputMode::Normal => app.status.clone(),
    };
    let move_keys = format!("{}/{}", bindings.down, bindings.up);
    let state_key = if bindings.cycle_state == " " {
        "space/enter".to_owned()
    } else {
        format!("{}/enter", bindings.cycle_state)
    };
    let hint_values = [
        (move_keys.as_str(), "move"),
        (bindings.filter.as_str(), "filter"),
        (bindings.sort.as_str(), "sort"),
        (bindings.search.as_str(), "search"),
        (bindings.add.as_str(), "add"),
        (state_key.as_str(), "state"),
        (bindings.delete.as_str(), "delete"),
        (bindings.help.as_str(), "help"),
        (bindings.quit.as_str(), "quit"),
    ];
    let mut command_spans = Vec::with_capacity(hint_values.len() * 3);
    for (index, (key, label)) in hint_values.into_iter().enumerate() {
        if index > 0 {
            command_spans.push(Span::raw("  "));
        }
        command_spans.push(Span::styled(
            key.to_owned(),
            Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
        ));
        command_spans.push(Span::styled(
            format!(" {label}"),
            Style::default().fg(theme.muted),
        ));
    }
    let commands = Line::from(command_spans);
    let status = Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled(mode_line, Style::default().fg(theme.muted)),
    ]);
    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(theme.border))
        .style(Style::default().bg(theme.background));
    frame.render_widget(Paragraph::new(vec![commands, status]).block(block), area);
}

fn render_overlay(frame: &mut Frame<'_>, app: &App, theme: &Theme) {
    let lines = match &app.mode {
        InputMode::Help => Some(vec![
            Line::from(Span::styled("TASKDECK / QUICK REFERENCE", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from("j / k or arrows    Move through tasks"),
            Line::from("f                  Cycle all / running / pending / done"),
            Line::from("o                  Cycle created / deadline / priority"),
            Line::from("/                  Search titles as you type"),
            Line::from("a                  Add a task"),
            Line::from("space / enter      Cycle running / pending / done"),
            Line::from("d                  Delete selected task"),
            Line::from("esc                Clear search or dismiss"),
            Line::from("q                  Quit"),
            Line::from(""),
            Line::from(Span::styled("Press ? / enter / esc to return", Style::default().fg(theme.muted))),
        ]),
        InputMode::ConfirmDelete => Some(vec![
            Line::from(Span::styled("REMOVE TASK", Style::default().fg(theme.urgent).add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from(app.selected_task().map(|task| task.title.as_str()).unwrap_or("No task selected")),
            Line::from(""),
            Line::from("[y] confirm     [n] keep task"),
        ]),
        _ => None,
    };

    if let Some(lines) = lines {
        let width = frame.area().width.min(64).saturating_sub(2);
        let height = (lines.len() as u16 + 2).min(frame.area().height.saturating_sub(2));
        let area = centered_rect(width, height, frame.area());
        frame.render_widget(Clear, area);
        frame.render_widget(
            Paragraph::new(lines)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(theme.accent))
                        .style(Style::default().fg(theme.foreground).bg(theme.panel_alt)),
                )
                .style(Style::default().fg(theme.foreground).bg(theme.panel_alt)),
            area,
        );
    }
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let width = width.min(area.width);
    let height = height.min(area.height);
    Rect {
        x: area.x + (area.width - width) / 2,
        y: area.y + (area.height - height) / 2,
        width,
        height,
    }
}

pub fn crossfade_effect(old: Buffer, area: Rect) -> Effect {
    fx::effect_fn_buf((old, area), (300, Interpolation::SineInOut), |(old, region), context, current| {
        let alpha = context.alpha();
        let left = region.x.max(current.area.x).max(old.area.x);
        let top = region.y.max(current.area.y).max(old.area.y);
        let right = region.right().min(current.area.right()).min(old.area.right());
        let bottom = region.bottom().min(current.area.bottom()).min(old.area.bottom());
        for y in top..bottom {
            let old_bar: Vec<u16> = (left..right)
                .filter(|x| is_progress_glyph(old[(*x, y)].symbol()))
                .collect();
            let new_bar: Vec<u16> = (left..right)
                .filter(|x| is_progress_glyph(current[(*x, y)].symbol()))
                .collect();
            let old_fill = old_bar
                .iter()
                .filter(|&&x| old[(x, y)].symbol() == "█")
                .count();
            let new_fill = new_bar
                .iter()
                .filter(|&&x| current[(x, y)].symbol() == "█")
                .count();
            for x in left..right {
                let old_cell = &old[(x, y)];
                let from_fg = old_cell.fg;
                let from_bg = old_cell.bg;
                let from_symbol = old_cell.symbol().to_owned();
                let cell = &mut current[(x, y)];
                cell.fg = blend(from_fg, cell.fg, alpha);
                cell.bg = blend(from_bg, cell.bg, alpha);
                if alpha < 0.5 {
                    cell.set_symbol(&from_symbol);
                }
            }
            if new_bar.len() == 8 {
                let start_fill = if old_bar == new_bar { old_fill } else { 0 };
                let fill = (start_fill as f32
                    + (new_fill as f32 - start_fill as f32) * alpha)
                    .round() as usize;
                for (index, x) in new_bar.into_iter().enumerate() {
                    current[(x, y)].set_symbol(if index < fill { "█" } else { "░" });
                }
            }
        }
    })
    .with_area(area)
}

fn is_progress_glyph(symbol: &str) -> bool {
    matches!(symbol, "█" | "░")
}

fn blend(from: ratatui::style::Color, to: ratatui::style::Color, alpha: f32) -> ratatui::style::Color {
    match (from, to) {
        (ratatui::style::Color::Rgb(fr, fg, fb), ratatui::style::Color::Rgb(tr, tg, tb)) => {
            let channel = |start: u8, end: u8| {
                (start as f32 + (end as f32 - start as f32) * alpha).round() as u8
            };
            ratatui::style::Color::Rgb(channel(fr, tr), channel(fg, tg), channel(fb, tb))
        }
        (_, target) if alpha >= 1.0 => target,
        (source, _) if alpha <= 0.0 => source,
        (_, target) => target,
    }
}

fn state_label(state: State) -> &'static str {
    match state {
        State::Running => "RUN",
        State::Pending => "WAIT",
        State::Done => "DONE",
    }
}

fn priority_label(priority: Priority) -> &'static str {
    match priority {
        Priority::Urgent => "URGENT",
        Priority::High => "HIGH",
        Priority::Medium => "MED",
        Priority::Low => "LOW",
    }
}

fn health_label(health: Health) -> &'static str {
    match health {
        Health::OnTrack => "ON TRACK",
        Health::Tight => "TIGHT",
        Health::Unknown => "UNKNOWN",
        Health::Done => "DONE",
    }
}

fn state_color(state: State, theme: &Theme) -> ratatui::style::Color {
    match state {
        State::Running => theme.accent,
        State::Pending => theme.medium,
        State::Done => theme.done,
    }
}

fn priority_color(priority: Priority, theme: &Theme) -> ratatui::style::Color {
    match priority {
        Priority::Urgent => theme.urgent,
        Priority::High => theme.high,
        Priority::Medium => theme.medium,
        Priority::Low => theme.low,
    }
}

fn health_color(health: Health, theme: &Theme) -> ratatui::style::Color {
    match health {
        Health::OnTrack => theme.on_track,
        Health::Tight => theme.tight,
        Health::Unknown => theme.unknown,
        Health::Done => theme.done,
    }
}

fn due_color(task: &Task, theme: &Theme) -> ratatui::style::Color {
    match task.health {
        Health::Tight => theme.tight,
        Health::Done => theme.done,
        _ if task.deadline.as_ref().is_some_and(|deadline| *deadline < Local::now()) => theme.urgent,
        _ => theme.muted,
    }
}

fn due_in(deadline: Option<&DateTime<Local>>) -> String {
    let Some(deadline) = deadline else {
        return "—".to_owned();
    };
    let seconds = deadline.signed_duration_since(Local::now()).num_seconds();
    let absolute = seconds.unsigned_abs();
    let duration = if absolute < 3_600 {
        format!("{}m", (absolute / 60).max(1))
    } else if absolute < 86_400 {
        format!("{}h", absolute / 3_600)
    } else {
        format!("{}d", absolute / 86_400)
    };
    if seconds < 0 {
        format!("{duration} late")
    } else if absolute < 3_600 {
        format!("in {duration}")
    } else {
        duration
    }
}

fn format_duration(duration: std::time::Duration) -> String {
    let total_minutes = duration.as_secs() / 60;
    if total_minutes == 0 {
        return "—".to_owned();
    }
    let days = total_minutes / (8 * 60);
    let hours = total_minutes % (8 * 60) / 60;
    let minutes = total_minutes % 60;
    if days > 0 {
        format!("{days}d {hours}h")
    } else if hours > 0 {
        format!("{hours}h {minutes:02}m")
    } else {
        format!("{minutes}m")
    }
}