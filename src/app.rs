use derive_deref::{Deref, DerefMut};
use ratatui::layout::Rect;
use ratzilla::event::KeyCode;
use ratzilla::event::KeyEvent;
use ratzilla::ratatui::widgets::Block;
use ratzilla::ratatui::Frame;
use std::collections::HashMap;
use tachyonfx::Effect;

use crate::pages::components::Clip;
use crate::pages::components::Login;
use crate::pages::components::Message;
use crate::pages::notfound::NotFound;
use crate::pages::Component;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Page {
    #[default]
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
    content: Box<dyn Component>,
    effect: Option<Effect>,
    exit_effect: Option<Effect>,
    area: Option<Rect>,
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
    pages: Pages,
    // Components
    components: UiComponents,
}

enum InputMode {
    Normal,
    Editing,
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
                    Page::Home,
                    View {
                        title: "Home".to_string(),
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
    pub fn handle_events(&mut self, key_event: KeyEvent) {
        let mut handled = None;
        self.pages.iter_mut().for_each(|(_, page)| {
            let handled_page = page.content.handle_events(key_event.clone());
            if handled_page.is_some() {
                handled = handled_page;
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
                        KeyCode::Char('h') => {
                            self.current_mode = Page::Home;
                        }
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

    pub fn draw(&self, frame: &mut Frame) {
        match self.pages.get(&self.current_mode) {
            Some(page) => {
                page.content.draw(frame);
            }
            None => NotFound::new().draw(frame),
        }
        //frame.render_effect(&mut self.intro_effect, area, Duration::from_millis(40));
    }
}
