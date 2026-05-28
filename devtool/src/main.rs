mod textarea;
mod tools;

use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use std::io;
use tools::{Action, Focus, Tool};

struct App {
    selected: usize,
    focus: Focus,
    tools: Vec<Box<dyn Tool>>,
}

impl App {
    fn new() -> Self {
        Self {
            selected: 0,
            focus: Focus::Sidebar,
            tools: vec![],
        }
    }
}

fn draw(frame: &mut ratatui::Frame, app: &mut App) {
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(frame.area());

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(18), Constraint::Min(0)])
        .split(root[0]);

    let sidebar_area = columns[0];
    let content_area = columns[1];
    let footer_area = root[1];

    let items: Vec<ListItem> = app.tools.iter().enumerate().map(|(i, t)| {
        let style = if i == app.selected {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        ListItem::new(Span::styled(t.name(), style))
    }).collect();

    frame.render_widget(
        List::new(items).block(Block::default().borders(Borders::ALL).title("tools")),
        sidebar_area,
    );

    if !app.tools.is_empty() {
        let focus = app.focus;
        app.tools[app.selected].render(frame, content_area, focus);
    } else {
        frame.render_widget(
            Paragraph::new("no tools").block(Block::default().borders(Borders::ALL)),
            content_area,
        );
    }

    let hints = if app.focus == Focus::Sidebar {
        "j/k: navigate  Enter: focus tool  q: quit".to_string()
    } else if !app.tools.is_empty() {
        let tool_hints = app.tools[app.selected].footer_hints();
        if tool_hints.is_empty() {
            "Esc: sidebar  ctrl+c: quit".to_string()
        } else {
            format!("Esc: sidebar  {}  ctrl+c: quit", tool_hints)
        }
    } else {
        "q: quit".to_string()
    };
    frame.render_widget(
        Paragraph::new(hints).style(Style::default().fg(Color::DarkGray)),
        footer_area,
    );
}

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut app = App::new();

    loop {
        terminal.draw(|f| draw(f, &mut app))?;

        let Event::Key(key) = event::read()? else { continue };

        match app.focus {
            Focus::Sidebar => match key.code {
                KeyCode::Char('q') => break,
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                KeyCode::Char('j') | KeyCode::Down => {
                    if app.selected + 1 < app.tools.len() {
                        app.selected += 1;
                    }
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    if app.selected > 0 {
                        app.selected -= 1;
                    }
                }
                KeyCode::Enter | KeyCode::Tab => {
                    if !app.tools.is_empty() {
                        app.focus = app.tools[app.selected].initial_focus();
                    }
                }
                _ => {
                    if !app.tools.is_empty() {
                        let focus = app.focus;
                        app.tools[app.selected].handle_key(key, focus);
                    }
                }
            },
            Focus::Input | Focus::Pattern => {
                let focus = app.focus;
                let action = app.tools[app.selected].handle_key(key, focus);
                match action {
                    Action::Quit => break,
                    Action::FocusSidebar => app.focus = Focus::Sidebar,
                    Action::Nothing => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
