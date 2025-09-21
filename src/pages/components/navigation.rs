use std::{collections::HashMap, fmt::Display, time::Duration};

use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use log::error;
use ratatui::{layout::Flex, prelude::*, widgets::*};
use tokio::sync::mpsc::UnboundedSender;
use tracing::trace;
use tui_input::{backend::crossterm::EventHandler, Input};

use crate::pages::{Component, Frame, InputMode, StatefulList};
use crate::{
    app::Mode,
    utils::{action::Action, key_event_to_string, Ctx},
};

#[derive(Default)]
pub struct Navigation {
    pub options: StatefulList<Mode>,
    pub action_tx: Option<UnboundedSender<Action>>,
    pub keymap: HashMap<KeyEvent, Action>,
    mouse: Option<MouseEvent>,
    current: Option<Mode>,
    areas: Vec<Rect>,
    area: Rect,
}

impl Navigation {
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
}

impl Component for Navigation {
    fn current_mode(&self) -> InputMode {
        InputMode::Normal
    }
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.action_tx = Some(tx);
        Ok(())
    }

    fn register_layout_handler(&mut self, area: Rect) -> Result<()> {
        self.area = area;
        let vertical = Layout::horizontal(
            Mode::ALL
                .iter()
                .map(|m| Constraint::Min(1))
                .collect::<Vec<Constraint>>(),
        );
        self.areas = vertical.split(self.area).to_vec();
        Ok(())
    }

    fn handle_mouse_events(&mut self, mouse: MouseEvent) -> Result<Option<Action>> {
        trace!("Mouse event: {:?}", mouse);
        let tx = self.action_tx.clone().unwrap();
        let mut action = None;
        self.areas.iter().enumerate().for_each(|(i, m)| {
            if let Some(mouse) = self.mouse {
                if mouse.kind == MouseEventKind::Moved {
                    if m.contains(Position::new(mouse.column, mouse.row)) {
                        self.current = Some(Mode::ALL[i]);
                    }
                    // if mouse moves outside of full area unselect
                    if !m.contains(Position::new(mouse.column, mouse.row))
                        && !self.area.contains(Position::new(mouse.column, mouse.row))
                    {
                        self.current = None;
                    }
                }
                if mouse.kind == MouseEventKind::Down(MouseButton::Left)
                    && m.contains(Position::new(
                        self.mouse.unwrap().column,
                        self.mouse.unwrap().row,
                    ))
                {
                    // Handle click event
                    self.options.state.select(Some(i));
                    action = Some(Action::ChangeMode(Mode::ALL[i]));
                }
            }
        });
        Ok(action)
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        let action = match key.code {
            KeyCode::Esc => Action::EnterNormal,
            KeyCode::Enter => Action::SelectOption,
            _ => return Ok(None),
        };
        Ok(Some(action))
    }

    fn update(&mut self, action: Action, ctx: &Ctx) -> Result<Option<Action>> {
        match action {
            Action::NextView => {
                self.options.next();
                let index = self.options.state.selected().unwrap_or(0);
                if self.options.items.get(index).is_some() {
                    return Ok(Some(Action::ChangeMode(self.options.items[index])));
                }
            }
            Action::Mouse(mouse) => {
                self.mouse.replace(mouse);
            }
            Action::PreviousView => {
                self.options.previous();
                let index = self.options.state.selected().unwrap_or(0);
                if self.options.items.get(index).is_some() {
                    return Ok(Some(Action::ChangeMode(self.options.items[index])));
                }
            }
            _ => (),
        }
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>) -> Result<()> {
        let block = Block::default().borders(Borders::BOTTOM);
        f.render_widget(block, self.area);
        Mode::ALL.iter().enumerate().for_each(|(i, m)| {
            let txt = match self.options.state.selected() {
                Some(thing) => match thing == i {
                    true => format!("[{}]", m),
                    false => format!(" {} ", m),
                },
                _ => format!(" {} ", m),
            };
            let style = match self.current {
                Some(t) => match t == Mode::ALL[i] {
                    true => Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                    false => Style::default().fg(Color::White),
                },
                _ => Style::default().fg(Color::White),
            };
            let p = Paragraph::new(txt).style(style);
            f.render_widget(p.centered(), self.areas[i]);
        });

        Ok(())
    }
}
