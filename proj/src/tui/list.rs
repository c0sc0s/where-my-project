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
            Constraint::Length(3),
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
                Row::new(["Repository", "Branch", "Path"]).style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
            )
            .widths(&[
                Constraint::Length(25),
                Constraint::Length(25),
                Constraint::Percentage(100),
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

    let footer_text = if let Some(path) = app.selected_path() {
        format!(
            "↑/↓ move  Enter select  / search  r refresh  q/Esc quit\n{}",
            path
        )
    } else {
        "↑/↓ move  Enter select  / search  r refresh  q/Esc quit".to_string()
    };
    let footer = Paragraph::new(footer_text).block(Block::default().borders(Borders::ALL));
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

fn find_common_prefix(paths: &[&str]) -> String {
    if paths.is_empty() {
        return String::new();
    }
    if paths.len() == 1 {
        return String::new();
    }

    let first = paths[0].as_bytes();
    let mut prefix_len = 0;

    'outer: for i in 0..first.len() {
        let ch = first[i];
        for path in &paths[1..] {
            if i >= path.len() || path.as_bytes()[i] != ch {
                break 'outer;
            }
        }
        prefix_len = i + 1;
    }

    // 回退到最后一个路径分隔符
    let prefix = &paths[0][..prefix_len];
    if let Some(pos) = prefix.rfind(|c| c == '\\' || c == '/') {
        paths[0][..pos].to_string()
    } else {
        String::new()
    }
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
        let paths: Vec<&str> = self
            .filtered
            .iter()
            .filter_map(|index| self.statuses.get(*index))
            .map(|s| s.instance.path.as_str())
            .collect();

        let common_prefix = find_common_prefix(&paths);

        self.filtered
            .iter()
            .filter_map(|index| self.statuses.get(*index))
            .map(|status| {
                let branch = status.git_status.branch.as_str();
                let path = if common_prefix.is_empty() {
                    status.instance.path.clone()
                } else {
                    format!(
                        "~/{}",
                        &status.instance.path[common_prefix.len()..]
                            .trim_start_matches('\\')
                            .trim_start_matches('/')
                    )
                };

                Row::new(vec![
                    Cell::from(status.instance.repo_name.clone()),
                    Cell::from(branch.to_string()),
                    Cell::from(path),
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

    let branch = status.git_status.branch.as_str();

    [&status.instance.repo_name, branch, &status.instance.path]
        .iter()
        .any(|value| value.to_lowercase().contains(needle))
}
