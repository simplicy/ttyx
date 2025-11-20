use std::collections::HashMap;

use derive_deref::{Deref, DerefMut};
use ratatui::widgets::Wrap;
use ratzilla::event::KeyEvent;
use ratzilla::ratatui::layout::{Constraint, Layout, Position};
use ratzilla::ratatui::style::{Modifier, Style, Stylize};
use ratzilla::ratatui::text::{Line, Span, Text};
use ratzilla::ratatui::widgets::{List, ListItem};
use ratzilla::ratatui::Frame;
use ratzilla::ratatui::{
    style::Color,
    widgets::{Block, Paragraph},
};
use ratzilla::{event::KeyCode, WebRenderer};
use tachyonfx::{Effect, EffectRenderer};

use crate::pages::Component;

/// App holds the state of the application
pub struct NotFound {}

impl Component for NotFound {
    fn draw(&self, frame: &mut Frame) {
        let vertical = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Fill(1),
        ]);
        let [_, input_area, _] = vertical.areas(frame.area());
        let input = Layout::vertical([
            Constraint::Min(1),
            Constraint::Length(2),
            Constraint::Length(3),
            Constraint::Min(1),
        ]);
        let [_, text_area, input_area, _] = input.areas(input_area);
        let title = Text::from(Line::from("[404]"));

        frame.render_widget(Paragraph::new(title).centered(), text_area);

        let text = Text::from(Line::from("Page not found!"));
        frame.render_widget(
            Paragraph::new(text).centered().wrap(Wrap { trim: false }),
            input_area,
        );
    }
}

impl NotFound {
    pub fn new() -> Self {
        Self {}
    }
}
