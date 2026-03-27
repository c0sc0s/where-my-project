use std::{io, time::Duration};

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table, TableState},
};

use crate::core::{manager::ProjectManager, models::InstanceStatus};

pub fn run() -> Result<Option<String>> {
    let mut app = App::load()?;
    let mut terminal = setup_terminal()?;

    let result = run_loop(&mut terminal, &mut app);
    restore_terminal(&mut terminal)?;
    result
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> Result<Option<String>> {
    loop {
        terminal.draw(|frame| draw(frame, app))?;

        if !event::poll(Duration::from_millis(200))? {
            continue;
        }

        let Event::Key(key) = event::read()? else {
            continue;
        };

        if key.kind != KeyEventKind::Press {
            continue;
        }

        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => return Ok(None),
            KeyCode::Char('/') => app.enter_filter_mode(),
            KeyCode::Char('r') if !app.filter_mode => app.reload()?,
            KeyCode::Enter if !app.filter_mode => return Ok(app.selected_path()),
            KeyCode::Up if !app.filter_mode => app.select_previous(),
            KeyCode::Down if !app.filter_mode => app.select_next(),
            KeyCode::Backspace if app.filter_mode => app.pop_filter(),
            KeyCode::Enter if app.filter_mode => app.filter_mode = false,
            KeyCode::Char(ch) if app.filter_mode => app.push_filter(ch),
            _ => {}
        }
    }
}

fn draw(frame: &mut Frame, app: &mut App) {
    let area = frame.size();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(2),
        ])
        .split(area);

    let title = if app.filter_mode {
        format!("Projects  Search: {}", app.filter)
    } else {
        "Projects".to_string()
    };

    let header =
        Paragraph::new(title).block(Block::default().borders(Borders::ALL).title("proj list"));
    frame.render_widget(header, chunks[0]);

    if app.filtered.is_empty() {
        let empty =
            Paragraph::new("No projects matched. Press / to search, r to refresh, q to quit.")
                .block(Block::default().borders(Borders::ALL).title("Results"));
        frame.render_widget(empty, chunks[1]);
    } else {
        let rows = app.filtered_rows();
        let table = Table::new(rows)
            .header(
                Row::new(["Alias", "Repo", "Branch", "Status", "Path"]).style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
            )
            .widths(&[
                Constraint::Length(18),
                Constraint::Length(18),
                Constraint::Length(18),
                Constraint::Length(18),
                Constraint::Min(24),
            ])
            .block(Block::default().borders(Borders::ALL).title("Results"))
            .highlight_style(
                Style::default()
                    .bg(Color::Rgb(32, 58, 96))
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("> ");

        frame.render_stateful_widget(table, chunks[1], &mut app.table_state);
    }

    let footer = Paragraph::new("↑/↓ move  Enter select  / search  r refresh  q/Esc quit")
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(footer, chunks[2]);

    if app.filter_mode {
        let popup = centered_rect(60, 3, area);
        frame.render_widget(Clear, popup);
        let search = Paragraph::new(app.filter.as_str())
            .block(Block::default().borders(Borders::ALL).title("Filter"));
        frame.render_widget(search, popup);
    }
}

fn centered_rect(width_percent: u16, height: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(height),
            Constraint::Min(0),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - width_percent) / 2),
            Constraint::Percentage(width_percent),
            Constraint::Percentage((100 - width_percent) / 2),
        ])
        .split(vertical[1])[1]
}

fn truncate(value: &str, max_chars: usize) -> String {
    let chars: Vec<char> = value.chars().collect();
    if chars.len() <= max_chars {
        return value.to_string();
    }

    let take = max_chars.saturating_sub(3);
    let truncated = chars.into_iter().take(take).collect::<String>();
    format!("{truncated}...")
}

