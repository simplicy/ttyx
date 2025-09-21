use std::{collections::HashMap, fmt::Display, time::Duration};

use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use log::error;
use ratatui::{layout::Spacing, prelude::*, widgets::*};
use reqwest::Client;
use serde_json::{json, Value};
use tokio::sync::mpsc::UnboundedSender;
use tracing::trace;
use tui_input::{backend::crossterm::EventHandler, Input};
use validator::Validate;

use super::{components::Loader, Component, Frame, InputMode};
use crate::{
    app::Mode,
    datastore::AuthenticationResponse,
    pages::{components::Modal, GeneralResponse},
    utils::{action::Action, key_event_to_string, Ctx, Error},
};

#[derive(Debug, Clone, Validate)]
struct Auth {
    #[validate(email)]
    email: String,
    #[validate(length(min = 3))]
    password: String,
}

enum Items {
    Email,
    Password,
    Submit,
    Switch,
    Local,
}

impl Items {
    pub const ALL: &'static [Self] = &[
        Self::Email,
        Self::Password,
        Self::Submit,
        Self::Switch,
        Self::Local,
    ];
}

#[derive(Default)]
pub struct Login {
    pub menu_index: usize,
    pub mode: InputMode,
    pub email: Input,
    pub password: Input,
    pub render_ticker: usize,
    pub action_tx: Option<UnboundedSender<Action>>,
    pub keymap: HashMap<KeyEvent, Action>,
    pub mouse: Option<MouseEvent>,
    area: Rect,
    areas: Vec<Rect>,
    loader: Loader,
}

impl Login {
    pub fn new() -> Self {
        Self {
            mode: InputMode::InsertUser,
            ..Default::default()
        }
    }

    fn current_mode(&self) -> InputMode {
        self.mode
    }

    pub fn render_tick(&mut self) {
        self.render_ticker = self.render_ticker.saturating_add(1);
    }

    pub fn login(&mut self) {
        let tx = self.action_tx.clone().unwrap();
        let user = self.email.value().to_string();
        let pass = self.password.value().to_string();
        // Validate the input
        let data = Auth {
            email: user.clone(),
            password: pass.clone(),
        };
        match data.validate() {
            Ok(_) => {}
            Err(e) => {
                log::error!("Validation error: {}", e);
                tx.send(Action::Toast(
                    "Validation Error".to_string(),
                    "Not a valid email.".to_string(),
                ))
                .unwrap();
                tx.send(Action::EnterNormal).unwrap();
                return;
            }
        }
        tokio::spawn(async move {
            tx.send(Action::EnterProcessing).unwrap();
            let req = reqwest::Client::new();
            match req
                .post("http://localhost:8080/api/auth/login")
                .basic_auth(user, Some(pass))
                .send()
                .await
            {
                Ok(res) => {
                    //convert to Calendar to return
                    let data = res.json::<AuthenticationResponse>().await.unwrap();
                    log::info!("Login successful: {:?}", data.authenticated);
                    match data.authenticated {
                        Some(true) => {
                            tx.send(Action::EnterNormal).unwrap();
                            tx.send(Action::LoggedIn).unwrap();
                            tx.send(Action::ChangeMode(Mode::Home)).unwrap();
                        }
                        Some(err) => {
                            log::error!("Failed to login: {}", data.message);
                            tx.send(Action::Toast("Validation Error".to_string(), data.message))
                                .unwrap();
                            tx.send(Action::EnterNormal).unwrap();
                        }
                        _ => {
                            log::error!("Failed to login");
                            tx.send(Action::Toast("Validation Error".to_string(), data.message))
                                .unwrap();
                            tx.send(Action::EnterNormal).unwrap();
                        }
                    }
                }
                Err(err) => {
                    log::error!("Failed to login: {}", err);

                    tx.send(Action::Toast(
                        "Validation Error".to_string(),
                        err.to_string(),
                    ))
                    .unwrap();
                    tx.send(Action::EnterNormal).unwrap();
                }
            };
        });
    }

