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
            tools: vec![
                Box::new(tools::json::JsonTool::new()),
                Box::new(tools::base64::Base64Tool::new()),
                Box::new(tools::jwt::JwtTool::new()),
                Box::new(tools::hash::HashTool::new()),
                Box::new(tools::url::UrlTool::new()),
                Box::new(tools::uuid::UuidTool::new()),
                Box::new(tools::timestamp::TimestampTool::new()),
                Box::new(tools::regex::RegexTool::new()),
                Box::new(tools::text_transform::TextTransformTool::new()),
                Box::new(tools::number_base::NumberBaseTool::new()),
                Box::new(tools::string_stats::StringStatsTool::new()),
                Box::new(tools::html_entity::HtmlEntityTool::new()),
                Box::new(tools::diff::DiffTool::new()),
                Box::new(tools::yaml_json::YamlJsonTool::new()),
                Box::new(tools::hmac_gen::HmacTool::new()),
                Box::new(tools::cron_expr::CronTool::new()),
            ],
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
        columns[0],
    );

    let focus = app.focus;
    app.tools[app.selected].render(frame, columns[1], focus);

    let hints = if app.focus == Focus::Sidebar {
        "j/k: navigate  Enter: focus tool  q: quit".to_string()
    } else {
        let tool_hints = app.tools[app.selected].footer_hints();
        if tool_hints.is_empty() {
            "Esc: sidebar  ctrl+c: quit".to_string()
        } else {
            format!("Esc: sidebar  {}  ctrl+c: quit", tool_hints)
        }
    };
    frame.render_widget(
        Paragraph::new(hints).style(Style::default().fg(Color::DarkGray)),
        root[1],
    );
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> io::Result<()> {
    loop {
        terminal.draw(|f| draw(f, app))?;

        let Event::Key(key) = event::read()? else { continue };

        match app.focus {
            Focus::Sidebar => match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return Ok(()),
                KeyCode::Char('j') | KeyCode::Down => {
                    if app.selected + 1 < app.tools.len() { app.selected += 1; }
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    if app.selected > 0 { app.selected -= 1; }
                }
                KeyCode::Enter | KeyCode::Tab => {
                    app.focus = app.tools[app.selected].initial_focus();
                }
                _ => {}
            },
            Focus::Input | Focus::Pattern => {
                let focus = app.focus;
                let action = app.tools[app.selected].handle_key(key, focus);
                match action {
                    Action::Quit => return Ok(()),
                    Action::FocusSidebar => app.focus = Focus::Sidebar,
                    Action::FocusInput => app.focus = Focus::Input,
                    Action::FocusPattern => app.focus = Focus::Pattern,
                    Action::Nothing => {}
                }
            }
        }
    }
}

fn run() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut app = App::new();
    let result = run_app(&mut terminal, &mut app);
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    result
}

fn main() -> io::Result<()> {
    run()
}
