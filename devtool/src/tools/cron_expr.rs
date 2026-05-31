use crate::textarea::TextArea;
use chrono::Utc;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::str::FromStr;
use super::{Action, Focus, Tool};

pub struct CronTool {
    input: TextArea,
}

impl CronTool {
    pub fn new() -> Self {
        Self { input: TextArea::new() }
    }
}

// The cron crate uses 7 fields (sec min hour dom month dow year).
// Accept standard 5-field (min hour dom month dow) by prepending seconds and appending year.
fn normalize(expr: &str) -> String {
    let fields: Vec<&str> = expr.split_whitespace().collect();
    match fields.len() {
        5 => format!("0 {} *", expr.trim()),
        6 => format!("{} *", expr.trim()),
        _ => expr.trim().to_string(),
    }
}

fn parse_and_schedule(expr: &str) -> Result<Vec<String>, String> {
    let normalized = normalize(expr);
    let schedule = cron::Schedule::from_str(&normalized).map_err(|e| e.to_string())?;
    let times: Vec<String> = schedule
        .upcoming(Utc)
        .take(8)
        .map(|t| t.format("%Y-%m-%d %H:%M:%S UTC").to_string())
        .collect();
    Ok(times)
}

impl Tool for CronTool {
    fn name(&self) -> &'static str { "Cron" }

    fn render(&mut self, frame: &mut Frame, area: Rect, focus: Focus) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(area);

        self.input.render(frame, chunks[0], matches!(focus, Focus::Input), " Expression (5-field or 6-field with seconds) ");

        let expr = self.input.lines[0].trim().to_string();
        let (lines, border_color) = if expr.is_empty() {
            let examples = vec![
                Line::from(Span::styled("examples:", Style::default().fg(Color::DarkGray))),
                Line::from(vec![
                    Span::styled("  * * * * *       ", Style::default().fg(Color::Yellow)),
                    Span::styled("every minute", Style::default().fg(Color::DarkGray)),
                ]),
                Line::from(vec![
                    Span::styled("  0 9 * * 1-5     ", Style::default().fg(Color::Yellow)),
                    Span::styled("weekdays at 09:00", Style::default().fg(Color::DarkGray)),
                ]),
                Line::from(vec![
                    Span::styled("  */15 * * * *    ", Style::default().fg(Color::Yellow)),
                    Span::styled("every 15 minutes", Style::default().fg(Color::DarkGray)),
                ]),
                Line::from(vec![
                    Span::styled("  0 0 1 * *       ", Style::default().fg(Color::Yellow)),
                    Span::styled("first of month at midnight", Style::default().fg(Color::DarkGray)),
                ]),
                Line::from(vec![
                    Span::styled("  30 9 * * MON    ", Style::default().fg(Color::Yellow)),
                    Span::styled("mondays at 09:30", Style::default().fg(Color::DarkGray)),
                ]),
            ];
            (examples, Color::DarkGray)
        } else {
            match parse_and_schedule(&expr) {
                Ok(times) => {
                    let normalized = normalize(&expr);
                    let mut out = vec![
                        Line::from(vec![
                            Span::styled("normalized  ", Style::default().fg(Color::DarkGray)),
                            Span::styled(normalized, Style::default().fg(Color::DarkGray)),
                        ]),
                        Line::from(""),
                        Line::from(Span::styled("next 8 fire times:", Style::default().fg(Color::DarkGray))),
                    ];
                    for (i, t) in times.iter().enumerate() {
                        out.push(Line::from(vec![
                            Span::styled(
                                format!("  {:>2}.  ", i + 1),
                                Style::default().fg(Color::DarkGray),
                            ),
                            Span::styled(t.clone(), Style::default().fg(Color::White)),
                        ]));
                    }
                    (out, Color::Green)
                }
                Err(e) => (
                    vec![Line::from(Span::styled(
                        format!("error: {e}"),
                        Style::default().fg(Color::Red),
                    ))],
                    Color::Red,
                ),
            }
        };

        frame.render_widget(
            Paragraph::new(lines).block(
                Block::default()
                    .title(" Schedule ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color)),
            ),
            chunks[1],
        );
    }

    fn handle_key(&mut self, key: KeyEvent, _focus: Focus) -> Action {
        match key.code {
            KeyCode::Esc => return Action::FocusSidebar,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return Action::Quit,
            _ => {}
        }
        self.input.handle_key_single(key);
        Action::Nothing
    }

    fn footer_hints(&self) -> String { String::new() }
}
