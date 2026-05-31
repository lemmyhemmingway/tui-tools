use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use unicode_width::UnicodeWidthStr;

pub struct TextArea {
    pub lines: Vec<String>,
    pub row: usize,
    pub col: usize,
    scroll: usize,
}

impl TextArea {
    pub fn new() -> Self {
        Self { lines: vec![String::new()], row: 0, col: 0, scroll: 0 }
    }

    pub fn content(&self) -> String {
        self.lines.join("\n")
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char(c) => self.insert_char(c),
            KeyCode::Enter => self.insert_newline(),
            KeyCode::Backspace => self.delete_backward(),
            KeyCode::Left => self.move_left(),
            KeyCode::Right => self.move_right(),
            KeyCode::Up => self.move_up(),
            KeyCode::Down => self.move_down(),
            _ => {}
        }
    }

    pub fn handle_key_single(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char(c) => self.insert_char(c),
            KeyCode::Backspace => self.delete_backward(),
            KeyCode::Left => self.move_left(),
            KeyCode::Right => self.move_right(),
            _ => {}
        }
    }

    fn insert_char(&mut self, c: char) {
        self.lines[self.row].insert(self.col, c);
        self.col += c.len_utf8();
    }

    fn insert_newline(&mut self) {
        let rest = self.lines[self.row].split_off(self.col);
        self.row += 1;
        self.lines.insert(self.row, rest);
        self.col = 0;
    }

    fn delete_backward(&mut self) {
        if self.col > 0 {
            let prev = self.lines[self.row][..self.col]
                .char_indices()
                .last()
                .map(|(i, _)| i)
                .unwrap_or(0);
            self.lines[self.row].remove(prev);
            self.col = prev;
        } else if self.row > 0 {
            let line = self.lines.remove(self.row);
            self.row -= 1;
            self.col = self.lines[self.row].len();
            self.lines[self.row].push_str(&line);
        }
    }

    fn move_left(&mut self) {
        if self.col > 0 {
            self.col = self.lines[self.row][..self.col]
                .char_indices()
                .last()
                .map(|(i, _)| i)
                .unwrap_or(0);
        } else if self.row > 0 {
            self.row -= 1;
            self.col = self.lines[self.row].len();
        }
    }

    fn move_right(&mut self) {
        let len = self.lines[self.row].len();
        if self.col < len {
            let c = self.lines[self.row][self.col..].chars().next().unwrap();
            self.col += c.len_utf8();
        } else if self.row + 1 < self.lines.len() {
            self.row += 1;
            self.col = 0;
        }
    }

    fn move_up(&mut self) {
        if self.row > 0 {
            self.row -= 1;
            self.col = self.col.min(self.lines[self.row].len());
        }
    }

    fn move_down(&mut self) {
        if self.row + 1 < self.lines.len() {
            self.row += 1;
            self.col = self.col.min(self.lines[self.row].len());
        }
    }

    fn adjust_scroll(&mut self, visible: usize) {
        if visible == 0 { return; }
        if self.row < self.scroll {
            self.scroll = self.row;
        } else if self.row >= self.scroll + visible {
            self.scroll = self.row + 1 - visible;
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect, focused: bool, title: &str) {
        let inner_h = area.height.saturating_sub(2) as usize;
        self.adjust_scroll(inner_h);

        let display: Vec<Line> = self.lines[self.scroll..]
            .iter()
            .map(|l| Line::from(l.clone()))
            .collect();

        let border_style = if focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        frame.render_widget(
            Paragraph::new(display).block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(border_style),
            ),
            area,
        );

        if focused && area.height > 2 {
            let cy = area.y + 1 + (self.row - self.scroll) as u16;
            let cx = area.x + 1 + UnicodeWidthStr::width(&self.lines[self.row][..self.col]) as u16;
            if cy < area.y + area.height - 1 && cx < area.x + area.width - 1 {
                frame.set_cursor_position((cx, cy));
            }
        }
    }
}
