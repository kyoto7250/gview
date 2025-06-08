use std::sync::{Arc, Mutex};

use crossterm::event::KeyCode;
use ratatui::{
    layout::Rect,
    style::{Color, Style, Stylize},
    widgets::{Block, Paragraph},
    Frame,
};

use crate::repository::RepositoryInfo;

use super::operatable_components::{Focus, Message, MultipleTimesOperation, OperatableComponent};

pub struct CommitViewer {
    focus: Focus,
    content: String,
    repository: Arc<Mutex<RepositoryInfo>>,
}

impl CommitViewer {
    pub fn new(repository: Arc<Mutex<RepositoryInfo>>) -> Self {
        Self {
            focus: Focus::Off,
            content: "".to_owned(),
            repository,
        }
    }

    fn _handle_message(&mut self, message: &Message) -> Message {
        match message {
            Message::MultipleTimes(MultipleTimesOperation::SetUp { repository }) => {
                let mut repository = repository.lock().unwrap();
                let (commit_id, commit_message) = repository.current_commit().unwrap();
                self.content = format!("{}: {}", commit_id, commit_message);
            }
            Message::MultipleTimes(MultipleTimesOperation::ChangeShowCommit) => {
                let mut repository = self.repository.lock().unwrap();
                let (commit_id, commit_message) = repository.current_commit().unwrap();
                self.content = format!("{}: {}", commit_id, commit_message);
            }
            _ => {}
        }
        Message::NoAction
    }
}

impl OperatableComponent for CommitViewer {
    fn draw(&mut self, frame: &mut Frame, rect: Rect) {
        let right_paragraph = Paragraph::new(self.content.to_owned())
            .block(title_block("current commit", self.focus));
        frame.render_widget(right_paragraph, rect);
    }
    fn process_focus(&mut self) {
        match self.focus {
            Focus::Off => self.focus = Focus::ON,
            Focus::ON => self.focus = Focus::Off,
        }
    }
    fn process_events(&mut self, events: KeyCode) -> Message {
        match events {
            KeyCode::Down => {
                let mut binding = self.repository.lock().unwrap();
                binding.set_parent_commit();
                return Message::MultipleTimes(MultipleTimesOperation::ChangeShowCommit);
            }
            KeyCode::Up => {
                let mut binding = self.repository.lock().unwrap();
                let _ = binding.set_next_commit();
                return Message::MultipleTimes(MultipleTimesOperation::ChangeShowCommit);
            }
            _ => {}
        }
        Message::NoAction
    }

    fn handle_message(&mut self, message: &Message) -> Message {
        // allow pattern
        // 1. MultipleTimes -> Once
        // 2. MultipleTimes -> NoAction
        // 3. Once -> NoAction
        // 4. NoAction -> NoAction
        match (message, self._handle_message(message)) {
            (Message::MultipleTimes(_), Message::MultipleTimes(_)) => unreachable!(),
            (Message::Once(_), Message::MultipleTimes(_)) => unreachable!(),
            (Message::Once(_), Message::Once(_)) => unreachable!(),
            (Message::NoAction, Message::MultipleTimes(_)) => unreachable!(),
            (Message::NoAction, Message::Once(_)) => unreachable!(),
            (_, new_message) => new_message,
        }
    }
}

fn title_block(title: &str, focus: Focus) -> Block {
    Block::bordered()
        .title(title.bold().into_left_aligned_line())
        .style(match focus {
            Focus::ON => Style::default(),
            Focus::Off => Style::default().fg(Color::DarkGray),
        })
}
