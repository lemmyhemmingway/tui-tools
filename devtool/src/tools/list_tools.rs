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

#[derive(Clone, Copy, PartialEq)]
enum Op {
    SortAsc,
    SortDesc,
    Dedup,
    Reverse,
    Trim,
    Count,
}

impl Op {
    fn label(self) -> &'static str {
        match self {
            Op::SortAsc  => "Sort A→Z",
            Op::SortDesc => "Sort Z→A",
            Op::Dedup    => "Deduplicate",
            Op::Reverse  => "Reverse",
            Op::Trim     => "Trim/Clean",
            Op::Count    => "Frequency",
        }
    }

    fn next(self) -> Self {
        match self {
            Op::SortAsc  => Op::SortDesc,
            Op::SortDesc => Op::Dedup,
            Op::Dedup    => Op::Reverse,
            Op::Reverse  => Op::Trim,
            Op::Trim     => Op::Count,
            Op::Count    => Op::SortAsc,
        }
    }

    fn prev(self) -> Self {
        match self {
            Op::SortAsc  => Op::Count,
            Op::SortDesc => Op::SortAsc,
            Op::Dedup    => Op::SortDesc,
            Op::Reverse  => Op::Dedup,
            Op::Trim     => Op::Reverse,
            Op::Count    => Op::Trim,
        }
    }
}

fn apply(op: Op, text: &str) -> String {
    let mut lines: Vec<String> = text.lines().map(|l| l.to_string()).collect();
    match op {
        Op::SortAsc => {
            lines.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
            lines.join("\n")
        }
        Op::SortDesc => {
            lines.sort_by(|a, b| b.to_lowercase().cmp(&a.to_lowercase()));
            lines.join("\n")
        }
        Op::Dedup => {
            let mut seen = std::collections::HashSet::new();
            lines.retain(|l| seen.insert(l.clone()));
            lines.join("\n")
        }
        Op::Reverse => {
            lines.reverse();
            lines.join("\n")
        }
        Op::Trim => {
            lines.iter()
                .map(|l| l.trim())
                .filter(|l| !l.is_empty())
                .collect::<Vec<_>>()
                .join("\n")
        }
        Op::Count => {
            let mut counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
            for l in &lines { *counts.entry(l.clone()).or_insert(0) += 1; }
            let mut pairs: Vec<(&String, &usize)> = counts.iter().collect();
            pairs.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
            let total = lines.len();
            let unique = counts.len();
            let mut out = format!("Total: {}  Unique: {}\n{}\n", total, unique, "─".repeat(26));
            for (line, count) in pairs {
                out.push_str(&format!("{:>5}×  {}\n", count, line));
            }
            out
        }
    }
}

pub struct ListToolsTool {
    input: TextArea,
    op: Op,
}

impl ListToolsTool {
    pub fn new() -> Self {
        Self { input: TextArea::new(), op: Op::SortAsc }
    }
}

const ALL_OPS: [Op; 6] = [Op::SortAsc, Op::SortDesc, Op::Dedup, Op::Reverse, Op::Trim, Op::Count];

impl Tool for ListToolsTool {
    fn name(&self) -> &'static str { "List Tools" }

    fn render(&mut self, frame: &mut Frame, area: Rect, focus: Focus) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        let left = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(3)])
            .split(chunks[0]);

        self.input.render(frame, left[0], matches!(focus, Focus::Input), " Input (one item per line) ");

        let spans: Vec<Span> = ALL_OPS.iter().flat_map(|&o| {
            let style = if o == self.op {
                Style::default().fg(Color::Black).bg(Color::Yellow)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            vec![
                Span::styled(format!(" {} ", o.label()), style),
                Span::styled(" ", Style::default()),
            ]
        }).collect();

        frame.render_widget(
            Paragraph::new(Line::from(spans))
                .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray))),
            left[1],
        );

        let text = self.input.content();
        let (output, color) = if text.trim().is_empty() {
            (vec![Line::from(Span::styled("paste a list above", Style::default().fg(Color::DarkGray)))], Color::DarkGray)
        } else {
            let result = apply(self.op, &text);
            (result.lines().map(|l| Line::from(l.to_string())).collect(), Color::Green)
        };

        frame.render_widget(
            Paragraph::new(output)
                .block(Block::default()
                    .title(format!(" {} ", self.op.label()))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(color))),
            chunks[1],
        );
    }

    fn handle_key(&mut self, key: KeyEvent, _focus: Focus) -> Action {
        match key.code {
            KeyCode::Esc     => return Action::FocusSidebar,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return Action::Quit,
            KeyCode::Tab     => self.op = self.op.next(),
            KeyCode::BackTab => self.op = self.op.prev(),
            _                => { self.input.handle_key(key); }
        }
        Action::Nothing
    }

    fn footer_hints(&self) -> String {
        "Tab/Shift+Tab: switch operation".to_string()
    }
}
