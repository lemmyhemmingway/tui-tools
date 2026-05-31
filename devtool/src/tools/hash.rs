use crate::textarea::TextArea;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use md5::Md5;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use sha1::Sha1;
use sha2::{Sha256, Sha512};
use super::{Action, Focus, Tool};

pub struct HashTool {
    input: TextArea,
}

impl HashTool {
    pub fn new() -> Self {
        Self { input: TextArea::new() }
    }
}

fn to_hex(bytes: impl AsRef<[u8]>) -> String {
    bytes.as_ref().iter().map(|b| format!("{b:02x}")).collect()
}

fn compute(data: &[u8]) -> [(&'static str, String); 4] {
    let md5_val = { use md5::Digest; to_hex(Md5::digest(data)) };
    let sha1_val = { use sha1::Digest; to_hex(Sha1::digest(data)) };
    let sha256_val = { use sha2::Digest; to_hex(Sha256::digest(data)) };
    let sha512_val = { use sha2::Digest; to_hex(Sha512::digest(data)) };
    [
        ("MD5    ", md5_val),
        ("SHA1   ", sha1_val),
        ("SHA256 ", sha256_val),
        ("SHA512 ", sha512_val),
    ]
}

impl Tool for HashTool {
    fn name(&self) -> &'static str { "Hash" }

    fn render(&mut self, frame: &mut Frame, area: Rect, focus: Focus) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(area);

        self.input.render(frame, chunks[0], matches!(focus, Focus::Input), " Input ");

        let data = self.input.content().into_bytes();
        let lines: Vec<Line> = if data.is_empty() {
            vec![Line::from(Span::styled("type something to hash", Style::default().fg(Color::DarkGray)))]
        } else {
            compute(&data).iter().map(|(label, value)| {
                Line::from(vec![
                    Span::styled(label.to_string(), Style::default().fg(Color::DarkGray)),
                    Span::styled(value.clone(), Style::default().fg(Color::White)),
                ])
            }).collect()
        };

        frame.render_widget(
            Paragraph::new(lines).block(
                Block::default().title(" Hashes ").borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan))
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
        self.input.handle_key(key);
        Action::Nothing
    }

    fn footer_hints(&self) -> String { String::new() }
}
