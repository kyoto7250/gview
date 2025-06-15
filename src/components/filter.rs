use crossterm::event::KeyCode;
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use regex::Regex;

use super::operatable_components::{
    Focus, Message, MultipleTimesOperation, OnceOperation, OperatableComponent,
};

#[allow(clippy::enum_variant_names)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FilterMode {
    PartialMatch,
    FuzzyMatch,
    RegularMatch,
}

impl FilterMode {
    fn next(self) -> FilterMode {
        match self {
            FilterMode::PartialMatch => FilterMode::FuzzyMatch,
            FilterMode::FuzzyMatch => FilterMode::RegularMatch,
            FilterMode::RegularMatch => FilterMode::PartialMatch,
        }
    }

    fn prev(self) -> FilterMode {
        match self {
            FilterMode::PartialMatch => FilterMode::RegularMatch,
            FilterMode::FuzzyMatch => FilterMode::PartialMatch,
            FilterMode::RegularMatch => FilterMode::FuzzyMatch,
        }
    }

    fn appearance(self) -> (String, Style) {
        match self {
            FilterMode::PartialMatch => {
                ("Partial Match".to_owned(), Style::default().fg(Color::Blue))
            }
            FilterMode::FuzzyMatch => ("Fuzzy Search".to_owned(), Style::default().fg(Color::Red)),
            FilterMode::RegularMatch => (
                "Regular Search".to_owned(),
                Style::default().fg(Color::Green),
            ),
        }
    }

    pub fn filter(self, items: Vec<String>, query: &String) -> Vec<String> {
        match self {
            FilterMode::PartialMatch => items
                .into_iter()
                .filter(|item| query.is_empty() || item.contains(query))
                .collect(),
            FilterMode::FuzzyMatch => {
                let matcher = SkimMatcherV2::default();
                let mut results = items
                    .into_iter()
                    .filter_map(|item| matcher.fuzzy_match(&item, query).map(|score| (item, score)))
                    .collect::<Vec<_>>();
                results.sort_by(|item, other| other.1.cmp(&item.1));
                results
                    .into_iter()
                    .map(|(item, _)| item)
                    .collect::<Vec<_>>()
            }
            FilterMode::RegularMatch => {
                if let Ok(re) = Regex::new(query) {
                    // TODO: check the regular expression behavior
                    items.into_iter().filter(|s| re.is_match(s)).collect()
                } else {
                    // TODO: popup regular expression error
                    vec!["error".to_owned()]
                }
            }
        }
    }
}

pub struct Filter {
    focus: Focus,
    mode: FilterMode,
    input: String,
    character_index: usize,
}

impl Filter {
    pub fn new() -> Self {
        Self {
            focus: Focus::Off,
            mode: FilterMode::PartialMatch,
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
            Message::Once(OnceOperation::JumpToFiler) => self.focus = Focus::Off,
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
        let (title, border_style) = self.mode.appearance();
        frame.render_widget(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(match self.focus {
                    Focus::Off => Style::default().fg(Color::DarkGray),
                    Focus::ON => border_style,
                }),
            rect,
        );

        let chunk = Layout::default()
            .vertical_margin(1)
            .horizontal_margin(1)
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(100)].as_ref())
            .split(rect)[0];

        let width = chunk.width;
        let input = self.input.to_owned();
        let overflow = input.len().saturating_sub(width as usize);
        if overflow > 0 {
            input[overflow..].clone_into(&mut input.to_owned())
        }

        let filter_paragraph = Paragraph::new(input).style(match self.focus {
            Focus::ON => Style::default(),
            Focus::Off => Style::default().fg(Color::DarkGray),
        });
        frame.render_widget(filter_paragraph, chunk);