    pub fn register(&mut self) {
        let tx = self.action_tx.clone().unwrap();
        let user = self.email.value().to_string();
        let pass = self.password.value().to_string();
        let data = Auth {
            email: user.clone(),
            password: pass.clone(),
        };
        match data.validate() {
            Ok(_) => {}
            Err(e) => {
                log::error!("Validation error: {}", e);

                tx.send(Action::Toast(
                    "Validation Error".to_string(),
                    "Failed to validate email.".to_string(),
                ))
                .unwrap();
                tx.send(Action::EnterNormal).unwrap();
                return;
            }
        }
        tokio::spawn(async move {
            tx.send(Action::EnterProcessing).unwrap();
            let req = reqwest::Client::new();
            match req
                .post("http://localhost:8080/api/auth/register")
                .basic_auth(user, Some(pass))
                .send()
                .await
            {
                Ok(res) => {
                    //convert to Calendar to return
                    let data = res.json::<AuthenticationResponse>().await.unwrap();
                    log::info!("Login successful: {:?}", data.authenticated);
                    match data.authenticated {
                        Some(true) => {
                            tx.send(Action::EnterNormal).unwrap();
                            tx.send(Action::LoggedIn).unwrap();
                            tx.send(Action::ChangeMode(Mode::Signup)).unwrap();
                        }
                        Some(false) => {
                            log::error!("Failed to login: {}", data.message);
                            tx.send(Action::Toast("Validation Error".to_string(), data.message))
                                .unwrap();
                            tx.send(Action::EnterNormal).unwrap();
                        }
                        _ => {
                            log::error!("Failed to login");
                            tx.send(Action::Toast("Validation Error".to_string(), data.message))
                                .unwrap();
                            tx.send(Action::EnterNormal).unwrap();
                        }
                    }
                }
                Err(err) => {
                    log::error!("Failed to login: {}", err);
                    tx.send(Action::Toast(
                        "Validation Error".to_string(),
                        err.to_string(),
                    ))
                    .unwrap();
                    tx.send(Action::EnterNormal).unwrap();
                }
            };
        });
    }

    pub fn keymap(mut self, keymap: HashMap<KeyEvent, Action>) -> Self {
        self.keymap = keymap;
        self
    }
}

