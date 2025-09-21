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

use crate::pages::{Component, Frame, InputMode, ScrollState};
use crate::{
    app::{App, Mode},
    utils::{action::Action, key_event_to_string, AppConfiguration, Ctx, Error},
};

#[derive(Debug, Clone)]
pub struct Filestats {
    title: String,
    ctime: DateTime<Utc>,
    markdown: String,
    keymap: HashMap<KeyEvent, Action>,
    mouse: Option<MouseEvent>,
    area: Rect,
    state: ScrollState,
    scrollable: bool,
    pub action_tx: Option<UnboundedSender<Action>>,
}

impl Filestats {
    pub fn new(markdown: String, title: String, ctime: DateTime<Utc>, state: ScrollState) -> Self {
        Self {
            state,
            ctime,
            title,
            markdown,
            scrollable: false,
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

impl Default for Filestats {
    fn default() -> Self {
        Self::new(
            "".to_string(),
            "".to_string(),
            DateTime::<Utc>::from(std::time::SystemTime::now()),
            ScrollState::new(0),
        )
    }
}
impl Component for Filestats {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.action_tx = Some(tx);
        Ok(())
    }

    fn register_layout_handler(&mut self, area: Rect) -> Result<()> {
        self.area = area;
        Ok(())
    }

    fn update(&mut self, action: Action, ctx: &Ctx) -> Result<Option<Action>> {
        match action {
            Action::ToggleSidebar => {
                self.scrollable = !self.scrollable;
            }
            Action::Forward => {
                if self.scrollable {
                    self.scroll_down()
                }
            }
            Action::Back => {
                if self.scrollable {
                    self.scroll_up()
                }
            }
            _ => (),
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame<'_>) -> Result<()> {
        let mut state = self.state;
        let area = self.area;
        frame.render_widget(Clear, area);
        frame.render_stateful_widget(self, area, &mut state);
        Ok(())
    }
}

impl StatefulWidget for &mut Filestats {
    type State = ScrollState;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let text = from_str(self.markdown.as_str());

        let [header, body] =
            Layout::vertical([Constraint::Length(1), Constraint::Fill(2)]).areas(area);

        let [name, ctime] =
            Layout::horizontal([Constraint::Percentage(70), Constraint::Percentage(30)])
                .areas(header);

        let [body, scrollbar] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Length(1)]).areas(body);

        state.view_size = body.height as usize;
        self.state = *state;
        state.position = state
            .position
            .min(text.height().saturating_sub(state.view_size));
        let header_line = Line::from(vec![
            Span::raw("File: "),
            Span::styled(self.title.clone(), (Color::White, Modifier::BOLD)),
        ]);
        Paragraph::new(header_line)
            .style(Style::default().bg(Color::Black))
            .render(name, buf);
        let time = self.ctime.naive_utc().date().clone().to_string()
            + " "
            + self
                .ctime
                .naive_utc()
                .time()
                .to_string()
                .split(".")
                .next()
                .unwrap();
        let time_line =
            Line::from(vec![Span::styled(time, (Color::White, Modifier::BOLD))]).right_aligned();
        Paragraph::new(time_line)
            .style(Style::default().bg(Color::Black))
            .render(ctime, buf);

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
