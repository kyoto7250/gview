use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders},
    Frame,
};

use super::filer::Filer;
use super::filter::Filter;
use crate::components::operatable_components::OperatableComponent;

pub struct Explorer {
    pub filter: Filter,
    pub filer: Filer,
    pub refresh: bool,
    pub items: Vec<String>,
}

impl Explorer {
    pub fn new() -> Self {
        Self {
            filter: Filter::new(),
            filer: Filer::new(),
            refresh: true,
            items: vec![],
        }
    }

    pub fn draw(&mut self, frame: &mut Frame, area: Rect) -> anyhow::Result<()> {
        // left frame
        frame.render_widget(
            Block::default().title("Explorer").borders(Borders::ALL),
            area,
        );

        // left chunks
        let chunks = Layout::default()
            .vertical_margin(1)
            .horizontal_margin(1)
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(2), Constraint::Min(1)].as_ref())
            .split(area);
        self.filter.draw(frame, chunks[0]);

        // TODO: should not clone
        self.filer.draw(frame, chunks[1]);
        Ok(())
    }
}
