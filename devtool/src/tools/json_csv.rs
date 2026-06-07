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

pub struct JsonCsvTool {
    input: TextArea,
}

impl JsonCsvTool {
    pub fn new() -> Self {
        Self { input: TextArea::new() }
    }
}

fn json_to_csv(text: &str) -> Result<String, String> {
    let val: serde_json::Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
    let arr = val.as_array().ok_or("expected a JSON array at the top level")?;

    if arr.is_empty() {
        return Ok(String::new());
    }

    let headers: Vec<String> = arr[0]
        .as_object()
        .ok_or("array items must be JSON objects")?
        .keys()
        .cloned()
        .collect();

    let mut wtr = csv::Writer::from_writer(vec![]);
    wtr.write_record(&headers).map_err(|e| e.to_string())?;

    for row in arr {
        let obj = row.as_object().ok_or("all array items must be JSON objects")?;
        let record: Vec<String> = headers.iter().map(|h| match obj.get(h) {
            Some(serde_json::Value::String(s)) => s.clone(),
            Some(v) => v.to_string(),
            None    => String::new(),
        }).collect();
        wtr.write_record(&record).map_err(|e| e.to_string())?;
    }

    String::from_utf8(wtr.into_inner().map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())
}

fn csv_to_json(text: &str) -> Result<String, String> {
    let mut rdr = csv::Reader::from_reader(text.as_bytes());
    let headers: Vec<String> = rdr.headers()
        .map_err(|e| e.to_string())?
        .iter()
        .map(|s| s.to_string())
        .collect();

    let mut rows: Vec<serde_json::Value> = vec![];
    for result in rdr.records() {
        let record = result.map_err(|e| e.to_string())?;
        let obj: serde_json::Map<String, serde_json::Value> = headers.iter()
            .zip(record.iter())
            .map(|(k, v)| (k.clone(), serde_json::Value::String(v.to_string())))
            .collect();
        rows.push(serde_json::Value::Object(obj));
    }

    serde_json::to_string_pretty(&rows).map_err(|e| e.to_string())
}

fn convert(text: &str) -> Result<(String, String), String> {
    let trimmed = text.trim();
    if trimmed.starts_with('[') {
        json_to_csv(trimmed).map(|out| ("JSON → CSV".to_string(), out))
    } else {
        csv_to_json(trimmed).map(|out| ("CSV → JSON".to_string(), out))
    }
}

impl Tool for JsonCsvTool {
    fn name(&self) -> &'static str { "JSON ↔ CSV" }

    fn render(&mut self, frame: &mut Frame, area: Rect, focus: Focus) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        self.input.render(frame, chunks[0], matches!(focus, Focus::Input), " Input (JSON array or CSV) ");

        let text = self.input.content();
        let (title, lines, color) = if text.trim().is_empty() {
            (
                " Output ".to_string(),
                vec![Line::from(Span::styled(
                    "paste a JSON array  or  CSV with headers",
                    Style::default().fg(Color::DarkGray),
                ))],
                Color::DarkGray,
            )
        } else {
            match convert(&text) {
                Ok((dir, out)) => (
                    format!(" {dir} "),
                    out.lines().map(|l| Line::from(l.to_string())).collect(),
                    Color::Green,
                ),
                Err(e) => (
                    " Error ".to_string(),
                    vec![Line::from(Span::styled(e, Style::default().fg(Color::Red)))],
                    Color::Red,
                ),
            }
        };

        frame.render_widget(
            Paragraph::new(lines)
                .block(Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(color))),
            chunks[1],
        );
    }

    fn handle_key(&mut self, key: KeyEvent, _focus: Focus) -> Action {
        match key.code {
            KeyCode::Esc => return Action::FocusSidebar,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return Action::Quit,
            _            => { self.input.handle_key(key); }
        }
        Action::Nothing
    }

    fn footer_hints(&self) -> String { String::new() }
}
