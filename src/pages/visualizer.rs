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

use super::{Component, Frame, InputMode};
use crate::{
    app::Mode,
    utils::{action::Action, key_event_to_string, Ctx},
};

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

pub struct Visualizer {
    mode: InputMode,
    pub menu_index: usize,
    pub input: Input,
    pub show: bool,
    pub progress: f64,
    pub sparkline: Signal<RandomSignal>,
    pub signals: Signals,
    pub action_tx: Option<UnboundedSender<Action>>,
    pub keymap: HashMap<KeyEvent, Action>,
    pub last_events: Vec<KeyEvent>,
    pub enhanced_graphics: bool,
    area: Rect,
    areas: Vec<Rect>,
}

impl Visualizer {
    pub fn new() -> Self {
        let mut rand_signal = RandomSignal::new(0, 100);
        let sparkline_points = rand_signal.by_ref().take(300).collect();
        let mut sin_signal = SinSignal::new(0.2, 4.0, 20.0);
        let sin1_points = sin_signal.by_ref().take(1000).collect();
        Self {
            input: Input::default(),
            mode: InputMode::Normal,
            show: false,
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
            enhanced_graphics: true,
            menu_index: 0,
            action_tx: None,
            keymap: HashMap::new(),
            last_events: vec![],
            area: Rect::default(),
            areas: vec![],
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

impl Component for Visualizer {
    fn current_mode(&self) -> InputMode {
        InputMode::Normal
    }
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.action_tx = Some(tx);
        Ok(())
    }

    fn register_layout_handler(&mut self, area: Rect) -> Result<()> {
        self.area = area;
        let constraints = vec![
            Constraint::Min(5),
            Constraint::Fill(1),
            Constraint::Length(1),
        ];
        let chunks = Layout::vertical(constraints).split(area);

        self.areas = chunks.to_vec();
        Ok(())
    }

    fn handle_mouse_events(&mut self, mouse: MouseEvent) -> Result<Option<Action>> {
        let tx = self.action_tx.clone().unwrap();

        Ok(None)
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        self.last_events.push(key);
        let action = match self.mode {
            InputMode::Normal => return Ok(None),
            InputMode::Processing => {
                self.input.handle_event(&crossterm::event::Event::Key(key));
                Action::Update
            }
            _ => return Ok(None),
        };
        Ok(Some(action))
    }

    fn update(&mut self, action: Action, ctx: &Ctx) -> Result<Option<Action>> {
        match action {
            Action::Tick => self.tick(),
            Action::Forward => self.menu_index = (self.menu_index + 1) % Mode::ALL.len(),
            Action::Back => {
                if self.menu_index == 0 {
                    self.menu_index = Mode::ALL.len() - 1;
                } else {
                    self.menu_index = (self.menu_index - 1) % Mode::ALL.len();
                }
            }
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
            .bar_set(if self.enhanced_graphics {
                symbols::bar::NINE_LEVELS
            } else {
                symbols::bar::THREE_LEVELS
            });
        frame.render_widget(sparkline, self.areas[0]);

        let chart = Chart::new(datasets)
            .block(Block::default())
            .x_axis(
                Axis::default()
                    .style(Style::default().fg(Color::Gray))
                    .bounds(self.signals.window),
            )
            .y_axis(
                Axis::default()
                    .style(Style::default().fg(Color::Gray))
                    .bounds([-20.0, 20.0]),
            );
        frame.render_widget(chart, self.areas[1]);

        // let label = format!("{:.2}%", self.progress * 100.0);
        // let gauge = Gauge::default()
        //     .block(Block::new())
        //     .gauge_style(
        //         Style::default()
        //             .fg(Color::Magenta)
        //             .bg(Color::Black)
        //             .add_modifier(Modifier::ITALIC | Modifier::BOLD),
        //     )
        //     .label(label)
        //     .use_unicode(self.enhanced_graphics)
        //     .ratio(self.progress);
        // frame.render_widget(gauge, chunks[2]);
        //
        let line_gauge = LineGauge::default()
            .block(Block::new())
            .filled_style(Style::default().fg(Color::Magenta))
            .line_set(if self.enhanced_graphics {
                symbols::line::THICK
            } else {
                symbols::line::NORMAL
            })
            .ratio(self.progress);
        frame.render_widget(line_gauge, self.areas[2]);
        Ok(())
    }
}
