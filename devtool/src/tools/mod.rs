pub mod base64;
pub mod hash;
pub mod html_entity;
pub mod json;
pub mod jwt;
pub mod number_base;
pub mod regex;
pub mod string_stats;
pub mod text_transform;
pub mod timestamp;
pub mod url;
pub mod uuid;

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
