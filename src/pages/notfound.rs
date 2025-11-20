use std::collections::HashMap;

use derive_deref::{Deref, DerefMut};
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
        let styles = Style::default().add_modifier(Modifier::RAPID_BLINK);
        let text = Text::from(Line::from("Testing")).patch_style(styles);
        let help_message = Paragraph::new(text);
        frame.render_widget(help_message, frame.area());
    }
}

impl NotFound {
    pub fn new() -> Self {
        Self {}
    }
}
