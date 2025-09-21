use std::{collections::HashMap, time::Duration};

use chrono::{DateTime, Local};
use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, MouseEvent};
use log::error;
use ratatui::{layout::Flex, prelude::*, widgets::*};
use textwrap::WordSeparator;
use tokio::sync::mpsc::UnboundedSender;
use tracing::trace;
use tui_input::{backend::crossterm::EventHandler, Input};

use super::{Component, Frame, InputMode, StatefulList};
use crate::utils::{action::Action, key_event_to_string, Ctx};

#[derive(Clone, Default)]
pub struct ChatMessage {
    pub message: String,
    pub ctime: DateTime<Local>,
    pub username: String,
}

#[derive(Default)]
pub struct Chat {
    pub show_chats: bool,
    pub show_users: bool,
    pub app_ticker: usize,
    pub render_ticker: usize,
    pub mode: InputMode,
    pub input: Input,
    pub action_tx: Option<UnboundedSender<Action>>,
    pub keymap: HashMap<KeyEvent, Action>,
    pub chats: StatefulList<ChatMessage>,
    pub last_events: Vec<KeyEvent>,
    area: Rect,
    areas: Vec<Rect>,
    overall_areas: Vec<Rect>,
    users_areas: Vec<Rect>,
    sub_areas: Vec<Rect>,
    banner_areas: Vec<Rect>,
    mouse: Option<MouseEvent>,
}

impl Chat {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn keymap(mut self, keymap: HashMap<KeyEvent, Action>) -> Self {
        self.keymap = keymap;
        self
    }

    pub fn tick(&mut self) {
        self.app_ticker = self.app_ticker.saturating_add(1);
        self.last_events.drain(..);
    }

    pub fn render_tick(&mut self) {
        self.render_ticker = self.render_ticker.saturating_add(1);
    }

    pub fn add(&mut self, s: String) {
        let s = ChatMessage {
            message: s,
            ctime: Local::now(),
            username: "Simplicy".to_string(),
        };
        self.chats.items.push(s)
    }
}

