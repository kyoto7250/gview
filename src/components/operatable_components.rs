use std::sync::{Arc, Mutex};

use crate::repository::RepositoryInfo;
use crossterm::event::KeyCode;
use ratatui::{layout::Rect, Frame};

use super::filter::FilterMode;

// rust enum pass the operation command
#[derive(Debug, PartialEq)]
pub enum Message {
    MultipleTimes(MultipleTimesOperation),
    Once(OnceOperation),
    NoAction,
    Error { _message: String },
}

#[derive(Debug)]
pub enum MultipleTimesOperation {
    Filtering {
        query: String,
        mode: FilterMode,
    },
    SetUp {
        repository: Arc<Mutex<RepositoryInfo>>,
    },
    ChangeShowCommit,
}

impl PartialEq for MultipleTimesOperation {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                MultipleTimesOperation::Filtering {
                    query: q1,
                    mode: m1,
                },
                MultipleTimesOperation::Filtering {
                    query: q2,
                    mode: m2,
                },
            ) => q1 == q2 && m1 == m2,
            (
                MultipleTimesOperation::ChangeShowCommit,
                MultipleTimesOperation::ChangeShowCommit,
            ) => true,
            (MultipleTimesOperation::SetUp { .. }, MultipleTimesOperation::SetUp { .. }) => true, // Compare by type only
            _ => false,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum OnceOperation {
    ShowFile { file: String },
    JumpToContentView,
    JumpToFiler,
    OpenCommitModal,
    CloseCommitModal,
    SetCommitById { commit_id: String },
    ShowHelpModal,
    CloseHelpModal,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Focus {
    Off,
    ON,
}

pub trait OperatableComponent {
    fn draw(&mut self, frame: &mut Frame, rect: Rect);
    fn process_focus(&mut self);
    fn process_events(&mut self, events: KeyCode) -> Message;
    fn handle_message(&mut self, message: &Message) -> Message;
}
