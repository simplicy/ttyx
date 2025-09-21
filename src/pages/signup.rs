use std::{collections::HashMap, fmt::Display, time::Duration};

use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, MouseEvent};
use log::error;
use ratatui::{layout::Flex, prelude::*, widgets::*};
use strum::Display;
use tokio::sync::mpsc::UnboundedSender;
use tracing::trace;
use tui_input::{backend::crossterm::EventHandler, Input};

use super::{Component, Frame, InputMode, StatefulList};
use crate::utils::{action::Action, key_event_to_string, AppConfiguration, Ctx};

#[derive(Display, Default, Copy, Clone, PartialEq, Eq)]
enum Items {
    #[default]
    Confirm,
    Username,
    Bio,
    Tos,
    Privacy,
}

impl Items {
    const ALL: &'static [Items] = &[
        Self::Confirm,
        Self::Username,
        Self::Bio,
        Self::Tos,
        Self::Privacy,
    ];
}

#[derive(Default)]
pub struct Settings {
    pub show: bool,
    pub mode: InputMode,
    pub confirm: Input,
    pub username: Input,
    pub bio: Input,
    options: StatefulList<Items>,
    selected_option: Option<Items>,
    pub action_tx: Option<UnboundedSender<Action>>,
    pub keymap: HashMap<KeyEvent, Action>,
    config: AppConfiguration,
    area: Rect,
    areas: Vec<Rect>,
}

impl Settings {
    pub fn new(conf: AppConfiguration) -> Self {
        Self {
            config: conf,
            options: StatefulList::with_items(Items::ALL.to_vec()),
            ..Default::default()
        }
    }
    fn draw_confirm(&self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        Ok(())
    }
    fn draw_tos(&self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        Ok(())
    }
    fn draw_privacy(&self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        Ok(())
    }
    fn draw_username(&self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        Ok(())
    }
    fn draw_bio(&self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        let layout = Layout::vertical([Constraint::Min(1)]);
        let [content_area] = layout.areas(area);
        // Make list of all config items
        let content_block =
            Paragraph::new("For support, please visit our website or contact us via email.").block(
                Block::default()
                    .borders(Borders::RIGHT)
                    .border_style(Style::default()),
            );
        f.render_widget(content_block, content_area);
        Ok(())
    }

    pub fn keymap(mut self, keymap: HashMap<KeyEvent, Action>) -> Self {
        self.keymap = keymap;
        self
    }
}

impl Component for Settings {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.action_tx = Some(tx);
        Ok(())
    }

    fn default_mode(&self) -> InputMode {
        InputMode::Normal
    }

    fn current_mode(&self) -> InputMode {
        self.mode
    }

    fn register_layout_handler(&mut self, area: Rect) -> Result<()> {
        self.area = area;
        let vertical = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .flex(Flex::Center)
        .vertical_margin(1);
        let [title_area, option_area, content_area] = vertical.areas(area);
        self.areas = vec![title_area, option_area, content_area];

        Ok(())
    }

    fn handle_mouse_events(&mut self, mouse: MouseEvent) -> Result<Option<Action>> {
        let tx = self.action_tx.clone().unwrap();
        Ok(None)
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        let action = match self.mode {
            InputMode::Normal => match key.code {
                KeyCode::Esc => Action::EnterNormal,
                KeyCode::Enter => Action::SelectOption,
                _ => {
                    self.confirm
                        .handle_event(&crossterm::event::Event::Key(key));
                    Action::Update
                }
            },
            InputMode::OptionInput => {
                match key.code {
                    KeyCode::Esc => Action::EnterNormal,
                    KeyCode::Backspace => {
                        self.mode = InputMode::Normal;
                        Action::Update
                    }
                    KeyCode::Enter => Action::SelectOption,
                    _ => {
                        if let Some(action) = self.keymap.get(&key) {
                            trace!(
                                "Key event: {} -> Action: {:?}",
                                key_event_to_string(&key),
                                action
                            );
                            return Ok(Some(action.clone()));
                        }
                        // If no action is found, we can just return None
                        return Ok(None);
                    }
                }
            }
            _ => return Ok(None),
        };
        Ok(Some(action))
    }

    fn update(&mut self, action: Action, ctx: &Ctx) -> Result<Option<Action>> {
        match action {
            Action::EnterNormal => {
                self.mode = InputMode::Normal;
                self.selected_option = None;
            }
            Action::SelectOption => {
                self.selected_option = self
                    .options
                    .state
                    .selected()
                    .and_then(|i| self.options.items.get(i).cloned())
            }
            Action::Forward => {
                match self.selected_option {
                    Some(thing) => {
                        // self.submenulist.next()
                    }
                    _ => self.options.next(),
                }
            }
            Action::Back => match self.selected_option {
                Some(thing) => {
                    // self.submenulist.previous()
                }
                _ => self.options.previous(),
            },
            _ => (),
        }
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>) -> Result<()> {
        // Make list of all config items
        let text = self
            .options
            .items
            .iter()
            .enumerate()
            .map(|(i, step)| ListItem::new(Text::from(step.to_string())))
            .collect::<Vec<_>>();

        let options = List::new(text).block(
            Block::default()
                .borders(Borders::RIGHT)
                .border_style(Style::default()),
        );

        match self.options.state.selected() {
            Some(index) => {
                if let Some(selected) = self.options.items.get(index) {
                    match selected {
                        Items::Confirm => self.draw_confirm(f, self.areas[2])?,
                        Items::Username => self.draw_username(f, self.areas[2])?,
                        Items::Bio => self.draw_bio(f, self.areas[2])?,
                        Items::Tos => self.draw_tos(f, self.areas[2])?,
                        Items::Privacy => self.draw_privacy(f, self.areas[2])?,
                    }
                } else {
                    error!("Selected index {} out of bounds", index);
                }
            }
            None => {
                // If no item is selected, we can just clear the content area
                f.render_widget(Clear, self.areas[2]);
            }
        };
        Ok(())
    }
}
