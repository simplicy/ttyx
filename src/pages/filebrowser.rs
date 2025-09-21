use std::{collections::HashMap, fmt::Display, time::Duration};

use chrono::{DateTime, Utc};
use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, MouseEvent};
use log::error;
use ratatui::{
    prelude::*,
    widgets::{
        canvas::{Canvas, Circle, Map, MapResolution, Rectangle},
        ListState, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget,
        StatefulWidgetRef, Widget, Wrap, *,
    },
};
use tokio::sync::mpsc::UnboundedSender;
use tracing::trace;
use tui_input::{backend::crossterm::EventHandler, Input};
use tui_markdown::from_str;

use super::{
    components::{Filepicker, Post},
    Component, Frame, InputMode, ScrollState, StatefulList,
};
use crate::{
    app::{App, Mode},
    pages::components::Filestats,
    utils::{action::Action, key_event_to_string, AppConfiguration, Ctx, Error, FileEntry},
};

pub struct Filebrowser {
    config: Option<AppConfiguration>,
    sidebar: bool,
    mode: InputMode,
    input: Input,
    action_tx: Option<UnboundedSender<Action>>,
    keymap: HashMap<KeyEvent, Action>,
    content: Filestats,
    picker: Filepicker,
    area: Rect,
}

impl Filebrowser {
    pub fn new() -> Self {
        Self {
            config: None,
            sidebar: true,
            mode: InputMode::Select,
            content: Filestats::default(),
            action_tx: None,
            area: Rect::default(),
            input: Input::default(),
            keymap: HashMap::new(),
            picker: Filepicker::new(false, None),
        }
    }

    pub fn keymap(mut self, keymap: HashMap<KeyEvent, Action>) -> Self {
        self.keymap = keymap;
        self
    }
}

impl Component for Filebrowser {
    fn register_config_handler(&mut self, config: AppConfiguration) -> Result<()> {
        self.config = Some(config.clone());
        self.picker.register_config_handler(config.clone())?;
        self.content.register_config_handler(config)?;
        Ok(())
    }
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.content.register_action_handler(tx.clone())?;
        self.picker.register_action_handler(tx.clone())?;
        self.action_tx = Some(tx);
        Ok(())
    }

    fn register_layout_handler(&mut self, area: Rect) -> Result<()> {
        self.area = area;

        let vertical = Layout::horizontal(match self.sidebar {
            true => vec![Constraint::Percentage(18), Constraint::Fill(1)],
            false => vec![Constraint::Fill(1)],
        })
        .split(self.area);
        let areas = vertical.to_vec();
        self.area = areas[0];
        self.content
            .register_layout_handler(areas[areas.len() - 1])?;
        self.picker.register_layout_handler(self.area)?;

        Ok(())
    }

    fn handle_mouse_events(&mut self, mouse: MouseEvent) -> Result<Option<Action>> {
        let tx = self.action_tx.clone().unwrap();
        Ok(None)
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        if self.sidebar {
            self.picker.handle_key_events(key)?;
        }
        let action = match self.mode {
            InputMode::Select => match key.code {
                KeyCode::Esc | KeyCode::Backspace => Action::ToggleSidebar,
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
            },
            _ => return Ok(None),
        };
        Ok(Some(action))
    }

    fn update(&mut self, action: Action, ctx: &Ctx) -> Result<Option<Action>> {
        self.content.update(action.clone(), ctx)?;
        self.picker.update(action.clone(), ctx)?;
        match action {
            Action::ToggleSidebar => {
                self.sidebar = !self.sidebar;
                match self.mode {
                    InputMode::Select => self.mode = InputMode::Normal,
                    _ => self.mode = InputMode::Select,
                }
            }
            Action::SelectOption => {
                match self.picker.files.state.selected() {
                    Some(index) => {
                        if let Some(selected) = self.picker.files.items.get(index) {
                            if selected.is_dir {
                                // Skip if directory
                                return Ok(None);
                            }
                            // If an item is selected, we can render the content area with the post
                            let file_path = selected.path.clone();
                            let markdown = std::fs::read_to_string(&file_path).map_err(|e| {
                                Error::Configuration(format!("Failed to read post file: {}", e))
                            })?;
                            let title = selected.name.replace(".md", "");
                            let ctime = selected.ctime.unwrap_or_default();
                            let mut view_size = self.area.height as usize;
                            let max = markdown.lines().count();
                            if max < view_size {
                                view_size = 0;
                            } else {
                                view_size = (view_size / 2) - view_size / 3; // Reserve one line for the scrollbar
                            }
                            let state = ScrollState::new(max - view_size);
                            self.content = Filestats::new(markdown, title, ctime, state);
                            //self.open_file = index;
                            // Render the post content
                        } else {
                            error!("Selected index {} out of bounds", index);
                        }
                    }
                    None => {
                        // If no item is selected, we can just clear the content area
                    }
                };
            }
            Action::Back => {}
            Action::Forward => {}
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>) -> Result<()> {
        // Set up areas
        if self.sidebar {
            self.picker.draw(f)?;
        }
        self.content.draw(f)?;
        Ok(())
    }
}
