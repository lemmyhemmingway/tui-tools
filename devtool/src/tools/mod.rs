pub mod base64;
pub mod bcrypt_tool;
pub mod cron_expr;
pub mod diff;
pub mod hash;
pub mod hmac_gen;
pub mod html_entity;
pub mod json;
pub mod json_csv;
pub mod jwt;
pub mod jwt_sign;
pub mod list_tools;
pub mod number_base;
pub mod password;
pub mod pem_decoder;
pub mod regex;
pub mod string_stats;
pub mod text_transform;
pub mod timestamp;
pub mod token_counter;
pub mod uri_parser;
pub mod url;
pub mod uuid;
pub mod yaml_json;

use crossterm::event::KeyEvent;
use ratatui::{layout::Rect, Frame};

#[derive(Clone, Copy, PartialEq)]
pub enum Focus {
    Sidebar,
    Input,
    Pattern,
}

pub enum Action {
    Quit,
    FocusSidebar,
    FocusInput,
    FocusPattern,
    Nothing,
}

pub trait Tool {
    fn name(&self) -> &'static str;
    fn render(&mut self, frame: &mut Frame, area: Rect, focus: Focus);
    fn handle_key(&mut self, key: KeyEvent, focus: Focus) -> Action;
    fn footer_hints(&self) -> String;
    fn initial_focus(&self) -> Focus {
        Focus::Input
    }
}
