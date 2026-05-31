use crate::textarea::TextArea;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use hmac::{Hmac, Mac};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use sha2::{Sha256, Sha512};
use super::{Action, Focus, Tool};

type HmacSha256 = Hmac<Sha256>;
type HmacSha512 = Hmac<Sha512>;

pub struct HmacTool {
    message: TextArea,
    key: TextArea,
}

impl HmacTool {
    pub fn new() -> Self {
        Self { message: TextArea::new(), key: TextArea::new() }
    }

    fn compute(&self) -> Option<[(&'static str, String); 2]> {
        let msg = self.message.content();
        let key = self.key.lines[0].as_bytes();

        let mut mac256 = HmacSha256::new_from_slice(key).ok()?;
        mac256.update(msg.as_bytes());
        let h256 = hex::encode(mac256.finalize().into_bytes());

        let mut mac512 = HmacSha512::new_from_slice(key).ok()?;
        mac512.update(msg.as_bytes());
        let h512 = hex::encode(mac512.finalize().into_bytes());

        Some([("HMAC-SHA256  ", h256), ("HMAC-SHA512  ", h512)])
    }
}

impl Tool for HmacTool {
    fn name(&self) -> &'static str { "HMAC" }

    fn initial_focus(&self) -> Focus { Focus::Pattern }

    fn render(&mut self, frame: &mut Frame, area: Rect, focus: Focus) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Percentage(50), Constraint::Min(0)])
            .split(area);

        self.key.render(frame, chunks[0], matches!(focus, Focus::Pattern), " Key (Tab) ");
        self.message.render(frame, chunks[1], matches!(focus, Focus::Input), " Message (Tab) ");

        let (lines, border_color) = match self.compute() {
            Some(hashes) => {
                let rows = hashes
                    .iter()
                    .map(|(label, value)| {
                        Line::from(vec![
                            Span::styled(label.to_string(), Style::default().fg(Color::DarkGray)),
                            Span::styled(value.clone(), Style::default().fg(Color::White)),
                        ])
                    })
                    .collect();
                (rows, Color::Green)
            }
            None => (
                vec![Line::from(Span::styled(
                    "enter a key and message above",
                    Style::default().fg(Color::DarkGray),
                ))],
                Color::DarkGray,
            ),
        };

        frame.render_widget(
            Paragraph::new(lines).block(
                Block::default()
                    .title(" Output ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color)),
            ),
            chunks[2],
        );
    }

    fn handle_key(&mut self, key: KeyEvent, focus: Focus) -> Action {
        match key.code {
            KeyCode::Esc => return Action::FocusSidebar,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return Action::Quit,
            KeyCode::Tab => return match focus {
                Focus::Pattern => Action::FocusInput,
                _ => Action::FocusPattern,
            },
            _ => {}
        }
        match focus {
            Focus::Pattern => self.key.handle_key_single(key),
            Focus::Input => self.message.handle_key(key),
            _ => {}
        }
        Action::Nothing
    }

    fn footer_hints(&self) -> String { "Tab: switch field".to_string() }
}
