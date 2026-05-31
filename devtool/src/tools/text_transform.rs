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

pub struct TextTransformTool {
    input: TextArea,
}

impl TextTransformTool {
    pub fn new() -> Self {
        Self { input: TextArea::new() }
    }

    fn transforms(text: &str) -> Vec<(&'static str, String)> {
        vec![
            ("UPPER          ", text.to_uppercase()),
            ("lower          ", text.to_lowercase()),
            ("Title Case     ", title_case(text)),
            ("camelCase      ", to_camel(text)),
            ("PascalCase     ", to_pascal(text)),
            ("snake_case     ", to_snake(text)),
            ("kebab-case     ", to_kebab(text)),
            ("SCREAMING_SNAKE", to_snake(text).to_uppercase()),
        ]
    }
}

fn split_words(text: &str) -> Vec<String> {
    let mut expanded = String::new();
    let chars: Vec<char> = text.chars().collect();
    for (i, &c) in chars.iter().enumerate() {
        if i > 0 && c.is_uppercase() {
            let prev = chars[i - 1];
            if prev.is_lowercase() || (prev.is_uppercase() && i + 1 < chars.len() && chars[i + 1].is_lowercase()) {
                expanded.push(' ');
            }
        }
        expanded.push(c);
    }
    expanded
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_lowercase())
        .collect()
}

fn title_case(text: &str) -> String {
    text.lines()
        .map(|line| {
            line.split_whitespace()
                .map(|w| {
                    let mut c = w.chars();
                    match c.next() {
                        None => String::new(),
                        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                    }
                })
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn to_camel(text: &str) -> String {
    let ws = split_words(text);
    let mut result = String::new();
    for (i, w) in ws.iter().enumerate() {
        if i == 0 {
            result.push_str(w);
        } else {
            let mut c = w.chars();
            if let Some(f) = c.next() {
                result.push_str(&f.to_uppercase().collect::<String>());
                result.push_str(c.as_str());
            }
        }
    }
    result
}

fn to_pascal(text: &str) -> String {
    split_words(text)
        .iter()
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect()
}

fn to_snake(text: &str) -> String {
    split_words(text).join("_")
}

fn to_kebab(text: &str) -> String {
    split_words(text).join("-")
}

impl Tool for TextTransformTool {
    fn name(&self) -> &'static str { "Text Transform" }

    fn render(&mut self, frame: &mut Frame, area: Rect, focus: Focus) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(area);

        self.input.render(frame, chunks[0], matches!(focus, Focus::Input), " Input ");

        let text = self.input.content();
        let lines: Vec<Line> = if text.trim().is_empty() {
            vec![Line::from(Span::styled(
                "type something above",
                Style::default().fg(Color::DarkGray),
            ))]
        } else {
            Self::transforms(&text)
                .into_iter()
                .map(|(label, value)| {
                    Line::from(vec![
                        Span::styled(label, Style::default().fg(Color::DarkGray)),
                        Span::styled(format!("  {value}"), Style::default().fg(Color::White)),
                    ])
                })
                .collect()
        };

        frame.render_widget(
            Paragraph::new(lines).block(
                Block::default()
                    .title(" Variants ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Green)),
            ),
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
        Action::Nothing
    }

    fn footer_hints(&self) -> String { String::new() }
}