        let cursor_position = std::cmp::min(chunk.x + self.character_index as u16, chunk.width);
        frame.set_cursor(cursor_position, chunk.y);
    }

    fn process_focus(&mut self) {
        match self.focus {
            Focus::Off => self.focus = Focus::ON,
            Focus::ON => self.focus = Focus::Off,
        }
    }

    fn process_events(&mut self, events: crossterm::event::KeyCode) -> Message {
        match events {
            KeyCode::Down => {
                self.mode = self.mode.prev();
                return Message::MultipleTimes(MultipleTimesOperation::Filtering {
                    query: self.input.to_owned(),
                    mode: self.mode,
                });
            }
            KeyCode::Up => {
                self.mode = self.mode.next();
                return Message::MultipleTimes(MultipleTimesOperation::Filtering {
                    query: self.input.to_owned(),
                    mode: self.mode,
                });
            }
            KeyCode::Char(char) => {
                self.enter_char(char);
                return Message::MultipleTimes(MultipleTimesOperation::Filtering {
                    query: self.input.to_owned(),
                    mode: self.mode,
                });
            }
            KeyCode::Enter => return Message::Once(OnceOperation::JumpToFiler),
            KeyCode::Backspace => {
                self.delete_char();
                return Message::MultipleTimes(MultipleTimesOperation::Filtering {
                    query: self.input.to_owned(),
                    mode: self.mode,
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

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_snapshot;
    use ratatui::{backend::TestBackend, Terminal};

    #[test]
    fn test_filter_mode_transitions() {
        assert_eq!(FilterMode::PartialMatch.next(), FilterMode::FuzzyMatch);
        assert_eq!(FilterMode::FuzzyMatch.next(), FilterMode::RegularMatch);
        assert_eq!(FilterMode::RegularMatch.next(), FilterMode::PartialMatch);

        assert_eq!(FilterMode::PartialMatch.prev(), FilterMode::RegularMatch);
        assert_eq!(FilterMode::FuzzyMatch.prev(), FilterMode::PartialMatch);
        assert_eq!(FilterMode::RegularMatch.prev(), FilterMode::FuzzyMatch);
    }

    #[test]
    fn test_filter_mode_partial_match() {
        let items = vec!["hello".to_string(), "world".to_string(), "help".to_string()];
        let query = "hel".to_string();
        let result = FilterMode::PartialMatch.filter(items, &query);
        assert_eq!(result, vec!["hello", "help"]);
    }

    #[test]
    fn test_filter_mode_partial_match_empty_query() {
        let items = vec!["hello".to_string(), "world".to_string()];
        let query = "".to_string();
        let result = FilterMode::PartialMatch.filter(items.clone(), &query);
        assert_eq!(result, items);
    }

    #[test]
    fn test_filter_mode_fuzzy_match() {
        let items = vec![
            "hello_world".to_string(),
            "help".to_string(),
            "world".to_string(),
        ];
        let query = "hlw".to_string();
        let result = FilterMode::FuzzyMatch.filter(items, &query);
        assert_eq!(result[0], "hello_world"); // Should match best
    }

    #[test]
    fn test_filter_mode_regular_match_valid() {
        let items = vec![
            "hello123".to_string(),
            "world456".to_string(),
            "test".to_string(),
        ];
        let query = r"\d+".to_string(); // Match digits
        let result = FilterMode::RegularMatch.filter(items, &query);
        assert_eq!(result, vec!["hello123", "world456"]);
    }

    #[test]
    fn test_filter_mode_regular_match_invalid() {
        let items = vec!["hello".to_string(), "world".to_string()];
        let query = "[".to_string(); // Invalid regex
        let result = FilterMode::RegularMatch.filter(items, &query);
        assert_eq!(result, vec!["error"]);
    }

    #[test]
    fn test_filter_cursor_movement() {
        let mut filter = Filter::new();
        filter.input = "hello".to_string();
        filter.character_index = 0;

        filter.move_cursor_right();
        assert_eq!(filter.character_index, 1);

        filter.move_cursor_right();
        assert_eq!(filter.character_index, 2);

        filter.move_cursor_left();
        assert_eq!(filter.character_index, 1);

        filter.move_cursor_left();
        assert_eq!(filter.character_index, 0);

        // Test bounds
        filter.move_cursor_left();
        assert_eq!(filter.character_index, 0); // Should stay at 0
    }

    #[test]
    fn test_filter_char_insertion() {
        let mut filter = Filter::new();
        filter.enter_char('h');
        assert_eq!(filter.input, "h");
        assert_eq!(filter.character_index, 1);

        filter.enter_char('i');
        assert_eq!(filter.input, "hi");
        assert_eq!(filter.character_index, 2);
    }

    #[test]
    fn test_filter_char_deletion() {
        let mut filter = Filter::new();
        filter.input = "hello".to_string();
        filter.character_index = 5;

        filter.delete_char();
        assert_eq!(filter.input, "hell");
        assert_eq!(filter.character_index, 4);

        filter.delete_char();
        assert_eq!(filter.input, "hel");
        assert_eq!(filter.character_index, 3);

        // Test deletion at beginning
        filter.character_index = 0;
        filter.delete_char();
        assert_eq!(filter.input, "hel"); // Should not change
        assert_eq!(filter.character_index, 0);
    }

    #[test]
    fn test_filter_draw_snapshot() {
        let mut filter = Filter::new();
        filter.input = "test input".to_string();
        filter.mode = FilterMode::FuzzyMatch;
        filter.focus = Focus::ON;

        let backend = TestBackend::new(40, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let rect = Rect::new(0, 0, 40, 10);
                filter.draw(frame, rect);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        assert_snapshot!(format!("{:?}", buffer));
    }

    #[test]
    fn test_filter_draw_focused_vs_unfocused() {
        let mut filter_focused = Filter::new();
        filter_focused.input = "test".to_string();
        filter_focused.focus = Focus::ON;

        let mut filter_unfocused = Filter::new();
        filter_unfocused.input = "test".to_string();
        filter_unfocused.focus = Focus::Off;

        let backend_focused = TestBackend::new(20, 5);
        let mut terminal_focused = Terminal::new(backend_focused).unwrap();

        let backend_unfocused = TestBackend::new(20, 5);
        let mut terminal_unfocused = Terminal::new(backend_unfocused).unwrap();

        terminal_focused
            .draw(|frame| {
                let rect = Rect::new(0, 0, 20, 5);
                filter_focused.draw(frame, rect);
            })
            .unwrap();

        terminal_unfocused
            .draw(|frame| {
                let rect = Rect::new(0, 0, 20, 5);
                filter_unfocused.draw(frame, rect);
            })
            .unwrap();

        let buffer_focused = terminal_focused.backend().buffer();
        let buffer_unfocused = terminal_unfocused.backend().buffer();

        assert_snapshot!("focused", format!("{:?}", buffer_focused));
        assert_snapshot!("unfocused", format!("{:?}", buffer_unfocused));
    }
}
