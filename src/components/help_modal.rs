use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem},
    Frame,
};

use super::operatable_components::{Focus, Message, OnceOperation, OperatableComponent};

pub struct HelpModal {
    visible: bool,
    focus: Focus,
    scroll_offset: usize,
}

impl HelpModal {
    pub fn new() -> Self {
        Self {
            visible: false,
            focus: Focus::Off,
            scroll_offset: 0,
        }
    }

    pub fn is_open(&self) -> bool {
        self.visible
    }

    fn get_help_content() -> Vec<ListItem<'static>> {
        vec![
            ListItem::new(Line::from(vec![Span::styled(
                "Global Keys:",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )])),
            ListItem::new(Line::from("")),
            Self::create_key_line("Tab", "Switch focus between panels"),
            Self::create_key_line("Ctrl+C", "Exit gview"),
            Self::create_key_line("<", "Decrease left panel width"),
            Self::create_key_line(">", "Increase left panel width"),
            Self::create_key_line("?", "Show this help modal"),
            Self::create_key_line("ESC", "Close help modal"),
            ListItem::new(Line::from("")),
            ListItem::new(Line::from(vec![Span::styled(
                "Filter Panel:",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )])),
            ListItem::new(Line::from("")),
            Self::create_key_line("Enter", "Apply filter"),
            Self::create_key_line("Ctrl+A", "Select all text"),
            ListItem::new(Line::from("")),
            ListItem::new(Line::from(vec![Span::styled(
                "File List Panel:",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )])),
            ListItem::new(Line::from("")),
            Self::create_key_line("↑/↓, j/k", "Navigate files"),
            Self::create_key_line("Enter", "Select file"),
            ListItem::new(Line::from("")),
            ListItem::new(Line::from(vec![Span::styled(
                "Commit Panel:",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )])),
            ListItem::new(Line::from("")),
            Self::create_key_line("o", "Open commit modal"),
            ListItem::new(Line::from("")),
            ListItem::new(Line::from(vec![Span::styled(
                "Content Viewer:",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )])),
            ListItem::new(Line::from("")),
            Self::create_key_line("↑/↓, j/k", "Scroll content vertically"),
            Self::create_key_line("←/→, h/l", "Scroll content horizontally"),
            Self::create_key_line("b", "Toggle blame view"),
            Self::create_key_line("n", "Toggle line numbers"),
            Self::create_key_line("g", "Go to GitHub (if available)"),
            ListItem::new(Line::from("")),
            ListItem::new(Line::from("")),
            ListItem::new(Line::from(vec![
                Span::styled("Use ", Style::default().fg(Color::Gray)),
                Span::styled("↑/↓", Style::default().fg(Color::Yellow)),
                Span::styled(" to scroll • Press ", Style::default().fg(Color::Gray)),
                Span::styled("ESC", Style::default().fg(Color::Yellow)),
                Span::styled(" to close", Style::default().fg(Color::Gray)),
            ])),
        ]
    }

    fn create_key_line(key: &'static str, description: &'static str) -> ListItem<'static> {
        ListItem::new(Line::from(vec![
            Span::styled(format!("{:12}", key), Style::default().fg(Color::Yellow)),
            Span::raw("  "),
            Span::styled(description, Style::default().fg(Color::White)),
        ]))
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
}

impl OperatableComponent for HelpModal {
    fn draw(&mut self, frame: &mut Frame, rect: Rect) {
        if !self.visible {
            return;
        }

        let popup_area = Self::centered_rect(80, 80, rect);

        // Clear the background
        frame.render_widget(Clear, popup_area);

        let block = Block::default()
            .title(" Key Configuration Help ")
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .style(Style::default().fg(Color::White));

        let help_content = Self::get_help_content();

        // Calculate visible area height (subtract 2 for borders)
        let inner_height = popup_area.height.saturating_sub(2) as usize;

        // Apply scroll offset to the content
        let visible_content: Vec<ListItem> = help_content
            .into_iter()
            .skip(self.scroll_offset)
            .take(inner_height)
            .collect();

        let help_list = List::new(visible_content)
            .block(block)
            .style(Style::default().fg(Color::White));

        frame.render_widget(help_list, popup_area);
    }

    fn process_focus(&mut self) {
        // Help modal doesn't need focus handling as it's always focused when visible
    }

