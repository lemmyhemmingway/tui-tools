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
use std::{collections::HashSet, fs, io, path::{Path, PathBuf}};

// ── Types ─────────────────────────────────────────────────────────────────────

struct Todo {
    text: String,
    done: bool,
    carried: bool,
}

enum Mode {
    Browse,
    Input,
    // (link_name, is_backlink)
    Follow { links: Vec<(String, bool)>, selected: usize },
}

struct App {
    todos: Vec<Todo>,
    state: ListState,
    mode: Mode,
    input: String,
    date: NaiveDate,
    dir: PathBuf,
    backlinks: Vec<String>,
    history: Vec<NaiveDate>,
}

// ── Link parsing ──────────────────────────────────────────────────────────────

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
    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut s = text;
    while let Some(i) = s.find("[[") {
        if i > 0 {
            spans.push(Span::styled(s[..i].to_string(), base));
        }
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
    if !s.is_empty() {
        spans.push(Span::styled(s.to_string(), base));
    }
    spans
}

// ── File format ───────────────────────────────────────────────────────────────

fn date_filename(date: NaiveDate) -> String {
    date.format("%d-%m-%Y.md").to_string()
}

fn parse_filename_date(name: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(name.strip_suffix(".md")?, "%d-%m-%Y").ok()
}

fn load_file(path: &Path) -> Vec<Todo> {
    let mut todos = Vec::new();
    for line in fs::read_to_string(path).unwrap_or_default().lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("- [ ] ") {
            let carried = rest.ends_with(" ↑");
            let text = rest.trim_end_matches(" ↑").to_string();
            todos.push(Todo { text, done: false, carried });
        } else if let Some(rest) = line.strip_prefix("- [x] ") {
            let carried = rest.ends_with(" ↑");
            let text = rest.trim_end_matches(" ↑").to_string();
            todos.push(Todo { text, done: true, carried });
        }
    }
    todos
}

fn save(app: &App) {
    let path = app.dir.join(date_filename(app.date));
    let header = format!("# {}\n\n", app.date.format("%d-%m-%Y"));
    let body: String = app.todos.iter().map(|t| {
        let mark = if t.done { "x" } else { " " };
        let carried = if t.carried { " ↑" } else { "" };
        format!("- [{}] {}{}\n", mark, t.text, carried)
    }).collect();

    let links: Vec<String> = {
        let mut seen = HashSet::new();
        let mut out = Vec::new();
        for t in &app.todos {
            for l in extract_links(&t.text) {
                if seen.insert(l.clone()) { out.push(l); }
            }
        }
        out
    };

    let mut content = header + &body;
    if !links.is_empty() {
        content.push_str("\n---\n\n**Links:** ");
        content.push_str(&links.iter().map(|l| format!("[[{}]]", l)).collect::<Vec<_>>().join(" "));
        content.push('\n');
    }

    fs::write(path, content).ok();
}

fn carryover(dir: &Path, today: NaiveDate) -> Vec<Todo> {
    let mut dates: Vec<NaiveDate> = fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let name = e.file_name();
            parse_filename_date(name.to_str()?)
        })
        .filter(|&d| d < today)
        .collect();
    dates.sort();

    for date in dates.iter().rev() {
        let unchecked: Vec<Todo> = load_file(&dir.join(date_filename(*date)))
            .into_iter()
            .filter(|t| !t.done)
            .map(|t| Todo { text: t.text, done: false, carried: true })
            .collect();
        if !unchecked.is_empty() {
            return unchecked;
        }
    }
    Vec::new()
}

fn find_backlinks(dir: &Path, today: NaiveDate) -> Vec<String> {
    let needle = format!("[[{}]]", today.format("%d-%m-%Y"));
    let mut found: Vec<String> = fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let name = e.file_name();
            let name = name.to_str()?;
            let date = parse_filename_date(name)?;
            if date == today { return None; }
            let content = fs::read_to_string(e.path()).ok()?;
            if content.contains(&needle) {
                Some(date.format("%d-%m-%Y").to_string())
            } else {
                None
            }
        })
        .collect();
    found.sort();
    found
}

// ── App ───────────────────────────────────────────────────────────────────────

impl App {
    fn new(dir: PathBuf) -> Self {
        let today = Local::now().date_naive();
        let path = dir.join(date_filename(today));
        let mut todos = load_file(&path);

        if !todos.iter().any(|t| t.carried) {
            todos.extend(carryover(&dir, today));
        }

        let backlinks = find_backlinks(&dir, today);

        let mut state = ListState::default();
        if !todos.is_empty() { state.select(Some(0)); }

        let app = Self {
            todos, state, mode: Mode::Browse,
            input: String::new(), date: today, dir, backlinks, history: Vec::new(),
        };
        save(&app);
        app
    }

