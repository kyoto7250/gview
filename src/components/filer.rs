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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::RepositoryInfo;
    use insta::assert_snapshot;
    use ratatui::{backend::TestBackend, Terminal};
    use std::sync::{Arc, Mutex};

    fn create_mock_repo() -> Arc<Mutex<RepositoryInfo>> {
        // Create a temporary test repo using the existing test setup
        use std::env;
        use std::time::{SystemTime, UNIX_EPOCH};

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let random_suffix = std::process::id();
        let test_dir =
            env::temp_dir().join(format!("gview_filer_test_{}_{}", timestamp, random_suffix));
        let _ = std::fs::remove_dir_all(&test_dir);
        std::fs::create_dir_all(&test_dir).unwrap();

        let repo = git2::Repository::init(&test_dir).unwrap();
        let signature = git2::Signature::new(
            "Test User",
            "test@example.com",
            &git2::Time::new(1234567890, 0),
        )
        .unwrap();
        let tree_id = {
            let mut index = repo.index().unwrap();
            index.write_tree().unwrap()
        };
        let tree = repo.find_tree(tree_id).unwrap();

        let _ = repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            "Initial commit",
            &tree,
            &[],
        );

        drop(tree);
        let oid = repo.head().unwrap().target().unwrap();

        // Use the public constructor instead of direct field access
        let repo_info = crate::repository::RepositoryInfo::_from_parts(repo, oid);
        Arc::new(Mutex::new(repo_info))
    }

    #[test]
    fn test_filer_navigation_up_down() {
        let mock_repo = create_mock_repo();
        let mut filer = Filer::new(mock_repo);
        filer.results = vec![
            "file1.txt".to_string(),
            "file2.txt".to_string(),
            "file3.txt".to_string(),
        ];
        filer.selected = 1;

        // Test moving up
        let message = filer.process_events(KeyCode::Up);
        assert_eq!(filer.selected, 0);
        if let Message::Once(OnceOperation::ShowFile { file }) = message {
            assert_eq!(file, "file1.txt");
        } else {
            panic!("Expected ShowFile message");
        }

        // Test moving up at boundary
        let message = filer.process_events(KeyCode::Up);
        assert_eq!(filer.selected, 0); // Should stay at 0
        assert_eq!(message, Message::NoAction);

        // Test moving down
        let message = filer.process_events(KeyCode::Down);
        assert_eq!(filer.selected, 1);
        if let Message::Once(OnceOperation::ShowFile { file }) = message {
            assert_eq!(file, "file2.txt");
        } else {
            panic!("Expected ShowFile message");
        }
    }

    #[test]
    fn test_filer_navigation_down_at_boundary() {
        let mock_repo = create_mock_repo();
        let mut filer = Filer::new(mock_repo);
        filer.results = vec!["file1.txt".to_string(), "file2.txt".to_string()];
        filer.selected = 1; // Last item

        let message = filer.process_events(KeyCode::Down);
        assert_eq!(filer.selected, 1); // Should stay at last item
        assert_eq!(message, Message::NoAction);
    }

    #[test]
    fn test_filer_horizontal_scrolling() {
        let mock_repo = create_mock_repo();
        let mut filer = Filer::new(mock_repo);
        filer.max_scroll = 10;
        filer.start_position = 5;

        // Test scrolling left
        filer.process_events(KeyCode::Left);
        assert_eq!(filer.start_position, 4);

        // Test scrolling left at boundary
        filer.start_position = 0;
        filer.process_events(KeyCode::Left);
        assert_eq!(filer.start_position, 0); // Should stay at 0

        // Test scrolling right
        filer.start_position = 5;
        filer.process_events(KeyCode::Right);
        assert_eq!(filer.start_position, 6);

        // Test scrolling right at boundary
        filer.start_position = 10;
        filer.process_events(KeyCode::Right);
        assert_eq!(filer.start_position, 10); // Should stay at max_scroll
    }

    #[test]
    fn test_filer_enter_key() {
        let mock_repo = create_mock_repo();
        let mut filer = Filer::new(mock_repo);

        let message = filer.process_events(KeyCode::Enter);
        assert_eq!(message, Message::Once(OnceOperation::JumpToContentView));
    }

    #[test]
    fn test_filer_focus_toggle() {
        let mock_repo = create_mock_repo();
        let mut filer = Filer::new(mock_repo);

        assert_eq!(filer.focus, Focus::Off);

        filer.process_focus();
        assert_eq!(filer.focus, Focus::ON);

        filer.process_focus();
        assert_eq!(filer.focus, Focus::Off);
    }

    #[test]
    fn test_filer_draw_snapshot() {
        let mock_repo = create_mock_repo();
        let mut filer = Filer::new(mock_repo);
        filer.results = vec![
            "src/main.rs".to_string(),
            "src/lib.rs".to_string(),
            "README.md".to_string(),
        ];
        filer.selected = 1;
        filer.focus = Focus::ON;

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let rect = ratatui::layout::Rect::new(0, 0, 80, 24);
                filer.draw(frame, rect);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        assert_snapshot!(format!("{:?}", buffer));
    }

    #[test]
    fn test_filer_draw_no_files_found() {
        let mock_repo = create_mock_repo();
        let mut filer = Filer::new(mock_repo);
        filer.results = vec!["not found".to_string()];
        filer.selected = 0;
        filer.focus = Focus::ON;

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let rect = ratatui::layout::Rect::new(0, 0, 80, 24);
                filer.draw(frame, rect);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        assert_snapshot!(format!("{:?}", buffer));
    }

    #[test]
    fn test_filer_draw_unfocused() {
        let mock_repo = create_mock_repo();
        let mut filer = Filer::new(mock_repo);
        filer.results = vec![
            "src/main.rs".to_string(),
            "src/lib.rs".to_string(),
            "README.md".to_string(),
        ];
        filer.selected = 0;
        filer.focus = Focus::Off;

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let rect = ratatui::layout::Rect::new(0, 0, 80, 24);
                filer.draw(frame, rect);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        assert_snapshot!(format!("{:?}", buffer));
    }

    #[test]
    fn test_filer_draw_long_filenames() {
        let mock_repo = create_mock_repo();
        let mut filer = Filer::new(mock_repo);
        filer.results = vec![
            "src/very/long/path/to/some/deeply/nested/file.rs".to_string(),
            "another/extremely/long/path/with/many/directories/file.txt".to_string(),
            "short.rs".to_string(),
        ];
        filer.selected = 1;
        filer.focus = Focus::ON;
        filer.start_position = 10;

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let rect = ratatui::layout::Rect::new(0, 0, 80, 24);
                filer.draw(frame, rect);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        assert_snapshot!(format!("{:?}", buffer));
    }
}
