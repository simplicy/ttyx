use std::{collections::HashMap, fmt::Display, time::Duration};

use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use log::error;
use ratatui::{
    layout::{Flex, Spacing},
    prelude::*,
    widgets::*,
};
use reqwest::Client;
use serde_json::{json, Value};
use tokio::sync::mpsc::UnboundedSender;
use tracing::trace;
use tui_input::{backend::crossterm::EventHandler, Input};

use crate::{
    app::Mode,
    datastore::AuthenticationResponse,
    pages::{Component, GeneralResponse},
    utils::{action::Action, key_event_to_string, Ctx, Error, InputMode},
};

#[derive(Default)]
pub struct Loader {
    pub mode: InputMode,
    pub render_ticker: usize,
    pub action_tx: Option<UnboundedSender<Action>>,
    area: Rect,
}

impl Loader {
    pub fn new() -> Self {
        Self::default()
    }

    fn current_mode(&self) -> InputMode {
        log::info!("Current mode: {:?}", self.mode.to_string());
        self.mode
    }

    pub fn render_tick(&mut self) {
        self.render_ticker = self.render_ticker.saturating_add(1);
    }
}

impl Component for Loader {
    fn current_mode(&self) -> InputMode {
        self.mode
    }
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.action_tx = Some(tx);
        Ok(())
    }

    fn register_layout_handler(&mut self, area: Rect) -> Result<()> {
        // Render the widget popup
        let vertical = Layout::vertical([Constraint::Length(1)]).flex(Flex::Center);
        let [load_area] = vertical.areas(area);
        self.area = load_area;
        Ok(())
    }

    fn update(&mut self, action: Action, ctx: &Ctx) -> Result<Option<Action>> {
        if action == Action::Render {
            self.render_tick()
        }
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>) -> Result<()> {
        // Setup Layout of page
        let sign = ["⡇", "⠏", "⠛", "⠹", "⢸", "⣰", "⣤", "⣆"];
        // cycle through code one string at a time
        let i = (self.render_ticker / 3) % sign.len();
        let loader = Paragraph::new(sign[i]).centered().block(Block::default());
        // cycle through code one string at a time
        f.render_widget(loader, self.area);

        Ok(())
    }
}
