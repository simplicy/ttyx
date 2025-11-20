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

pub struct EffectConfig {
    effect: Option<Effect>,
    duration_ms: Option<u64>,
    interpolation: tachyonfx::Interpolation,
}
impl Default for EffectConfig {
    fn default() -> Self {
        EffectConfig {
            effect: None,
            duration_ms: Some(500),
            interpolation: tachyonfx::Interpolation::Linear,
        }
    }
}

/// used for pages and components
pub struct View {
    title: String,
    pub content: Box<dyn Component>,
    effect: Option<Effect>,
    exit_effect: Option<Effect>,
    area: Option<Rect>,
}
enum InputMode {
    Normal,
    Editing,
}

#[derive(Deref, DerefMut)]
pub struct Pages(pub HashMap<Page, View>);

#[derive(Deref, DerefMut)]
pub struct UiComponents(Vec<View>);

/// App holds the state of the application
pub struct App {
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
            components: UiComponents(Vec::new()),
            input_mode: InputMode::Normal,
            current_mode: Page::default(),
            pages: Pages(HashMap::from([
                (
                    Page::Login,
                    View {
                        title: "Login".to_string(),
                        content: Box::new(login),
                        effect: None,
                        area: None,
                        exit_effect: None,
                    },
                ),
                (
                    Page::Settings,
                    View {
                        title: "Settings".to_string(),
                        area: None,
                        content: Box::new(input),
                        effect: None,
                        exit_effect: None,
                    },
                ),
            ])),
        }
    }

    pub fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        for (_, page) in self.pages.iter_mut() {
            page.content.register_action_handler(tx.clone())?;
        }
        for component in self.components.iter_mut() {
            component.content.register_action_handler(tx.clone())?;
        }
        Ok(())
    }

    pub fn handle_events(&mut self, key_event: KeyEvent) {
        let mut handled = None;
        // handle events for only current page
        self.pages.iter_mut().for_each(|(page_type, page)| {
            if *page_type == self.current_mode {
                let handled_page = page.content.handle_events(key_event.clone());
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
                        _ => {}
                    }
                }
                InputMode::Editing => {
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
                _ => {}
            }
        }
        Ok(None)
    }

    pub fn run(&mut self, frame: &mut Frame, rx: &mut UnboundedReceiver<Action>) -> Result<()> {
        // Send over actions to be handled
        self.handle_actions(rx)?;
        // Show page
        match self.pages.get(&self.current_mode) {
            Some(page) => {
                page.content.draw(frame);
            }
            None => NotFound::new().draw(frame),
        }
        // Handle the Window title
        if self.pages.get(&self.current_mode).is_some() {
            utils::set_document_title(&format!(
                "{} - {}",
                APP_NAME,
                self.pages
                    .get(&self.current_mode)
                    .map(|p| p.title.as_str())
                    .unwrap_or("Not Found"),
            ))
            .ok();
        }
        Ok(())
        //frame.render_effect(&mut self.intro_effect, area, Duration::from_millis(40));
    }
}
