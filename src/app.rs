use crate::{
    components::{
        commit_modal::CommitModal,
        commit_viewer::CommitViewer,
        content_viewer::ContentViewer,
        filer::Filer,
        filter::Filter,
        help_modal::HelpModal,
        operatable_components::{
            Message, MultipleTimesOperation, OnceOperation, OperatableComponent,
        },
    },
    repository::RepositoryInfo,
};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Flex, Layout},
    terminal::Terminal,
    Frame,
};
use std::{
    io::{self, Stdout},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

// A simple alias for the terminal type used in this example.
pub type Tui = Terminal<CrosstermBackend<Stdout>>;

#[derive(Clone, Copy, Debug, PartialEq)]
enum FocusState {
    Filter,
    Filer,
    Commit,
    Viewer,
}

impl FocusState {
    fn next(self) -> FocusState {
        match self {
            FocusState::Filter => FocusState::Filer,
            FocusState::Filer => FocusState::Commit,
            FocusState::Commit => FocusState::Viewer,
            FocusState::Viewer => FocusState::Filter,
        }
    }
}

pub struct App {
    left_main_chunk_percentage: u16,
    should_exit: bool,
    last_tick: Instant,
    focus_state: FocusState,
    filter: Filter,
    filer: Filer,
    commit_viewer: CommitViewer,
    content_viewer: ContentViewer,
    commit_modal: CommitModal,
    help_modal: HelpModal,
}

impl App {
    const TICK_RATE: Duration = Duration::from_millis(50);

    pub fn new(repository_info: RepositoryInfo) -> App {
        let repository = Arc::new(Mutex::new(repository_info));
        let mut app = Self {
            left_main_chunk_percentage: 15,
            should_exit: false,
            last_tick: Instant::now(),
            focus_state: FocusState::Filter,
            filter: Filter::new(),
            filer: Filer::new(Arc::clone(&repository)),
            commit_viewer: CommitViewer::new(Arc::clone(&repository)),
            content_viewer: ContentViewer::new(Arc::clone(&repository)),
            commit_modal: CommitModal::new(Arc::clone(&repository)),
            help_modal: HelpModal::new(),
        };
        app.handle_message(Message::MultipleTimes(MultipleTimesOperation::SetUp {
            repository: Arc::clone(&repository),
        }));
        app
    }

    fn process_focus(&mut self) {
        match self.focus_state {
            FocusState::Commit => self.commit_viewer.process_focus(),
            FocusState::Filter => self.filter.process_focus(),
            FocusState::Filer => self.filer.process_focus(),
            FocusState::Viewer => self.content_viewer.process_focus(),
        }
    }

    fn process_events(&mut self, code: KeyCode) -> Message {
        // If help modal is open, handle help modal events first
        if self.help_modal.is_open() {
            return self.help_modal.process_events(code);
        }

        // If commit modal is open, handle commit modal events next
        if self.commit_modal.is_open() {
            return self.commit_modal.process_events(code);
        }

        match self.focus_state {
            FocusState::Commit => self.commit_viewer.process_events(code),
            FocusState::Filter => self.filter.process_events(code),
            FocusState::Filer => self.filer.process_events(code),
            FocusState::Viewer => self.content_viewer.process_events(code),
        }
    }

    #[allow(unconditional_recursion)]
    fn handle_message(&mut self, message: Message) {
        // handle itself
        match &message {
            Message::NoAction => return,
            Message::Once(OnceOperation::JumpToContentView) => {
                self.focus_state = FocusState::Viewer
            }
            Message::Once(OnceOperation::JumpToFiler) => self.focus_state = FocusState::Filer,
            Message::Once(OnceOperation::SetCommitById { commit_id }) => {
                // Close modal and set commit
                let commit_id = commit_id.clone();
                let success = {
                    if let Ok(mut repo) = self.commit_viewer.repository.lock() {
                        repo.set_commit_by_id(&commit_id).is_ok()
                    } else {
                        false
                    }
                };

                self.handle_message(Message::Once(OnceOperation::CloseCommitModal));
                if success {
                    self.handle_message(Message::MultipleTimes(
                        MultipleTimesOperation::ChangeShowCommit,
                    ));
                }
                return; // Early return to avoid processing this message further
            }
            _ => {}
        }

        let new_message = self.filer.handle_message(&message);
        self.handle_message(new_message);

        let new_message = self.filter.handle_message(&message);
        self.handle_message(new_message);

        let new_message = self.content_viewer.handle_message(&message);
        self.handle_message(new_message);

        let new_message = self.commit_viewer.handle_message(&message);
        self.handle_message(new_message);

        let new_message = self.commit_modal.handle_message(&message);
        self.handle_message(new_message);

        let new_message = self.help_modal.handle_message(&message);
        self.handle_message(new_message);
    }

    pub fn run(&mut self, terminal: &mut Tui) -> io::Result<()> {
        while !self.should_exit {
            terminal.draw(|frame| {
                let _ = self.draw(frame);
            })?;
            self.handle_events()?;
            if self.last_tick.elapsed() >= Self::TICK_RATE {
                self.last_tick = Instant::now();
            }
        }
        Ok(())
    }

    fn handle_events(&mut self) -> io::Result<()> {
        let timeout = Self::TICK_RATE.saturating_sub(self.last_tick.elapsed());
        while event::poll(timeout)? {
            if let Event::Key(event) = event::read()? {
                if event.kind == KeyEventKind::Press {
                    match event {
                        event::KeyEvent {
                            code: event::KeyCode::Tab,
                            ..
                        } => {
                            self.process_focus();
                            self.focus_state = self.focus_state.next();
                            self.process_focus();
                        }
                        event::KeyEvent {
                            code: event::KeyCode::Char('c'),
                            modifiers: event::KeyModifiers::CONTROL,
                            ..
                        } => self.should_exit = true,
                        event::KeyEvent {
                            code: event::KeyCode::Char('<'),
                            ..
                        } => {
                            self.left_main_chunk_percentage =
                                self.left_main_chunk_percentage.saturating_sub(5).max(15);
                        }
                        event::KeyEvent {
                            code: event::KeyCode::Char('>'),
                            ..
                        } => {
                            self.left_main_chunk_percentage =
                                (self.left_main_chunk_percentage + 5).min(70);
                        }
                        event::KeyEvent {
                            code: event::KeyCode::Char('?'),
                            ..
                        } => self.handle_message(Message::Once(OnceOperation::ShowHelpModal)),
                        _ => {
                            let message = self.process_events(event.code);
                            self.handle_message(message)
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub fn draw(&mut self, frame: &mut Frame) -> anyhow::Result<()> {
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(self.left_main_chunk_percentage),
                Constraint::Percentage((100_u16).saturating_sub(self.left_main_chunk_percentage)),
            ])
            .split(frame.size());

        let left_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(1)].as_ref())
            .split(main_chunks[0]);

        // chunks[0], chunks[1]
        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .flex(Flex::Legacy)
            .constraints([Constraint::Length(3), Constraint::Length(5)].as_ref())
            .split(main_chunks[1]);

        self.filter.draw(frame, left_chunks[0]);
        self.filer.draw(frame, left_chunks[1]);
        self.commit_viewer.draw(frame, right_chunks[0]);
        self.content_viewer.draw(frame, right_chunks[1]);

        // Draw modals on top if they're open
        self.commit_modal.draw(frame, frame.size());
        self.help_modal.draw(frame, frame.size());

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_focus_state_next_transitions() {
        assert_eq!(FocusState::Filter.next(), FocusState::Filer);
        assert_eq!(FocusState::Filer.next(), FocusState::Commit);
        assert_eq!(FocusState::Commit.next(), FocusState::Viewer);
        assert_eq!(FocusState::Viewer.next(), FocusState::Filter);
    }

    #[test]
    fn test_focus_state_cycle_complete() {
        let mut state = FocusState::Filter;

        // Test complete cycle
        state = state.next();
        assert_eq!(state, FocusState::Filer);

        state = state.next();
        assert_eq!(state, FocusState::Commit);

        state = state.next();
        assert_eq!(state, FocusState::Viewer);

        state = state.next();
        assert_eq!(state, FocusState::Filter); // Back to start
    }

    #[test]
    fn test_app_draw_normal_state() {
        use crate::repository::RepositoryInfo;
        use insta::assert_snapshot;
        use ratatui::{backend::TestBackend, Terminal};
        use std::env;
        use std::time::{SystemTime, UNIX_EPOCH};

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let random_suffix = std::process::id();
        let test_dir =
            env::temp_dir().join(format!("gview_app_test_{}_{}", timestamp, random_suffix));
        let _ = std::fs::remove_dir_all(&test_dir);
        std::fs::create_dir_all(&test_dir).unwrap();

        let repo = git2::Repository::init(&test_dir).unwrap();

        // Create a test file
        let test_file_path = test_dir.join("test.txt");
        std::fs::write(&test_file_path, "Hello, world!").unwrap();

        let signature = git2::Signature::new(
            "Test User",
            "test@localhost",
            &git2::Time::new(1234567890, 0),
        )
        .unwrap();
        let tree_id = {
            let mut index = repo.index().unwrap();
            index.add_path(std::path::Path::new("test.txt")).unwrap();
            index.write().unwrap();
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

        let mut app = App::new(repo_info);

        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let _ = app.draw(frame);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        assert_snapshot!(format!("{:?}", buffer));
    }

    #[test]
    fn test_app_draw_with_help_modal_open() {
        use crate::repository::RepositoryInfo;
        use insta::assert_snapshot;
        use ratatui::{backend::TestBackend, Terminal};
        use std::env;
        use std::time::{SystemTime, UNIX_EPOCH};

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let random_suffix = std::process::id();
        let test_dir = env::temp_dir().join(format!(
            "gview_app_help_test_{}_{}",
            timestamp, random_suffix
        ));
        let _ = std::fs::remove_dir_all(&test_dir);
        std::fs::create_dir_all(&test_dir).unwrap();

        let repo = git2::Repository::init(&test_dir).unwrap();

        // Create a test file
        let test_file_path = test_dir.join("test.txt");
        std::fs::write(&test_file_path, "Hello, world!").unwrap();

        let signature = git2::Signature::new(
            "Test User",
            "test@localhost",
            &git2::Time::new(1234567890, 0),
        )
        .unwrap();
        let tree_id = {
            let mut index = repo.index().unwrap();
            index.add_path(std::path::Path::new("test.txt")).unwrap();
            index.write().unwrap();
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

        let mut app = App::new(repo_info);

        // Open help modal
        app.handle_message(Message::Once(OnceOperation::ShowHelpModal));

        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let _ = app.draw(frame);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        assert_snapshot!(format!("{:?}", buffer));
    }

    #[test]
    fn test_app_draw_help_modal_large_terminal() {
        use crate::repository::RepositoryInfo;
        use insta::assert_snapshot;
        use ratatui::{backend::TestBackend, Terminal};
        use std::env;
        use std::time::{SystemTime, UNIX_EPOCH};

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let random_suffix = std::process::id();
        let test_dir = env::temp_dir().join(format!(
            "gview_app_large_test_{}_{}",
            timestamp, random_suffix
        ));
        let _ = std::fs::remove_dir_all(&test_dir);
        std::fs::create_dir_all(&test_dir).unwrap();

        let repo = git2::Repository::init(&test_dir).unwrap();

        // Create a test file
        let test_file_path = test_dir.join("test.txt");
        std::fs::write(&test_file_path, "Hello, world!").unwrap();

        let signature = git2::Signature::new(
            "Test User",
            "test@localhost",
            &git2::Time::new(1234567890, 0),
        )
        .unwrap();
        let tree_id = {
            let mut index = repo.index().unwrap();
            index.add_path(std::path::Path::new("test.txt")).unwrap();
            index.write().unwrap();
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

        let mut app = App::new(repo_info);

        // Open help modal
        app.handle_message(Message::Once(OnceOperation::ShowHelpModal));

        let backend = TestBackend::new(150, 50);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let _ = app.draw(frame);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        assert_snapshot!(format!("{:?}", buffer));
    }

    #[test]
    fn test_app_draw_help_modal_small_terminal() {
        use crate::repository::RepositoryInfo;
        use insta::assert_snapshot;
        use ratatui::{backend::TestBackend, Terminal};
        use std::env;
        use std::time::{SystemTime, UNIX_EPOCH};

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let random_suffix = std::process::id();
        let test_dir = env::temp_dir().join(format!(
            "gview_app_small_test_{}_{}",
            timestamp, random_suffix
        ));
        let _ = std::fs::remove_dir_all(&test_dir);
        std::fs::create_dir_all(&test_dir).unwrap();

        let repo = git2::Repository::init(&test_dir).unwrap();

        // Create a test file
        let test_file_path = test_dir.join("test.txt");
        std::fs::write(&test_file_path, "Hello, world!").unwrap();

        let signature = git2::Signature::new(
            "Test User",
            "test@localhost",
            &git2::Time::new(1234567890, 0),
        )
        .unwrap();
        let tree_id = {
            let mut index = repo.index().unwrap();
            index.add_path(std::path::Path::new("test.txt")).unwrap();
            index.write().unwrap();
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

        let mut app = App::new(repo_info);

        // Open help modal
        app.handle_message(Message::Once(OnceOperation::ShowHelpModal));

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let _ = app.draw(frame);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        assert_snapshot!(format!("{:?}", buffer));
    }
}
