use crate::textarea::TextArea;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use similar::{ChangeTag, TextDiff};
use super::{Action, Focus, Tool};

pub struct DiffTool {
    before: TextArea,
    after: TextArea,
}

impl DiffTool {
    pub fn new() -> Self {
        Self { before: TextArea::new(), after: TextArea::new() }
    }
}

impl Tool for DiffTool {
    fn name(&self) -> &'static str { "Diff" }

    fn initial_focus(&self) -> Focus { Focus::Pattern }

    fn render(&mut self, frame: &mut Frame, area: Rect, focus: Focus) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(35), Constraint::Percentage(35), Constraint::Min(0)])
            .split(area);

        let inputs = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[0]);

        self.before.render(frame, inputs[0], matches!(focus, Focus::Pattern), " Before (Tab) ");
        self.after.render(frame, inputs[1], matches!(focus, Focus::Input), " After (Tab) ");

        let before_text = self.before.content();
        let after_text = self.after.content();
        let diff = TextDiff::from_lines(before_text.as_str(), after_text.as_str());

        let mut lines: Vec<Line> = Vec::new();
        let mut changes = 0usize;

        for change in diff.iter_all_changes() {
            let (prefix, style) = match change.tag() {
                ChangeTag::Delete => {
                    changes += 1;
                    ("-", Style::default().fg(Color::Red))
                }
                ChangeTag::Insert => {
                    changes += 1;
                    ("+", Style::default().fg(Color::Green))
                }
                ChangeTag::Equal => (" ", Style::default().fg(Color::DarkGray)),
            };
            let text = change.to_string();
            let text = text.trim_end_matches('\n');
            lines.push(Line::from(vec![
                Span::styled(prefix, style.add_modifier(Modifier::BOLD)),
                Span::styled(format!(" {text}"), style),
            ]));
        }

        let title = if before_text.is_empty() && after_text.is_empty() {
            " Diff ".to_string()
        } else if changes == 0 {
            " Diff — identical ".to_string()
        } else {
            format!(" Diff — {changes} change{} ", if changes == 1 { "" } else { "s" })
        };

        let border_color = if before_text.is_empty() && after_text.is_empty() {
            Color::DarkGray
        } else if changes == 0 {
            Color::Green
        } else {
            Color::Yellow
        };

        frame.render_widget(
            Paragraph::new(lines).block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color)),
            ),
            chunks[1],
        );

        // stats row
        let before_lines = before_text.lines().count();
        let after_lines = after_text.lines().count();
        let stats = format!(" before: {before_lines} lines   after: {after_lines} lines ");
        frame.render_widget(
            Paragraph::new(stats).style(Style::default().fg(Color::DarkGray)).block(
                Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)),
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
            Focus::Pattern => self.before.handle_key(key),
            Focus::Input => self.after.handle_key(key),
            _ => {}
        }
        Action::Nothing
    }

    fn footer_hints(&self) -> String { "Tab: switch panel".to_string() }
}
