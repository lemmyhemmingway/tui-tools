use crate::textarea::TextArea;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use hmac::{Hmac, Mac};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use sha2::Sha256;
use super::{Action, Focus, Tool};

type HmacSha256 = Hmac<Sha256>;

pub struct JwtSignTool {
    input: TextArea,
    secret: TextArea,
}

impl JwtSignTool {
    pub fn new() -> Self {
        Self { input: TextArea::new(), secret: TextArea::new() }
    }

    fn is_jwt(s: &str) -> bool {
        s.trim().split('.').count() == 3
    }

    fn sign(payload_json: &str, secret: &str) -> Result<String, String> {
        serde_json::from_str::<serde_json::Value>(payload_json)
            .map_err(|e| format!("invalid JSON: {e}"))?;
        let header = r#"{"alg":"HS256","typ":"JWT"}"#;
        let enc_header = URL_SAFE_NO_PAD.encode(header);
        let enc_payload = URL_SAFE_NO_PAD.encode(payload_json.trim());
        let signing_input = format!("{enc_header}.{enc_payload}");
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
            .map_err(|e| e.to_string())?;
        mac.update(signing_input.as_bytes());
        let sig = URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes());
        Ok(format!("{signing_input}.{sig}"))
    }

    fn verify(token: &str, secret: &str) -> Result<String, String> {
        let parts: Vec<&str> = token.trim().split('.').collect();
        if parts.len() != 3 {
            return Err("not a valid JWT (need 3 parts)".to_string());
        }
        let signing_input = format!("{}.{}", parts[0], parts[1]);
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
            .map_err(|e| e.to_string())?;
        mac.update(signing_input.as_bytes());
        let expected = URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes());
        if expected != parts[2] {
            return Err("invalid signature".to_string());
        }
        let payload_bytes = URL_SAFE_NO_PAD.decode(parts[1])
            .map_err(|_| "invalid base64 in payload".to_string())?;
        let payload_str = String::from_utf8(payload_bytes)
            .map_err(|_| "payload is not valid UTF-8".to_string())?;
        let pretty = serde_json::from_str::<serde_json::Value>(&payload_str)
            .ok()
            .and_then(|v| serde_json::to_string_pretty(&v).ok())
            .unwrap_or(payload_str);
        Ok(pretty)
    }
}

impl Tool for JwtSignTool {
    fn name(&self) -> &'static str { "JWT Sign" }

    fn initial_focus(&self) -> Focus { Focus::Input }

    fn render(&mut self, frame: &mut Frame, area: Rect, focus: Focus) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Length(3), Constraint::Min(0)])
            .split(area);

        let input_text = self.input.content();
        let is_jwt = Self::is_jwt(&input_text);
        let input_title = if is_jwt {
            " JWT token to verify "
        } else {
            " JSON payload to sign "
        };
        self.input.render(frame, chunks[0], matches!(focus, Focus::Input), input_title);
        self.secret.render(frame, chunks[1], matches!(focus, Focus::Pattern), " Secret (HS256) (Tab) ");

        let secret = self.secret.lines[0].clone();
        let (lines, border_color, title) = if input_text.trim().is_empty() {
            let hints = vec![
                Line::from(Span::styled(
                    "paste a JSON payload to sign, or a JWT token to verify",
                    Style::default().fg(Color::DarkGray),
                )),
            ];
            (hints, Color::DarkGray, " Output ".to_string())
        } else if is_jwt {
            match Self::verify(&input_text, &secret) {
                Ok(payload) => {
                    let mut out = vec![
                        Line::from(vec![
                            Span::styled("✓ ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                            Span::styled("signature valid", Style::default().fg(Color::Green)),
                        ]),
                        Line::from(""),
                    ];
                    for l in payload.lines() {
                        out.push(Line::from(l.to_string()));
                    }
                    (out, Color::Green, " Verified Payload ".to_string())
                }
                Err(e) => (
                    vec![Line::from(vec![
                        Span::styled("✗ ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                        Span::styled(e, Style::default().fg(Color::Red)),
                    ])],
                    Color::Red,
                    " Verification ".to_string(),
                ),
            }
        } else {
            match Self::sign(&input_text, &secret) {
                Ok(token) => {
                    let out = token.split('.').enumerate().map(|(i, part)| {
                        let color = [Color::Cyan, Color::Magenta, Color::Yellow][i];
                        Line::from(Span::styled(
                            if i < 2 { format!("{part}.") } else { part.to_string() },
                            Style::default().fg(color),
                        ))
                    }).collect();
                    (out, Color::Green, " Signed JWT ".to_string())
                }
                Err(e) => (
                    vec![Line::from(Span::styled(
                        format!("error: {e}"),
                        Style::default().fg(Color::Red),
                    ))],
                    Color::Red,
                    " Output ".to_string(),
                ),
            }
        };

        frame.render_widget(
            Paragraph::new(lines).block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color)),
            ),
            chunks[2],
        );
    }

    fn handle_key(&mut self, key: KeyEvent, focus: Focus) -> Action {
        match key.code {
            KeyCode::Esc => return Action::FocusSidebar,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return Action::Quit,
            KeyCode::Tab => return match focus {
                Focus::Input => Action::FocusPattern,
                _ => Action::FocusInput,
            },
            _ => {}
        }
        match focus {
            Focus::Input => self.input.handle_key(key),
            Focus::Pattern => self.secret.handle_key_single(key),
            _ => {}
        }
        Action::Nothing
    }

    fn footer_hints(&self) -> String { "Tab: switch field  auto-detects JWT vs JSON".to_string() }
}
