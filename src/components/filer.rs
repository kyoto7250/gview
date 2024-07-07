use std::cmp::min;

use crossterm::event::KeyCode;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

use super::operatable_components::{
    Focus, Message, MultipleTimesOperation, OnceOperation, OperatableComponent,
};

pub struct Filer {
    focus: Focus,
    selected: usize,
    query: String,
    items: Vec<String>,
    results: Vec<String>,
}

impl Filer {
    pub fn new() -> Self {
        Self {
            focus: Focus::OFF,
            selected: 0,
            query: "".to_owned(),
            items: vec![],
            results: vec![],
        }
    }

    fn _handle_message(&mut self, message: &Message) -> Message {
        match message {
            Message::Once(OnceOperation::JumpToContentView) => self.focus = Focus::OFF,
            Message::Once(OnceOperation::SetUp { repository }) => {
                let binding = repository.clone();
                let mut repository = binding.lock().unwrap();
                let items = repository.recursive_walk().unwrap();
                self.items = items.clone();
                self.results = items;
            }
            Message::MultipleTimes(MultipleTimesOperation::Filtering { query }) => {
                self.focus = Focus::ON;
                self.query = query.to_owned();
                self.results = self
                    .items
                    .clone()
                    .into_iter()
                    .filter(|item| self.query.is_empty() || item.contains(&self.query))
                    .collect();

                if self.results.is_empty() {
                    self.results.push("not found".to_owned())
                }

                self.selected = min(self.selected, self.results.len().saturating_sub(1));
                return Message::Once(OnceOperation::ShowFile {
                    file: self.results[self.selected].to_owned(),
                });
            }
            _ => {}
        }
        Message::NoAction
    }
}

impl<'a> OperatableComponent for Filer {
    fn draw(&mut self, frame: &mut Frame, rect: Rect) {
        let list_items: Vec<ListItem> = self
            .results
            .iter()
            .map(|item| ListItem::new(item.to_owned()))
            .collect();

        let list = List::new(list_items)
            .block(Block::default().borders(Borders::NONE))
            .highlight_symbol(">> ")
            .style(match self.focus {
                Focus::ON => Style::default(),
                Focus::OFF => Style::default().fg(Color::DarkGray),
            });

        let mut list_state = ListState::default();
        list_state.select(Some(self.selected));
        frame.render_stateful_widget(list, rect, &mut list_state);
    }

    fn process_focus(&mut self) {
        match self.focus {
            Focus::OFF => self.focus = Focus::ON,
            Focus::ON => self.focus = Focus::OFF,
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
                if self.selected + 1 <= self.results.len().saturating_sub(1) {
                    self.selected += 1;
                    return Message::Once(OnceOperation::ShowFile {
                        file: self.results[self.selected].clone(),
                    });
                }
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
