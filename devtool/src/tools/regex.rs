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

pub struct RegexTool {
    pattern: TextArea,
    test_input: TextArea,
}

impl RegexTool {
    pub fn new() -> Self {
        Self { pattern: TextArea::new(), test_input: TextArea::new() }
    }

    fn matches(&self) -> Result<Vec<(String, usize, usize)>, String> {
        let pat = self.pattern.lines[0].trim();
        if pat.is_empty() { return Ok(vec![]); }
        let re = regex::Regex::new(pat).map_err(|e| e.to_string())?;
        let text = self.test_input.content();
        Ok(re.find_iter(&text).map(|m| (m.as_str().to_string(), m.start(), m.end())).collect())
    }
}

impl Tool for RegexTool {
    fn name(&self) -> &'static str { "Regex" }

    fn initial_focus(&self) -> Focus { Focus::Pattern }

    fn render(&mut self, frame: &mut Frame, area: Rect, focus: Focus) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Percentage(50), Constraint::Min(0)])
            .split(area);

        self.pattern.render(frame, chunks[0], matches!(focus, Focus::Pattern), " Pattern ");
        self.test_input.render(frame, chunks[1], matches!(focus, Focus::Input), " Test String (Tab to switch) ");

        let result = self.matches();
        let (lines, border_color) = match result {
            Ok(ref ms) => {
                let mut out: Vec<Line> = vec![Line::from(Span::styled(
                    if ms.is_empty() { "no matches".to_string() } else { format!("{} match{}", ms.len(), if ms.len() == 1 { "" } else { "es" }) },
                    Style::default().fg(if ms.is_empty() { Color::DarkGray } else { Color::Green }),
                ))];
                for (i, (text, start, end)) in ms.iter().enumerate() {
                    out.push(Line::from(vec![
                        Span::styled(format!("  [{}] ", i + 1), Style::default().fg(Color::DarkGray)),
                        Span::styled(format!("{text:?}"), Style::default().fg(Color::White)),
                        Span::styled(format!("  {start}..{end}"), Style::default().fg(Color::DarkGray)),
                    ]));
                }
                (out, Color::Green)
            }
            Err(ref e) => {
                (vec![Line::from(Span::styled(
                    format!("error: {e}"),
                    Style::default().fg(Color::Red),
                ))], Color::Red)
            }
        };

        frame.render_widget(
            Paragraph::new(lines).block(
                Block::default().title(" Matches ").borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color))
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
            Focus::Pattern => self.pattern.handle_key_single(key),
            Focus::Input => self.test_input.handle_key(key),
            _ => {}
        }
        Action::Nothing
    }

    fn footer_hints(&self) -> String { "Tab: switch field".to_string() }
}