fn format_status(status: &InstanceStatus) -> String {
    if status.git_status.is_clean {
        return "clean".to_string();
    }

    let mut parts = Vec::new();
    if status.git_status.modified_count > 0 {
        parts.push(format!("M{}", status.git_status.modified_count));
    }
    if status.git_status.untracked_count > 0 {
        parts.push(format!("?{}", status.git_status.untracked_count));
    }
    if status.git_status.ahead_count > 0 {
        parts.push(format!("↑{}", status.git_status.ahead_count));
    }
    parts.join(" ")
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    Ok(Terminal::new(CrosstermBackend::new(stdout))?)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

struct App {
    statuses: Vec<InstanceStatus>,
    filtered: Vec<usize>,
    filter: String,
    filter_mode: bool,
    table_state: TableState,
}

impl App {
    fn load() -> Result<Self> {
        let mut app = Self {
            statuses: ProjectManager::load()?.statuses()?,
            filtered: Vec::new(),
            filter: String::new(),
            filter_mode: false,
            table_state: TableState::default(),
        };
        app.refresh_filter();
        Ok(app)
    }

    fn reload(&mut self) -> Result<()> {
        self.statuses = ProjectManager::load()?.statuses()?;
        self.refresh_filter();
        Ok(())
    }

    fn enter_filter_mode(&mut self) {
        self.filter_mode = true;
    }

    fn push_filter(&mut self, ch: char) {
        self.filter.push(ch);
        self.refresh_filter();
    }

    fn pop_filter(&mut self) {
        self.filter.pop();
        self.refresh_filter();
    }

    fn select_previous(&mut self) {
        let Some(selected) = self.table_state.selected() else {
            self.table_state.select(Some(0));
            return;
        };

        let next = selected.saturating_sub(1);
        self.table_state.select(Some(next));
    }

    fn select_next(&mut self) {
        if self.filtered.is_empty() {
            self.table_state.select(None);
            return;
        }

        let next = match self.table_state.selected() {
            Some(selected) if selected + 1 < self.filtered.len() => selected + 1,
            Some(selected) => selected,
            None => 0,
        };
        self.table_state.select(Some(next));
    }

    fn selected_path(&self) -> Option<String> {
        let selected = self.table_state.selected()?;
        let index = *self.filtered.get(selected)?;
        Some(self.statuses.get(index)?.instance.path.clone())
    }

    fn filtered_rows(&self) -> Vec<Row<'static>> {
        self.filtered
            .iter()
            .filter_map(|index| self.statuses.get(*index))
            .map(|status| {
                let alias = status.instance.alias.as_deref().unwrap_or("-");
                let repo = status.instance.repo_name.as_str();
                let branch = status.git_status.branch.as_str();
                let path = status.instance.path.as_str();

                Row::new(vec![
                    Cell::from(truncate(alias, 16)),
                    Cell::from(truncate(repo, 16)),
                    Cell::from(truncate(branch, 16)),
                    Cell::from(truncate(&format_status(status), 16)),
                    Cell::from(truncate(path, 80)),
                ])
            })
            .collect()
    }

    fn refresh_filter(&mut self) {
        let needle = self.filter.to_lowercase();
        self.filtered = self
            .statuses
            .iter()
            .enumerate()
            .filter(|(_, status)| matches_filter(status, &needle))
            .map(|(index, _)| index)
            .collect();

        let next_selection = if self.filtered.is_empty() {
            None
        } else {
            Some(
                self.table_state
                    .selected()
                    .unwrap_or(0)
                    .min(self.filtered.len() - 1),
            )
        };
        self.table_state.select(next_selection);
    }
}

fn matches_filter(status: &InstanceStatus, needle: &str) -> bool {
    if needle.is_empty() {
        return true;
    }

    let alias = status.instance.alias.as_deref().unwrap_or_default();
    let branch = status.git_status.branch.as_str();

    [
        alias,
        &status.instance.repo_name,
        branch,
        &status.instance.path,
    ]
    .iter()
    .any(|value| value.to_lowercase().contains(needle))
}
