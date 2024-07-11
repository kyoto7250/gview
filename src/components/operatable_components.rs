use std::sync::{Arc, Mutex};

use crate::repository::RepositoryInfo;
use crossterm::event::KeyCode;
use ratatui::{layout::Rect, Frame};

// rust enum pass the operation command
pub enum Message {
    MultipleTimes(MultipleTimesOperation),
    Once(OnceOperation),
    NoAction,
    Error { message: String },
}

pub enum MultipleTimesOperation {
    Filtering {
        query: String,
    },
    SetUp {
        repository: Arc<Mutex<RepositoryInfo>>,
    },
    ChangeShowCommit,
}

pub enum OnceOperation {
    ShowFile { file: String },
    JumpToContentView,
    JumpToFiler,
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
