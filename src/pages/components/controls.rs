use std::{
    collections::HashMap,
    fmt::Display,
    time::{Duration, Instant},
};

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
pub struct Controls {
    keymap: HashMap<KeyEvent, Action>,
    mouse: Option<MouseEvent>,
    area: Rect,
    start_time: Instant,
    state: ScrollState,
    playing: bool,
    time: u64,
    pub progress: f64,
    pub action_tx: Option<UnboundedSender<Action>>,
}

impl Controls {
    pub fn new() -> Self {
        let state = ScrollState::new(0);
        Self {
            state,
            mouse: None,
            start_time: Instant::now(),
            progress: f64::default(),
            action_tx: None,
            time: 0,
            playing: false,
            area: Rect::default(),
            keymap: HashMap::new(),
        }
    }

    fn tick(&mut self) -> Result<()> {
        if self.playing {
            let now = Instant::now();
            let elapsed = (now - self.start_time).as_secs_f64();
            if elapsed >= 1.0 {
                log::info!("Ticking controls, elapsed: {}", elapsed);
                self.start_time = now;
                self.time += 1;
            }
        }
        Ok(())
    }
}

impl Default for Controls {
    fn default() -> Self {
        Self::new()
    }
}
impl Component for Controls {
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
            Action::Tick => self.tick()?,
            Action::PausePlay => self.playing = !self.playing,
            Action::Forward => {}
            Action::Back => {}
            _ => (),
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame<'_>) -> Result<()> {
        let mut state = self.state;
        let area = self.area;
        frame.render_widget(Clear, area); //this clears out the background
        frame.render_stateful_widget(self, area, &mut state);
        Ok(())
    }
}

impl StatefulWidget for &mut Controls {
    type State = ScrollState;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Block::bordered()
        //     .border_type(BorderType::Rounded)
        //     .title_top(Line::from("Controls"))
        //     .style(Style::default().fg(Color::White))
        //     .render(area, buf);
        //let text = Text::from("1093 - YEAT (2093)");

        let [header, body, footer] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(2),
            Constraint::Length(1),
        ])
        //.margin(1)
        .areas(area);
        // Display time as 00:00
        let time: String = format!("{:02}:{:02}", self.time / 60, self.time % 60);
        LineGauge::default()
            .ratio(self.progress)
            .label(
                //Seconds into minutes
                time + " ⏪︎"
                    + match self.playing {
                        true => " ⏸︎",
                        false => " ⏯",
                    }
                    + " ⏩︎ ",
            )
            .block(Block::new())
            .filled_style(Style::default().fg(Color::Magenta))
            .line_set(symbols::line::THICK)
            .render(footer, buf);

        state.view_size = body.height as usize;
        self.state = *state;
        // state.position = state
        //     .position
        //     .min(text.height().saturating_sub(state.view_size));
        let header_line = Line::from(vec![
            Span::raw("Now Playing: "),
            Span::styled(
                "1093 - YEAT (2093)".to_string(),
                (Color::White, Modifier::BOLD),
            ),
        ]);
        Paragraph::new(header_line)
            .style(Style::default().bg(Color::Black))
            .render(header, buf);
    }
}
