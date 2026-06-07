use crate::textarea::TextArea;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use super::{Action, Focus, Tool};

pub struct UriParserTool {
    input: TextArea,
}

impl UriParserTool {
    pub fn new() -> Self {
        Self { input: TextArea::new() }
    }
}

fn parse(raw: &str) -> Result<Vec<(&'static str, String)>, String> {
    let u = url::Url::parse(raw.trim()).map_err(|e| e.to_string())?;
    let mut rows: Vec<(&'static str, String)> = vec![];

    rows.push(("Scheme    ", u.scheme().to_string()));

    if !u.username().is_empty() {
        rows.push(("Username  ", u.username().to_string()));
        if let Some(pw) = u.password() {
            rows.push(("Password  ", pw.to_string()));
        }
    }

    if let Some(host) = u.host_str() {
        rows.push(("Host      ", host.to_string()));
    }

    if let Some(port) = u.port() {
        rows.push(("Port      ", port.to_string()));
    } else if let Some(port) = u.port_or_known_default() {
        rows.push(("Port      ", format!("{port} (default)")));
    }

    let path = u.path();
    if !path.is_empty() && path != "/" {
        rows.push(("Path      ", path.to_string()));
        let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        for (i, seg) in segments.iter().enumerate() {
            rows.push(("  segment ", format!("[{i}] {seg}")));
        }
    }

    if u.query().is_some() {
        let raw_q = u.query().unwrap_or("");
        rows.push(("Query     ", raw_q.to_string()));
        for (k, v) in u.query_pairs() {
            rows.push(("  param   ", format!("{k} = {v}")));
        }
    }

    if let Some(frag) = u.fragment() {
        rows.push(("Fragment  ", frag.to_string()));
    }

    Ok(rows)
}

impl Tool for UriParserTool {
    fn name(&self) -> &'static str { "URI Parser" }

    fn render(&mut self, frame: &mut Frame, area: Rect, focus: Focus) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(area);

        self.input.render(frame, chunks[0], matches!(focus, Focus::Input), " URI / URL ");

        let raw = self.input.content();
        let (lines, color) = if raw.trim().is_empty() {
            (
                vec![Line::from(Span::styled(
                    "paste a URL above  e.g. https://user:pw@host:8080/path?k=v#frag",
                    Style::default().fg(Color::DarkGray),
                ))],
                Color::DarkGray,
            )
        } else {
            match parse(&raw) {
                Ok(rows) => {
                    let lines = rows.into_iter().map(|(label, value)| {
                        Line::from(vec![
                            Span::styled(label, Style::default().fg(Color::DarkGray)),
                            Span::styled(format!("  {value}"), Style::default().fg(Color::White)),
                        ])
                    }).collect();
                    (lines, Color::Green)
                }
                Err(e) => (
                    vec![Line::from(Span::styled(e, Style::default().fg(Color::Red)))],
                    Color::Red,
                ),
            }
        };

        frame.render_widget(
            Paragraph::new(lines)
                .block(Block::default()
                    .title(" Parsed ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(color))),
            chunks[1],
        );
    }

    fn handle_key(&mut self, key: KeyEvent, _focus: Focus) -> Action {
        match key.code {
            KeyCode::Esc => return Action::FocusSidebar,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return Action::Quit,
            _ => { self.input.handle_key_single(key); }
        }
        Action::Nothing
    }

    fn footer_hints(&self) -> String { String::new() }
}
