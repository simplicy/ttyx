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
    components::{Controls, Filepicker, Post, Wave},
    Component, Frame, InputMode, ScrollState, StatefulList,
};
use crate::{
    app::{App, Mode},
    utils::{action::Action, key_event_to_string, AppConfiguration, Ctx, Error, FileEntry},
};

enum ContentType {
    Visualizer,
    Queue,
    Playlist,
}

enum SideBarType {
    Picker,
    Songs,
    Playlists,
    Queue,
}

pub struct MusicPlayer {
    config: Option<AppConfiguration>,
    sidebar: bool,
    mode: InputMode,
    input: Input,
    action_tx: Option<UnboundedSender<Action>>,
    keymap: HashMap<KeyEvent, Action>,
    controls: Controls,
    picker: Filepicker,
    wave: Wave,
    area: Rect,
    areas: Vec<Rect>,
}

impl MusicPlayer {
    pub fn new() -> Self {
        Self {
            config: None,
            sidebar: true,
            mode: InputMode::Select,
            action_tx: None,
            area: Rect::default(),
            input: Input::default(),
            keymap: HashMap::new(),
            controls: Controls::new(),
            picker: Filepicker::new(false, None),
            wave: Wave::new(),
            areas: Vec::new(),
        }
    }

    pub fn keymap(mut self, keymap: HashMap<KeyEvent, Action>) -> Self {
        self.keymap = keymap;
        self
    }
}

impl Component for MusicPlayer {
    fn register_config_handler(&mut self, config: AppConfiguration) -> Result<()> {
        self.config = Some(config.clone());
        self.picker.register_config_handler(config.clone())?;
        self.wave.register_config_handler(config.clone())?;
        Ok(())
    }

    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.picker.register_action_handler(tx.clone())?;
        self.wave.register_action_handler(tx.clone())?;
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
        let horizontal = Layout::vertical([Constraint::Percentage(20), Constraint::Fill(1)])
            .split(vertical[vertical.len() - 1]);
        self.area = vertical[0];
        self.controls.register_layout_handler(horizontal[0])?;
        self.wave
            .register_layout_handler(horizontal[horizontal.len() - 1])?;
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
        self.picker.update(action.clone(), ctx)?;
        self.controls.update(action.clone(), ctx)?;
        self.wave.update(action.clone(), ctx)?;
        if action == Action::ToggleSidebar {
            self.sidebar = !self.sidebar;
            match self.mode {
                InputMode::Select => self.mode = InputMode::Normal,
                _ => self.mode = InputMode::Select,
            }
        }
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>) -> Result<()> {
        // Set up areas
        if self.sidebar {
            self.picker.draw(f)?;
        }
        self.controls.draw(f)?;
        self.wave.draw(f)?;
        Ok(())
    }
}
