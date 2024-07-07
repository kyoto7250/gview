use std::sync::{Arc, Mutex};

use crossterm::event::KeyCode;
use ratatui::{layout::Rect, Frame};

use crate::repository::RepositoryInfo;

// rust enum pass the operation command
pub enum Message {
    MultipleTimes(MultipleTimesOperation),
    Once(OnceOperation),
    NoAction,
    Error { message: String },
}

pub enum MultipleTimesOperation {
    Filtering { query: String },
}

pub enum OnceOperation {
    SetUp {
        repository: Arc<Mutex<RepositoryInfo>>,
    },
    ShowFile {
        file: String,
    },
    JumpToContentView,
}

#[derive(Clone, Copy, Debug)]
pub enum Focus {
    OFF,
    ON,
}

pub trait OperatableComponent {
    fn draw(&mut self, frame: &mut Frame, rect: Rect);
    fn process_focus(&mut self);
    fn process_events(&mut self, events: KeyCode) -> Message;
    fn handle_message(&mut self, message: &Message) -> Message;
}
