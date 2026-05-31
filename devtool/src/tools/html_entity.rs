use crate::textarea::TextArea;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use super::{Action, Focus, Tool};

pub struct HtmlEntityTool {
    input: TextArea,
}

impl HtmlEntityTool {
    pub fn new() -> Self {
        Self { input: TextArea::new() }
    }
}

fn encode(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            c if c.is_ascii() => out.push(c),
            c => out.push_str(&format!("&#{};", c as u32)),
        }
    }
    out
}

fn decode(text: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    while !rest.is_empty() {
        if let Some(pos) = rest.find('&') {
            out.push_str(&rest[..pos]);
            rest = &rest[pos..];
            if let Some(end) = rest.find(';') {
                let entity = &rest[1..end];
                let replacement = match entity {
                    "amp" => Some('&'),
                    "lt" => Some('<'),
                    "gt" => Some('>'),
                    "quot" => Some('"'),
                    "apos" => Some('\''),
                    "nbsp" => Some('\u{00A0}'),
                    "copy" => Some('\u{00A9}'),
                    "reg" => Some('\u{00AE}'),
                    "trade" => Some('\u{2122}'),
                    "mdash" => Some('\u{2014}'),
                    "ndash" => Some('\u{2013}'),
                    "laquo" => Some('\u{00AB}'),
                    "raquo" => Some('\u{00BB}'),
                    "hellip" => Some('\u{2026}'),
                    e if e.starts_with("#x") || e.starts_with("#X") => {
                        u32::from_str_radix(&e[2..], 16).ok().and_then(char::from_u32)
                    }
                    e if e.starts_with('#') => {
                        e[1..].parse::<u32>().ok().and_then(char::from_u32)
                    }
                    _ => None,
                };
                if let Some(ch) = replacement {
                    out.push(ch);
                    rest = &rest[end + 1..];
                } else {
                    out.push('&');
                    rest = &rest[1..];
                }
            } else {
                out.push('&');
                rest = &rest[1..];
            }
        } else {
            out.push_str(rest);
            break;
        }
    }
    out
}

impl Tool for HtmlEntityTool {
    fn name(&self) -> &'static str { "HTML Entity" }

    fn render(&mut self, frame: &mut Frame, area: Rect, focus: Focus) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(40),
                Constraint::Percentage(30),
                Constraint::Min(0),
            ])
            .split(area);

        self.input.render(frame, chunks[0], matches!(focus, Focus::Input), " Input ");

        let text = self.input.content();
        let encoded = encode(&text);
        let decoded = decode(&text);

        let enc_lines: Vec<Line> = encoded.lines().map(|l| Line::from(l.to_string())).collect();
        let dec_lines: Vec<Line> = decoded.lines().map(|l| Line::from(l.to_string())).collect();

        frame.render_widget(
            Paragraph::new(enc_lines).block(
                Block::default()
                    .title(" Encoded ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            ),
            chunks[1],
        );

        frame.render_widget(
            Paragraph::new(dec_lines).block(
                Block::default()
                    .title(" Decoded ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Magenta)),
            ),
            chunks[2],
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
        self.input.handle_key(key);
        Action::Nothing
    }

    fn footer_hints(&self) -> String { String::new() }
}
