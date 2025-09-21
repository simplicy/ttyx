use std::{collections::HashMap, time::Duration};

use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, MouseEvent};
use log::error;
use ratatui::{prelude::*, widgets::*};
use tokio::sync::mpsc::UnboundedSender;
use tracing::trace;
use tui_input::{backend::crossterm::EventHandler, Input};

use super::{Component, Frame};
use crate::utils::{action::Action, key_event_to_string, Ctx, InputMode};

#[derive(Default)]
pub struct Home {
    pub render_ticker: usize,
    pub mode: InputMode,
    pub input: Input,
    pub action_tx: Option<UnboundedSender<Action>>,
    pub keymap: HashMap<KeyEvent, Action>,
    pub text: Vec<String>,
    pub last_events: Vec<KeyEvent>,
    area: Rect,
    areas: Vec<Rect>,
}

impl Home {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn keymap(mut self, keymap: HashMap<KeyEvent, Action>) -> Self {
        self.keymap = keymap;
        self
    }

    pub fn render_tick(&mut self) {
        self.render_ticker = self.render_ticker.saturating_add(1);
    }

    pub fn add(&mut self, s: String) {
        self.text.push(s)
    }

    pub fn schedule_increment(&mut self, i: usize) {
        let tx = self.action_tx.clone().unwrap();
        tokio::spawn(async move {
            tx.send(Action::EnterProcessing).unwrap();
            tokio::time::sleep(Duration::from_secs(1)).await;
            tx.send(Action::Increment(i)).unwrap();
            tx.send(Action::EnterNormal).unwrap();
        });
    }

    pub fn schedule_decrement(&mut self, i: usize) {
        let tx = self.action_tx.clone().unwrap();
        tokio::spawn(async move {
            tx.send(Action::EnterProcessing).unwrap();
            tokio::time::sleep(Duration::from_secs(1)).await;
            tx.send(Action::Decrement(i)).unwrap();
            tx.send(Action::EnterNormal).unwrap();
        });
    }
}

impl Component for Home {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.action_tx = Some(tx);
        Ok(())
    }

    fn current_mode(&self) -> InputMode {
        self.mode
    }

    fn register_layout_handler(&mut self, area: Rect) -> Result<()> {
        self.area = area;
        let rects = Layout::default()
            .constraints([Constraint::Percentage(100), Constraint::Min(3)].as_ref())
            .split(area);
        self.areas = rects.to_vec();
        Ok(())
    }

    fn handle_mouse_events(&mut self, mouse: MouseEvent) -> Result<Option<Action>> {
        let tx = self.action_tx.clone().unwrap();

        Ok(None)
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        self.last_events.push(key);
        let action = match self.mode {
            InputMode::Normal | InputMode::Processing => return Ok(None),
            InputMode::Insert => match key.code {
                KeyCode::Esc => Action::EnterNormal,
                KeyCode::Enter => {
                    if let Some(sender) = &self.action_tx {
                        if let Err(e) =
                            sender.send(Action::CompleteInput(self.input.value().to_string()))
                        {
                            error!("Failed to send action: {:?}", e);
                        }
                    }
                    Action::EnterNormal
                }
                _ => {
                    self.input.handle_event(&crossterm::event::Event::Key(key));
                    Action::Update
                }
            },
            _ => return Ok(None),
        };
        Ok(Some(action))
    }

    fn update(&mut self, action: Action, ctx: &Ctx) -> Result<Option<Action>> {
        match action {
            Action::Render => self.render_tick(),
            Action::ScheduleIncrement => self.schedule_increment(1),
            Action::ScheduleDecrement => self.schedule_decrement(1),
            Action::CompleteInput(s) => self.add(s),
            Action::EnterNormal => {
                self.mode = InputMode::Normal;
            }
            Action::EnterInsert => {
                self.mode = InputMode::Insert;
            }
            Action::EnterProcessing => {
                self.mode = InputMode::Processing;
            }
            _ => (),
        }
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>) -> Result<()> {
        let mut text: Vec<Line> = self
            .text
            .clone()
            .iter()
            .map(|l| Line::from(l.clone()))
            .collect();
        text.insert(0, "".into());
        text.insert(
            0,
            "Type into input and hit enter to display here".dim().into(),
        );
        text.insert(0, "".into());
        text.insert(0, format!("Render Ticker: {}", self.render_ticker).into());
        text.insert(0, "".into());
        text.insert(
            0,
            Line::from(vec![
                "Press ".into(),
                Span::styled("j", Style::default().fg(Color::Red)),
                " or ".into(),
                Span::styled("k", Style::default().fg(Color::Red)),
                " to ".into(),
                Span::styled("increment", Style::default().fg(Color::Yellow)),
                " or ".into(),
                Span::styled("decrement", Style::default().fg(Color::Yellow)),
                ".".into(),
            ]),
        );
        text.insert(0, "".into());

        f.render_widget(
            Paragraph::new(text)
                .block(
                    Block::default()
                        .title("ratatui async template")
                        .title_alignment(Alignment::Center)
                        .borders(Borders::ALL)
                        .border_style(match self.mode {
                            InputMode::Processing => Style::default().fg(Color::Yellow),
                            _ => Style::default(),
                        })
                        .border_type(BorderType::Rounded),
                )
                .style(Style::default().fg(Color::Cyan))
                .alignment(Alignment::Center),
            self.areas[0],
        );
        let width = self.areas[1].width.max(3) - 3; // keep 2 for borders and 1 for cursor
        let scroll = self.input.visual_scroll(width as usize);
        let input = Paragraph::new(self.input.value())
            .style(match self.mode {
                InputMode::Insert => Style::default().fg(Color::Yellow),
                _ => Style::default(),
            })
            .scroll((0, scroll as u16))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(Line::from(vec![
                        Span::raw("Enter Input InputMode "),
                        Span::styled("(Press ", Style::default().fg(Color::DarkGray)),
                        Span::styled(
                            "/",
                            Style::default()
                                .add_modifier(Modifier::BOLD)
                                .fg(Color::Gray),
                        ),
                        Span::styled(" to start, ", Style::default().fg(Color::DarkGray)),
                        Span::styled(
                            "ESC",
                            Style::default()
                                .add_modifier(Modifier::BOLD)
                                .fg(Color::Gray),
                        ),
                        Span::styled(" to finish)", Style::default().fg(Color::DarkGray)),
                    ])),
            );
        f.render_widget(input, self.areas[1]);
        if self.mode == InputMode::Insert {
            f.set_cursor_position(Position {
                x: (self.areas[1].x + 1 + self.input.cursor() as u16)
                    .min(self.areas[1].x + self.areas[1].width - 2),
                y: self.areas[1].y + 1,
            })
        }

        f.render_widget(
            Block::default()
                .title(ratatui::widgets::block::Title::from(format!(
                    "{:?}",
                    &self
                        .last_events
                        .iter()
                        .map(key_event_to_string)
                        .collect::<Vec<_>>()
                )))
                .title_alignment(Alignment::Right)
                .title_style(Style::default().add_modifier(Modifier::BOLD)),
            Rect {
                x: self.area.x + 1,
                y: self.area.height.saturating_sub(1),
                width: self.area.width.saturating_sub(2),
                height: 1,
            },
        );

        Ok(())
    }
}
