use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use uuid::Uuid;
use super::{Action, Focus, Tool};

pub struct UuidTool {
    current: String,
    history: Vec<String>,
}

impl UuidTool {
    pub fn new() -> Self {
        let first = Uuid::new_v4().to_string();
        Self { current: first.clone(), history: vec![first] }
    }

    fn generate(&mut self) {
        let id = Uuid::new_v4().to_string();
        self.current = id.clone();
        self.history.push(id);
        if self.history.len() > 10 {
            self.history.remove(0);
        }
    }
}

impl Tool for UuidTool {
    fn name(&self) -> &'static str { "UUID" }

    fn render(&mut self, frame: &mut Frame, area: Rect, _focus: Focus) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(area);

        frame.render_widget(
            Paragraph::new(Span::styled(
                self.current.clone(),
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            ))
            .block(Block::default().title(" Generated ").borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green))),
            chunks[0],
        );

        let hist_lines: Vec<Line> = self.history.iter().rev().skip(1)
            .map(|id| Line::from(Span::styled(id.clone(), Style::default().fg(Color::DarkGray))))
            .collect();
        frame.render_widget(
            Paragraph::new(hist_lines)
                .block(Block::default().title(" History ").borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray))),
            chunks[1],
        );
    }

    fn handle_key(&mut self, key: KeyEvent, _focus: Focus) -> Action {
        match key.code {
            KeyCode::Esc => return Action::FocusSidebar,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return Action::Quit,
            KeyCode::Enter | KeyCode::Char(' ') => self.generate(),
            _ => {}
        }
        Action::Nothing
    }

    fn footer_hints(&self) -> String { "Enter/Space: new UUID".to_string() }
}