    fn process_events(&mut self, key_code: KeyCode) -> Message {
        if !self.visible {
            return Message::NoAction;
        }

        match key_code {
            KeyCode::Esc => Message::Once(OnceOperation::CloseHelpModal),
            KeyCode::Up => {
                self.scroll_offset = self.scroll_offset.saturating_sub(1);
                Message::NoAction
            }
            KeyCode::Down => {
                let help_content = Self::get_help_content();
                let max_scroll = help_content.len().saturating_sub(1);
                if self.scroll_offset < max_scroll {
                    self.scroll_offset += 1;
                }
                Message::NoAction
            }
            _ => Message::NoAction, // Consume all other events when help is open
        }
    }

    fn handle_message(&mut self, message: &Message) -> Message {
        match message {
            Message::Once(OnceOperation::ShowHelpModal) => {
                self.visible = true;
                self.focus = Focus::ON;
            }
            Message::Once(OnceOperation::CloseHelpModal) => {
                self.visible = false;
                self.focus = Focus::Off;
                self.scroll_offset = 0; // Reset scroll when closing
            }
            _ => {}
        }
        Message::NoAction
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_snapshot;
    use ratatui::{backend::TestBackend, Terminal};

    #[test]
    fn test_help_modal_initial_state() {
        let help_modal = HelpModal::new();
        assert!(!help_modal.is_open());
        assert_eq!(help_modal.focus, Focus::Off);
    }

    #[test]
    fn test_help_modal_show_and_hide() {
        let mut help_modal = HelpModal::new();

        // Initially closed
        assert!(!help_modal.is_open());

        // Show modal
        let message = Message::Once(OnceOperation::ShowHelpModal);
        help_modal.handle_message(&message);
        assert!(help_modal.is_open());
        assert_eq!(help_modal.focus, Focus::ON);

        // Hide modal
        let message = Message::Once(OnceOperation::CloseHelpModal);
        help_modal.handle_message(&message);
        assert!(!help_modal.is_open());
        assert_eq!(help_modal.focus, Focus::Off);
    }

    #[test]
    fn test_help_modal_key_events() {
        let mut help_modal = HelpModal::new();

        // When modal is closed, should return NoAction
        let message = help_modal.process_events(KeyCode::Esc);
        assert_eq!(message, Message::NoAction);

        // Open modal
        help_modal.handle_message(&Message::Once(OnceOperation::ShowHelpModal));

        // ESC should close modal
        let message = help_modal.process_events(KeyCode::Esc);
        assert_eq!(message, Message::Once(OnceOperation::CloseHelpModal));

        // Other keys should be consumed
        let message = help_modal.process_events(KeyCode::Enter);
        assert_eq!(message, Message::NoAction);

        let message = help_modal.process_events(KeyCode::Char('a'));
        assert_eq!(message, Message::NoAction);
    }

    #[test]
    fn test_help_modal_draw_closed() {
        let mut help_modal = HelpModal::new();
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let rect = ratatui::layout::Rect::new(0, 0, 80, 24);
                help_modal.draw(frame, rect);
            })
            .unwrap();