impl Component for Chat {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.action_tx = Some(tx);
        Ok(())
    }

    fn current_mode(&self) -> InputMode {
        self.mode
    }

    fn register_layout_handler(&mut self, area: Rect) -> Result<()> {
        self.area = area;
        // Setup overall horizontal layout
        let overall_horizontal = Layout::horizontal(match self.show_chats {
            true => vec![Constraint::Percentage(18), Constraint::Fill(1)],
            false => vec![Constraint::Fill(1)],
        })
        .flex(Flex::Start)
        .spacing(1)
        .split(area);
        self.overall_areas = overall_horizontal.to_vec();
        // Handle Chat list view
        let rects = Layout::default()
            .constraints([Constraint::Percentage(100), Constraint::Min(2)].as_ref())
            .flex(Flex::Start)
            .split(overall_horizontal[overall_horizontal.len() - 1]);
        self.sub_areas = rects.to_vec();
        // Handle User list view
        let horizontal = Layout::horizontal(match self.show_users {
            true => vec![Constraint::Fill(1), Constraint::Percentage(15)],
            false => vec![Constraint::Fill(1)],
        })
        .flex(Flex::Start)
        .split(rects[0]);
        self.users_areas = horizontal.to_vec();
        let vertical = Layout::vertical([Constraint::Length(2), Constraint::Fill(1)])
            .flex(Flex::Start)
            .split(horizontal[0]);
        self.banner_areas = vertical.to_vec();
        // Handle Message View
        let horizontal = Layout::horizontal([
            Constraint::Percentage(5),
            Constraint::Max(18),
            Constraint::Fill(1),
        ])
        .flex(Flex::Start)
        .split(vertical[1]);
        self.areas = horizontal.to_vec();
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
                        if self.input.value().is_empty() {
                            return Ok(None);
                        }
                        if let Err(e) =
                            sender.send(Action::CompleteInput(self.input.value().to_string()))
                        {
                            error!("Failed to send action: {:?}", e);
                        }
                        self.input.reset();
                    }
                    return Ok(None);
                }
                _ => {
                    self.input.handle_event(&crossterm::event::Event::Key(key));
                    return Ok(None);
                }
            },
            _ => return Ok(None),
        };
        Ok(Some(action))
    }

    fn update(&mut self, action: Action, ctx: &Ctx) -> Result<Option<Action>> {
        match action {
            Action::Tick => self.tick(),
            Action::Render => self.render_tick(),
            Action::Forward => {
                if self.mode != InputMode::Processing {
                    self.chats.next()
                }
            }
            Action::Back => {
                if self.mode != InputMode::Processing {
                    self.chats.previous()
                }
            }
            Action::CompleteInput(s) => self.add(s),
            Action::ToggleChats => self.show_chats = !self.show_chats,
            Action::ToggleUsers => self.show_users = !self.show_users,
            Action::EnterNormal => {
                self.mode = InputMode::Normal;
            }
            Action::EnterInput => {
                self.chats.state.select(None);
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
        // Show Chats
        if self.show_chats {
            let block = Block::new()
                .border_type(BorderType::Rounded)
                .borders(Borders::RIGHT);
            f.render_widget(block, self.overall_areas[0]);
        }

        // Show Users
        if self.show_users {
            let block = Block::new()
                .title("Users")
                .border_type(BorderType::Rounded)
                .borders(Borders::LEFT);
            f.render_widget(block, self.users_areas[self.users_areas.len() - 1]);
        }

        // Render Banner TODO: make it dynamic
        let block = Block::new()
            .border_type(BorderType::Rounded)
            .borders(Borders::BOTTOM);
        f.render_widget(block, self.banner_areas[0]);

        // Render Chats
        let times: Vec<Line> = self
            .chats
            .items
            .iter()
            .map(|l| Line::from(l.ctime.format("%H:%m").to_string()).left_aligned())
            .collect();
        let names = self
            .chats
            .items
            .iter()
            .map(|l| Line::from(l.username.clone() + " ").right_aligned())
            .collect::<Vec<_>>();
        let text = self
            .chats
            .items
            .iter()
            .map(|l| {
                let msg = l.message.as_str();
                let word = textwrap::wrap(msg, self.areas[2].width as usize - 4);
                let word = word
                    .into_iter()
                    .fold(String::new(), |acc, w| acc + &w + " ");
                ListItem::from(Line::from(" ".to_string() + word.as_str()).left_aligned())
            })
            .collect::<Vec<_>>();
        f.render_widget(
            Paragraph::new(times)
                .style(Style::default().fg(Color::Cyan))
                .alignment(Alignment::Center),
            self.areas[0],
        );
        f.render_stateful_widget(
            List::new(names)
                .highlight_style(Style::default().add_modifier(Modifier::BOLD))
                .block(Block::default().borders(Borders::RIGHT))
                .style(Style::default().fg(Color::Cyan)),
            self.areas[1],
            &mut self.chats.state,
        );

        // Stateful List of Messages
        let messages = List::new(text)
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .highlight_symbol("> ");
        f.render_stateful_widget(messages, self.areas[2], &mut self.chats.state);

        // Input View
        let width = self.sub_areas[1].width.max(3) - 3; // keep 2 for borders and 1 for cursor
        let scroll = self.input.visual_scroll(width as usize);
        let input = Paragraph::new(self.input.value())
            .style(match self.mode {
                InputMode::Insert => Style::default().fg(Color::Yellow),
                _ => Style::default(),
            })
            .scroll((0, scroll as u16))
            .block(
                Block::default()
                    .borders(Borders::TOP)
                    .border_type(BorderType::Rounded)
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
        f.render_widget(input, self.sub_areas[1]);
        if self.mode == InputMode::Insert {
            f.set_cursor_position(Position::new(
                (self.sub_areas[1].x + self.input.cursor() as u16)
                    .min(self.sub_areas[1].x + self.sub_areas[1].width - 2),
                self.sub_areas[1].y + 1,
            ))
        }
        Ok(())
    }
}
