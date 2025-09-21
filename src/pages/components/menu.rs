use std::{collections::HashMap, fmt::Display, time::Duration};

use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, MouseEvent};
use log::error;
use ratatui::{layout::Flex, prelude::*, widgets::*};
use tokio::sync::mpsc::UnboundedSender;
use tracing::trace;
use tui_input::{backend::crossterm::EventHandler, Input};

use crate::pages::{Component, Frame, InputMode, StatefulList};
use crate::{
    app::Mode,
    utils::{action::Action, key_event_to_string, Ctx},
    APP_NAME,
};

#[derive(Default)]
pub struct Menu {
    pub show: bool,
    pub options: StatefulList<Mode>,
    pub action_tx: Option<UnboundedSender<Action>>,
    pub keymap: HashMap<KeyEvent, Action>,
    area: Rect,
    areas: Vec<Rect>,
}

impl Menu {
    pub fn new() -> Self {
        Self {
            options: StatefulList::with_items(Mode::ALL.to_vec()),
            ..Default::default()
        }
    }

    pub fn keymap(mut self, keymap: HashMap<KeyEvent, Action>) -> Self {
        self.keymap = keymap;
        self
    }

    /// helper function to create a centered rect using up certain percentage of the available rect `r`
    fn popup_area(area: Rect) -> Rect {
        let vertical = Layout::vertical([Constraint::Length(Mode::ALL.len() as u16 + 2)])
            .flex(Flex::Start)
            .vertical_margin(2);
        let horizontal = Layout::horizontal([Constraint::Percentage(15)]).flex(Flex::Start);
        let [area] = vertical.areas(area);
        let [area] = horizontal.areas(area);
        area
    }
}

impl Component for Menu {
    fn current_mode(&self) -> InputMode {
        InputMode::Normal
    }
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.action_tx = Some(tx);
        Ok(())
    }

    fn register_layout_handler(&mut self, area: Rect) -> Result<()> {
        // Render the widget popup
        self.area = Self::popup_area(area);
        // Get Areas
        let layout = Layout::vertical([Constraint::Min(1)]);
        let [bottom_area] = layout.areas(self.area);
        let block = Block::bordered();
        let content_area = block.inner(bottom_area);

        self.areas = vec![content_area];
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
            KeyCode::Esc => {
                self.show = false;
                Action::Update
            }
            _ => return Ok(None),
        };
        Ok(Some(action))
    }

    fn update(&mut self, action: Action, ctx: &Ctx) -> Result<Option<Action>> {
        if !self.show {
            return Ok(None);
        }
        match action {
            Action::Forward => self.options.next(),
            Action::SelectOption => {
                let index = self.options.state.selected().unwrap_or(0);
                if self.options.items.get(index).is_some() {
                    return Ok(Some(Action::ChangeMode(self.options.items[index].clone())));
                }
            }
            Action::Back => self.options.previous(),
            _ => (),
        }
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>) -> Result<()> {
        if self.show {
            // Blocks for popup and button area
            let block = Block::default()
                .borders(Borders::TOP | Borders::RIGHT | Borders::BOTTOM)
                .border_type(BorderType::Rounded)
                .title_top(Line::from("Menu").left_aligned())
                .style(Style::default().fg(Color::White));

            // Render the widgets
            f.render_widget(Clear, self.area); //this clears out the background

            let text = self
                .options
                .items
                .iter()
                .enumerate()
                .map(|(i, mode)| ListItem::new(vec![Line::from(mode.to_string())]))
                .collect::<Vec<_>>();

            let options = List::new(text).highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::Yellow)
                    .fg(Color::White),
            );

            f.render_widget(block, self.area);
            f.render_stateful_widget(options, self.areas[0], &mut self.options.state);
        }
        Ok(())
    }
}
