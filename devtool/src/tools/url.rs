use crate::textarea::TextArea;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};
use super::{Action, Focus, Tool};

pub struct UrlTool {
    input: TextArea,
}

impl UrlTool {
    pub fn new() -> Self {
        Self { input: TextArea::new() }
    }
}

fn percent_encode(s: &str) -> String {
    let mut out = String::new();
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char);
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

fn percent_decode(s: &str) -> Result<String, String> {
    let mut bytes: Vec<u8> = Vec::new();
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '%' && i + 2 < chars.len() {
            let hex: String = [chars[i + 1], chars[i + 2]].iter().collect();
            match u8::from_str_radix(&hex, 16) {
                Ok(b) => { bytes.push(b); i += 3; }
                Err(_) => { bytes.push(b'%'); i += 1; }
            }
        } else if chars[i] == '+' {
            bytes.push(b' ');
            i += 1;
        } else {
            let mut buf = [0u8; 4];
            bytes.extend_from_slice(chars[i].encode_utf8(&mut buf).as_bytes());
            i += 1;
        }
    }
    String::from_utf8(bytes).map_err(|e| e.to_string())
}

impl Tool for UrlTool {
    fn name(&self) -> &'static str { "URL" }

    fn render(&mut self, frame: &mut Frame, area: Rect, focus: Focus) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(30), Constraint::Percentage(30)])
            .split(area);

        self.input.render(frame, chunks[0], matches!(focus, Focus::Input), " Input ");

        let content = self.input.content();

        let encoded = percent_encode(&content);
        frame.render_widget(
            Paragraph::new(encoded)
                .wrap(Wrap { trim: false })
                .block(Block::default().title(" Encoded ").borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan))),
            chunks[1],
        );

        let (dec_text, dec_color) = match percent_decode(&content) {
            Ok(s) => (s, Color::Green),
            Err(e) => (format!("decode error: {e}"), Color::Red),
        };
        let dec_lines: Vec<Line> = dec_text.lines().map(|l| Line::from(l.to_owned())).collect();
        frame.render_widget(
            Paragraph::new(dec_lines)
                .block(Block::default().title(" Decoded ").borders(Borders::ALL)
                    .border_style(Style::default().fg(dec_color))),
            chunks[2],
        );
    }

    fn handle_key(&mut self, key: KeyEvent, _focus: Focus) -> Action {
        match key.code {
            KeyCode::Esc => return Action::FocusSidebar,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return Action::Quit,
            _ => {}
        }
        self.input.handle_key(key);
        Action::Nothing
    }

    fn footer_hints(&self) -> String { String::new() }
}
