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
pub struct Quit {
    pub show: bool,
    pub menu_index: usize,
    pub action_tx: Option<UnboundedSender<Action>>,
    pub keymap: HashMap<KeyEvent, Action>,
    area: Rect,
    content_area: Rect,
}

impl Quit {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn keymap(mut self, keymap: HashMap<KeyEvent, Action>) -> Self {
        self.keymap = keymap;
        self
    }
    /// helper function to create a centered rect using up certain percentage of the available rect `r`
    fn quit_area(area: Rect) -> Rect {
        let vertical = Layout::vertical([Constraint::Length(3)])
            .flex(Flex::Start)
            .vertical_margin(3);
        let horizontal = Layout::horizontal([Constraint::Percentage(15)]).flex(Flex::Center);
        let [area] = vertical.areas(area);
        let [area] = horizontal.areas(area);
        area
    }
}

impl Component for Quit {
    fn current_mode(&self) -> InputMode {
        InputMode::Normal
    }
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.action_tx = Some(tx);
        Ok(())
    }

    fn register_layout_handler(&mut self, area: Rect) -> Result<()> {
        let area = Self::quit_area(area);
        let layout = Layout::horizontal([Constraint::Min(1)]);
        let block = Block::bordered();
        // Get Areas
        let outer_area = area;
        let [bottom_area] = layout.areas(outer_area);
        let bottom_area = block.inner(bottom_area);

        self.area = outer_area;
        self.content_area = bottom_area;
        Ok(())
    }

    fn handle_mouse_events(&mut self, mouse: MouseEvent) -> Result<Option<Action>> {
        let tx = self.action_tx.clone().unwrap();

        Ok(None)
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        if !self.show {
            return Ok(None);
        }
        let action = match key.code {
            KeyCode::Esc => Action::ClosePopup,
            KeyCode::Enter => Action::SelectOption,
            KeyCode::Char('h') => Action::Back,
            KeyCode::Char('l') => Action::Forward,
            _ => return Ok(None),
        };
        Ok(Some(action))
    }

    fn update(&mut self, action: Action, ctx: &Ctx) -> Result<Option<Action>> {
        if self.show {
            match action {
                Action::Forward => {
                    self.menu_index = (self.menu_index + 1) % InputMode::CONFIRM.len()
                }
                Action::SelectOption => {
                    match self.menu_index {
                        0 => {
                            // No action, just close the popup
                            self.show = false;
                        }
                        1 => {
                            // Yes, close the application
                            if let Some(sender) = &self.action_tx {
                                if let Err(e) = sender.send(Action::Quit) {
                                    error!("Failed to send action: {:?}", e);
                                }
                            }
                            self.show = false;
                        }
                        _ => {}
                    }
                }
                Action::Back => {
                    if self.menu_index == 0 {
                        self.menu_index = InputMode::CONFIRM.len() - 1;
                    } else {
                        self.menu_index = (self.menu_index - 1) % InputMode::CONFIRM.len();
                    }
                }
                Action::ClosePopup => {
                    self.show = false;
                }
                _ => (),
            }
        }
        if action == Action::ToggleShowQuit {
            self.menu_index = 0;
            self.show = true;
        }
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>) -> Result<()> {
        if self.show {
            // Blocks for popup and button area
            let block = Block::bordered()
                .border_type(BorderType::Rounded)
                .title_top(Line::from("Close Application?").centered())
                .style(Style::default().fg(Color::White));
            // Prep the widgets
            let text = Line::from(vec![
                Span::from("No").style(match self.menu_index {
                    0 => Style::default().bg(Color::Yellow),
                    _ => Style::default(),
                }),
                "|".into(),
                Span::from("Yes").style(match self.menu_index {
                    1 => Style::default().bg(Color::Yellow),
                    _ => Style::default(),
                }),
            ]);
            let confirm = Paragraph::new(text).bold().left_aligned();

            // Render the widgets
            f.render_widget(Clear, self.area); //this clears out the background

            f.render_widget(block, self.area);
            f.render_widget(confirm, self.content_area);
        }
        Ok(())
    }
}
