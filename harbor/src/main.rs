use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Terminal,
};

fn projects_dir() -> PathBuf {
    std::env::var("PROJECTS_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
            PathBuf::from(home).join("github.com")
        })
}

fn get_projects(dir: &Path) -> Vec<String> {
    let mut projects = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    projects.push(name.to_string());
                }
            }
        }
    }
    projects.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
    projects
}

fn fuzzy_match(item: &str, query: &str) -> bool {
    if query.is_empty() {
        return true;
    }
    let item = item.to_lowercase();
    let query = query.to_lowercase();
    let mut item_chars = item.chars();
    for q in query.chars() {
        if !item_chars.any(|c| c == q) {
            return false;
        }
    }
    true
}

struct App {
    all_projects: Vec<String>,
    filtered: Vec<String>,
    query: String,
    list_state: ListState,
}

impl App {
    fn new(projects: Vec<String>) -> Self {
        let filtered = projects.clone();
        let mut list_state = ListState::default();
        if !filtered.is_empty() {
            list_state.select(Some(0));
        }
        Self { all_projects: projects, filtered, query: String::new(), list_state }
    }

    fn update_filter(&mut self) {
        self.filtered = self
            .all_projects
            .iter()
            .filter(|p| fuzzy_match(p, &self.query))
            .cloned()
            .collect();

        if self.filtered.is_empty() {
            self.list_state.select(None);
        } else {
            let i = self.list_state.selected().unwrap_or(0).min(self.filtered.len() - 1);
            self.list_state.select(Some(i));
        }
    }

    fn selected(&self) -> Option<&str> {
        self.list_state.selected().and_then(|i| self.filtered.get(i)).map(|s| s.as_str())
    }

    fn move_down(&mut self) {
        if self.filtered.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => (i + 1).min(self.filtered.len() - 1),
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn move_up(&mut self) {
        if self.filtered.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(0) | None => 0,
            Some(i) => i - 1,
        };
        self.list_state.select(Some(i));
    }
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    projects: Vec<String>,
) -> io::Result<Option<String>> {
    let mut app = App::new(projects);

    loop {
        terminal.draw(|f| {
            let area = f.area();

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(1)])
                .split(area);

            // Search bar
            let input_text = Line::from(vec![
                Span::styled(" > ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(app.query.as_str(), Style::default().fg(Color::White)),
                Span::styled("█", Style::default().fg(Color::Cyan)),
            ]);
            let input = Paragraph::new(input_text).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan))
                    .title(Span::styled(
                        " harbor ",
                        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                    )),
            );
            f.render_widget(input, chunks[0]);

            // Project list
            let items: Vec<ListItem> = app
                .filtered
                .iter()
                .map(|p| {
                    ListItem::new(Line::from(vec![
                        Span::raw("  "),
                        Span::styled(p, Style::default().fg(Color::White)),
                    ]))
                })
                .collect();

            let title = format!(" {} projects ", app.filtered.len());
            let list = List::new(items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::DarkGray))
                        .title(Span::styled(title, Style::default().fg(Color::DarkGray))),
                )
                .highlight_style(
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol("▶ ");

            f.render_stateful_widget(list, chunks[1], &mut app.list_state);

            // Help line
            let help = Paragraph::new(Line::from(vec![
                Span::styled("  ↑↓ / ctrl+jk", Style::default().fg(Color::DarkGray)),
                Span::styled("  navigate", Style::default().fg(Color::DarkGray)),
                Span::raw("   "),
                Span::styled("enter", Style::default().fg(Color::DarkGray)),
                Span::styled("  open session", Style::default().fg(Color::DarkGray)),
                Span::raw("   "),
                Span::styled("esc", Style::default().fg(Color::DarkGray)),
                Span::styled("  quit", Style::default().fg(Color::DarkGray)),
            ]));
            f.render_widget(help, chunks[2]);
        })?;

        if let Event::Key(key) = event::read()? {
            match (key.code, key.modifiers) {
                (KeyCode::Esc, _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                    return Ok(None);
                }
                (KeyCode::Enter, _) => {
                    return Ok(app.selected().map(|s| s.to_string()));
                }
                (KeyCode::Down, _) | (KeyCode::Char('j'), KeyModifiers::CONTROL) => {
                    app.move_down();
                }
                (KeyCode::Up, _) | (KeyCode::Char('k'), KeyModifiers::CONTROL) => {
                    app.move_up();
                }
                (KeyCode::Backspace, _) => {
                    app.query.pop();
                    app.update_filter();
                }
                (KeyCode::Char(c), KeyModifiers::NONE) | (KeyCode::Char(c), KeyModifiers::SHIFT) => {
                    app.query.push(c);
                    app.update_filter();
                }
                _ => {}
            }
        }
    }
}

fn in_tmux() -> bool {
    std::env::var("TMUX").is_ok()
}

fn tmux_session_exists(name: &str) -> bool {
    Command::new("tmux")
        .args(["has-session", "-t", name])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn open_session(project: &str, path: &Path) {
    let session = project.replace(['.', ' ', ':'], "_");
    let path_str = path.to_str().unwrap_or(".");

    if tmux_session_exists(&session) {
        if in_tmux() {
            Command::new("tmux")
                .args(["switch-client", "-t", &session])
                .status()
                .ok();
        } else {
            Command::new("tmux")
                .args(["attach-session", "-t", &session])
                .status()
                .ok();
        }
    } else if in_tmux() {
        Command::new("tmux")
            .args(["new-session", "-ds", &session, "-c", path_str])
            .status()
            .ok();
        Command::new("tmux")
            .args(["switch-client", "-t", &session])
            .status()
            .ok();
    } else {
        Command::new("tmux")
            .args(["new-session", "-s", &session, "-c", path_str])
            .status()
            .ok();
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = projects_dir();
    let projects = get_projects(&dir);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal, projects);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Ok(Some(project)) = result {
        let project_path = dir.join(&project);
        open_session(&project, &project_path);
    }

    Ok(())
}
