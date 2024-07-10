use crossterm::event::KeyCode;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::Paragraph,
    Frame,
};

use super::operatable_components::{
    Focus, Message, MultipleTimesOperation, OnceOperation, OperatableComponent,
};

pub struct Filter {
    focus: Focus,
    input: String,
    character_index: usize,
}

impl Filter {
    pub fn new() -> Self {
        Self {
            focus: Focus::OFF,
            input: "".to_owned(),
            character_index: 0,
        }
    }

    fn enter_char(&mut self, char: char) {
        let index = self.byte_index();
        self.input.insert(index, char);
        self.move_cursor_right();
    }

    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.character_index.saturating_sub(1);
        self.character_index = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.character_index.saturating_add(1);
        self.character_index = self.clamp_cursor(cursor_moved_right);
    }

    fn byte_index(&self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.input.len())
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.chars().count())
    }

    fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.character_index != 0;
        if is_not_cursor_leftmost {
            // Method "remove" is not used on the saved text for deleting the selected char.
            // Reason: Using remove on String works on bytes instead of the chars.
            // Using remove would require special care because of char boundaries.

            let current_index = self.character_index;
            let from_left_to_current_index = current_index - 1;

            // Getting all characters before the selected character.
            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
            // Getting all characters after selected character.
            let after_char_to_delete = self.input.chars().skip(current_index);

            // Put all characters together except the selected one.
            // By leaving the selected one out, it is forgotten and therefore deleted.
            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    fn _handle_message(&mut self, message: &Message) -> Message {
        match message {
            Message::Once(OnceOperation::JumpToFiler) => self.focus = Focus::OFF,
            Message::MultipleTimes(MultipleTimesOperation::SetUp { repository: _ }) => {
                self.focus = Focus::ON
            }
            _ => {}
        }

        Message::NoAction
    }
}

impl OperatableComponent for Filter {
    fn draw(&mut self, frame: &mut Frame, rect: Rect) {
        // TODO: should scroll
        let filter_paragraph = Paragraph::new(self.input.to_owned()).style(match self.focus {
            Focus::ON => Style::default(),
            Focus::OFF => Style::default().fg(Color::DarkGray),
        });
        frame.render_widget(filter_paragraph, rect);
        frame.set_cursor(rect.x + self.character_index as u16, rect.y);
    }

    fn process_focus(&mut self) {
        match self.focus {
            Focus::OFF => self.focus = Focus::ON,
            Focus::ON => self.focus = Focus::OFF,
        }
    }

    fn process_events(&mut self, events: crossterm::event::KeyCode) -> Message {
        match events {
            KeyCode::Char(char) => {
                self.enter_char(char);
                return Message::MultipleTimes(MultipleTimesOperation::Filtering {
                    query: self.input.to_owned(),
                });
            }
            KeyCode::Enter => return Message::Once(OnceOperation::JumpToFiler),
            KeyCode::Backspace => {
                self.delete_char();
                return Message::MultipleTimes(MultipleTimesOperation::Filtering {
                    query: self.input.to_owned(),
                });
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
