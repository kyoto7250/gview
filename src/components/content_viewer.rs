use std::sync::{Arc, Mutex};

use crossterm::event::KeyCode;
use ratatui::{
    layout::Rect,
    style::{Color, Style, Stylize},
    widgets::{Block, Paragraph, Wrap},
    Frame,
};

use crate::repository::{CommitRow, RepositoryInfo};

use super::operatable_components::{Focus, Message, OnceOperation, OperatableComponent};

pub enum ShowMode {
    WithLine,
    WithBlame,
    NoLine,
}

impl ShowMode {
    fn next(&mut self) -> ShowMode {
        match self {
            Self::WithLine => Self::WithBlame,
            Self::WithBlame => Self::NoLine,
            Self::NoLine => Self::WithLine,
        }
    }

    fn concat(&mut self, rows: Vec<CommitRow>) -> String {
        match self {
            Self::NoLine => rows
                .iter()
                .map(|row| row.line.to_owned())
                .collect::<Vec<String>>()
                .join("\n"),
            Self::WithLine => {
                // TODO: maximum size
                rows.iter()
                    .map(|row| format!("{} | {} ", row.number, row.line.to_owned()))
                    .collect::<Vec<String>>()
                    .join("\n")
            }
            Self::WithBlame => {
                // TODO: maximum size
                rows.iter()
                    .map(|row| format!("{} | {} ", row.commit, row.line.to_owned()))
                    .collect::<Vec<String>>()
                    .join("\n")
            }
        }
    }
}

pub struct ContentViewer {
    focus: Focus,
    title: String,
    content: String,
    context_size: usize,
    scroll_position: usize,
    height: usize,
    repository: Arc<Mutex<RepositoryInfo>>,
    mode: ShowMode,
}

impl ContentViewer {
    pub fn new(repository: Arc<Mutex<RepositoryInfo>>) -> Self {
        Self {
            focus: Focus::Off,
            title: "Content Viewer".to_owned(),
            content: "".to_owned(),
            repository,
            context_size: 0,
            height: 0,
            scroll_position: 0,
            mode: ShowMode::WithLine,
        }
    }

    fn _handle_message(&mut self, message: &Message) -> Message {
        match message {
            Message::Once(OnceOperation::ShowFile { file }) => {
                // update content view
                file.clone_into(&mut self.title);
                let mut repository = self.repository.lock().unwrap();

                if let Ok(rows) = repository.get_content(file.to_owned()) {
                    self.content = self.mode.concat(rows);
                    self.scroll_position = 0
                } else {
                    return Message::Error {
                        _message: "failed to get content".to_owned(),
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
            .skip(self.scroll_position)
            .take(rect.height as usize)
            .map(|line| format!("{}\n", line))
            .collect();

        let paragraph = Paragraph::new(contents.concat())
            .block(title_block(&self.title, self.focus))
            .wrap(Wrap { trim: false });

        self.context_size = Paragraph::new(self.content.clone()).line_count(rect.width);
        self.height = rect.height as usize;
        frame.render_widget(paragraph, rect)
    }

    fn process_focus(&mut self) {
        match self.focus {
            Focus::Off => self.focus = Focus::ON,
            Focus::ON => self.focus = Focus::Off,
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
                // 4 is the using frame size
                if self.scroll_position < 4 + self.context_size.saturating_sub(1 + self.height) {
                    self.scroll_position += 1;
                }
            }
            KeyCode::Char('o') => {
                self.mode = self.mode.next();
                let mut repository = self.repository.lock().unwrap();
                if let Ok(rows) = repository.get_content(self.title.to_owned()) {
                    self.content = self.mode.concat(rows);
                    self.scroll_position = 0
                } else {
                    return Message::Error {
                        _message: "failed to get content".to_owned(),
                    };
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
