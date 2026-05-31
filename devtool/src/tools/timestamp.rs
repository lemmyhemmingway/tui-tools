use crate::textarea::TextArea;
use chrono::{DateTime, Local, NaiveDate, TimeZone, Utc};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use super::{Action, Focus, Tool};

pub struct TimestampTool {
    input: TextArea,
}

impl TimestampTool {
    pub fn new() -> Self {
        Self { input: TextArea::new() }
    }

    fn parse(&self) -> Option<(DateTime<Utc>, bool)> {
        let s = self.input.lines[0].trim();
        if s.is_empty() {
            return Some((Utc::now(), true));
        }
        if let Ok(n) = s.parse::<i64>() {
            let dt = if n.abs() >= 10_000_000_000 {
                Utc.timestamp_millis_opt(n).single()?
            } else {
                Utc.timestamp_opt(n, 0).single()?
            };
            return Some((dt, false));
        }
        if let Ok(dt) = s.parse::<DateTime<Utc>>() {
            return Some((dt, false));
        }
        if let Ok(dt) = s.parse::<DateTime<Local>>() {
            return Some((dt.with_timezone(&Utc), false));
        }
        if let Ok(d) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
            let dt = d.and_hms_opt(0, 0, 0)?.and_utc();
            return Some((dt, false));
        }
        None
    }
}

impl Tool for TimestampTool {
    fn name(&self) -> &'static str { "Timestamp" }

    fn render(&mut self, frame: &mut Frame, area: Rect, focus: Focus) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(area);

        self.input.render(frame, chunks[0], matches!(focus, Focus::Input), " Input (unix or ISO) ");

        let (lines, border_color) = match self.parse() {
            Some((dt, is_now)) => {
                let local = dt.with_timezone(&Local);
                let label = if is_now { " [current time] " } else { "" };
                let rows: Vec<Line> = vec![
                    row("Unix (s)  ", &format!("{}{}", dt.timestamp(), label)),
                    row("Unix (ms) ", &dt.timestamp_millis().to_string()),
                    row("UTC       ", &dt.format("%Y-%m-%dT%H:%M:%SZ").to_string()),
                    row("Local     ", &local.format("%Y-%m-%d %H:%M:%S %z").to_string()),
                    row("Day       ", &dt.format("%A, %B %-d, %Y").to_string()),
                ];
                (rows, Color::Green)
            }
            None => {
                let s = self.input.lines[0].trim().to_string();
                if s.is_empty() {
                    (vec![], Color::DarkGray)
                } else {
                    (vec![Line::from(Span::styled(
                        format!("cannot parse: {s}"),
                        Style::default().fg(Color::Red),
                    ))], Color::Red)
                }
            }
        };

        frame.render_widget(
            Paragraph::new(lines).block(
                Block::default().title(" Output ").borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color))
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

fn row(label: &str, value: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(label.to_string(), Style::default().fg(Color::DarkGray)),
        Span::styled(value.to_string(), Style::default().fg(Color::White)),
    ])
}
