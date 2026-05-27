use std::collections::HashSet;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

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

fn history_path() -> PathBuf {
    env::var("HISTFILE")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
            PathBuf::from(home).join(".zsh_history")
        })
}

fn parse_history(path: &Path) -> Vec<String> {
    let bytes = fs::read(path).unwrap_or_default();
    let content = String::from_utf8_lossy(&bytes).into_owned();

    let mut commands: Vec<String> = Vec::new();
    let mut current: Option<String> = None;

    for line in content.lines() {
        // Extended format: `: 1234567890:0;command`
        if line.starts_with(": ") {
            if let Some(pos) = line.find(';') {
                if let Some(cmd) = current.take() {
                    commands.push(cmd);
                }
                let cmd = line[pos + 1..].to_string();
                if !cmd.is_empty() {
                    current = Some(cmd);
                }
                continue;
            }
        }

        if let Some(ref mut cmd) = current {
            if cmd.ends_with('\\') {
                cmd.pop();
                cmd.push('\n');
                cmd.push_str(line);
                continue;
            } else {
                commands.push(current.take().unwrap());
            }
        }

        if !line.is_empty() && current.is_none() {
            commands.push(line.to_string());
        }
    }
    if let Some(cmd) = current {
        commands.push(cmd);
    }

    // Dedup, keep most recent occurrence, most recent first
    let mut seen = HashSet::new();
    commands
        .into_iter()
        .rev()
        .filter(|cmd| seen.insert(cmd.clone()))
        .collect()
}

fn fuzzy_match(item: &str, query: &str) -> bool {
    if query.is_empty() {
        return true;
    }
    let item = item.to_lowercase();
    let query = query.to_lowercase();
    let mut chars = item.chars();
    for q in query.chars() {
        if !chars.any(|c| c == q) {
            return false;
        }
    }
    true
}

struct App {
    all: Vec<String>,
    filtered: Vec<String>,
    query: String,
    list_state: ListState,
}

impl App {
    fn new(commands: Vec<String>) -> Self {
        let filtered = commands.clone();
        let mut list_state = ListState::default();
        if !filtered.is_empty() {
            list_state.select(Some(0));
        }
        Self { all: commands, filtered, query: String::new(), list_state }
    }

    fn update_filter(&mut self) {
        self.filtered =
            self.all.iter().filter(|c| fuzzy_match(c, &self.query)).cloned().collect();
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
    commands: Vec<String>,
) -> io::Result<Option<String>> {
    let mut app = App::new(commands);

    loop {
        terminal.draw(|f| {
            let area = f.area();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(1)])
                .split(area);

            let input_text = Line::from(vec![
                Span::styled(" > ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
                Span::styled(app.query.as_str(), Style::default().fg(Color::White)),
                Span::styled("█", Style::default().fg(Color::Magenta)),
            ]);
            let input = Paragraph::new(input_text).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Magenta))
                    .title(Span::styled(
                        " recall ",
                        Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
                    )),
            );
            f.render_widget(input, chunks[0]);

            let items: Vec<ListItem> = app
                .filtered
                .iter()
                .map(|cmd| {
                    let display = cmd.lines().next().unwrap_or(cmd);
                    ListItem::new(Line::from(vec![
                        Span::raw("  "),
                        Span::styled(display, Style::default().fg(Color::White)),
                    ]))
                })
                .collect();

            let title = format!(" {} commands ", app.filtered.len());
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
                        .bg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol("▶ ");

            f.render_stateful_widget(list, chunks[1], &mut app.list_state);

            let help = Paragraph::new(Line::from(vec![
                Span::styled("  ↑↓ / ctrl+jk", Style::default().fg(Color::DarkGray)),
                Span::styled("  navigate", Style::default().fg(Color::DarkGray)),
                Span::raw("   "),
                Span::styled("enter", Style::default().fg(Color::DarkGray)),
                Span::styled("  paste to prompt", Style::default().fg(Color::DarkGray)),
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let output_path = args.get(1).cloned();

    let commands = parse_history(&history_path());

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal, commands);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Ok(Some(cmd)) = result {
        if let Some(path) = output_path {
            fs::write(path, &cmd)?;
        } else {
            print!("{}", cmd);
        }
    }

    Ok(())
}
