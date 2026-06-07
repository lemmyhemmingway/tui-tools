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

pub struct BcryptTool {
    plaintext:  TextArea, // Pattern focus
    hash_field: TextArea, // Input focus
    result:     String,
    result_ok:  Option<bool>,
}

impl BcryptTool {
    pub fn new() -> Self {
        Self {
            plaintext:  TextArea::new(),
            hash_field: TextArea::new(),
            result:     String::new(),
            result_ok:  None,
        }
    }

    fn compute(&mut self) {
        let pw   = self.plaintext.content();
        let hash = self.hash_field.content().trim().to_string();

        if pw.trim().is_empty() {
            self.result    = String::new();
            self.result_ok = None;
            return;
        }

        if hash.is_empty() {
            match bcrypt::hash(pw.trim(), 12) {
                Ok(h)  => { self.result = h;              self.result_ok = Some(true); }
                Err(e) => { self.result = e.to_string();  self.result_ok = Some(false); }
            }
        } else {
            match bcrypt::verify(pw.trim(), &hash) {
                Ok(true)  => { self.result = "✓  Password matches the hash".to_string(); self.result_ok = Some(true); }
                Ok(false) => { self.result = "✗  Password does NOT match".to_string();   self.result_ok = Some(false); }
                Err(e)    => { self.result = format!("Error: {e}");                       self.result_ok = Some(false); }
            }
        }
    }
}

impl Tool for BcryptTool {
    fn name(&self) -> &'static str { "Bcrypt" }

    fn initial_focus(&self) -> Focus { Focus::Pattern }

    fn render(&mut self, frame: &mut Frame, area: Rect, focus: Focus) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Length(3), Constraint::Min(0)])
            .split(area);

        self.plaintext.render(frame, chunks[0], matches!(focus, Focus::Pattern), " Plaintext password ");
        self.hash_field.render(frame, chunks[1], matches!(focus, Focus::Input),  " Hash (leave empty to generate) ");

        let verify_mode = !self.hash_field.content().trim().is_empty();
        let mode_label = if verify_mode { "Verify mode" } else { "Hash mode — cost 12" };

        let (text, color) = if self.result.is_empty() {
            (
                vec![
                    Line::from(Span::styled(
                        format!("{mode_label} — press Enter to compute"),
                        Style::default().fg(Color::DarkGray),
                    )),
                ],
                Color::DarkGray,
            )
        } else if self.result_ok == Some(true) {
            (vec![Line::from(Span::styled(&self.result, Style::default().fg(Color::Green)))], Color::Green)
        } else {
            (vec![Line::from(Span::styled(&self.result, Style::default().fg(Color::Red)))], Color::Red)
        };

        frame.render_widget(
            Paragraph::new(text)
                .block(Block::default()
                    .title(format!(" Result — {mode_label} "))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(color))),
            chunks[2],
        );
    }

    fn handle_key(&mut self, key: KeyEvent, focus: Focus) -> Action {
        match key.code {
            KeyCode::Esc => return Action::FocusSidebar,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return Action::Quit,
            KeyCode::Tab => return if matches!(focus, Focus::Pattern) { Action::FocusInput } else { Action::FocusPattern },
            KeyCode::Enter => self.compute(),
            _ => {
                match focus {
                    Focus::Pattern => self.plaintext.handle_key_single(key),
                    Focus::Input   => self.hash_field.handle_key_single(key),
                    _ => {}
                }
                self.result    = String::new();
                self.result_ok = None;
            }
        }
        Action::Nothing
    }

    fn footer_hints(&self) -> String {
        "Tab: switch fields  Enter: hash / verify".to_string()
    }
}
