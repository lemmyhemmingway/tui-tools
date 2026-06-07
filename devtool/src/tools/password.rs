use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use super::{Action, Focus, Tool};

pub struct PasswordTool {
    length: usize,
    upper: bool,
    lower: bool,
    digits: bool,
    symbols: bool,
    password: String,
    opt: usize, // 0=length, 1=upper, 2=lower, 3=digits, 4=symbols
}

fn gen_password(length: usize, upper: bool, lower: bool, digits: bool, symbols: bool) -> String {
    let mut pool: Vec<u8> = Vec::new();
    if upper   { pool.extend_from_slice(b"ABCDEFGHIJKLMNOPQRSTUVWXYZ"); }
    if lower   { pool.extend_from_slice(b"abcdefghijklmnopqrstuvwxyz"); }
    if digits  { pool.extend_from_slice(b"0123456789"); }
    if symbols { pool.extend_from_slice(b"!@#$%^&*()-_=+[]{}|;:,.<>?"); }
    if pool.is_empty() {
        return "(enable at least one character set)".to_string();
    }
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..length).map(|_| pool[rng.gen_range(0..pool.len())] as char).collect()
}

impl PasswordTool {
    pub fn new() -> Self {
        let mut t = Self {
            length: 20,
            upper: true,
            lower: true,
            digits: true,
            symbols: true,
            password: String::new(),
            opt: 0,
        };
        t.regen();
        t
    }

    fn regen(&mut self) {
        self.password = gen_password(self.length, self.upper, self.lower, self.digits, self.symbols);
    }

    fn toggle_current(&mut self) {
        match self.opt {
            1 => self.upper   = !self.upper,
            2 => self.lower   = !self.lower,
            3 => self.digits  = !self.digits,
            4 => self.symbols = !self.symbols,
            _ => {}
        }
        self.regen();
    }
}

impl Tool for PasswordTool {
    fn name(&self) -> &'static str { "Password Gen" }

    fn render(&mut self, frame: &mut Frame, area: Rect, _focus: Focus) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(5), Constraint::Length(5), Constraint::Min(0)])
            .split(area);

        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                &self.password,
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            )))
            .block(Block::default()
                .title(" Password ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)))
            .alignment(Alignment::Center),
            chunks[0],
        );

        let sel = |i: usize| {
            if self.opt == i { Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD) }
            else             { Style::default().fg(Color::White) }
        };
        let chk = |b: bool| if b { "[x]" } else { "[ ]" };

        let opts = Line::from(vec![
            Span::styled(format!(" Length: {:>3}   ", self.length), sel(0)),
            Span::styled(format!("{} Uppercase   ", chk(self.upper)),   sel(1)),
            Span::styled(format!("{} Lowercase   ", chk(self.lower)),   sel(2)),
            Span::styled(format!("{} Digits   ",   chk(self.digits)),   sel(3)),
            Span::styled(format!("{} Symbols",     chk(self.symbols)),  sel(4)),
        ]);

        frame.render_widget(
            Paragraph::new(opts)
                .block(Block::default()
                    .title(" Options  (Tab: select  ↑↓: length  Space: toggle) ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray))),
            chunks[1],
        );
    }

    fn handle_key(&mut self, key: KeyEvent, _focus: Focus) -> Action {
        match key.code {
            KeyCode::Esc => return Action::FocusSidebar,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return Action::Quit,
            KeyCode::Tab     => self.opt = (self.opt + 1) % 5,
            KeyCode::BackTab => self.opt = if self.opt == 0 { 4 } else { self.opt - 1 },
            KeyCode::Enter   => self.regen(),
            KeyCode::Char(' ') => {
                if self.opt == 0 { self.regen(); } else { self.toggle_current(); }
            }
            KeyCode::Up => {
                if self.length < 128 { self.length += 1; self.regen(); }
            }
            KeyCode::Down => {
                if self.length > 4 { self.length -= 1; self.regen(); }
            }
            _ => {}
        }
        Action::Nothing
    }

    fn footer_hints(&self) -> String {
        "Tab: cycle options  ↑↓: length  Space: toggle/regen  Enter: regenerate".to_string()
    }
}
