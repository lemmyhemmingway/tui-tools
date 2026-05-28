use crate::textarea::TextArea;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use serde_json::Value;
use super::{Action, Focus, Tool};

pub struct JsonTool {
    input: TextArea,
    output: String,
    is_error: bool,
}

impl JsonTool {
    pub fn new() -> Self {
        Self { input: TextArea::new(), output: String::new(), is_error: false }
    }
}

fn format_json(input: &str) -> Result<String, String> {
    if input.trim().is_empty() {
        return Ok(String::new());
    }
    let v: Value = serde_json::from_str(input).map_err(|e| e.to_string())?;
    serde_json::to_string_pretty(&v).map_err(|e| e.to_string())
}

impl Tool for JsonTool {
    fn name(&self) -> &'static str { "JSON" }

    fn render(&mut self, frame: &mut Frame, area: Rect, focus: Focus) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        self.input.render(frame, chunks[0], matches!(focus, Focus::Input), "Input");

        let output_style = if self.is_error {
            Style::default().fg(Color::Red)
        } else {
            Style::default().fg(Color::Green)
        };
        frame.render_widget(
            Paragraph::new(self.output.clone())
                .style(output_style)
                .block(Block::default().borders(Borders::ALL).title("Output")),
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
        self.input.handle_key(key);
        match format_json(&self.input.content()) {
            Ok(s) => { self.output = s; self.is_error = false; }
            Err(e) => { self.output = e; self.is_error = true; }
        }
        Action::Nothing
    }

    fn footer_hints(&self) -> String { String::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_valid_json() {
        let result = format_json(r#"{"a":1}"#).unwrap();
        assert_eq!(result, "{\n  \"a\": 1\n}");
    }

    #[test]
    fn test_format_invalid_json() {
        assert!(format_json("not json").is_err());
    }

    #[test]
    fn test_format_empty() {
        assert_eq!(format_json("").unwrap(), "");
    }
}
