use std::{collections::HashMap, fmt::Display, time::Duration};

use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, MouseEvent};
use log::error;
use rand::{
    distr::{Distribution, Uniform},
    rngs::ThreadRng,
};
use ratatui::{prelude::*, widgets::*};
use tokio::sync::mpsc::UnboundedSender;
use tracing::trace;
use tui_input::{backend::crossterm::EventHandler, Input};

use crate::{
    app::Mode,
    utils::{action::Action, key_event_to_string, Ctx},
};
use crate::{pages::Component, utils::InputMode};

pub struct Signal<S: Iterator> {
    source: S,
    pub points: Vec<S::Item>,
    tick_rate: usize,
}

impl<S> Signal<S>
where
    S: Iterator,
{
    fn on_tick(&mut self) {
        self.points.drain(0..self.tick_rate);
        self.points
            .extend(self.source.by_ref().take(self.tick_rate));
    }
}

pub struct Signals {
    pub sigs: Vec<Signal<SinSignal>>,
    pub window: [f64; 2],
}

impl Signals {
    fn on_tick(&mut self) {
        for signal in &mut self.sigs {
            signal.on_tick();
        }
        //     self.window[0] += 1.0;
        //     self.window[1] += 1.0;
    }
}

pub struct SinSignal {
    x: f64,
    interval: f64,
    period: f64,
    scale: f64,
}

impl SinSignal {
    pub const fn new(interval: f64, period: f64, scale: f64) -> Self {
        Self {
            x: 0.0,
            interval,
            period,
            scale,
        }
    }
}

impl Iterator for SinSignal {
    type Item = (f64, f64);
    fn next(&mut self) -> Option<Self::Item> {
        let point = (self.x, (self.x * 1.0 / self.period).sin() * self.scale);
        self.x += self.interval;
        Some(point)
    }
}

#[derive(Clone)]
pub struct RandomSignal {
    distribution: Uniform<u64>,
    rng: ThreadRng,
}

impl RandomSignal {
    pub fn new(lower: u64, upper: u64) -> Self {
        Self {
            distribution: Uniform::try_from(lower..upper).unwrap(),
            rng: rand::rng(),
        }
    }
}

impl Iterator for RandomSignal {
    type Item = u64;
    fn next(&mut self) -> Option<u64> {
        Some(self.distribution.sample(&mut self.rng))
    }
}

pub struct Wave {
    mode: InputMode,
    pub progress: f64,
    pub sparkline: Signal<RandomSignal>,
    pub signals: Signals,
    pub action_tx: Option<UnboundedSender<Action>>,
    pub keymap: HashMap<KeyEvent, Action>,
    area: Rect,
}

impl Wave {
    pub fn new() -> Self {
        let mut rand_signal = RandomSignal::new(0, 100);
        let sparkline_points = rand_signal.by_ref().take(300).collect();
        let mut sin_signal = SinSignal::new(0.2, 4.0, 20.0);
        let sin1_points = sin_signal.by_ref().take(1000).collect();
        Self {
            mode: InputMode::Normal,
            progress: 0.0,
            sparkline: Signal {
                source: rand_signal,
                points: sparkline_points,
                tick_rate: 1,
            },
            signals: Signals {
                sigs: vec![Signal {
                    source: sin_signal,
                    points: sin1_points,
                    tick_rate: 1,
                }],
                window: [0.0, 50.0],
            },
            action_tx: None,
            keymap: HashMap::new(),
            area: Rect::default(),
        }
    }

    pub fn tick(&mut self) {
        self.signals.on_tick();
        self.sparkline.on_tick();
        self.progress += 0.01;
        if self.progress > 1.0 {
            self.progress = 0.0;
        }
    }

    pub fn keymap(mut self, keymap: HashMap<KeyEvent, Action>) -> Self {
        self.keymap = keymap;
        self
    }
}

impl Component for Wave {
    fn current_mode(&self) -> InputMode {
        InputMode::Normal
    }
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.action_tx = Some(tx);
        Ok(())
    }

    fn register_layout_handler(&mut self, area: Rect) -> Result<()> {
        self.area = area;
        let constraints = vec![Constraint::Fill(1)];
        let chunks = Layout::vertical(constraints).split(area);
        self.area = chunks[0];

        Ok(())
    }

    fn handle_mouse_events(&mut self, mouse: MouseEvent) -> Result<Option<Action>> {
        let tx = self.action_tx.clone().unwrap();

        Ok(None)
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        let action = match self.mode {
            InputMode::Normal => return Ok(None),
            _ => return Ok(None),
        };
        Ok(Some(action))
    }

    fn update(&mut self, action: Action, ctx: &Ctx) -> Result<Option<Action>> {
        match action {
            Action::Tick => self.tick(),
            _ => (),
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame<'_>) -> Result<()> {
        let datasets = self
            .signals
            .sigs
            .iter()
            .map(|signal| {
                Dataset::default()
                    .marker(symbols::Marker::Dot)
                    .style(Style::default().fg(Color::Cyan))
                    .data(&signal.points)
            })
            .collect::<Vec<_>>();
        let sparkline = Sparkline::default()
            .block(Block::new())
            .style(Style::default().fg(Color::Green))
            .data(&self.sparkline.points)
            .bar_set(symbols::bar::NINE_LEVELS);

        frame.render_widget(Clear, self.area);
        frame.render_widget(sparkline, self.area);
        Ok(())
    }
}
