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

pub struct YamlJsonTool {
    input: TextArea,
}

impl YamlJsonTool {
    pub fn new() -> Self {
        Self { input: TextArea::new() }
    }
}

fn convert(text: &str) -> Result<(&'static str, String), String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(String::new());
    }
    // JSON is strict — try it first. If it parses, output YAML.
    if let Ok(val) = serde_json::from_str::<serde_json::Value>(trimmed) {
        let yaml = serde_yaml::to_string(&val).map_err(|e| e.to_string())?;
        return Ok(("JSON → YAML", yaml));
    }
    // Fall back to YAML → JSON.
    let val: serde_json::Value = serde_yaml::from_str(trimmed).map_err(|e| e.to_string())?;
    let json = serde_json::to_string_pretty(&val).map_err(|e| e.to_string())?;
    Ok(("YAML → JSON", json))
}

impl Tool for YamlJsonTool {
    fn name(&self) -> &'static str { "YAML ↔ JSON" }

    fn render(&mut self, frame: &mut Frame, area: Rect, focus: Focus) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Min(0)])
            .split(area);

        self.input.render(frame, chunks[0], matches!(focus, Focus::Input), " Input (JSON or YAML) ");

        let text = self.input.content();
        let (lines, title, border_color): (Vec<Line>, String, Color) = match convert(&text) {
            Ok((label, output)) => {
                let out_lines = output.lines().map(|l| Line::from(l.to_string())).collect();
                (out_lines, format!(" {label} "), Color::Green)
            }
            Err(e) if e.is_empty() => {
                (vec![Line::from(Span::styled(
                    "paste JSON or YAML above",
                    Style::default().fg(Color::DarkGray),
                ))], " Output ".to_string(), Color::DarkGray)
            }
            Err(e) => {
                (vec![Line::from(Span::styled(
                    format!("error: {e}"),
                    Style::default().fg(Color::Red),
                ))], " Output ".to_string(), Color::Red)
            }
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
