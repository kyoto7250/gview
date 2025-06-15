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
}
