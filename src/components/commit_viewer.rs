use std::sync::{Arc, Mutex};

use crossterm::event::KeyCode;
use ratatui::{
    layout::Rect,
    style::{Color, Style, Stylize},
    widgets::{Block, Paragraph},
    Frame,
};

use crate::repository::RepositoryInfo;

use super::operatable_components::{
    Focus, Message, MultipleTimesOperation, OnceOperation, OperatableComponent,
};

pub struct CommitViewer {
    focus: Focus,
    content: String,
    pub repository: Arc<Mutex<RepositoryInfo>>,
}

impl CommitViewer {
    pub fn new(repository: Arc<Mutex<RepositoryInfo>>) -> Self {
        Self {
            focus: Focus::Off,
            content: "".to_owned(),
            repository,
        }
    }

    fn _handle_message(&mut self, message: &Message) -> Message {
        match message {
            Message::MultipleTimes(MultipleTimesOperation::SetUp { repository }) => {
                let mut repository = repository.lock().unwrap();
                let (commit_id, commit_message) = repository.current_commit().unwrap();
                self.content = format!("{}: {}", commit_id, commit_message);
            }
            Message::MultipleTimes(MultipleTimesOperation::ChangeShowCommit) => {
                let mut repository = self.repository.lock().unwrap();
                let (commit_id, commit_message) = repository.current_commit().unwrap();
                self.content = format!("{}: {}", commit_id, commit_message);
            }
            _ => {}
        }
        Message::NoAction
    }
}

impl OperatableComponent for CommitViewer {
    fn draw(&mut self, frame: &mut Frame, rect: Rect) {
        let right_paragraph = Paragraph::new(self.content.to_owned())
            .block(title_block("current commit (g: go to commit)", self.focus));
        frame.render_widget(right_paragraph, rect);
    }
    fn process_focus(&mut self) {
        match self.focus {
            Focus::Off => self.focus = Focus::ON,
            Focus::ON => self.focus = Focus::Off,
        }
    }
    fn process_events(&mut self, events: KeyCode) -> Message {
        match events {
            KeyCode::Down => {
                let mut binding = self.repository.lock().unwrap();
                binding.set_parent_commit();
                return Message::MultipleTimes(MultipleTimesOperation::ChangeShowCommit);
            }
            KeyCode::Up => {
                let mut binding = self.repository.lock().unwrap();
                let _ = binding.set_next_commit();
                return Message::MultipleTimes(MultipleTimesOperation::ChangeShowCommit);
            }
            KeyCode::Char('g') => {
                return Message::Once(OnceOperation::OpenCommitModal);
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
            "gview_commit_viewer_test_{}_{}",
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
    fn test_commit_viewer_draw_empty() {
        let mock_repo = create_mock_repo();
        let mut commit_viewer = CommitViewer::new(mock_repo);
        commit_viewer.focus = Focus::ON;
        commit_viewer.content = "".to_string();

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let rect = ratatui::layout::Rect::new(0, 0, 80, 24);
                commit_viewer.draw(frame, rect);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        assert_snapshot!(format!("{:?}", buffer));
    }

    #[test]
    fn test_commit_viewer_draw_with_commit() {
        let mock_repo = create_mock_repo();
        let mut commit_viewer = CommitViewer::new(mock_repo);
        commit_viewer.focus = Focus::ON;
        commit_viewer.content = "abc123def456: Initial commit message".to_string();

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let rect = ratatui::layout::Rect::new(0, 0, 80, 24);
                commit_viewer.draw(frame, rect);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        assert_snapshot!(format!("{:?}", buffer));
    }

    #[test]
    fn test_commit_viewer_draw_unfocused() {
        let mock_repo = create_mock_repo();
        let mut commit_viewer = CommitViewer::new(mock_repo);
        commit_viewer.focus = Focus::Off;
        commit_viewer.content = "def789ghi012: Add new feature implementation".to_string();

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let rect = ratatui::layout::Rect::new(0, 0, 80, 24);
                commit_viewer.draw(frame, rect);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        assert_snapshot!(format!("{:?}", buffer));
    }

    #[test]
    fn test_commit_viewer_draw_long_commit_message() {
        let mock_repo = create_mock_repo();
        let mut commit_viewer = CommitViewer::new(mock_repo);
        commit_viewer.focus = Focus::ON;
        commit_viewer.content = "abcdef123456: This is a very long commit message that should demonstrate how the commit viewer handles longer text content that might wrap or be truncated depending on the terminal width".to_string();

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let rect = ratatui::layout::Rect::new(0, 0, 80, 24);
                commit_viewer.draw(frame, rect);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        assert_snapshot!(format!("{:?}", buffer));
    }

    #[test]
    fn test_commit_viewer_draw_multiline_commit() {
        let mock_repo = create_mock_repo();
        let mut commit_viewer = CommitViewer::new(mock_repo);
        commit_viewer.focus = Focus::ON;
        commit_viewer.content = "commit123: Fix critical bug\n\nThis commit addresses a critical issue where the application\nwould crash under certain conditions. The fix includes:\n- Better error handling\n- Input validation\n- Memory management improvements".to_string();

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let rect = ratatui::layout::Rect::new(0, 0, 80, 24);
                commit_viewer.draw(frame, rect);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        assert_snapshot!(format!("{:?}", buffer));
    }
}
