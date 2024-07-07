use ratatui::{
    layout::Rect,
    style::{Color, Style, Stylize},
    widgets::{Block, Paragraph},
    Frame,
};

use super::operatable_components::{Focus, Message, OnceOperation, OperatableComponent};

pub struct CommitViewer {
    focus: Focus,
    content: String,
}

impl CommitViewer {
    pub fn new() -> Self {
        Self {
            focus: Focus::OFF,
            content: "".to_owned(),
        }
    }

    fn _handle_message(&mut self, message: &Message) -> Message {
        match message {
            Message::MultipleTimes(_) => {}
            Message::Once(operation) => {
                match operation {
                    OnceOperation::SetUp { repository } => {
                        // set commit
                        let binding = repository.clone();
                        let mut repository = binding.lock().unwrap();
                        let (commit_id, commit_message) = repository.current_commit().unwrap();
                        self.content = format!("{}: {}", commit_id, commit_message);
                    }
                    _ => {}
                }
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
            Focus::OFF => self.focus = Focus::ON,
            Focus::ON => self.focus = Focus::OFF,
        }
    }
    fn process_events(&mut self, events: crossterm::event::KeyCode) -> Message {
        Message::NoAction
    }

    fn handle_message(&mut self, message: &Message) -> Message {
        // allow pattern
        // 1. MultipleTimes -> Once
        // 2. MultipleTimes -> NoAction
        // 3. Once -> NoAction
        // 4. NoAction -> NoAction
        return match (message, self._handle_message(message)) {
            (Message::MultipleTimes(_), Message::MultipleTimes(_)) => unreachable!(),
            (Message::Once(_), Message::MultipleTimes(_)) => unreachable!(),
            (Message::Once(_), Message::Once(_)) => unreachable!(),
            (Message::NoAction, Message::MultipleTimes(_)) => unreachable!(),
            (Message::NoAction, Message::Once(_)) => unreachable!(),
            (_, new_message) => new_message,
        };
    }
}

fn title_block(title: &str, focus: Focus) -> Block {
    return Block::bordered()
        .title(title.bold().into_left_aligned_line())
        .style(match focus {
            Focus::ON => Style::default(),
            Focus::OFF => Style::default().fg(Color::DarkGray),
        });
}
