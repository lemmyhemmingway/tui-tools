use chrono::{Local, NaiveDate};
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Terminal,
};
use std::{fs, io, path::{Path, PathBuf}};

// ── Types ─────────────────────────────────────────────────────────────────────

struct Jot {
    time: String,
    text: String,
}

enum Mode {
    Input,
    LinkComplete { link_start: usize, matches: Vec<String>, selected: usize },
    // (link_name, is_backlink)
    Follow { links: Vec<(String, bool)>, selected: usize },
}

struct App {
    jots: Vec<Jot>,
    input: String,
    mode: Mode,
    date: NaiveDate,
    dir: PathBuf,
    notes_root: PathBuf,
    all_notes: Vec<String>,
    backlinks: Vec<String>,
    history: Vec<NaiveDate>,
    state: ListState,
}

enum Action {
    Quit,
    Submit,
    InputChar(char),
    Backspace,
    ScrollUp,
    ScrollDown,
    OpenFollow,
    GoBack,
    FollowUp,
    FollowDown,
    FollowConfirm,
    FollowCancel,
    LinkConfirm,
    LinkCancel,
    LinkChar(char),
    LinkBackspace,
    LinkUp,
    LinkDown,
    Nothing,
}

// ── Text helpers ──────────────────────────────────────────────────────────────

fn extract_links(text: &str) -> Vec<String> {
    let mut links = Vec::new();
    let mut s = text;
    while let Some(i) = s.find("[[") {
        s = &s[i + 2..];
        if let Some(j) = s.find("]]") {
            links.push(s[..j].to_string());
            s = &s[j + 2..];
        } else {
            break;
        }
    }
    links
}

fn render_text(text: &str, base: Style) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    let mut s = text;
    while let Some(i) = s.find("[[") {
        if i > 0 { spans.push(Span::styled(s[..i].to_string(), base)); }
        s = &s[i + 2..];
        if let Some(j) = s.find("]]") {
            spans.push(Span::styled(
                format!("[[{}]]", &s[..j]),
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ));
            s = &s[j + 2..];
        } else {
            spans.push(Span::styled(format!("[[{}", s), base));
            s = "";
        }
    }
    if !s.is_empty() { spans.push(Span::styled(s.to_string(), base)); }
    spans
}

// ── Note scanning ─────────────────────────────────────────────────────────────

fn scan_notes(root: &Path) -> Vec<String> {
    let mut names = Vec::new();
    let Ok(entries) = fs::read_dir(root) else { return names };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            names.extend(scan_notes(&path));
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                names.push(stem.to_string());
            }
        }
    }
    names.sort();
    names.dedup();
    names
}

fn collect_backlinks(dir: &Path, needle: &str, found: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_backlinks(&path, needle, found);
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            if let Ok(content) = fs::read_to_string(&path) {
                if content.contains(needle) {
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        found.push(stem.to_string());
                    }
                }
            }
        }
    }
}

fn find_backlinks(notes_root: &Path, date: NaiveDate) -> Vec<String> {
    let needle = format!("[[{}]]", date.format("%d-%m-%Y"));
    let mut found = Vec::new();
    collect_backlinks(notes_root, &needle, &mut found);
    found.sort();
    found.dedup();
    found
}

fn create_note(root: &Path, name: &str) {
    if name.is_empty() { return; }
    let path = root.join(format!("{}.md", name));
    if !path.exists() {
        fs::write(path, format!("# {}\n\n", name)).ok();
    }
}

// ── File I/O ──────────────────────────────────────────────────────────────────

fn date_filename(date: NaiveDate) -> String {
    date.format("%d-%m-%Y.md").to_string()
}

fn load_file(path: &Path) -> Vec<Jot> {
    let mut jots = Vec::new();
    for line in fs::read_to_string(path).unwrap_or_default().lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("- ") {
            if rest.len() > 6
                && rest.as_bytes().get(2) == Some(&b':')
                && rest.as_bytes().get(5) == Some(&b' ')
            {
                jots.push(Jot { time: rest[..5].to_string(), text: rest[6..].to_string() });
            }
        }
    }
    jots
}

