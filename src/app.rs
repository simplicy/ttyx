use crate::utils::{Action, Result};
use derive_deref::{Deref, DerefMut};
use ratatui::layout::Rect;
use ratzilla::event::KeyCode;
use ratzilla::event::KeyEvent;
use ratzilla::ratatui::Frame;
use ratzilla::utils;
use std::collections::HashMap;
use tachyonfx::Effect;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::UnboundedSender;

use crate::pages::components::Clip;
use crate::pages::components::Message;
use crate::pages::notfound::NotFound;
use crate::pages::Component;
use crate::pages::Login;
use crate::APP_NAME;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Page {
    #[default]
    Login,
    Home,
    Settings,
    Help,
}

enum InputMode {
    Normal,
    Other,
}
pub struct View(Box<dyn Component>);

#[derive(Deref, DerefMut)]
pub struct Pages(pub HashMap<Page, View>);

#[derive(Deref, DerefMut)]
pub struct UiComponents(Vec<View>);

/// App holds the state of the application
pub struct App {
    // Tx sender
    tx: Option<UnboundedSender<Action>>,
    /// Current input mode
    input_mode: InputMode,
    // Current application mode/page
    current_mode: Page,
    // Page/view currently being displayed
    pub pages: Pages,
    // Components
    pub components: UiComponents,
}

impl App {
    pub fn new() -> Self {
        let input = Message::new();
        let clip = Clip::new();
        let login = Login::new();
        Self {
            tx: None,
            components: UiComponents(Vec::new()),
            input_mode: InputMode::Normal,
            current_mode: Page::default(),
            pages: Pages(HashMap::from([
                (Page::Login, View(Box::new(login))),
                (Page::Settings, View(Box::new(input))),
                (Page::Help, View(Box::new(clip))),
            ])),
        }
    }

    pub fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        for (_, page) in self.pages.iter_mut() {
            page.0.register_action_handler(tx.clone())?;
        }
        for component in self.components.iter_mut() {
            component.0.register_action_handler(tx.clone())?;
        }
        Ok(())
    }
    pub fn handle_mouse(&mut self, mouse_event: ratzilla::event::MouseEvent) {
        // handle events for only current page
        self.pages.iter_mut().for_each(|(page_type, page)| {
            if *page_type == self.current_mode {
                page.0.handle_mouse(mouse_event.clone()).ok();
            }
        });
    }

    pub fn handle_events(&mut self, key_event: KeyEvent) {
        let mut handled = None;
        // handle events for only current page
        self.pages.iter_mut().for_each(|(page_type, page)| {
            if *page_type == self.current_mode {
                let handled_page = page.0.handle_events(key_event.clone());
                if handled_page.is_some() {
                    handled = handled_page;
                }
            }
        });
        if handled.is_none() || handled == Some(false) {
            match self.input_mode {
                InputMode::Normal => {
                    match key_event.code {
                        KeyCode::Char('q') => {
                            // Exit application
                            self.current_mode = Page::Settings;
                        }
                        KeyCode::Char('h') => self.current_mode = Page::Login,
                        KeyCode::Char('m') => self.current_mode = Page::Help,
                        _ => {}
                    }
                }
                InputMode::Other => {
                    match key_event.code {
                        KeyCode::Char('q') => {
                            // Exit application
                            self.current_mode = Page::Settings;
                        }
                        KeyCode::Char('h') => {
                            self.current_mode = Page::Home;
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    fn handle_actions(&mut self, rx: &mut UnboundedReceiver<Action>) -> Result<Option<Action>> {
        while let Ok(action) = rx.try_recv() {
            match action {
                Action::ChangePage(page) => {
                    self.current_mode = page;
                }
                Action::SubmitEmail(email) => {
                    // TODO: Subimt to api endpoint
                    self.current_mode = Page::Settings;
                    let tx = match self.tx.clone() {
                        Some(tx) => tx,
                        _ => return Ok(None),
                    };
                    // tokio::spawn(async move {
                    //     tx.send(Action::EnterProcessing).unwrap();
                    //     let req = reqwest::Client::new();
                    //     match req
                    //         .post("http://localhost:8080/api/auth/register")
                    //         .bearer_auth("Some otkne")
                    //         .send()
                    //         .await
                    //     {
                    //         Ok(res) => {
                    //             //convert to Calendar to return
                    //             let data = res.text().await.unwrap_or_default();
                    //             log::info!("Login successful: {:?}", data);
                    //         }
                    //         Err(err) => {
                    //             log::error!("Failed to login: {}", err);
                    //             tx.send(Action::EnterNormal).unwrap();
                    //         }
                    //     };
                    // });
                }
                _ => {}
            }
        }
        Ok(None)
    }

    pub fn run(
        &mut self,
        frame: &mut Frame,
        rx: &mut UnboundedReceiver<Action>,
        tx: &UnboundedSender<Action>,
    ) -> Result<()> {
        // Send over actions to be handled
        self.handle_actions(rx)?;
        // Show page
        match self.pages.get_mut(&self.current_mode) {
            Some(page) => {
                page.0.draw(frame);
            }
            None => NotFound::new().draw(frame),
        }
        // Handle the Window title
        if let Some(get) = self.pages.get(&self.current_mode) {
            utils::set_document_title(&format!("{} - {:?}", APP_NAME, self.current_mode)).ok();
        }
        Ok(())
        //frame.render_effect(&mut self.intro_effect, area, Duration::from_millis(40));
    }
}
