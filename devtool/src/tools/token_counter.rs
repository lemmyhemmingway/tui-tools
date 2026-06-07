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

pub struct TokenCounterTool {
    input: TextArea,
}

impl TokenCounterTool {
    pub fn new() -> Self {
        Self { input: TextArea::new() }
    }
}

fn count_words(text: &str) -> usize {
    text.split_whitespace().count()
}

// Standard OpenAI/Anthropic heuristic: ~4 chars per token for English prose
fn approx_tokens(chars: usize) -> usize {
    ((chars as f64) / 4.0).ceil() as usize
}

fn pct_bar(tokens: usize, limit: usize) -> (String, Color) {
    let ratio = tokens as f64 / limit as f64;
    let color = if ratio > 1.0 { Color::Red }
                else if ratio > 0.75 { Color::Yellow }
                else { Color::Green };
    let pct = (ratio * 100.0).min(9999.9);
    let filled = ((ratio.min(1.0)) * 20.0) as usize;
    let bar = format!("[{}{}] {:.1}%",
        "█".repeat(filled),
        "░".repeat(20 - filled),
        pct,
    );
    (bar, color)
}

fn row<'a>(label: &'static str, value: String, color: Color) -> Line<'a> {
    Line::from(vec![
        Span::styled(format!("{:<22}", label), Style::default().fg(Color::DarkGray)),
        Span::styled(value, Style::default().fg(color)),
    ])
}

const LIMITS: &[(&str, usize)] = &[
    ("GPT-3.5 (16K)",        16_384),
    ("GPT-4 (128K)",        128_000),
    ("Claude (200K)",       200_000),
    ("Gemini 1.5 (1M)",   1_000_000),
];

impl Tool for TokenCounterTool {
    fn name(&self) -> &'static str { "Token Counter" }

    fn render(&mut self, frame: &mut Frame, area: Rect, focus: Focus) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Min(0)])
            .split(area);

        self.input.render(frame, chunks[0], matches!(focus, Focus::Input), " Input ");

        let text = self.input.content();
        let lines = if text.trim().is_empty() {
            vec![Line::from(Span::styled(
                "paste text above — shows token estimates and context window usage",
                Style::default().fg(Color::DarkGray),
            ))]
        } else {
            let chars  = text.chars().count();
            let bytes  = text.len();
            let words  = count_words(&text);
            let lcount = text.lines().count();
            let tokens = approx_tokens(chars);

            let mut out: Vec<Line> = vec![
                row("Characters",            chars.to_string(),  Color::White),
                row("Bytes (UTF-8)",         bytes.to_string(),  Color::White),
                row("Words",                 words.to_string(),  Color::White),
                row("Lines",                 lcount.to_string(), Color::White),
                Line::from(""),
                row("≈ Tokens",              tokens.to_string(), Color::Yellow),
                Line::from(Span::styled(
                    "  ~4 chars/token (English prose estimate)",
                    Style::default().fg(Color::DarkGray),
                )),
                Line::from(""),
                Line::from(Span::styled("Context window usage:", Style::default().fg(Color::DarkGray))),
            ];

            for (model, limit) in LIMITS {
                let (bar, color) = pct_bar(tokens, *limit);
                out.push(row(model, bar, color));
            }

            out
        };

        frame.render_widget(
            Paragraph::new(lines)
                .block(Block::default()
                    .title(" Statistics ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray))),
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