fn append_jot(dir: &Path, date: NaiveDate, jot: &Jot) {
    let path = dir.join(date_filename(date));
    let mut content = fs::read_to_string(&path).unwrap_or_default();
    if content.is_empty() {
        content = format!("# {}\n\n", date.format("%d-%m-%Y"));
    }
    content.push_str(&format!("- {} {}\n", jot.time, jot.text));
    fs::write(path, content).ok();
}

// ── App ───────────────────────────────────────────────────────────────────────

impl App {
    fn new(dir: PathBuf, notes_root: PathBuf) -> Self {
        let today = Local::now().date_naive();
        let jots = load_file(&dir.join(date_filename(today)));
        let all_notes = scan_notes(&notes_root);
        let backlinks = find_backlinks(&notes_root, today);
        let mut state = ListState::default();
        if !jots.is_empty() { state.select(Some(jots.len() - 1)); }
        Self {
            jots, input: String::new(), mode: Mode::Input, date: today,
            dir, notes_root, all_notes, backlinks, history: Vec::new(), state,
        }
    }

    fn submit(&mut self) {
        let text = self.input.trim().to_string();
        if text.is_empty() { return; }
        let jot = Jot { time: Local::now().format("%H:%M").to_string(), text };
        append_jot(&self.dir, self.date, &jot);
        self.jots.push(jot);
        self.state.select(Some(self.jots.len() - 1));
        self.input.clear();
        self.all_notes = scan_notes(&self.notes_root);
    }

    fn all_jot_links(&self) -> Vec<String> {
        let mut seen = std::collections::HashSet::new();
        self.jots.iter()
            .flat_map(|j| extract_links(&j.text))
            .filter(|l| seen.insert(l.clone()))
            .collect()
    }

    fn all_visible_links(&self) -> Vec<(String, bool)> {
        let mut links: Vec<(String, bool)> = self.all_jot_links()
            .into_iter()
            .map(|l| (l, false))
            .collect();
        links.extend(self.backlinks.iter().map(|l| (l.clone(), true)));
        links
    }

    fn navigate_to_date(&mut self, date_str: &str) {
        if let Ok(date) = NaiveDate::parse_from_str(date_str, "%d-%m-%Y") {
            self.history.push(self.date);
            self.date = date;
            self.jots = load_file(&self.dir.join(date_filename(date)));
            self.backlinks = find_backlinks(&self.notes_root, date);
            let sel = if self.jots.is_empty() { None } else { Some(self.jots.len() - 1) };
            self.state.select(sel);
        }
    }

    fn go_back(&mut self) {
        if let Some(date) = self.history.pop() {
            self.date = date;
            self.jots = load_file(&self.dir.join(date_filename(date)));
            self.backlinks = find_backlinks(&self.notes_root, date);
            let sel = if self.jots.is_empty() { None } else { Some(self.jots.len() - 1) };
            self.state.select(sel);
        }
    }

    fn follow_selected(&mut self) -> Option<String> {
        if let Mode::Follow { ref links, selected } = self.mode {
            let name = links.get(selected)?.0.clone();
            self.mode = Mode::Input;
            Some(name)
        } else {
            None
        }
    }

    fn open_follow(&mut self) {
        let links = self.all_visible_links();
        if !links.is_empty() {
            self.mode = Mode::Follow { links, selected: 0 };
        }
    }

    // ── Link complete ──────────────────────────────────────────────────────────

    fn enter_link_mode(&mut self) {
        self.all_notes = scan_notes(&self.notes_root);
        let link_start = self.input.len() - 2;
        let matches = self.all_notes.clone();
        self.mode = Mode::LinkComplete { link_start, matches, selected: 0 };
    }

