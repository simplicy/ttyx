use std::{collections::HashMap, fmt::Display, time::Duration};

use chrono::{DateTime, Utc};
use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use log::error;
use ratatui::{
    prelude::*,
    widgets::{
        canvas::{Canvas, Circle, Map, MapResolution, Rectangle},
        ListState, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget,
        Widget, Wrap, *,
    },
};
use tokio::sync::mpsc::UnboundedSender;
use tracing::trace;
use tui_input::{backend::crossterm::EventHandler, Input};
use tui_markdown::from_str;

use crate::pages::{Component, Frame, InputMode};
use crate::{
    app::{App, Mode},
    utils::{action::Action, key_event_to_string, AppConfiguration, Ctx, Error},
};

#[derive(Debug, Clone)]
pub struct MouseList {
    config: Option<AppConfiguration>,
    keymap: HashMap<KeyEvent, Action>,
    mouse: Option<MouseEvent>,
    area: Rect,
    pub items: Vec<String>,
    pub action_tx: Option<UnboundedSender<Action>>,
    pub state: MouseListState,
}

impl MouseList {
    pub fn new(items: Vec<String>, state: MouseListState) -> Self {
        Self {
            config: None,
            state,
            items,
            mouse: None,
            action_tx: None,
            area: Rect::default(),
            keymap: HashMap::new(),
        }
    }
    pub fn scroll_top(&mut self) {
        self.state.scroll_top();
    }
    pub fn scroll_up(&mut self) {
        self.state.scroll_up();
    }

    pub fn scroll_down(&mut self) {
        self.state.scroll_down();
    }
}

impl Default for MouseList {
    fn default() -> Self {
        Self::new(Vec::new(), MouseListState::new(0))
    }
}
impl Component for MouseList {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.action_tx = Some(tx);
        Ok(())
    }

    fn register_layout_handler(&mut self, area: Rect) -> Result<()> {
        self.area = area;
        let num = self.items.len();
        let layout = Layout::vertical(
            self.items
                .iter()
                .map(|_| Constraint::Length(1))
                .collect::<Vec<_>>(),
        );
        let areas = layout.split(area);
        self.state.areas = areas.to_vec();
        Ok(())
    }

    fn handle_mouse_events(&mut self, mouse: MouseEvent) -> Result<Option<Action>> {
        let tx = self.action_tx.clone();
        let action = None;
        let areas = self.state.areas.clone();
        log::info!("Mouse event: {:?}", areas);
        if let Some(mouse) = self.mouse {
            areas.iter().enumerate().for_each(|(i, m)| {
                if m.contains(Position::new(
                    self.mouse.unwrap().column,
                    self.mouse.unwrap().row,
                )) {
                    log::info!("Mouse event: {:?}", mouse);
                    self.state.select(Some(i));
                }
            });
        }
        Ok(action)
    }

    fn update(&mut self, action: Action, ctx: &Ctx) -> Result<Option<Action>> {
        match action {
            Action::EnterNormal => {}
            Action::Forward => {}
            Action::Back => {}
            _ => (),
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame<'_>) -> Result<()> {
        let mut state = self.state.clone();
        let area = self.area;
        frame.render_widget(Clear, area);
        frame.render_stateful_widget(self, area, &mut state);
        Ok(())
    }
}

/// necessary as ScrollbarState fields are private
#[derive(Debug, Clone)]
pub struct MouseListState {
    pub position: usize,
    pub view_size: usize,
    pub areas: Vec<Rect>,
    pub selected: Option<usize>,
    pub max: usize,
}

impl MouseListState {
    pub fn new(max: usize) -> MouseListState {
        MouseListState {
            position: 0,
            view_size: 1,
            areas: Vec::new(),
            selected: None,
            max,
        }
    }

    fn select(&mut self, index: Option<usize>) {
        self.selected = index;
        if let Some(i) = index {
            if i >= self.max {
                error!("Index out of bounds: {} >= {}", i, self.max);
            } else {
                self.position = i;
            }
        }
    }

    fn scroll_down(&mut self) {
        self.position = self.position.saturating_add(1);
    }

    fn scroll_up(&mut self) {
        self.position = self.position.saturating_sub(1);
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

impl From<&mut MouseListState> for ScrollbarState {
    fn from(state: &mut MouseListState) -> ScrollbarState {
        ScrollbarState::new(state.max.saturating_sub(state.view_size)).position(state.position)
    }
}

impl From<&mut MouseListState> for ListState {
    fn from(state: &mut MouseListState) -> ListState {
        ListState::default().with_selected(Some(state.position))
    }
}

impl StatefulWidget for &mut MouseList {
    type State = MouseListState;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let [body, scrollbar] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Length(1)]).areas(area);
        self.state.view_size = body.height as usize;
        state.position = self
            .state
            .position
            .min(self.items.len().saturating_sub(state.view_size));
        let position = state.position as u16;
        self.items.iter().enumerate().for_each(|(i, item)| {
            let item_area = *state.areas.get(i).unwrap_or(&Rect::default());
            let item_text = Paragraph::new(item.as_str())
                .style(match self.state.selected == Some(i) {
                    true => Style::default()
                        .bg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                    false => Style::default(),
                })
                //.scroll((position, 0))
                .wrap(Wrap { trim: false });
            item_text.render(item_area, buf);
        });
        // Paragraph::new("data")
        //     .scroll((position, 0))
        //     .wrap(Wrap { trim: false })
        //     .render(body, buf);

        let mut scrollbar_state = state.into();
        Scrollbar::new(ScrollbarOrientation::VerticalRight).render(
            scrollbar,
            buf,
            &mut scrollbar_state,
        );
    }
}
