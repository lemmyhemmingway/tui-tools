use crate::textarea::TextArea;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use super::{Action, Focus, Tool};

pub struct JwtTool {
    input: TextArea,
}

impl JwtTool {
    pub fn new() -> Self {
        Self { input: TextArea::new() }
    }

    fn decode(&self) -> (String, String, String) {
        let token: String = self.input.content().split_whitespace().collect();
        if token.is_empty() {
            return (String::new(), String::new(), String::new());
        }
        let parts: Vec<&str> = token.splitn(3, '.').collect();
        if parts.len() != 3 {
            return (String::from("Invalid: expected header.payload.signature"), String::new(), String::new());
        }
        let decode_part = |s: &str| -> String {
            match URL_SAFE_NO_PAD.decode(s) {
                Ok(bytes) => serde_json::from_slice::<serde_json::Value>(&bytes)
                    .ok()
                    .and_then(|v| serde_json::to_string_pretty(&v).ok())
                    .unwrap_or_else(|| String::from_utf8_lossy(&bytes).into_owned()),
                Err(e) => format!("decode error: {e}"),
            }
        };
        (decode_part(parts[0]), decode_part(parts[1]), parts[2].to_string())
    }
}

impl Tool for JwtTool {
    fn name(&self) -> &'static str { "JWT" }

    fn render(&mut self, frame: &mut Frame, area: Rect, focus: Focus) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(5), Constraint::Min(0)])
            .split(area);

        self.input.render(frame, chunks[0], matches!(focus, Focus::Input), " Token ");

        let (header, payload, sig) = self.decode();

        let inner = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(45), Constraint::Percentage(45), Constraint::Min(0)])
            .split(chunks[1]);

        let h_color = if header.starts_with("Invalid") || header.starts_with("decode error") {
            Color::Red
        } else {
            Color::Cyan
        };
        let h_lines: Vec<Line> = header.lines().map(|l| Line::from(l.to_owned())).collect();
        frame.render_widget(
            Paragraph::new(h_lines).block(
                Block::default().title(" Header ").borders(Borders::ALL)
                    .border_style(Style::default().fg(h_color))
            ),
            inner[0],
        );

        let p_color = if payload.is_empty() { Color::DarkGray } else { Color::Green };
        let p_lines: Vec<Line> = payload.lines().map(|l| Line::from(l.to_owned())).collect();
        frame.render_widget(
            Paragraph::new(p_lines).block(
                Block::default().title(" Payload ").borders(Borders::ALL)
                    .border_style(Style::default().fg(p_color))
            ),
            inner[1],
        );

        let sig_display = if sig.len() > 60 {
            format!("{}…", &sig[..60])
        } else {
            sig
        };
        frame.render_widget(
            Paragraph::new(sig_display).block(
                Block::default().title(" Signature ").borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray))
            ),
            inner[2],
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
