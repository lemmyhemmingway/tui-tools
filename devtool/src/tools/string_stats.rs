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

pub struct StringStatsTool {
    input: TextArea,
}

impl StringStatsTool {
    pub fn new() -> Self {
        Self { input: TextArea::new() }
    }
}

struct Stats {
    chars: usize,
    chars_no_space: usize,
    bytes: usize,
    words: usize,
    lines: usize,
    sentences: usize,
    unique_chars: usize,
    longest_word: usize,
    avg_word_len: f64,
}

fn compute_stats(text: &str) -> Stats {
    let chars = text.chars().count();
    let chars_no_space = text.chars().filter(|c| !c.is_whitespace()).count();
    let bytes = text.len();
    let lines = text.lines().count().max(if text.is_empty() { 0 } else { 1 });
    let words: Vec<&str> = text.split_whitespace().collect();
    let word_count = words.len();
    let sentences = text.chars().filter(|&c| c == '.' || c == '!' || c == '?').count();
    let mut seen = std::collections::HashSet::new();
    for c in text.chars() { seen.insert(c); }
    let unique_chars = seen.len();
    let longest_word = words.iter().map(|w| w.chars().count()).max().unwrap_or(0);
    let avg_word_len = if word_count == 0 {
        0.0
    } else {
        words.iter().map(|w| w.chars().count()).sum::<usize>() as f64 / word_count as f64
    };
    Stats { chars, chars_no_space, bytes, words: word_count, lines, sentences, unique_chars, longest_word, avg_word_len }
}

impl Tool for StringStatsTool {
    fn name(&self) -> &'static str { "String Stats" }

    fn render(&mut self, frame: &mut Frame, area: Rect, focus: Focus) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Min(0)])
            .split(area);

        self.input.render(frame, chunks[0], matches!(focus, Focus::Input), " Input ");

        let text = self.input.content();
        let (lines, border_color) = if text.is_empty() {
            (vec![Line::from(Span::styled(
                "type something above",
                Style::default().fg(Color::DarkGray),
            ))], Color::DarkGray)
        } else {
            let s = compute_stats(&text);
            let rows = vec![
                row("Characters       ", &s.chars.to_string()),
                row("Chars (no space) ", &s.chars_no_space.to_string()),
                row("Bytes (UTF-8)    ", &s.bytes.to_string()),
                row("Words            ", &s.words.to_string()),
                row("Lines            ", &s.lines.to_string()),
                row("Sentences        ", &s.sentences.to_string()),
                row("Unique chars     ", &s.unique_chars.to_string()),
                row("Longest word     ", &format!("{} chars", s.longest_word)),
                row("Avg word length  ", &format!("{:.1} chars", s.avg_word_len)),
            ];
            (rows, Color::Green)
        };

        frame.render_widget(
            Paragraph::new(lines).block(
                Block::default()
                    .title(" Stats ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color)),
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

fn row(label: &str, value: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(label.to_string(), Style::default().fg(Color::DarkGray)),
        Span::styled(value.to_string(), Style::default().fg(Color::White)),
    ])
}
