use crate::{
    components::{
        commit_viewer::CommitViewer,
        content_viewer::ContentViewer,
        explorer::Explorer,
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
    iter::Once,
    ops::Mul,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

// A simple alias for the terminal type used in this example.
pub type Tui = Terminal<CrosstermBackend<Stdout>>;

#[derive(Clone, Copy, Debug)]
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
    repository_info: Arc<Mutex<RepositoryInfo>>,
    left_main_chunk_percentage: u16,
    should_exit: bool,
    last_tick: Instant,
    focus_state: FocusState,
    explorer: Explorer,
    commit_viewer: CommitViewer,
    content_viewer: ContentViewer,
}

impl App {
    const TICK_RATE: Duration = Duration::from_millis(50);

    pub fn new(repository_info: RepositoryInfo) -> App {
        let repository = Arc::new(Mutex::new(repository_info));
        let mut app = Self {
            repository_info: Arc::clone(&repository),
            left_main_chunk_percentage: 15,
            should_exit: false,
            last_tick: Instant::now(),
            focus_state: FocusState::Filter,
            explorer: Explorer::new(),
            commit_viewer: CommitViewer::new(),
            content_viewer: ContentViewer::new(Arc::clone(&repository)),
        };
        app.handle_message(Message::MultipleTimes(MultipleTimesOperation::SetUp {
            repository: Arc::clone(&repository),
        }));
        app
    }

    fn process_focus(&mut self) {
        match self.focus_state {
            FocusState::Commit => self.commit_viewer.process_focus(),
            FocusState::Filter => self.explorer.filter.process_focus(),
            FocusState::Filer => self.explorer.filer.process_focus(),
            FocusState::Viewer => self.content_viewer.process_focus(),
        }
    }

    fn process_events(&mut self, code: KeyCode) -> Message {
        match self.focus_state {
            FocusState::Commit => self.commit_viewer.process_events(code),
            FocusState::Filter => self.explorer.filter.process_events(code),
            FocusState::Filer => self.explorer.filer.process_events(code),
            FocusState::Viewer => self.content_viewer.process_events(code),
        }
    }

    #[allow(unconditional_recursion)]
    fn handle_message(&mut self, message: Message) {
        // handle itself
        match message {
            Message::NoAction => return,
            Message::Once(OnceOperation::JumpToContentView) => {
                self.focus_state = FocusState::Viewer
            }
            Message::Once(OnceOperation::JumpToFiler) => self.focus_state = FocusState::Filer,
            _ => {}
        }

        let new_message = self.explorer.filer.handle_message(&message);
        self.handle_message(new_message);

        let new_message = self.explorer.filter.handle_message(&message);
        self.handle_message(new_message);

        let new_message = self.content_viewer.handle_message(&message);
        self.handle_message(new_message);

        let new_message = self.commit_viewer.handle_message(&message);
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
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Tab => {
                            self.process_focus();
                            self.focus_state = self.focus_state.next();
                            self.process_focus();
                        }
                        KeyCode::Char('q') => self.should_exit = true,
                        KeyCode::Char('<') => {
                            self.left_main_chunk_percentage =
                                self.left_main_chunk_percentage.saturating_sub(5).max(15);
                        }
                        KeyCode::Char('>') => {
                            self.left_main_chunk_percentage =
                                (self.left_main_chunk_percentage + 5).min(70);
                        }
                        _ => {
                            let message = self.process_events(key.code);
                            self.handle_message(message)
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub fn draw(&mut self, frame: &mut Frame) -> anyhow::Result<()> {
        let left_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(self.left_main_chunk_percentage),
                Constraint::Percentage((100_u16).saturating_sub(self.left_main_chunk_percentage)),
            ])
            .split(frame.size());

        // chunks[0], chunks[1]
        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .flex(Flex::Legacy)
            .constraints([Constraint::Length(3), Constraint::Length(5)].as_ref())
            .split(left_chunks[1]);

        let _ = self.explorer.draw(frame, left_chunks[0]);
        self.commit_viewer.draw(frame, right_chunks[0]);
        self.content_viewer.draw(frame, right_chunks[1]);
        Ok(())
    }
}