    fn toggle(&mut self) {
        if let Some(i) = self.state.selected() {
            if let Some(t) = self.todos.get_mut(i) { t.done = !t.done; }
            save(self);
        }
    }

    fn delete(&mut self) {
        if let Some(i) = self.state.selected() {
            if i < self.todos.len() {
                self.todos.remove(i);
                let sel = if self.todos.is_empty() { None }
                          else { Some(i.min(self.todos.len() - 1)) };
                self.state.select(sel);
                save(self);
            }
        }
    }

    fn confirm_add(&mut self) {
        let text = self.input.trim().to_string();
        if !text.is_empty() {
            self.todos.push(Todo { text, done: false, carried: false });
            self.state.select(Some(self.todos.len() - 1));
            save(self);
        }
        self.input.clear();
        self.mode = Mode::Browse;
    }

    fn up(&mut self) {
        if self.todos.is_empty() { return; }
        let i = self.state.selected().map(|i| if i == 0 { 0 } else { i - 1 }).unwrap_or(0);
        self.state.select(Some(i));
    }

    fn down(&mut self) {
        if self.todos.is_empty() { return; }
        let i = self.state.selected()
            .map(|i| (i + 1).min(self.todos.len() - 1))
            .unwrap_or(0);
        self.state.select(Some(i));
    }

    fn all_links(&self) -> Vec<String> {
        let mut seen = HashSet::new();
        self.todos.iter()
            .flat_map(|t| extract_links(&t.text))
            .filter(|l| seen.insert(l.clone()))
            .collect()
    }

    fn all_visible_links(&self) -> Vec<(String, bool)> {
        let mut links: Vec<(String, bool)> = self.all_links()
            .into_iter()
            .map(|l| (l, false))
            .collect();
        links.extend(self.backlinks.iter().map(|l| (l.clone(), true)));
        links
    }

    fn navigate_to(&mut self, name: &str) {
        if let Ok(date) = NaiveDate::parse_from_str(name, "%d-%m-%Y") {
            self.history.push(self.date);
            self.date = date;
            self.todos = load_file(&self.dir.join(date_filename(date)));
            self.backlinks = find_backlinks(&self.dir, date);
            let sel = if self.todos.is_empty() { None } else { Some(0) };
            self.state.select(sel);
        }
    }

    fn go_back(&mut self) {
        if let Some(date) = self.history.pop() {
            self.date = date;
            self.todos = load_file(&self.dir.join(date_filename(date)));
            self.backlinks = find_backlinks(&self.dir, date);
            let sel = if self.todos.is_empty() { None } else { Some(0) };
            self.state.select(sel);
        }
    }