impl Component for Login {
    fn current_mode(&self) -> InputMode {
        self.mode
    }
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.action_tx = Some(tx.clone());
        self.loader.action_tx = Some(tx);
        Ok(())
    }

    fn register_layout_handler(&mut self, area: Rect) -> Result<()> {
        self.area = area;
        let horizontal = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Min(10),
            Constraint::Fill(1),
        ]);
        let vertical = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Max(3),
            Constraint::Max(3),
            Constraint::Max(1),
            Constraint::Max(1),
            Constraint::Fill(1),
        ])
        .horizontal_margin(10)
        .spacing(Spacing::Space(1));
        let buttons = Layout::horizontal([Constraint::Length(12), Constraint::Length(12)])
            .horizontal_margin(3)
            .spacing(Spacing::Space(1));

        let local_button = Layout::horizontal([Constraint::Max(24)])
            .horizontal_margin(3)
            .spacing(Spacing::Space(1));

        // get areas for layouts
        let [_, center_area, _] = horizontal.areas(self.area);
        let [_, userinput_area, passinput_area, button_area, bottom_area, _] =
            vertical.areas(center_area);
        let [submit_area, register_area] = buttons.areas(button_area);
        let [local_area] = local_button.areas(bottom_area);

        self.loader.register_layout_handler(passinput_area)?;

        self.areas = [
            userinput_area,
            passinput_area,
            submit_area,
            register_area,
            local_area,
        ]
        .to_vec();
        Ok(())
    }

    fn handle_mouse_events(&mut self, mouse: MouseEvent) -> Result<Option<Action>> {
        let tx = self.action_tx.clone().unwrap();
        for (i, area) in self.areas.iter().enumerate() {
            if let Some(mouse) = self.mouse {
                if mouse.kind == MouseEventKind::Down(MouseButton::Left)
                    && area.contains(Position::new(
                        self.mouse.unwrap().column,
                        self.mouse.unwrap().row,
                    ))
                {
                    // Handle click event
                    self.menu_index = i;
                    match Items::ALL[i] {
                        Items::Email => {
                            self.mode = InputMode::InsertUser;
                            tx.send(Action::CompleteInput(self.email.value().to_string()))?;
                        }
                        Items::Password => {
                            self.mode = InputMode::InsertPass;
                            tx.send(Action::CompleteInput(self.password.value().to_string()))?;
                        }
                        Items::Submit => tx.send(Action::Login)?,
                        Items::Switch => tx.send(Action::Register)?,
                        // Change later to bring to local user setup screen
                        Items::Local => tx.send(Action::Home)?,
                    }
                }
            }
        }
        Ok(None)
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        let action = match self.mode {
            InputMode::Normal => match key.code {
                KeyCode::Enter => Action::SelectItem,
                _ => return Ok(None),
            },
            InputMode::InsertPass => match key.code {
                KeyCode::Esc => Action::EnterNormal,
                KeyCode::Enter | KeyCode::Tab => {
                    if let Some(sender) = &self.action_tx {
                        if let Err(e) =
                            sender.send(Action::CompleteInput(self.password.value().to_string()))
                        {
                            error!("Failed to send action: {:?}", e);
                        }
                    }
                    self.menu_index += 1;
                    Action::EnterNormal
                }
                KeyCode::BackTab => {
                    if let Some(sender) = &self.action_tx {
                        if let Err(e) =
                            sender.send(Action::CompleteInput(self.password.value().to_string()))
                        {
                            error!("Failed to send action: {:?}", e);
                        }
                    }
                    self.menu_index -= 1;
                    self.mode = InputMode::InsertUser;
                    return Ok(None);
                }
                _ => {
                    self.password
                        .handle_event(&crossterm::event::Event::Key(key));
                    return Ok(None);
                }
            },
            InputMode::InsertUser => match key.code {
                KeyCode::Esc => Action::EnterNormal,
                KeyCode::Enter | KeyCode::Tab => {
                    if let Some(sender) = &self.action_tx {
                        if let Err(e) =
                            sender.send(Action::CompleteInput(self.email.value().to_string()))
                        {
                            error!("Failed to send action: {:?}", e);
                        }
                    }
                    self.mode = InputMode::InsertPass;
                    self.menu_index += 1;
                    return Ok(None);
                }
                KeyCode::BackTab => {
                    if let Some(sender) = &self.action_tx {
                        if let Err(e) =
                            sender.send(Action::CompleteInput(self.password.value().to_string()))
                        {
                            error!("Failed to send action: {:?}", e);
                        }
                    }
                    self.menu_index += 1;
                    Action::EnterNormal
                }
                _ => {
                    self.email.handle_event(&crossterm::event::Event::Key(key));
                    return Ok(None);
                }
            },
            _ => return Ok(None),
        };
        Ok(Some(action))
    }

    fn update(&mut self, action: Action, ctx: &Ctx) -> Result<Option<Action>> {
        self.loader.update(action.clone(), ctx)?;
        match action {
            Action::Render => self.render_tick(),
            Action::Mouse(mouse) => self.mouse = Some(mouse),
            Action::Forward => {
                if self.mode == InputMode::Normal {
                    self.menu_index = (self.menu_index + 1) % Items::ALL.len();
                }
            }
            Action::Back => {
                if self.mode == InputMode::Normal {
                    if self.menu_index == 0 {
                        self.menu_index = Items::ALL.len() - 1;
                    } else {
                        self.menu_index = (self.menu_index - 1) % Items::ALL.len();
                    }
                }
            }
            Action::EnterProcessing => {
                self.mode = InputMode::Processing;
            }
            Action::Login => self.login(),
            Action::Register => self.register(),
            Action::Home => {
                self.mode = InputMode::Normal;
                return Ok(Some(Action::ChangeMode(Mode::Home)));
            }
            Action::EnterNormal => {
                self.mode = InputMode::Normal;
            }
            Action::SelectItem => {
                match Items::ALL[self.menu_index] {
                    Items::Email => self.mode = InputMode::InsertUser,
                    Items::Password => self.mode = InputMode::InsertPass,
                    Items::Switch => {
                        return Ok(Some(Action::Register));
                    }
                    Items::Submit => {
                        return Ok(Some(Action::Login));
                    }
                    Items::Local => {
                        return Ok(Some(Action::Home));
                    }
                };
            }
            _ => (),
        }
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>) -> Result<()> {
        // Create a block for the input area

        let uwidth = self.areas[0].width.max(3) - 3; // keep 2 for borders and 1 for cursor
        let uscroll = self.email.visual_scroll(uwidth as usize);
        let user_input = Paragraph::new(self.email.value())
            .style(match Items::ALL[self.menu_index] {
                Items::Email => Style::default().fg(Color::Yellow),
                _ => Style::default(),
            })
            .scroll((0, uscroll as u16))
            .block(Block::default().borders(Borders::ALL).title_bottom("Email"));
        let pwidth = self.areas[1].width.max(3) - 3; // keep 2 for borders and 1 for cursor
        let pscroll = self.password.visual_scroll(pwidth as usize);
        let pass_input = Paragraph::new("•".repeat(self.password.value().len()))
            .style(match Items::ALL[self.menu_index] {
                Items::Password => Style::default().fg(Color::Yellow),
                _ => Style::default(),
            })
            .scroll((0, pscroll as u16))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title_bottom("Password"),
            );
        let submit_button = Paragraph::new("Sign In")
            .block(Block::new().style(match Items::ALL[self.menu_index] {
                Items::Submit => Style::default().bg(Color::Yellow).fg(Color::White),
                _ => Style::default().bg(Color::DarkGray).fg(Color::White),
            }))
            .alignment(ratatui::layout::Alignment::Center);
        let register_button = Paragraph::new("Register")
            .block(Block::new().style(match Items::ALL[self.menu_index] {
                Items::Switch => Style::default().bg(Color::Yellow).fg(Color::White),
                _ => Style::default().bg(Color::DarkGray).fg(Color::White),
            }))
            .alignment(ratatui::layout::Alignment::Center);
        let local_button = Paragraph::new("Local Account")
            .block(Block::new().style(match Items::ALL[self.menu_index] {
                Items::Local => Style::default().bg(Color::Yellow).fg(Color::White),
                _ => Style::default().bg(Color::DarkGray).fg(Color::White),
            }))
            .alignment(ratatui::layout::Alignment::Center);

        if self.mode == InputMode::InsertUser {
            f.set_cursor_position(Position::new(
                (self.areas[0].x + 1 + self.email.cursor() as u16)
                    .min(self.areas[0].x + self.areas[0].width - 2),
                self.areas[0].y + 1,
            ))
        }
        if self.mode == InputMode::InsertPass {
            f.set_cursor_position(Position::new(
                (self.areas[1].x + 1 + self.password.cursor() as u16)
                    .min(self.areas[1].x + self.areas[1].width - 2),
                self.areas[1].y + 1,
            ))
        }

        let tx = self.action_tx.clone().unwrap();

        if self.mode == InputMode::Processing {
            self.loader.draw(f)?;
        } else {
            // Render the widgets
            f.render_widget(local_button, self.areas[4]);
            f.render_widget(submit_button, self.areas[2]);
            f.render_widget(register_button, self.areas[3]);
            f.render_widget(user_input, self.areas[0]);
            f.render_widget(pass_input, self.areas[1]);
        }

        Ok(())
    }
}
