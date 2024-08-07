use std::{
    cmp::min,
    sync::{Arc, Mutex},
};

use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

use crate::repository::RepositoryInfo;

use super::{
    filter::FilterMode,
    operatable_components::{
        Focus, Message, MultipleTimesOperation, OnceOperation, OperatableComponent,
    },
};

pub struct Filer {
    focus: Focus,
    selected: usize,
    query: String,
    start_position: usize,
    max_scroll: usize,
    mode: FilterMode,
    repository: Arc<Mutex<RepositoryInfo>>,
    items: Vec<String>,
    results: Vec<String>,
}

impl Filer {
    pub fn new(repository: Arc<Mutex<RepositoryInfo>>) -> Self {
        Self {
            focus: Focus::Off,
            selected: 0,
            query: "".to_owned(),
            start_position: 0,
            max_scroll: 0,
            mode: FilterMode::PartialMatch,
            repository,
            items: vec![],
            results: vec![],
        }
    }

    fn _handle_message(&mut self, message: &Message) -> Message {
        match message {
            Message::Once(OnceOperation::JumpToContentView) => self.focus = Focus::Off,
            Message::Once(OnceOperation::JumpToFiler) => self.focus = Focus::ON,
            Message::MultipleTimes(MultipleTimesOperation::SetUp { repository: _ }) => {
                let mut binding = self.repository.lock().unwrap();
                let items = binding.recursive_walk().unwrap();
                self.items.clone_from(&items);
                self.results = items;
                return Message::Once(OnceOperation::ShowFile {
                    file: self.results[0].to_owned(),
                });
            }
            Message::MultipleTimes(MultipleTimesOperation::ChangeShowCommit) => {
                let mut binding = self.repository.lock().unwrap();
                let items = binding.recursive_walk().unwrap();
                self.items.clone_from(&items);
                self.results = self.mode.filter(items.clone(), &self.query);
                if self.results.is_empty() {
                    self.results.push("not found".to_owned())
                }

                self.selected = min(self.selected, self.results.len().saturating_sub(1));
                self.start_position = 0;
                return Message::Once(OnceOperation::ShowFile {
                    file: self.results[self.selected].to_owned(),
                });
            }
            Message::MultipleTimes(MultipleTimesOperation::Filtering { query, mode }) => {
                query.clone_into(&mut self.query);
                self.mode = *mode;
                self.results = self.mode.filter(self.items.clone(), query);
                if self.results.is_empty() {
                    self.results.push("not found".to_owned())
                }

                self.selected = min(self.selected, self.results.len().saturating_sub(1));
                self.start_position = 0;
                return Message::Once(OnceOperation::ShowFile {
                    file: self.results[self.selected].to_owned(),
                });
            }
            _ => {}
        }
        Message::NoAction
    }
}

impl OperatableComponent for Filer {
    fn draw(&mut self, frame: &mut Frame, rect: Rect) {
        let title = if self.results.len() == 1 && self.results[0] == "not found" {
            "0 files".to_string()
        } else {
            format!("{} files", self.results.len())
        };
        frame.render_widget(Block::default().title(title).borders(Borders::ALL), rect);

        let chunk = Layout::default()
            .vertical_margin(1)
            .horizontal_margin(1)
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(100)].as_ref())
            .split(rect)[0];

        let list_items: Vec<ListItem> = self
            .results
            .iter()
            .map(|item| {
                if self.start_position < item.len() {
                    ListItem::new(item[self.start_position..].to_owned())
                } else {
                    ListItem::new("".to_owned())
                }
            })
            .collect();

        // 3 is the size of ">> "
        self.max_scroll = self
            .results
            .iter()
            .map(String::len)
            .max()
            .unwrap_or(0)
            .saturating_sub(chunk.width as usize - 3);
        let list = List::new(list_items)
            .block(Block::default().borders(Borders::NONE))
            .highlight_symbol(">> ")
            .style(match self.focus {
                Focus::ON => Style::default(),
                Focus::Off => Style::default().fg(Color::DarkGray),
            });

        let mut list_state = ListState::default();
        list_state.select(Some(self.selected));
        frame.render_stateful_widget(list, chunk, &mut list_state);
    }

    fn process_focus(&mut self) {
        match self.focus {
            Focus::Off => self.focus = Focus::ON,
            Focus::ON => self.focus = Focus::Off,
        }
    }
    fn process_events(&mut self, code: KeyCode) -> Message {
        match code {
            KeyCode::Up => {
                if self.selected > 0 {
                    self.selected -= 1;
                    return Message::Once(OnceOperation::ShowFile {
                        file: self.results[self.selected].clone(),
                    });
                }
            }
            KeyCode::Down => {
                if self.selected < self.results.len().saturating_sub(1) {
                    self.selected += 1;
                    return Message::Once(OnceOperation::ShowFile {
                        file: self.results[self.selected].clone(),
                    });
                }
            }
            KeyCode::Left => {
                if self.start_position > 0 {
                    self.start_position -= 1
                }
            }
            KeyCode::Right => {
                self.start_position += 1;
                self.start_position = std::cmp::min(self.start_position, self.max_scroll)
            }
            KeyCode::Enter => return Message::Once(OnceOperation::JumpToContentView),
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
