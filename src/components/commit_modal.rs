use std::sync::{Arc, Mutex};

use crossterm::event::KeyCode;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

use crate::repository::RepositoryInfo;

use super::operatable_components::{Focus, Message, OnceOperation, OperatableComponent};

pub struct CommitModal {
    focus: Focus,
    is_open: bool,
    commits: Vec<(String, String)>,
    list_state: ListState,
    repository: Arc<Mutex<RepositoryInfo>>,
}

impl CommitModal {
    pub fn new(repository: Arc<Mutex<RepositoryInfo>>) -> Self {
        Self {
            focus: Focus::Off,
            is_open: false,
            commits: Vec::new(),
            list_state: ListState::default(),
            repository,
        }
    }

    pub fn is_open(&self) -> bool {
        self.is_open
    }

    fn load_commits(&mut self) {
        if let Ok(mut repo) = self.repository.lock() {
            if let Ok(history) = repo.get_commit_history() {
                let current_commit_id = repo.get_current_commit_id();
                self.commits = history;

                // Find the current commit position and select it
                let current_position = self
                    .commits
                    .iter()
                    .position(|(id, _)| id == &current_commit_id)
                    .unwrap_or(0);

                if !self.commits.is_empty() {
                    self.list_state.select(Some(current_position));
                }
            }
        }
    }

    fn open(&mut self) {
        self.is_open = true;
        self.focus = Focus::ON;
        self.load_commits();
    }

    fn close(&mut self) {
        self.is_open = false;
        self.focus = Focus::Off;
        self.list_state.select(None);
    }

    fn get_selected_commit_id(&self) -> Option<String> {
        if let Some(selected) = self.list_state.selected() {
            if selected < self.commits.len() {
                return Some(self.commits[selected].0.clone());
            }
        }
        None
    }

    fn _handle_message(&mut self, message: &Message) -> Message {
        match message {
            Message::Once(OnceOperation::OpenCommitModal) => {
                self.open();
            }
            Message::Once(OnceOperation::CloseCommitModal) => {
                self.close();
            }
            _ => {}
        }
        Message::NoAction
    }
}

impl OperatableComponent for CommitModal {
    fn draw(&mut self, frame: &mut Frame, _rect: Rect) {
        if !self.is_open {
            return;
        }

        let area = frame.size();
        let popup_area = centered_rect(80, 80, area);

        frame.render_widget(Clear, popup_area);

        let block = Block::bordered()
            .title("All Commit History (Press Enter to select, Esc to cancel)")
            .style(match self.focus {
                Focus::ON => Style::default(),
                Focus::Off => Style::default().fg(Color::DarkGray),
            });

        let inner_area = block.inner(popup_area);
        frame.render_widget(block, popup_area);

        if self.commits.is_empty() {
            let empty_msg = Paragraph::new("No commits found")
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true });
            frame.render_widget(empty_msg, inner_area);
            return;
        }

        let items: Vec<ListItem> = self
            .commits
            .iter()
            .map(|(id, message)| {
                let short_id = &id[..std::cmp::min(8, id.len())];
                let content = Line::from(vec![
                    Span::styled(short_id, Style::default().fg(Color::Yellow)),
                    Span::raw(" "),
                    Span::raw(message),
                ]);
                ListItem::new(content)
            })
            .collect();

        let list = List::new(items)
            .highlight_style(Style::default().bg(Color::Blue))
            .highlight_symbol("→ ");

        frame.render_stateful_widget(list, inner_area, &mut self.list_state);
    }

    fn process_focus(&mut self) {
        match self.focus {
            Focus::Off => self.focus = Focus::ON,
            Focus::ON => self.focus = Focus::Off,
        }
    }

    fn process_events(&mut self, events: KeyCode) -> Message {
        if !self.is_open {
            return Message::NoAction;
        }

        match events {
            KeyCode::Esc => {
                return Message::Once(OnceOperation::CloseCommitModal);
            }
            KeyCode::Enter => {
                if let Some(commit_id) = self.get_selected_commit_id() {
                    return Message::Once(OnceOperation::SetCommitById { commit_id });
                }
            }
            KeyCode::Up => {
                let selected = self.list_state.selected().unwrap_or(0);
                if selected > 0 {
                    self.list_state.select(Some(selected - 1));
                }
            }
            KeyCode::Down => {
                let selected = self.list_state.selected().unwrap_or(0);
                if selected < self.commits.len().saturating_sub(1) {
                    self.list_state.select(Some(selected + 1));
                }
            }
            _ => {}
        }
        Message::NoAction
    }

    fn handle_message(&mut self, message: &Message) -> Message {
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

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
