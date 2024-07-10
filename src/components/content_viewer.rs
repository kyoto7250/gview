use std::sync::{Arc, Mutex};

use crossterm::event::KeyCode;
use ratatui::{
    layout::Rect,
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Paragraph, Wrap},
    Frame,
};

use crate::repository::RepositoryInfo;

use super::operatable_components::{Focus, Message, OnceOperation, OperatableComponent};

pub struct ContentViewer {
    focus: Focus,
    title: String,
    content: String,
    context_size: usize,
    scroll_position: usize,
    repository: Arc<Mutex<RepositoryInfo>>,
}

impl ContentViewer {
    pub fn new(repository: Arc<Mutex<RepositoryInfo>>) -> Self {
        Self {
            focus: Focus::OFF,
            title: "Content Viewer".to_owned(),
            content: "".to_owned(),
            repository: repository,
            context_size: 0,
            scroll_position: 0,
        }
    }

    fn _handle_message(&mut self, message: &Message) -> Message {
        match message {
            Message::Once(OnceOperation::ShowFile { file }) => {
                // update content view
                self.title = file.to_owned();
                let mut repository = self.repository.lock().unwrap();

                if let Ok(content) = repository.get_content(file.to_owned()) {
                    self.content = content;
                    self.scroll_position = 0
                } else {
                    return Message::Error {
                        message: "failed to get content".to_owned(),
                    };
                }
            }
            Message::Once(OnceOperation::JumpToContentView) => self.focus = Focus::ON,
            _ => {}
        }
        Message::NoAction
    }
}

impl OperatableComponent for ContentViewer {
    fn draw(&mut self, frame: &mut Frame, rect: Rect) {
        let contents: Vec<String> = self
            .content
            .lines()
            .into_iter()
            .skip(self.scroll_position)
            .take(rect.height as usize)
            .map(|line| String::from(format!("{}\n", line)))
            .collect();

        let paragraph = Paragraph::new(contents.concat())
            .block(title_block(&self.title, self.focus))
            .wrap(Wrap { trim: false });

        self.context_size = Paragraph::new(self.content.clone()).line_count(rect.width);
        frame.render_widget(paragraph, rect)
    }

    fn process_focus(&mut self) {
        match self.focus {
            Focus::OFF => self.focus = Focus::ON,
            Focus::ON => self.focus = Focus::OFF,
        }
    }

    fn process_events(&mut self, events: KeyCode) -> Message {
        match events {
            KeyCode::Up => {
                if self.scroll_position > 0 {
                    self.scroll_position -= 1;
                }
            }
            KeyCode::Down => {
                if self.scroll_position + 1 <= self.context_size.saturating_sub(1) {
                    self.scroll_position += 1;
                }
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
