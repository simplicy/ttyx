use std::{collections::HashMap, fmt::Display, time::Duration};

use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, MouseEvent};
use log::error;
use ratatui::{
    prelude::*,
    widgets::{
        canvas::{Canvas, Circle, Map, MapResolution, Rectangle},
        *,
    },
};
use tokio::sync::mpsc::UnboundedSender;
use tracing::trace;
use tui_input::{backend::crossterm::EventHandler, Input};

use super::{Component, Frame, InputMode};
use crate::{
    app::Mode,
    utils::{action::Action, key_event_to_string, Ctx},
};

pub struct Server<'a> {
    pub name: &'a str,
    pub location: &'a str,
    pub coords: (f64, f64),
    pub status: &'a str,
}

#[derive(Default)]
pub struct WorldMap<'a> {
    pub show: bool,
    mode: InputMode,
    pub servers: Vec<Server<'a>>,
    pub input: Input,
    pub menu_index: usize,
    pub action_tx: Option<UnboundedSender<Action>>,
    pub keymap: HashMap<KeyEvent, Action>,
    pub enhanced_graphics: bool,
    area: Rect,
    areas: Vec<Rect>,
}

impl WorldMap<'_> {
    pub fn new() -> Self {
        Self {
            enhanced_graphics: true,
            servers: vec![
                Server {
                    name: "NorthAmerica-1",
                    location: "New York City",
                    coords: (40.71, -74.00),
                    status: "Up",
                },
                Server {
                    name: "Europe-1",
                    location: "Paris",
                    coords: (48.85, 2.35),
                    status: "Failure",
                },
                Server {
                    name: "SouthAmerica-1",
                    location: "São Paulo",
                    coords: (-23.54, -46.62),
                    status: "Up",
                },
                Server {
                    name: "Asia-1",
                    location: "Singapore",
                    coords: (1.35, 103.86),
                    status: "Up",
                },
            ],
            ..Default::default()
        }
    }

    pub fn keymap(mut self, keymap: HashMap<KeyEvent, Action>) -> Self {
        self.keymap = keymap;
        self
    }
}

impl Component for WorldMap<'_> {
    fn current_mode(&self) -> InputMode {
        InputMode::Normal
    }
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.action_tx = Some(tx);
        Ok(())
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
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

    fn register_layout_handler(&mut self, area: Rect) -> Result<()> {
        self.area = area;
        let vertical = Layout::vertical([Constraint::Min(1)]);
        let [main_area] = vertical.areas(self.area);
        self.areas = vec![main_area];
        Ok(())
    }

    fn handle_mouse_events(&mut self, mouse: MouseEvent) -> Result<Option<Action>> {
        let tx = self.action_tx.clone().unwrap();

        Ok(None)
    }

    fn update(&mut self, action: Action, ctx: &Ctx) -> Result<Option<Action>> {
        match action {
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

    fn draw(&mut self, f: &mut Frame<'_>) -> Result<()> {
        let map = Canvas::default()
            .block(Block::default())
            .paint(|ctx| {
                ctx.draw(&Map {
                    color: Color::White,
                    resolution: MapResolution::High,
                });
                ctx.layer();
                ctx.draw(&Rectangle {
                    x: 0.0,
                    y: 30.0,
                    width: 10.0,
                    height: 10.0,
                    color: Color::Yellow,
                });
                ctx.draw(&Circle {
                    x: self.servers[2].coords.1,
                    y: self.servers[2].coords.0,
                    radius: 10.0,
                    color: Color::Green,
                });
                for (i, s1) in self.servers.iter().enumerate() {
                    for s2 in &self.servers[i + 1..] {
                        ctx.draw(&canvas::Line {
                            x1: s1.coords.1,
                            y1: s1.coords.0,
                            y2: s2.coords.0,
                            x2: s2.coords.1,
                            color: Color::Yellow,
                        });
                    }
                }
                for server in &self.servers {
                    let color = if server.status == "Up" {
                        Color::Green
                    } else {
                        Color::Red
                    };
                    ctx.print(
                        server.coords.1,
                        server.coords.0,
                        Span::styled("X", Style::default().fg(color)),
                    );
                }
            })
            .marker(if self.enhanced_graphics {
                symbols::Marker::Braille
            } else {
                symbols::Marker::Dot
            })
            .x_bounds([-180.0, 180.0])
            .y_bounds([-90.0, 90.0]);
        f.render_widget(map, self.areas[0]);

        Ok(())
    }
}
