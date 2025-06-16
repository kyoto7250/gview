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
    fn concat(&mut self, rows: Vec<CommitRow>) -> String {
        match self {
            Self::NoLine => rows
                .iter()
                .map(|row| row.line.to_owned())
                .collect::<Vec<String>>()
                .join("\n"),
            Self::WithLine => {
                let max_line_number = rows.iter().map(|row| row.number).max().unwrap_or(0);
                let width = max_line_number.to_string().len();
                rows.iter()
                    .map(|row| {
                        format!(
                            "{:width$} | {} ",
                            row.number,
                            row.line.to_owned(),
                            width = width
                        )
                    })
                    .collect::<Vec<String>>()
                    .join("\n")
            }
            Self::WithBlame => rows
                .iter()
                .map(|row| format!("{} | {} ", row.commit, row.line.to_owned()))
                .collect::<Vec<String>>()
                .join("\n"),
        }
    }
}

pub struct ContentViewer {
    focus: Focus,
    title: String,
    content: String,
    context_size: usize,
    scroll_position: usize,
    horizontal_scroll: usize,
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
            horizontal_scroll: 0,
            mode: ShowMode::WithLine,
        }
    }

    fn toggle_line_numbers(&mut self) {
        self.mode = match self.mode {
            ShowMode::NoLine => ShowMode::WithLine,
            ShowMode::WithLine => ShowMode::NoLine,
            ShowMode::WithBlame => ShowMode::WithLine,
        };
        self.refresh_content();
    }

    fn toggle_blame_mode(&mut self) {
        self.mode = match self.mode {
            ShowMode::NoLine => ShowMode::WithBlame,
            ShowMode::WithLine => ShowMode::WithBlame,
            ShowMode::WithBlame => ShowMode::NoLine,
        };
        self.refresh_content();
    }

    fn refresh_content(&mut self) {
        let mut repository = match self.repository.lock() {
            Ok(repo) => repo,
            Err(_) => return,
        };
        if let Ok(rows) = repository.get_content(self.title.to_owned()) {
            self.content = self.mode.concat(rows);
            self.scroll_position = 0;
        }
    }

    fn _handle_message(&mut self, message: &Message) -> Message {
        match message {
            Message::Once(OnceOperation::ShowFile { file }) => {
                // update content view
                file.clone_into(&mut self.title);
                let mut repository = match self.repository.lock() {
                    Ok(repo) => repo,
                    Err(_) => {
                        return Message::Error {
                            _message: "Failed to acquire repository lock".to_owned(),
                        }
                    }
                };

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
            .map(|line| {
                let line_chars: Vec<char> = line.chars().collect();
                let start = self.horizontal_scroll.min(line_chars.len());
                let visible_line: String = line_chars.iter().skip(start).collect();
                format!("{}\n", visible_line)
            })
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
            KeyCode::Up | KeyCode::Char('k') => {
                if self.scroll_position > 0 {
                    self.scroll_position -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                // 4 is the using frame size
                if self.scroll_position < 4 + self.context_size.saturating_sub(1 + self.height) {
                    self.scroll_position += 1;
                }
            }
            KeyCode::Left | KeyCode::Char('h') => {
                if self.horizontal_scroll > 0 {
                    self.horizontal_scroll -= 1;
                }
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.horizontal_scroll += 1;
            }
            KeyCode::Char('n') => {
                self.toggle_line_numbers();
            }
            KeyCode::Char('b') => {
                self.toggle_blame_mode();
            }
            KeyCode::Char('g') => {
                if self.title != "not found" && !self.title.is_empty() {
                    let current_line = self.scroll_position + 1;
                    let repository = match self.repository.lock() {
                        Ok(repo) => repo,
                        Err(_) => {
                            return Message::Error {
                                _message: "Failed to acquire repository lock".to_owned(),
                            }
                        }
                    };
                    if let Err(e) = repository.open_file_in_browser(&self.title, current_line) {
                        return Message::Error {
                            _message: format!("Failed to open in browser: {}", e),
                        };
                    }
                }
            }
            _ => {}
        }
        Message::NoAction
    }

    fn handle_message(&mut self, message: &Message) -> Message {
        self._handle_message(message)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::RepositoryInfo;
    use insta::assert_snapshot;
    use ratatui::{backend::TestBackend, Terminal};
    use std::sync::{Arc, Mutex};

    fn create_mock_repo() -> Arc<Mutex<RepositoryInfo>> {
        use std::env;
        use std::time::{SystemTime, UNIX_EPOCH};

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let random_suffix = std::process::id();
        let test_dir = env::temp_dir().join(format!(
            "gview_content_viewer_test_{}_{}",
            timestamp, random_suffix
        ));
        let _ = std::fs::remove_dir_all(&test_dir);
        std::fs::create_dir_all(&test_dir).unwrap();

        let repo = git2::Repository::init(&test_dir).unwrap();
        let signature = git2::Signature::new(
            "Test User",
            "test@localhost",
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

        let repo_info = RepositoryInfo::_from_parts(repo, oid);
        Arc::new(Mutex::new(repo_info))
    }

    #[test]
    fn test_content_viewer_draw_empty() {
        let mock_repo = create_mock_repo();
        let mut content_viewer = ContentViewer::new(mock_repo);
        content_viewer.focus = Focus::ON;
        content_viewer.title = "test.rs".to_string();
        content_viewer.content = "".to_string();

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let rect = ratatui::layout::Rect::new(0, 0, 80, 24);
                content_viewer.draw(frame, rect);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        assert_snapshot!(format!("{:?}", buffer));
    }

    #[test]
    fn test_content_viewer_draw_with_content() {
        let mock_repo = create_mock_repo();
        let mut content_viewer = ContentViewer::new(mock_repo);
        content_viewer.focus = Focus::ON;
        content_viewer.title = "main.rs".to_string();
        content_viewer.content = "fn main() {\n    println!(\"Hello, world!\");\n}\n\nfn another_function() {\n    // Some comment\n    let x = 42;\n    println!(\"x = {}\", x);\n}".to_string();

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let rect = ratatui::layout::Rect::new(0, 0, 80, 24);
                content_viewer.draw(frame, rect);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        assert_snapshot!(format!("{:?}", buffer));
    }

    #[test]
    fn test_content_viewer_draw_unfocused() {
        let mock_repo = create_mock_repo();
        let mut content_viewer = ContentViewer::new(mock_repo);
        content_viewer.focus = Focus::Off;
        content_viewer.title = "lib.rs".to_string();
        content_viewer.content = "pub fn add(left: usize, right: usize) -> usize {\n    left + right\n}\n\n#[cfg(test)]\nmod tests {\n    use super::*;\n\n    #[test]\n    fn it_works() {\n        let result = add(2, 2);\n        assert_eq!(result, 4);\n    }\n}".to_string();

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let rect = ratatui::layout::Rect::new(0, 0, 80, 24);
                content_viewer.draw(frame, rect);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        assert_snapshot!(format!("{:?}", buffer));
    }

    #[test]
    fn test_content_viewer_draw_with_line_numbers() {
        let mock_repo = create_mock_repo();
        let mut content_viewer = ContentViewer::new(mock_repo);
        content_viewer.focus = Focus::ON;
        content_viewer.title = "example.rs".to_string();
        content_viewer.mode = ShowMode::WithLine;
        content_viewer.content = "1 | use std::collections::HashMap;\n2 | \n3 | fn main() {\n4 |     let mut map = HashMap::new();\n5 |     map.insert(\"key\", \"value\");\n6 |     println!(\"{:?}\", map);\n7 | }".to_string();

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let rect = ratatui::layout::Rect::new(0, 0, 80, 24);
                content_viewer.draw(frame, rect);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        assert_snapshot!(format!("{:?}", buffer));
    }

    #[test]
    fn test_content_viewer_draw_with_blame() {
        let mock_repo = create_mock_repo();
        let mut content_viewer = ContentViewer::new(mock_repo);
        content_viewer.focus = Focus::ON;
        content_viewer.title = "blame_example.rs".to_string();
        content_viewer.mode = ShowMode::WithBlame;
        content_viewer.content = "abc123f | use std::io;\nabc123f | \n456def9 | fn main() -> Result<(), Box<dyn std::error::Error>> {\n456def9 |     let input = std::io::stdin();\n789ghi2 |     println!(\"Input received\");\n789ghi2 |     Ok(())\nabc123f | }".to_string();

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let rect = ratatui::layout::Rect::new(0, 0, 80, 24);
                content_viewer.draw(frame, rect);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        assert_snapshot!(format!("{:?}", buffer));
    }

    #[test]
    fn test_content_viewer_draw_scrolled() {
        let mock_repo = create_mock_repo();
        let mut content_viewer = ContentViewer::new(mock_repo);
        content_viewer.focus = Focus::ON;
        content_viewer.title = "scrolled.rs".to_string();
        content_viewer.scroll_position = 3;
        content_viewer.content = "Line 1\nLine 2\nLine 3\nLine 4\nLine 5\nLine 6\nLine 7\nLine 8\nLine 9\nLine 10\nLine 11\nLine 12\nLine 13\nLine 14\nLine 15\nLine 16\nLine 17\nLine 18\nLine 19\nLine 20".to_string();

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let rect = ratatui::layout::Rect::new(0, 0, 80, 24);
                content_viewer.draw(frame, rect);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        assert_snapshot!(format!("{:?}", buffer));
    }

    #[test]
    fn test_content_viewer_line_number_alignment() {
        let mock_repo = create_mock_repo();
        let mut content_viewer = ContentViewer::new(mock_repo);
        content_viewer.focus = Focus::ON;
        content_viewer.title = "alignment_test.rs".to_string();
        content_viewer.mode = ShowMode::WithLine;

        // Test with line numbers up to 120 to verify proper alignment
        let mut content_lines = Vec::new();
        for i in 1..=120 {
            content_lines.push(format!("{:3} | Line {}", i, i));
        }
        content_viewer.content = content_lines.join("\n");

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let rect = ratatui::layout::Rect::new(0, 0, 80, 24);
                content_viewer.draw(frame, rect);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        assert_snapshot!(format!("{:?}", buffer));
    }

    #[test]
    fn test_key_bindings_navigation() {
        let mock_repo = create_mock_repo();
        let mut content_viewer = ContentViewer::new(mock_repo);
        content_viewer.content = "Line 1\nLine 2\nLine 3\nLine 4\nLine 5".to_string();
        content_viewer.context_size = 10;
        content_viewer.height = 5;

        // Test j (down) key
        let message = content_viewer.process_events(KeyCode::Char('j'));
        assert_eq!(message, Message::NoAction);
        assert_eq!(content_viewer.scroll_position, 1);

        // Test k (up) key
        let message = content_viewer.process_events(KeyCode::Char('k'));
        assert_eq!(message, Message::NoAction);
        assert_eq!(content_viewer.scroll_position, 0);

        // Test Down arrow
        let message = content_viewer.process_events(KeyCode::Down);
        assert_eq!(message, Message::NoAction);
        assert_eq!(content_viewer.scroll_position, 1);

        // Test Up arrow
        let message = content_viewer.process_events(KeyCode::Up);
        assert_eq!(message, Message::NoAction);
        assert_eq!(content_viewer.scroll_position, 0);
    }

    #[test]
    fn test_horizontal_scrolling() {
        let mock_repo = create_mock_repo();
        let mut content_viewer = ContentViewer::new(mock_repo);
        content_viewer.content =
            "This is a very long line that should be scrollable horizontally".to_string();

        // Initial state
        assert_eq!(content_viewer.horizontal_scroll, 0);

        // Test l (right) key
        let message = content_viewer.process_events(KeyCode::Char('l'));
        assert_eq!(message, Message::NoAction);
        assert_eq!(content_viewer.horizontal_scroll, 1);

        // Test h (left) key
        let message = content_viewer.process_events(KeyCode::Char('h'));
        assert_eq!(message, Message::NoAction);
        assert_eq!(content_viewer.horizontal_scroll, 0);

        // Test h when already at 0 (should stay at 0)
        let message = content_viewer.process_events(KeyCode::Char('h'));
        assert_eq!(message, Message::NoAction);
        assert_eq!(content_viewer.horizontal_scroll, 0);

        // Test Right arrow
        let message = content_viewer.process_events(KeyCode::Right);
        assert_eq!(message, Message::NoAction);
        assert_eq!(content_viewer.horizontal_scroll, 1);

        // Test Left arrow
        let message = content_viewer.process_events(KeyCode::Left);
        assert_eq!(message, Message::NoAction);
        assert_eq!(content_viewer.horizontal_scroll, 0);
    }

    #[test]
    fn test_toggle_line_numbers() {
        let mock_repo = create_mock_repo();
        let mut content_viewer = ContentViewer::new(mock_repo);
        content_viewer.title = "test.rs".to_string();

        // Initial mode should be WithLine
        assert!(matches!(content_viewer.mode, ShowMode::WithLine));

        // Toggle to NoLine
        content_viewer.toggle_line_numbers();
        assert!(matches!(content_viewer.mode, ShowMode::NoLine));

        // Toggle back to WithLine
        content_viewer.toggle_line_numbers();
        assert!(matches!(content_viewer.mode, ShowMode::WithLine));

        // Test from blame mode
        content_viewer.mode = ShowMode::WithBlame;
        content_viewer.toggle_line_numbers();
        assert!(matches!(content_viewer.mode, ShowMode::WithLine));
    }

    #[test]
    fn test_toggle_blame_mode() {
        let mock_repo = create_mock_repo();
        let mut content_viewer = ContentViewer::new(mock_repo);
        content_viewer.title = "test.rs".to_string();

        // Initial mode should be WithLine
        assert!(matches!(content_viewer.mode, ShowMode::WithLine));

        // Toggle to WithBlame
        content_viewer.toggle_blame_mode();
        assert!(matches!(content_viewer.mode, ShowMode::WithBlame));

        // Toggle to NoLine
        content_viewer.toggle_blame_mode();
        assert!(matches!(content_viewer.mode, ShowMode::NoLine));

        // Toggle back to WithBlame
        content_viewer.toggle_blame_mode();
        assert!(matches!(content_viewer.mode, ShowMode::WithBlame));
    }

    #[test]
    fn test_key_bindings_mode_toggle() {
        let mock_repo = create_mock_repo();
        let mut content_viewer = ContentViewer::new(mock_repo);
        content_viewer.title = "test.rs".to_string();

        // Test 'n' key for line number toggle
        let message = content_viewer.process_events(KeyCode::Char('n'));
        assert_eq!(message, Message::NoAction);

        // Test 'b' key for blame mode toggle
        let message = content_viewer.process_events(KeyCode::Char('b'));
        assert_eq!(message, Message::NoAction);
    }

    #[test]
    fn test_horizontal_scroll_with_content_display() {
        let mock_repo = create_mock_repo();
        let mut content_viewer = ContentViewer::new(mock_repo);
        content_viewer.focus = Focus::ON;
        content_viewer.title = "horizontal_test.rs".to_string();
        content_viewer.content =
            "This is a very long line that needs horizontal scrolling to view completely"
                .to_string();
        content_viewer.horizontal_scroll = 10;

        let backend = TestBackend::new(40, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let rect = ratatui::layout::Rect::new(0, 0, 40, 10);
                content_viewer.draw(frame, rect);
            })
            .unwrap();

        // The content should be horizontally scrolled
        let buffer = terminal.backend().buffer();
        let first_line: String = buffer
            .content()
            .iter()
            .take(40)
            .map(|cell| cell.symbol().chars().next().unwrap_or(' '))
            .collect();
        // Should not contain "This is" at the beginning since it's scrolled
        assert!(!first_line.trim_start().starts_with("This is"));
    }

    #[test]
    fn test_show_mode_concat_functionality() {
        use git2::Oid;
        let oid1 = Oid::from_str("abc123456789abcd1234567890abcdef12345678").unwrap();
        let oid2 = Oid::from_str("def456789012cdef1234567890abcdef12345678").unwrap();

        let commit_rows = vec![
            crate::repository::CommitRow {
                _author: "Test Author".to_string(),
                number: 1,
                line: "fn main() {".to_string(),
                commit: oid1,
            },
            crate::repository::CommitRow {
                _author: "Test Author".to_string(),
                number: 2,
                line: "    println!(\"Hello\");".to_string(),
                commit: oid2,
            },
        ];

        // Test NoLine mode
        let mut mode = ShowMode::NoLine;
        let result = mode.concat(commit_rows.clone());
        assert_eq!(result, "fn main() {\n    println!(\"Hello\");");

        // Test WithLine mode
        let mut mode = ShowMode::WithLine;
        let result = mode.concat(commit_rows.clone());
        assert!(result.contains("1 | fn main() { "));
        assert!(result.contains("2 |     println!(\"Hello\"); "));

        // Test WithBlame mode
        let mut mode = ShowMode::WithBlame;
        let result = mode.concat(commit_rows);
        assert!(result.contains("abc123456789abcd1234567890abcdef12345678 | fn main() { "));
        assert!(
            result.contains("def456789012cdef1234567890abcdef12345678 |     println!(\"Hello\"); ")
        );
    }

    #[test]
    fn test_scroll_boundary_conditions() {
        let mock_repo = create_mock_repo();
        let mut content_viewer = ContentViewer::new(mock_repo);
        content_viewer.content = "Line 1\nLine 2\nLine 3".to_string();
        content_viewer.context_size = 3;
        content_viewer.height = 2;

        // Test vertical scroll up at boundary
        content_viewer.scroll_position = 0;
        let _message = content_viewer.process_events(KeyCode::Char('k'));
        assert_eq!(content_viewer.scroll_position, 0); // Should stay at 0

        // Test vertical scroll down within bounds
        let _message = content_viewer.process_events(KeyCode::Char('j'));
        assert!(content_viewer.scroll_position > 0);

        // Test horizontal scroll left at boundary
        content_viewer.horizontal_scroll = 0;
        let _message = content_viewer.process_events(KeyCode::Char('h'));
        assert_eq!(content_viewer.horizontal_scroll, 0); // Should stay at 0

        // Test horizontal scroll right
        let _message = content_viewer.process_events(KeyCode::Char('l'));
        assert_eq!(content_viewer.horizontal_scroll, 1);
    }
}
