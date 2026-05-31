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

pub struct NumberBaseTool {
    input: TextArea,
}

impl NumberBaseTool {
    pub fn new() -> Self {
        Self { input: TextArea::new() }
    }
}

fn parse_number(s: &str) -> Option<u64> {
    let s = s.trim();
    if s.is_empty() { return None; }
    if s.starts_with("0x") || s.starts_with("0X") {
        u64::from_str_radix(s[2..].trim(), 16).ok()
    } else if s.starts_with("0b") || s.starts_with("0B") {
        u64::from_str_radix(s[2..].trim(), 2).ok()
    } else if s.starts_with("0o") || s.starts_with("0O") {
        u64::from_str_radix(s[2..].trim(), 8).ok()
    } else if s.chars().all(|c| c.is_ascii_hexdigit()) && s.chars().any(|c| c.is_ascii_alphabetic()) {
        // Looks like bare hex (contains a-f/A-F with no prefix)
        u64::from_str_radix(s, 16).ok()
    } else {
        s.parse::<u64>().ok()
    }
}

fn format_binary_groups(n: u64) -> String {
    if n == 0 { return "0".to_string(); }
    let bits = format!("{n:b}");
    // Group into nibbles (4 bits) from the right
    let pad = (4 - bits.len() % 4) % 4;
    let padded = format!("{:0>width$}", bits, width = bits.len() + pad);
    padded
        .as_bytes()
        .chunks(4)
        .map(|c| std::str::from_utf8(c).unwrap())
        .collect::<Vec<_>>()
        .join(" ")
}

impl Tool for NumberBaseTool {
    fn name(&self) -> &'static str { "Number Base" }

    fn render(&mut self, frame: &mut Frame, area: Rect, focus: Focus) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(area);

        self.input.render(
            frame, chunks[0], matches!(focus, Focus::Input),
            " Input (decimal, 0x hex, 0b binary, 0o octal, or bare hex) ",
        );

        let raw = self.input.lines[0].trim().to_string();
        let (lines, border_color) = if raw.is_empty() {
            (vec![Line::from(Span::styled(
                "enter a number above",
                Style::default().fg(Color::DarkGray),
            ))], Color::DarkGray)
        } else {
            match parse_number(&raw) {
                Some(n) => {
                    let rows = vec![
                        row("Decimal ", &format!("{n}")),
                        row("Hex     ", &format!("0x{n:X}  ({n:x})")),
                        row("Octal   ", &format!("0o{n:o}")),
                        row("Binary  ", &format!("0b{}  ({})", format!("{n:b}"), format_binary_groups(n))),
                    ];
                    (rows, Color::Green)
                }
                None => {
                    (vec![Line::from(Span::styled(
                        format!("cannot parse: {raw}"),
                        Style::default().fg(Color::Red),
                    ))], Color::Red)
                }
            }
        };

        frame.render_widget(
            Paragraph::new(lines).block(
                Block::default()
                    .title(" Bases ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color)),
            ),
            chunks[1],
        );
    }

    fn handle_key(&mut self, key: KeyEvent, _focus: Focus) -> Action {
        match key.code {
            KeyCode::Esc => return Action::FocusSidebar,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                return Action::Quit
            }
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
