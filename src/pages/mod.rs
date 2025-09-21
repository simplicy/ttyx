use color_eyre::eyre::Result;
use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::{
    layout::Rect,
    widgets::{ListState, ScrollbarState},
};
use serde::{Deserialize, Serialize};
use strum::Display;
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    tui::{Event, Frame},
    utils::{action::Action, AppConfiguration, Ctx, InputMode},
};

pub mod blog;
pub mod chat;
pub mod components;
pub mod filebrowser;
pub mod home;
pub mod login;
pub mod music;
pub mod setting;
pub mod settings;
pub mod signup;
pub mod visualizer;
pub mod worldmap;

/// Data wrapper for a general response
#[derive(Serialize, Deserialize)]
pub struct GeneralResponse<D> {
    pub data: D,
    pub message: String,
}

/// necessary as ScrollbarState fields are private
#[derive(Debug, Clone, Copy)]
pub struct ScrollState {
    pub position: usize,
    pub view_size: usize,
    pub max: usize,
}

impl ScrollState {
    pub fn new(max: usize) -> ScrollState {
        ScrollState {
            position: 0,
            view_size: 1,
            max,
        }
    }

    fn scroll_down(&mut self) {
        self.position = self.position.saturating_add(1);
        if self.position > self.max.saturating_sub(self.view_size - self.view_size / 6) {
            self.position = self.max
        }
    }

    fn scroll_up(&mut self) {
        if self.position >= self.max {
            self.position = self.max.saturating_sub(self.view_size - self.view_size / 6);
        } else {
            self.position = self.position.saturating_sub(1);
        }
    }

    fn scroll_page_down(&mut self) {
        self.position = self.position.saturating_add(self.view_size);
    }

    fn scroll_page_up(&mut self) {
        self.position = self.position.saturating_sub(self.view_size);
    }

    fn scroll_top(&mut self) {
        self.position = 0;
    }

    fn scroll_bottom(&mut self) {
        self.position = self.max.saturating_sub(self.view_size);
    }
}

impl From<&mut ScrollState> for ScrollbarState {
    fn from(state: &mut ScrollState) -> ScrollbarState {
        ScrollbarState::new(state.max.saturating_sub(state.view_size)).position(state.position)
    }
}

impl From<&mut ScrollState> for ListState {
    fn from(state: &mut ScrollState) -> ListState {
        ListState::default().with_selected(Some(state.position))
    }
}

#[derive(Default)]
pub struct StatefulList<T> {
    pub state: ListState,
    pub items: Vec<T>,
}

impl<T> StatefulList<T> {
    pub fn with_items(items: Vec<T>) -> Self {
        Self {
            state: ListState::default(),
            items,
        }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => (i + 1) % self.items.len(),
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}

pub trait Component {
    #[allow(unused_variables)]
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        Ok(())
    }
    #[allow(unused_variables)]
    fn register_layout_handler(&mut self, area: Rect) -> Result<()> {
        Ok(())
    }

    #[allow(unused_variables)]
    fn register_config_handler(&mut self, config: AppConfiguration) -> Result<()> {
        Ok(())
    }
    fn init(&mut self) -> Result<()> {
        Ok(())
    }
    fn default_mode(&self) -> InputMode {
        InputMode::default()
    }
    fn current_mode(&self) -> InputMode {
        InputMode::default()
    }
    fn handle_events(&mut self, event: Option<Event>) -> Result<Option<Action>> {
        let r = match event {
            Some(Event::Key(key_event)) => self.handle_key_events(key_event)?,
            Some(Event::Mouse(mouse_event)) => self.handle_mouse_events(mouse_event)?,
            _ => None,
        };
        Ok(r)
    }
    fn unfocus(&mut self) -> Result<()> {
        Ok(())
    }

    fn focus(&mut self) -> Result<()> {
        Ok(())
    }

    fn is_focused(&self) -> bool {
        true
    }

    #[allow(unused_variables)]
    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        Ok(None)
    }
    #[allow(unused_variables)]
    fn handle_mouse_events(&mut self, mouse: MouseEvent) -> Result<Option<Action>> {
        Ok(None)
    }
    #[allow(unused_variables)]
    fn update(&mut self, action: Action, ctx: &Ctx) -> Result<Option<Action>> {
        Ok(None)
    }
    fn draw(&mut self, f: &mut Frame<'_>) -> Result<()>;
}
//// ANCHOR_END: component