    fn stats(&self) -> (usize, usize) {
        (self.todos.iter().filter(|t| t.done).count(), self.todos.len())
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
    let (done, total) = app.stats();
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            app.date.format("%d %B %Y").to_string(),
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("  {}/{} done", done, total),
            Style::default().fg(Color::DarkGray),
        ),
    ]))
    .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Green))
        .title(Span::styled(" todo ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))));
    f.render_widget(header, chunks[0]);

    // Todo list
    let items: Vec<ListItem> = app.todos.iter().map(|t| {
        let check_sym = if t.done { "●" } else { "○" };
        let (check_style, text_style) = if t.done {
            (Style::default().fg(Color::DarkGray), Style::default().fg(Color::DarkGray).add_modifier(Modifier::CROSSED_OUT))
        } else if t.carried {
            (Style::default().fg(Color::Yellow), Style::default().fg(Color::Yellow))
        } else {
            (Style::default().fg(Color::White), Style::default().fg(Color::White))
        };

        let mut spans: Vec<Span<'static>> = vec![
            Span::raw("  "),
            Span::styled(format!("{}  ", check_sym), check_style),
        ];
        spans.extend(render_text(&t.text, text_style));
        if t.carried {
            spans.push(Span::styled("  ↑", Style::default().fg(Color::Yellow).add_modifier(Modifier::DIM)));
        }
        ListItem::new(Line::from(spans))
    }).collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)))
        .highlight_style(Style::default().bg(Color::DarkGray).fg(Color::White).add_modifier(Modifier::BOLD))
        .highlight_symbol("▶ ");
    f.render_stateful_widget(list, chunks[1], &mut app.state);

    // Footer
    match &app.mode {
        Mode::Input => {
            let footer = Paragraph::new(Line::from(vec![
                Span::styled(" > ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::styled(app.input.clone(), Style::default().fg(Color::White)),
                Span::styled("█", Style::default().fg(Color::Green)),
            ]))
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Green)));
            f.render_widget(footer, chunks[2]);
        }
        Mode::Browse | Mode::Follow { .. } => {
            let fwd = app.all_links();
            let bwd = &app.backlinks;
            let mut spans: Vec<Span<'static>> = vec![Span::raw("  ")];

            if !fwd.is_empty() {
                spans.push(Span::styled("→ ", Style::default().fg(Color::DarkGray)));
                for (i, l) in fwd.iter().enumerate() {
                    if i > 0 { spans.push(Span::raw(" ")); }
                    spans.push(Span::styled(format!("[[{}]]", l), Style::default().fg(Color::Cyan)));
                }
                spans.push(Span::raw("   "));
            }
            if !bwd.is_empty() {
                spans.push(Span::styled("← ", Style::default().fg(Color::DarkGray)));
                for (i, l) in bwd.iter().enumerate() {
                    if i > 0 { spans.push(Span::raw(" ")); }
                    spans.push(Span::styled(format!("[[{}]]", l), Style::default().fg(Color::Magenta)));
                }
                spans.push(Span::raw("   "));
            }

            spans.extend([
                Span::styled("a", Style::default().fg(Color::DarkGray)),
                Span::styled(" add  ", Style::default().fg(Color::DarkGray)),
                Span::styled("space", Style::default().fg(Color::DarkGray)),
                Span::styled(" toggle  ", Style::default().fg(Color::DarkGray)),
                Span::styled("d", Style::default().fg(Color::DarkGray)),
                Span::styled(" delete  ", Style::default().fg(Color::DarkGray)),
                Span::styled("f", Style::default().fg(Color::DarkGray)),
                Span::styled(" follow  ", Style::default().fg(Color::DarkGray)),
                Span::styled("b", Style::default().fg(Color::DarkGray)),
                Span::styled(" back  ", Style::default().fg(Color::DarkGray)),
                Span::styled("q", Style::default().fg(Color::DarkGray)),
                Span::styled(" quit", Style::default().fg(Color::DarkGray)),
            ]);

            let footer = Paragraph::new(Line::from(spans))
                .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)));
            f.render_widget(footer, chunks[2]);
        }
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
    let dir = std::env::var("TODO_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(std::env::var("HOME").unwrap_or_default())
                .join("notes")
                .join("todos")
        });
    fs::create_dir_all(&dir)?;

    let mut app = App::new(dir);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;

    loop {
        terminal.draw(|f| draw(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match app.mode {
                Mode::Browse => match (key.code, key.modifiers) {
                    (KeyCode::Char('q'), _)
                    | (KeyCode::Esc, _)
                    | (KeyCode::Char('c'), KeyModifiers::CONTROL) => break,

                    (KeyCode::Char('a'), _) => app.mode = Mode::Input,
                    (KeyCode::Char(' '), _) | (KeyCode::Enter, _) => app.toggle(),
                    (KeyCode::Char('d'), _) => app.delete(),
                    (KeyCode::Up, _) | (KeyCode::Char('k'), _) => app.up(),
                    (KeyCode::Down, _) | (KeyCode::Char('j'), _) => app.down(),
                    (KeyCode::Char('f'), _) => {
                        let links = app.all_visible_links();
                        if !links.is_empty() {
                            app.mode = Mode::Follow { links, selected: 0 };
                        }
                    }
                    (KeyCode::Char('b'), _) | (KeyCode::Backspace, _) => app.go_back(),
                    _ => {}
                },
                Mode::Input => match key.code {
                    KeyCode::Enter => app.confirm_add(),
                    KeyCode::Esc => { app.input.clear(); app.mode = Mode::Browse; }
                    KeyCode::Backspace => { app.input.pop(); }
                    KeyCode::Char(c) => app.input.push(c),
                    _ => {}
                },
                Mode::Follow { links, selected } => {
                    match (key.code, key.modifiers) {
                        (KeyCode::Esc, _)
                        | (KeyCode::Char('q'), _)
                        | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                            app.mode = Mode::Browse;
                        }
                        (KeyCode::Up, _) | (KeyCode::Char('k'), _) => {
                            app.mode = Mode::Follow {
                                selected: selected.saturating_sub(1),
                                links,
                            };
                        }
                        (KeyCode::Down, _) | (KeyCode::Char('j'), _) => {
                            let max = links.len().saturating_sub(1);
                            app.mode = Mode::Follow {
                                selected: (selected + 1).min(max),
                                links,
                            };
                        }
                        (KeyCode::Enter, _) => {
                            let link_name = links.get(selected).map(|(name, _)| name.clone());
                            app.mode = Mode::Browse;
                            if let Some(name) = link_name {
                                app.navigate_to(&name);
                            }
                        }
                        _ => {
                            app.mode = Mode::Follow { links, selected };
                        }
                    }
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