    fn update_link_matches(&mut self) {
        let link_start = match &self.mode {
            Mode::LinkComplete { link_start, .. } => *link_start,
            _ => return,
        };
        let query = self.input[link_start + 2..].to_lowercase();
        let new_matches: Vec<String> = self.all_notes.iter()
            .filter(|n| n.to_lowercase().contains(&query))
            .cloned()
            .collect();
        if let Mode::LinkComplete { matches, selected, .. } = &mut self.mode {
            *selected = (*selected).min(new_matches.len().saturating_sub(1));
            *matches = new_matches;
        }
    }

    fn link_backspace(&mut self) {
        let link_start = match &self.mode {
            Mode::LinkComplete { link_start, .. } => *link_start,
            _ => return,
        };
        if self.input.len() <= link_start + 2 {
            self.input.truncate(link_start);
            self.mode = Mode::Input;
        } else {
            self.input.pop();
            self.update_link_matches();
        }
    }

    fn link_push(&mut self, c: char) {
        self.input.push(c);
        self.update_link_matches();
    }

    fn link_up(&mut self) {
        if let Mode::LinkComplete { selected, .. } = &mut self.mode {
            *selected = selected.saturating_sub(1);
        }
    }

    fn link_down(&mut self) {
        let link_start = match &self.mode {
            Mode::LinkComplete { link_start, .. } => *link_start,
            _ => return,
        };
        let (selected, total) = match &self.mode {
            Mode::LinkComplete { selected, matches, .. } => {
                let has_create = !self.input[link_start + 2..].is_empty();
                (*selected, matches.len() + if has_create { 1 } else { 0 })
            }
            _ => return,
        };
        if let Mode::LinkComplete { selected: sel, .. } = &mut self.mode {
            *sel = (selected + 1).min(total.saturating_sub(1));
        }
    }

    fn cancel_link(&mut self) {
        let link_start = match &self.mode {
            Mode::LinkComplete { link_start, .. } => *link_start,
            _ => return,
        };
        self.input.truncate(link_start);
        self.mode = Mode::Input;
    }

    fn confirm_link(&mut self) {
        let (link_start, selected, matches) = match &self.mode {
            Mode::LinkComplete { link_start, selected, matches } => (*link_start, *selected, matches.clone()),
            _ => return,
        };
        let query = self.input[link_start + 2..].to_string();

        let name = if selected < matches.len() {
            matches[selected].clone()
        } else if !query.is_empty() {
            create_note(&self.notes_root, &query);
            self.all_notes = scan_notes(&self.notes_root);
            query
        } else {
            self.mode = Mode::Input;
            return;
        };

        self.input.truncate(link_start);
        self.input.push_str(&format!("[[{}]]", name));
        self.mode = Mode::Input;
    }

    fn scroll_up(&mut self) {
        if self.jots.is_empty() { return; }
        let i = self.state.selected().map(|i| i.saturating_sub(1)).unwrap_or(0);
        self.state.select(Some(i));
    }

    fn scroll_down(&mut self) {
        if self.jots.is_empty() { return; }
        let i = self.state.selected().map(|i| (i + 1).min(self.jots.len() - 1)).unwrap_or(0);
        self.state.select(Some(i));
    }
}

// ── Drawing ───────────────────────────────────────────────────────────────────

fn popup_rect(list_area: Rect, item_count: usize) -> Rect {
    let height = (item_count as u16 + 2).clamp(3, 12).min(list_area.height);
    let width = (list_area.width * 3 / 4).max(30).min(list_area.width);
    Rect {
        x: list_area.x + (list_area.width.saturating_sub(width)) / 2,
        y: list_area.y + list_area.height.saturating_sub(height),
        width,
        height,
    }
}

