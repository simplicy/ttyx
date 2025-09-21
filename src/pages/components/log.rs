use std::{collections::HashMap, fmt::Display, time::Duration};

use chrono::{DateTime, Utc};
use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, MouseEvent};
use log::error;
use ratatui::{layout::Flex, prelude::*, widgets::*};
use tokio::sync::mpsc::UnboundedSender;
use tracing::trace;
use tui_input::{backend::crossterm::EventHandler, Input};

use crate::pages::{Component, Frame, InputMode, ScrollState};
use crate::utils::AppConfiguration;
use crate::{
    app::Mode,
    utils::{action::Action, key_event_to_string, Ctx},
    APP_NAME,
};

#[derive(Debug, Clone)]
pub struct Log {
    pub show: bool,
    action_tx: Option<UnboundedSender<Action>>,
    keymap: HashMap<KeyEvent, Action>,
    area: Rect,
    config: Option<AppConfiguration>,
    last_refresh: DateTime<Utc>,
    refresh_rate: Duration,
    log: String,
    pub state: ScrollState,
}

impl Log {
    pub fn new() -> Self {
        let state = ScrollState::new(0);
        Self {
            show: false,
            state,
            action_tx: None,
            keymap: HashMap::new(),
            area: Rect::default(),
            refresh_rate: Duration::from_secs(5),
            config: None,
            last_refresh: Utc::now(),
            log: String::from("test text"),
        }
    }

    pub fn keymap(mut self, keymap: HashMap<KeyEvent, Action>) -> Self {
        self.keymap = keymap;
        self
    }

    pub fn tick(&mut self) {
        let log_path = match self.config.clone() {
            Some(conf) => {
                let path = conf.config.app_data_path + "/" + APP_NAME + ".log";
                shellexpand::tilde(&path).to_string()
            }
            _ => {
                error!("Configuration not set, cannot read log file");
                return;
            }
        };
        self.last_refresh = Utc::now();
        self.log = match std::fs::read_to_string(&log_path) {
            Ok(log) => log,
            Err(e) => {
                error!("Failed to read log file: {}", e);
                String::from("Failed to read log file")
            }
        };
        self.state = ScrollState::new(self.log.lines().count());
        self.state.scroll_bottom();
    }

    /// helper function to create a centered rect using up certain percentage of the available rect `r`
    fn popup_area(area: Rect) -> Rect {
        let vertical = Layout::vertical([Constraint::Percentage(75)])
            .flex(Flex::Center)
            .vertical_margin(2);
        let horizontal = Layout::horizontal([Constraint::Percentage(80)])
            .flex(Flex::Center)
            .horizontal_margin(2);
        let [area] = horizontal.areas(area);
        let [area] = vertical.areas(area);
        area
    }
}

impl Default for Log {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for Log {
    fn current_mode(&self) -> InputMode {
        InputMode::Normal
    }
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.action_tx = Some(tx);
        Ok(())
    }
    fn register_config_handler(&mut self, config: AppConfiguration) -> Result<()> {
        self.config = Some(config);
        Ok(())
    }

    fn register_layout_handler(&mut self, area: Rect) -> Result<()> {
        self.area = Self::popup_area(area);
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
        if self.show {
            match action {
                Action::Tick => self.tick(),
                Action::ToggleLog => {
                    self.state.scroll_bottom();
                    self.show = !self.show
                }
                Action::Forward => self.state.scroll_down(),
                Action::Back => self.state.scroll_up(),
                _ => (),
            }
        } else if action == Action::ToggleLog {
            self.state.scroll_bottom();
            self.show = !self.show
        }
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>) -> Result<()> {
        if self.show {
            // Blocks for popup and button area
            let block = Block::bordered()
                .border_type(BorderType::Rounded)
                .title_top(Line::from("Log"))
                .style(Style::default().fg(Color::White));
            // Prep the widgets
            f.render_widget(Clear, self.area); //this clears out the background
            let mut state = self.state;
            f.render_widget(block.clone(), self.area);
            // make inner area for text
            let inner = block.inner(self.area);
            f.render_stateful_widget(&mut self.clone(), inner, &mut state);
            self.state = state;
        }
        Ok(())
    }
}

impl StatefulWidget for &mut Log {
    type State = ScrollState;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let text = Text::from(self.log.clone());

        let [body] = Layout::vertical([Constraint::Fill(1)]).areas(area);
        let [body, scrollbar] =
            Layout::horizontal([Constraint::Percentage(95), Constraint::Length(10)]).areas(body);

        self.state.view_size = body.height as usize;
        state.view_size = self.state.view_size;
        state.position = state
            .position
            .min(text.height().saturating_sub(state.view_size));
        let position = state
            .position
            .min(text.height().saturating_sub(state.view_size)) as u16;
        Paragraph::new(text.clone())
            .scroll((position, 0))
            .wrap(Wrap { trim: false })
            .render(body, buf);
        let mut scrollbar_state = state.into();
        Scrollbar::new(ScrollbarOrientation::VerticalRight).render(
            scrollbar,
            buf,
            &mut scrollbar_state,
        );
    }
}