        // When closed, should render nothing significant
        let buffer = terminal.backend().buffer();
        // Most of the buffer should be empty/spaces
        let content_str = format!("{:?}", buffer);
        assert!(!content_str.contains("Key Configuration Help"));
    }

    #[test]
    fn test_help_modal_draw_open() {
        let mut help_modal = HelpModal::new();
        help_modal.handle_message(&Message::Once(OnceOperation::ShowHelpModal));

        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let rect = ratatui::layout::Rect::new(0, 0, 120, 40);
                help_modal.draw(frame, rect);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        assert_snapshot!(format!("{:?}", buffer));
    }

    #[test]
    fn test_help_modal_content() {
        let help_content = HelpModal::get_help_content();

        // Should have content
        assert!(!help_content.is_empty());

        // Should contain key sections
        let content_text = format!("{:?}", help_content);
        assert!(content_text.contains("Tab"));
        assert!(content_text.contains("Ctrl+C"));
        assert!(content_text.contains("Filter Panel"));
        assert!(content_text.contains("File List Panel"));
        assert!(content_text.contains("Commit Panel"));
        assert!(content_text.contains("Content Viewer"));
        assert!(content_text.contains("ESC"));
    }

    #[test]
    fn test_help_modal_centered_rect() {
        let full_rect = Rect::new(0, 0, 100, 50);
        let centered = HelpModal::centered_rect(80, 60, full_rect);

        // Should be centered
        assert_eq!(centered.x, 10); // (100 - 80) / 2
        assert_eq!(centered.y, 10); // (50 - 30) / 2  (60% of 50 = 30)
        assert_eq!(centered.width, 80);
        assert_eq!(centered.height, 30); // 60% of 50
    }

    #[test]
    fn test_help_modal_process_focus() {
        let mut help_modal = HelpModal::new();

        // Should not crash and should not change state
        let initial_focus = help_modal.focus;
        help_modal.process_focus();
        assert_eq!(help_modal.focus, initial_focus);
    }

    #[test]
    fn test_help_modal_draw_different_sizes() {
        let mut help_modal = HelpModal::new();
        help_modal.handle_message(&Message::Once(OnceOperation::ShowHelpModal));

        // Test with small terminal
        let backend = TestBackend::new(40, 20);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let rect = ratatui::layout::Rect::new(0, 0, 40, 20);
                help_modal.draw(frame, rect);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        assert_snapshot!(format!("{:?}", buffer), @r#"
        Buffer {
            area: Rect { x: 0, y: 0, width: 40, height: 20 },
            content: [
                "                                        ",
                "                                        ",
                "    ╔ Key Configuration Help ══════╗    ",
                "    ║Global Keys:                  ║    ",
                "    ║                              ║    ",
                "    ║Tab           Switch focus bet║    ",
                "    ║Ctrl+C        Exit gview      ║    ",
                "    ║<             Decrease left pa║    ",
                "    ║>             Increase left pa║    ",
                "    ║?             Show this help m║    ",
                "    ║ESC           Close help modal║    ",
                "    ║                              ║    ",
                "    ║Filter Panel:                 ║    ",
                "    ║                              ║    ",
                "    ║Enter         Apply filter    ║    ",
                "    ║Ctrl+A        Select all text ║    ",
                "    ║                              ║    ",
                "    ╚══════════════════════════════╝    ",
                "                                        ",
                "                                        ",
            ],
            styles: [
                x: 0, y: 0, fg: Reset, bg: Reset, underline: Reset, modifier: NONE,
                x: 4, y: 2, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 36, y: 2, fg: Reset, bg: Reset, underline: Reset, modifier: NONE,
                x: 4, y: 3, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 5, y: 3, fg: Cyan, bg: Reset, underline: Reset, modifier: BOLD,
                x: 17, y: 3, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 36, y: 3, fg: Reset, bg: Reset, underline: Reset, modifier: NONE,
                x: 4, y: 4, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 36, y: 4, fg: Reset, bg: Reset, underline: Reset, modifier: NONE,
                x: 4, y: 5, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 5, y: 5, fg: Yellow, bg: Reset, underline: Reset, modifier: NONE,
                x: 17, y: 5, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 36, y: 5, fg: Reset, bg: Reset, underline: Reset, modifier: NONE,
                x: 4, y: 6, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 5, y: 6, fg: Yellow, bg: Reset, underline: Reset, modifier: NONE,
                x: 17, y: 6, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 36, y: 6, fg: Reset, bg: Reset, underline: Reset, modifier: NONE,
                x: 4, y: 7, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 5, y: 7, fg: Yellow, bg: Reset, underline: Reset, modifier: NONE,
                x: 17, y: 7, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 36, y: 7, fg: Reset, bg: Reset, underline: Reset, modifier: NONE,
                x: 4, y: 8, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 5, y: 8, fg: Yellow, bg: Reset, underline: Reset, modifier: NONE,
                x: 17, y: 8, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 36, y: 8, fg: Reset, bg: Reset, underline: Reset, modifier: NONE,
                x: 4, y: 9, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 5, y: 9, fg: Yellow, bg: Reset, underline: Reset, modifier: NONE,
                x: 17, y: 9, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 36, y: 9, fg: Reset, bg: Reset, underline: Reset, modifier: NONE,
                x: 4, y: 10, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 5, y: 10, fg: Yellow, bg: Reset, underline: Reset, modifier: NONE,
                x: 17, y: 10, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 36, y: 10, fg: Reset, bg: Reset, underline: Reset, modifier: NONE,
                x: 4, y: 11, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 36, y: 11, fg: Reset, bg: Reset, underline: Reset, modifier: NONE,
                x: 4, y: 12, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 5, y: 12, fg: Cyan, bg: Reset, underline: Reset, modifier: BOLD,
                x: 18, y: 12, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 36, y: 12, fg: Reset, bg: Reset, underline: Reset, modifier: NONE,
                x: 4, y: 13, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 36, y: 13, fg: Reset, bg: Reset, underline: Reset, modifier: NONE,
                x: 4, y: 14, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 5, y: 14, fg: Yellow, bg: Reset, underline: Reset, modifier: NONE,
                x: 17, y: 14, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 36, y: 14, fg: Reset, bg: Reset, underline: Reset, modifier: NONE,
                x: 4, y: 15, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 5, y: 15, fg: Yellow, bg: Reset, underline: Reset, modifier: NONE,
                x: 17, y: 15, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 36, y: 15, fg: Reset, bg: Reset, underline: Reset, modifier: NONE,
                x: 4, y: 16, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 36, y: 16, fg: Reset, bg: Reset, underline: Reset, modifier: NONE,
                x: 4, y: 17, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 36, y: 17, fg: Reset, bg: Reset, underline: Reset, modifier: NONE,
            ]
        }
        "#);

        // Test with large terminal
        let backend = TestBackend::new(150, 50);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let rect = ratatui::layout::Rect::new(0, 0, 150, 50);
                help_modal.draw(frame, rect);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        assert_snapshot!(format!("{:?}", buffer), @r#"
        Buffer {
            area: Rect { x: 0, y: 0, width: 150, height: 50 },
            content: [
                "                                                                                                                                                      ",
                "                                                                                                                                                      ",
                "                                                                                                                                                      ",
                "                                                                                                                                                      ",
                "                                                                                                                                                      ",
                "               ╔ Key Configuration Help ══════════════════════════════════════════════════════════════════════════════════════════════╗               ",
                "               ║Global Keys:                                                                                                          ║               ",
                "               ║                                                                                                                      ║               ",
                "               ║Tab           Switch focus between panels                                                                             ║               ",
                "               ║Ctrl+C        Exit gview                                                                                              ║               ",
                "               ║<             Decrease left panel width                                                                               ║               ",
                "               ║>             Increase left panel width                                                                               ║               ",
                "               ║?             Show this help modal                                                                                    ║               ",
                "               ║ESC           Close help modal                                                                                        ║               ",
                "               ║                                                                                                                      ║               ",
                "               ║Filter Panel:                                                                                                         ║               ",
                "               ║                                                                                                                      ║               ",
                "               ║Enter         Apply filter                                                                                            ║               ",
                "               ║Ctrl+A        Select all text                                                                                         ║               ",
                "               ║                                                                                                                      ║               ",
                "               ║File List Panel:                                                                                                      ║               ",
                "               ║                                                                                                                      ║               ",
                "               ║↑/↓, j/k      Navigate files                                                                                          ║               ",
                "               ║Enter         Select file                                                                                             ║               ",
                "               ║                                                                                                                      ║               ",
                "               ║Commit Panel:                                                                                                         ║               ",
                "               ║                                                                                                                      ║               ",
                "               ║o             Open commit modal                                                                                       ║               ",
                "               ║                                                                                                                      ║               ",
                "               ║Content Viewer:                                                                                                       ║               ",
                "               ║                                                                                                                      ║               ",
                "               ║↑/↓, j/k      Scroll content vertically                                                                               ║               ",
                "               ║←/→, h/l      Scroll content horizontally                                                                             ║               ",
                "               ║b             Toggle blame view                                                                                       ║               ",
                "               ║n             Toggle line numbers                                                                                     ║               ",
                "               ║g             Go to GitHub (if available)                                                                             ║               ",
                "               ║                                                                                                                      ║               ",
                "               ║                                                                                                                      ║               ",
                "               ║Use ↑/↓ to scroll • Press ESC to close                                                                                ║               ",
                "               ║                                                                                                                      ║               ",
                "               ║                                                                                                                      ║               ",
                "               ║                                                                                                                      ║               ",
                "               ║                                                                                                                      ║               ",
                "               ║                                                                                                                      ║               ",
                "               ╚══════════════════════════════════════════════════════════════════════════════════════════════════════════════════════╝               ",
                "                                                                                                                                                      ",
                "                                                                                                                                                      ",
                "                                                                                                                                                      ",
                "                                                                                                                                                      ",
                "                                                                                                                                                      ",
            ],
            styles: [
                x: 0, y: 0, fg: Reset, bg: Reset, underline: Reset, modifier: NONE,
                x: 15, y: 5, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 135, y: 5, fg: Reset, bg: Reset, underline: Reset, modifier: NONE,
                x: 15, y: 6, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 16, y: 6, fg: Cyan, bg: Reset, underline: Reset, modifier: BOLD,
                x: 28, y: 6, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 135, y: 6, fg: Reset, bg: Reset, underline: Reset, modifier: NONE,
                x: 15, y: 7, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 135, y: 7, fg: Reset, bg: Reset, underline: Reset, modifier: NONE,
                x: 15, y: 8, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 16, y: 8, fg: Yellow, bg: Reset, underline: Reset, modifier: NONE,
                x: 28, y: 8, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 135, y: 8, fg: Reset, bg: Reset, underline: Reset, modifier: NONE,
                x: 15, y: 9, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 16, y: 9, fg: Yellow, bg: Reset, underline: Reset, modifier: NONE,
                x: 28, y: 9, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 135, y: 9, fg: Reset, bg: Reset, underline: Reset, modifier: NONE,
                x: 15, y: 10, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 16, y: 10, fg: Yellow, bg: Reset, underline: Reset, modifier: NONE,
                x: 28, y: 10, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 135, y: 10, fg: Reset, bg: Reset, underline: Reset, modifier: NONE,
                x: 15, y: 11, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 16, y: 11, fg: Yellow, bg: Reset, underline: Reset, modifier: NONE,
                x: 28, y: 11, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 135, y: 11, fg: Reset, bg: Reset, underline: Reset, modifier: NONE,
                x: 15, y: 12, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 16, y: 12, fg: Yellow, bg: Reset, underline: Reset, modifier: NONE,
                x: 28, y: 12, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 135, y: 12, fg: Reset, bg: Reset, underline: Reset, modifier: NONE,
                x: 15, y: 13, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 16, y: 13, fg: Yellow, bg: Reset, underline: Reset, modifier: NONE,
                x: 28, y: 13, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 135, y: 13, fg: Reset, bg: Reset, underline: Reset, modifier: NONE,
                x: 15, y: 14, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 135, y: 14, fg: Reset, bg: Reset, underline: Reset, modifier: NONE,
                x: 15, y: 15, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 16, y: 15, fg: Cyan, bg: Reset, underline: Reset, modifier: BOLD,
                x: 29, y: 15, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 135, y: 15, fg: Reset, bg: Reset, underline: Reset, modifier: NONE,
                x: 15, y: 16, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 135, y: 16, fg: Reset, bg: Reset, underline: Reset, modifier: NONE,
                x: 15, y: 17, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 16, y: 17, fg: Yellow, bg: Reset, underline: Reset, modifier: NONE,
                x: 28, y: 17, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 135, y: 17, fg: Reset, bg: Reset, underline: Reset, modifier: NONE,
                x: 15, y: 18, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 16, y: 18, fg: Yellow, bg: Reset, underline: Reset, modifier: NONE,
                x: 28, y: 18, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 135, y: 18, fg: Reset, bg: Reset, underline: Reset, modifier: NONE,
                x: 15, y: 19, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 135, y: 19, fg: Reset, bg: Reset, underline: Reset, modifier: NONE,
                x: 15, y: 20, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 16, y: 20, fg: Cyan, bg: Reset, underline: Reset, modifier: BOLD,
                x: 32, y: 20, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 135, y: 20, fg: Reset, bg: Reset, underline: Reset, modifier: NONE,
                x: 15, y: 21, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 135, y: 21, fg: Reset, bg: Reset, underline: Reset, modifier: NONE,
                x: 15, y: 22, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 16, y: 22, fg: Yellow, bg: Reset, underline: Reset, modifier: NONE,
                x: 28, y: 22, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 135, y: 22, fg: Reset, bg: Reset, underline: Reset, modifier: NONE,
                x: 15, y: 23, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 16, y: 23, fg: Yellow, bg: Reset, underline: Reset, modifier: NONE,
                x: 28, y: 23, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 135, y: 23, fg: Reset, bg: Reset, underline: Reset, modifier: NONE,
                x: 15, y: 24, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 135, y: 24, fg: Reset, bg: Reset, underline: Reset, modifier: NONE,
                x: 15, y: 25, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 16, y: 25, fg: Cyan, bg: Reset, underline: Reset, modifier: BOLD,
                x: 29, y: 25, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 135, y: 25, fg: Reset, bg: Reset, underline: Reset, modifier: NONE,
                x: 15, y: 26, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 135, y: 26, fg: Reset, bg: Reset, underline: Reset, modifier: NONE,
                x: 15, y: 27, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 16, y: 27, fg: Yellow, bg: Reset, underline: Reset, modifier: NONE,
                x: 28, y: 27, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 135, y: 27, fg: Reset, bg: Reset, underline: Reset, modifier: NONE,
                x: 15, y: 28, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 135, y: 28, fg: Reset, bg: Reset, underline: Reset, modifier: NONE,
                x: 15, y: 29, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 16, y: 29, fg: Cyan, bg: Reset, underline: Reset, modifier: BOLD,
                x: 31, y: 29, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 135, y: 29, fg: Reset, bg: Reset, underline: Reset, modifier: NONE,
                x: 15, y: 30, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 135, y: 30, fg: Reset, bg: Reset, underline: Reset, modifier: NONE,
                x: 15, y: 31, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 16, y: 31, fg: Yellow, bg: Reset, underline: Reset, modifier: NONE,
                x: 28, y: 31, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 135, y: 31, fg: Reset, bg: Reset, underline: Reset, modifier: NONE,
                x: 15, y: 32, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 16, y: 32, fg: Yellow, bg: Reset, underline: Reset, modifier: NONE,
                x: 28, y: 32, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 135, y: 32, fg: Reset, bg: Reset, underline: Reset, modifier: NONE,
                x: 15, y: 33, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 16, y: 33, fg: Yellow, bg: Reset, underline: Reset, modifier: NONE,
                x: 28, y: 33, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 135, y: 33, fg: Reset, bg: Reset, underline: Reset, modifier: NONE,
                x: 15, y: 34, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 16, y: 34, fg: Yellow, bg: Reset, underline: Reset, modifier: NONE,
                x: 28, y: 34, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 135, y: 34, fg: Reset, bg: Reset, underline: Reset, modifier: NONE,
                x: 15, y: 35, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 16, y: 35, fg: Yellow, bg: Reset, underline: Reset, modifier: NONE,
                x: 28, y: 35, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 135, y: 35, fg: Reset, bg: Reset, underline: Reset, modifier: NONE,
                x: 15, y: 36, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 135, y: 36, fg: Reset, bg: Reset, underline: Reset, modifier: NONE,
                x: 15, y: 37, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 135, y: 37, fg: Reset, bg: Reset, underline: Reset, modifier: NONE,
                x: 15, y: 38, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 16, y: 38, fg: Gray, bg: Reset, underline: Reset, modifier: NONE,
                x: 20, y: 38, fg: Yellow, bg: Reset, underline: Reset, modifier: NONE,
                x: 23, y: 38, fg: Gray, bg: Reset, underline: Reset, modifier: NONE,
                x: 42, y: 38, fg: Yellow, bg: Reset, underline: Reset, modifier: NONE,
                x: 45, y: 38, fg: Gray, bg: Reset, underline: Reset, modifier: NONE,
                x: 54, y: 38, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 135, y: 38, fg: Reset, bg: Reset, underline: Reset, modifier: NONE,
                x: 15, y: 39, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 135, y: 39, fg: Reset, bg: Reset, underline: Reset, modifier: NONE,
                x: 15, y: 40, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 135, y: 40, fg: Reset, bg: Reset, underline: Reset, modifier: NONE,
                x: 15, y: 41, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 135, y: 41, fg: Reset, bg: Reset, underline: Reset, modifier: NONE,
                x: 15, y: 42, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 135, y: 42, fg: Reset, bg: Reset, underline: Reset, modifier: NONE,
                x: 15, y: 43, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 135, y: 43, fg: Reset, bg: Reset, underline: Reset, modifier: NONE,
                x: 15, y: 44, fg: White, bg: Reset, underline: Reset, modifier: NONE,
                x: 135, y: 44, fg: Reset, bg: Reset, underline: Reset, modifier: NONE,
            ]
        }
        "#);
    }
}
