use std::{collections::HashMap, fmt::Display, time::Duration};

use super::Modal;
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
pub struct Popup {
    pub popups: Vec<Modal>,
    pub menu_index: usize,
    pub action_tx: Option<UnboundedSender<Action>>,
    pub keymap: HashMap<KeyEvent, Action>,
    area: Rect,
    content_area: Rect,
}

impl Popup {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn keymap(mut self, keymap: HashMap<KeyEvent, Action>) -> Self {
        self.keymap = keymap;
        self
    }
    /// helper function to create a centered rect using up certain percentage of the available rect `r`
    fn popup_area(area: Rect, percent_x: u16) -> Rect {
        let vertical = Layout::vertical([Constraint::Percentage(15)]).flex(Flex::Center);
        let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
        let [area] = vertical.areas(area);
        let [area] = horizontal.areas(area);
        area
    }
}

impl Component for Popup {
    fn current_mode(&self) -> InputMode {
        InputMode::Normal
    }
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.action_tx = Some(tx);
        Ok(())
    }
    fn register_layout_handler(&mut self, area: Rect) -> Result<()> {
        let area = Self::popup_area(area, 15);
        let outer_area = area;
        self.area = area;
        // Get Areas
        let block = Block::bordered();
        let layout = Layout::vertical([Constraint::Min(1)]);
        let [bottom_area] = layout.areas(outer_area);
        let bottom_area = block.inner(bottom_area);
        self.content_area = bottom_area;
        Ok(())
    }

    fn handle_mouse_events(&mut self, mouse: MouseEvent) -> Result<Option<Action>> {
        let tx = self.action_tx.clone().unwrap();

        Ok(None)
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        let action = match key.code {
            KeyCode::Esc => Action::ClosePopup,
            KeyCode::Enter => Action::SelectOption,
            _ => return Ok(None),
        };
        Ok(Some(action))
    }

    fn update(&mut self, action: Action, ctx: &Ctx) -> Result<Option<Action>> {
        if !self.popups.is_empty() {
            match action {
                Action::Forward => {
                    self.menu_index = (self.menu_index + 1) % InputMode::CONFIRM.len()
                }
                Action::Back => {
                    if self.menu_index == 0 {
                        self.menu_index = InputMode::CONFIRM.len() - 1;
                    } else {
                        self.menu_index = (self.menu_index - 1) % InputMode::CONFIRM.len();
                    }
                }
                Action::Popup(title, body) => {
                    log::info!("Popup-ing {}", title);
                    self.popups.push(Modal {
                        title: Some(title),
                        content: body,
                        subaction: None,
                        duration: Duration::from_secs(5),
                    });
                }
                Action::SelectOption => {
                    match self.menu_index {
                        0 => {
                            // No action, just close the popup
                            self.popups.pop();
                            // Don't Do Something
                        }
                        1 => {
                            let thing = self.popups[0].subaction.clone();
                            self.popups.pop();
                            // Do Something
                            return Ok(thing);
                        }
                        _ => {}
                    }
                }
                Action::ClosePopup => {
                    self.popups.pop();
                }
                _ => {}
            }
        } else if let Action::Popup(title, body) = action {
            log::info!("Toasting {}", title);
            self.popups.push(Modal {
                title: Some(title),
                content: body,
                subaction: None,
                duration: Duration::from_secs(5),
            });

            // self.toasts.push(Modal {
            //     title: Some(title),
            //     content: body,
            //     subaction: None,
            // });
        }
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>) -> Result<()> {
        if self.popups.is_empty() {
            return Ok(());
        }
        let title = self.popups[0]
            .title
            .clone()
            .unwrap_or_else(|| "Popup".to_string());
        let content = self.popups[0].content.clone();
        // Blocks for popup and button area
        let block = Block::bordered()
            .title_top(Line::from(title).left_aligned())
            .title_top(Line::from("[x]").right_aligned())
            .style(Style::default().bg(Color::Black).fg(Color::White));

        // Prep the widgets
        let text = vec![
            Line::from(content).centered(),
            Line::from(""),
            match self.popups[0].subaction {
                Some(_) => Line::from(vec![
                    Span::from("No").style(match self.menu_index {
                        0 => Style::default().bg(Color::Yellow),
                        _ => Style::default(),
                    }),
                    "|".into(),
                    Span::from("Yes").style(match self.menu_index {
                        1 => Style::default().bg(Color::Yellow),
                        _ => Style::default(),
                    }),
                ]),
                None => Line::from(""),
            },
        ];
        let confirm = Paragraph::new(text)
            .wrap(Wrap { trim: false })
            .bold()
            .left_aligned();

        let outer_area = self.area;
        // Render the widgets
        f.render_widget(Clear, outer_area); //this clears out the background

        f.render_widget(block, outer_area);
        f.render_widget(confirm, self.content_area);

        Ok(())
    }
}
