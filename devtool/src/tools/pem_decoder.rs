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

pub struct PemDecoderTool {
    input: TextArea,
}

impl PemDecoderTool {
    pub fn new() -> Self {
        Self { input: TextArea::new() }
    }
}

fn decode(pem_str: &str) -> Result<Vec<(&'static str, String)>, String> {
    use x509_parser::extensions::GeneralName;

    let (_, pem) = x509_parser::pem::parse_x509_pem(pem_str.trim().as_bytes())
        .map_err(|_| "failed to parse PEM — paste a -----BEGIN CERTIFICATE----- block".to_string())?;
    let cert = pem.parse_x509()
        .map_err(|_| "failed to parse X.509 certificate".to_string())?;

    let mut rows: Vec<(&'static str, String)> = vec![];

    rows.push(("Subject    ", cert.subject().to_string()));
    rows.push(("Issuer     ", cert.issuer().to_string()));
    rows.push(("Not Before ", cert.validity().not_before.to_string()));
    rows.push(("Not After  ", cert.validity().not_after.to_string()));

    let serial = cert
        .tbs_certificate
        .raw_serial()
        .iter()
        .map(|b| format!("{b:02X}"))
        .collect::<Vec<_>>()
        .join(":");
    rows.push(("Serial     ", serial));

    let self_signed = cert.subject() == cert.issuer();
    rows.push(("Self-signed", self_signed.to_string()));

    rows.push(("Sig Alg    ", cert.signature_algorithm.algorithm.to_string()));

    if let Ok(Some(san_ext)) = cert.subject_alternative_name() {
        for gn in &san_ext.value.general_names {
            let val = match gn {
                GeneralName::DNSName(n) => format!("DNS: {n}"),
                GeneralName::RFC822Name(n) => format!("Email: {n}"),
                GeneralName::URI(u) => format!("URI: {u}"),
                GeneralName::IPAddress(b) if b.len() == 4 => {
                    format!("IP: {}.{}.{}.{}", b[0], b[1], b[2], b[3])
                }
                GeneralName::IPAddress(b) if b.len() == 16 => {
                    let groups: Vec<String> = b.chunks(2).map(|c| format!("{:02x}{:02x}", c[0], c[1])).collect();
                    format!("IP: {}", groups.join(":"))
                }
                _ => format!("{gn:?}"),
            };
            rows.push(("SAN        ", val));
        }
    }

    Ok(rows)
}

impl Tool for PemDecoderTool {
    fn name(&self) -> &'static str { "PEM Decoder" }

    fn render(&mut self, frame: &mut Frame, area: Rect, focus: Focus) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(45), Constraint::Min(0)])
            .split(area);

        self.input.render(frame, chunks[0], matches!(focus, Focus::Input), " Input (paste -----BEGIN CERTIFICATE-----) ");

        let text = self.input.content();
        let (lines, border_color) = if text.trim().is_empty() {
            (vec![Line::from(Span::styled(
                "paste a PEM certificate above",
                Style::default().fg(Color::DarkGray),
            ))], Color::DarkGray)
        } else {
            match decode(&text) {
                Ok(rows) => {
                    let out = rows.into_iter().map(|(label, value)| {
                        Line::from(vec![
                            Span::styled(label, Style::default().fg(Color::DarkGray)),
                            Span::styled(format!("  {value}"), Style::default().fg(Color::White)),
                        ])
                    }).collect();
                    (out, Color::Green)
                }
                Err(e) => (
                    vec![Line::from(Span::styled(e, Style::default().fg(Color::Red)))],
                    Color::Red,
                ),
            }
        };

        frame.render_widget(
            Paragraph::new(lines).block(
                Block::default()
                    .title(" Certificate ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color)),
            ),
            chunks[1],
        );
    }

    fn handle_key(&mut self, key: KeyEvent, _focus: Focus) -> Action {
        match key.code {
            KeyCode::Esc => return Action::FocusSidebar,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return Action::Quit,
            _ => {}
        }
        self.input.handle_key(key);
        Action::Nothing
    }

    fn footer_hints(&self) -> String { String::new() }
}
