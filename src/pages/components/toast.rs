use std::time::Instant;
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

enum Location {
    Center,
    Left,
    Right,
    BottomLeft,
    BottomRight,
}

pub struct Toast {
    pub toasts: Vec<Modal>,
    pub menu_index: usize,
    pub action_tx: Option<UnboundedSender<Action>>,
    pub keymap: HashMap<KeyEvent, Action>,
    render_start_time: Instant,
    render_frames: u32,
    area: Rect,
    content_area: Rect,
}

impl Toast {
    pub fn new() -> Self {
        Self {
            toasts: vec![Modal {
                title: Some("Toast".to_string()),
                content: "This is a toast message.".to_string(),
                subaction: None,
                duration: Duration::from_secs(5),
            }],
            render_start_time: Instant::now(),
            render_frames: 0,
            menu_index: 0,
            action_tx: None,
            keymap: HashMap::new(),
            area: Rect::default(),
            content_area: Rect::default(),
        }
    }

    fn render_tick(&mut self) -> Result<()> {
        self.render_frames += 1;
        let now = Instant::now();
        let elapsed = (now - self.render_start_time).as_secs_f64();
        if elapsed >= 1.0 {
            // Update the duration on the toasts
            if let Some(toast) = self.toasts.first_mut() {
                if let Some(duration) = toast.duration.checked_sub(Duration::from_secs(1)) {
                    toast.duration = duration;
                }
            }
            self.render_start_time = now;
            self.render_frames = 0;
            if self.toasts.first().map_or(false, |t| t.duration.is_zero()) {
                self.toasts.remove(0);
            }
        }
        Ok(())
    }

    pub fn keymap(mut self, keymap: HashMap<KeyEvent, Action>) -> Self {
        self.keymap = keymap;
        self
    }
    /// helper function to create a centered rect using up certain percentage of the available rect `r`
    fn toast_area(area: Rect, percent_x: u16) -> Rect {
        let vertical = Layout::vertical([Constraint::Percentage(15)]).flex(Flex::Start);
        let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::End);
        let [area] = vertical.areas(area);
        let [area] = horizontal.areas(area);
        area
    }
}

impl Component for Toast {
    fn current_mode(&self) -> InputMode {
        InputMode::Normal
    }
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.action_tx = Some(tx);
        Ok(())
    }
    fn register_layout_handler(&mut self, area: Rect) -> Result<()> {
        let area = Self::toast_area(area, 15);
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
            KeyCode::Esc => Action::CloseToast,
            KeyCode::Enter => Action::SelectOption,
            _ => return Ok(None),
        };
        Ok(Some(action))
    }

    fn update(&mut self, action: Action, ctx: &Ctx) -> Result<Option<Action>> {
        match action {
            Action::Render => self.render_tick()?,
            Action::Toast(title, body) => {
                log::info!("Toasting {}", title);
                self.toasts.push(Modal {
                    title: Some(title),
                    content: body,
                    subaction: None,
                    duration: Duration::from_secs(5),
                });
            }
            Action::CloseToast => {
                self.toasts.pop();
            }
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>) -> Result<()> {
        if self.toasts.is_empty() {
            return Ok(());
        }
        let title = self.toasts[0]
            .title
            .clone()
            .unwrap_or_else(|| "Toast".to_string());
        let content = self.toasts[0].content.clone();
        // Blocks for popup and button area
        let block = Block::bordered()
            .title_top(Line::from(title).left_aligned())
            .style(Style::default().bg(Color::Black).fg(Color::White));

        // Render the widgets
        f.render_widget(Clear, self.area); //this clears out the background

        let paragraph = Paragraph::new(content)
            .wrap(Wrap { trim: false })
            .bold()
            .left_aligned();
        f.render_widget(paragraph, self.content_area);
        f.render_widget(block, self.area);

        Ok(())
    }
}