fn draw(f: &mut ratatui::Frame, app: &mut App) {
    let area = f.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(3)])
        .split(area);

    // Header
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            app.date.format("%d %B %Y").to_string(),
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("  {} notes", app.jots.len()),
            Style::default().fg(Color::DarkGray),
        ),
    ]))
    .block(Block::default().borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green))
        .title(Span::styled(" jot ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))));
    f.render_widget(header, chunks[0]);

    // Jots list
    let items: Vec<ListItem> = app.jots.iter().map(|j| {
        let mut spans: Vec<Span<'static>> = vec![
            Span::raw("  "),
            Span::styled(format!("{}  ", j.time), Style::default().fg(Color::DarkGray)),
        ];
        spans.extend(render_text(&j.text, Style::default().fg(Color::White)));
        ListItem::new(Line::from(spans))
    }).collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)))
        .highlight_style(Style::default().bg(Color::DarkGray).fg(Color::White))
        .highlight_symbol("");
    f.render_stateful_widget(list, chunks[1], &mut app.state);

    // Input bar
    let input_display = app.input.clone();
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" > ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::styled(input_display, Style::default().fg(Color::White)),
        Span::styled("█", Style::default().fg(Color::Green)),
    ]))
    .block(Block::default().borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green)));
    f.render_widget(footer, chunks[2]);

    // Link complete popup
    if let Mode::LinkComplete { matches, selected, link_start } = &app.mode {
        let selected = *selected;
        let link_start = *link_start;
        let query = app.input[link_start + 2..].to_string();
        let has_create = !query.is_empty();
        let total = matches.len() + if has_create { 1 } else { 0 };

        let popup_area = popup_rect(chunks[1], total.max(1));
        f.render_widget(Clear, popup_area);

        let mut popup_items: Vec<ListItem> = matches.iter().map(|name| {
            ListItem::new(Line::from(vec![
                Span::raw("  "),
                Span::styled(name.clone(), Style::default().fg(Color::White)),
            ]))
        }).collect();

        if has_create {
            popup_items.push(ListItem::new(Line::from(vec![
                Span::styled("  + ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::styled(format!("[[{}]]", query), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ])));
        } else if popup_items.is_empty() {
            popup_items.push(ListItem::new(Line::from(
                Span::styled("  type to search or create...", Style::default().fg(Color::DarkGray)),
            )));
        }

        let mut popup_state = ListState::default();
        if total > 0 { popup_state.select(Some(selected)); }

        let popup = List::new(popup_items)
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(Span::styled(" [[ ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))))
            .highlight_style(Style::default().bg(Color::DarkGray))
            .highlight_symbol("▶ ");

        f.render_stateful_widget(popup, popup_area, &mut popup_state);
    }

    // Follow popup
    if let Mode::Follow { links, selected } = &app.mode {
        let selected = *selected;
        let popup_area = popup_rect(chunks[1], links.len().max(1));
        f.render_widget(Clear, popup_area);

        let popup_items: Vec<ListItem> = if links.is_empty() {
            vec![ListItem::new(Line::from(
                Span::styled("  no links on this page", Style::default().fg(Color::DarkGray)),
            ))]
        } else {
            links.iter().map(|(name, is_back)| {
                let (arrow, color) = if *is_back {
                    ("← ", Color::Magenta)
                } else {
                    ("→ ", Color::Cyan)
                };
                ListItem::new(Line::from(vec![
                    Span::styled(format!("  {}", arrow), Style::default().fg(color)),
                    Span::styled(format!("[[{}]]", name), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                ]))
            }).collect()
        };

        let mut popup_state = ListState::default();
        if !links.is_empty() { popup_state.select(Some(selected)); }

        let popup = List::new(popup_items)
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(Span::styled(" follow ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))))
            .highlight_style(Style::default().bg(Color::DarkGray))
            .highlight_symbol("▶ ");

        f.render_stateful_widget(popup, popup_area, &mut popup_state);
    }
}

// ── Main ──────────────────────────────────────────────────────────────────────

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = std::env::var("JOT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(std::env::var("HOME").unwrap_or_default())
                .join("notes").join("jots")
        });
    fs::create_dir_all(&dir)?;

    let notes_root = dir.parent().map(PathBuf::from).unwrap_or_else(|| dir.clone());
    fs::create_dir_all(&notes_root)?;

    let mut app = App::new(dir, notes_root);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;

    loop {
        terminal.draw(|f| draw(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
            let action = match &app.mode {
                Mode::Input => match key.code {
                    KeyCode::Esc => Action::Quit,
                    KeyCode::Char('c') if ctrl => Action::Quit,
                    KeyCode::Char('f') if ctrl => Action::OpenFollow,
                    KeyCode::Char('b') if ctrl => Action::GoBack,
                    KeyCode::Enter => Action::Submit,
                    KeyCode::Backspace => Action::Backspace,
                    KeyCode::Up => Action::ScrollUp,
                    KeyCode::Char('k') if ctrl => Action::ScrollUp,
                    KeyCode::Down => Action::ScrollDown,
                    KeyCode::Char('j') if ctrl => Action::ScrollDown,
                    KeyCode::Char(c) => Action::InputChar(c),
                    _ => Action::Nothing,
                },
                Mode::LinkComplete { .. } => match key.code {
                    KeyCode::Char('c') if ctrl => Action::Quit,
                    KeyCode::Esc => Action::LinkCancel,
                    KeyCode::Enter => Action::LinkConfirm,
                    KeyCode::Backspace => Action::LinkBackspace,
                    KeyCode::Up => Action::LinkUp,
                    KeyCode::Char('k') if ctrl => Action::LinkUp,
                    KeyCode::Down => Action::LinkDown,
                    KeyCode::Char('j') if ctrl => Action::LinkDown,
                    KeyCode::Char(c) => Action::LinkChar(c),
                    _ => Action::Nothing,
                },
                Mode::Follow { links, selected } => match key.code {
                    KeyCode::Char('c') if ctrl => Action::Quit,
                    KeyCode::Esc => Action::FollowCancel,
                    KeyCode::Enter => Action::FollowConfirm,
                    KeyCode::Up => Action::FollowUp,
                    KeyCode::Char('k') if ctrl => Action::FollowUp,
                    KeyCode::Down => Action::FollowDown,
                    KeyCode::Char('j') if ctrl => {
                        // avoid conflict: ctrl+j in follow is navigate down
                        let _ = (links, selected); // suppress unused warning
                        Action::FollowDown
                    }
                    _ => Action::Nothing,
                },
            };

            match action {
                Action::Quit => break,
                Action::Submit => app.submit(),
                Action::Backspace => { app.input.pop(); }
                Action::ScrollUp => app.scroll_up(),
                Action::ScrollDown => app.scroll_down(),
                Action::OpenFollow => app.open_follow(),
                Action::GoBack => app.go_back(),
                Action::InputChar('[') => {
                    app.input.push('[');
                    if app.input.ends_with("[[") { app.enter_link_mode(); }
                }
                Action::InputChar(c) => app.input.push(c),
                Action::LinkConfirm => app.confirm_link(),
                Action::LinkCancel => app.cancel_link(),
                Action::LinkBackspace => app.link_backspace(),
                Action::LinkChar(c) => app.link_push(c),
                Action::LinkUp => app.link_up(),
                Action::LinkDown => app.link_down(),
                Action::FollowCancel => app.mode = Mode::Input,
                Action::FollowUp => {
                    if let Mode::Follow { ref mut selected, .. } = app.mode {
                        *selected = selected.saturating_sub(1);
                    }
                }
                Action::FollowDown => {
                    let max = if let Mode::Follow { ref links, .. } = app.mode {
                        links.len().saturating_sub(1)
                    } else { 0 };
                    if let Mode::Follow { ref mut selected, .. } = app.mode {
                        *selected = (*selected + 1).min(max);
                    }
                }
                Action::FollowConfirm => {
                    if let Some(name) = app.follow_selected() {
                        if NaiveDate::parse_from_str(&name, "%d-%m-%Y").is_ok() {
                            app.navigate_to_date(&name);
                        } else {
                            // Named note — open in $EDITOR
                            let path = app.notes_root.join(format!("{}.md", name));
                            disable_raw_mode()?;
                            execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
                            terminal.show_cursor()?;
                            let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
                            std::process::Command::new(&editor).arg(&path).status().ok();
                            enable_raw_mode()?;
                            execute!(io::stdout(), EnterAlternateScreen)?;
                            terminal.clear()?;
                        }
                    }
                }
                Action::Nothing => {}
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
