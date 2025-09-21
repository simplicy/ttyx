use std::{collections::HashMap, fmt::Display, time::Duration};

use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, MouseEvent};
use log::error;
use ratatui::{layout::Flex, prelude::*, widgets::*};
use tokio::sync::mpsc::UnboundedSender;
use tracing::trace;
use tui_input::{backend::crossterm::EventHandler, Input};

use crate::pages::{Component, Frame, InputMode};
use crate::{
    app::Mode,
    utils::{action::Action, key_event_to_string, Ctx},
    APP_NAME,
};

#[derive(Default)]
pub struct Help {
    pub show: bool,
    menu_index: usize,
    pub action_tx: Option<UnboundedSender<Action>>,
    pub keymap: HashMap<KeyEvent, Action>,
    area: Rect,
    content_area: Rect,
}

impl Help {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn keymap(mut self, keymap: HashMap<KeyEvent, Action>) -> Self {
        self.keymap = keymap;
        self
    }

    /// helper function to create a centered rect using up certain percentage of the available rect `r`
    fn popup_area(area: Rect) -> Rect {
        let vertical = Layout::vertical([Constraint::Min(3)])
            .flex(Flex::End)
            .vertical_margin(2);
        let horizontal = Layout::horizontal([Constraint::Percentage(17)])
            .flex(Flex::End)
            .horizontal_margin(2);
        let [area] = horizontal.areas(area);
        let [area] = vertical.areas(area);
        area
    }
}

impl Component for Help {
    fn current_mode(&self) -> InputMode {
        InputMode::Normal
    }
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.action_tx = Some(tx);
        Ok(())
    }

    fn register_layout_handler(&mut self, area: Rect) -> Result<()> {
        let area = Self::popup_area(area);
        // Get Areas
        let outer_area = area;
        let layout = Layout::vertical([Constraint::Min(1)]);
        let [bottom_area] = layout.areas(outer_area);
        let block = Block::bordered();

        let content_area = block.inner(bottom_area);
        self.area = outer_area;
        self.content_area = content_area;
        Ok(())
    }

    fn handle_mouse_events(&mut self, mouse: MouseEvent) -> Result<Option<Action>> {
        let tx = self.action_tx.clone().unwrap();

        Ok(None)
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        let action = match key.code {
            KeyCode::Esc => {
                self.show = false;
                Action::Update
            }
            KeyCode::Enter => Action::SelectOption,
            _ => return Ok(None),
        };
        Ok(Some(action))
    }

    fn update(&mut self, action: Action, ctx: &Ctx) -> Result<Option<Action>> {
        match action {
            Action::ToggleShowHelp => {
                self.show = !self.show;
            }
            _ => (),
        }
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>) -> Result<()> {
        if self.show {
            // Blocks for popup and button area
            let block = Block::bordered()
                .border_type(BorderType::Rounded)
                .title_top(Line::from("Key-Bindings").right_aligned())
                .style(Style::default().fg(Color::White));
            // Prep the widgets
            let text = vec![Line::from("somehintg").centered(), Line::from("")];
            let content = Paragraph::new(text)
                .wrap(Wrap { trim: false })
                .bold()
                .left_aligned();

            // Render the widgets
            f.render_widget(Clear, self.area); //this clears out the background

            f.render_widget(block, self.area);
        }
        Ok(())
    }
}
