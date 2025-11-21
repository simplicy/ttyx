use std::cell::RefCell;

use layout::Offset;
use ratatui::{style::Stylize, Frame};
use ratzilla::{
    event::KeyEvent,
    ratatui::{prelude::*, widgets::Clear},
    widgets::Hyperlink,
};
use tachyonfx::{
    fx::{self, RepeatMode},
    CenteredShrink, Duration, Effect, EffectRenderer, Interpolation,
};

use crate::pages::Component;

#[derive(Clone)]
pub struct Clip {
    intro_effect: Effect,
    menu_effect: Option<Effect>,
    text: RefCell<String>,
}

impl Default for Clip {
    fn default() -> Self {
        let text = format!(
            "Press Ctrl+C to copy.\n\
            Press Ctrl+V to paste."
        );
        Self {
            text: RefCell::new(text),
            menu_effect: None,
            intro_effect: fx::sequence(&[
                // fx::ping_pong(fx::sweep_in(
                //     Motion::LeftToRight,
                //     10,
                //     0,
                //     Color::Black,
                //     EffectTimer::from_ms(3000, Interpolation::QuadIn),
                // )),
                fx::coalesce((3000, Interpolation::SineOut)),
                fx::sleep(1000),
                fx::repeat(
                    fx::hsl_shift(
                        Some([120.0, 25.0, 25.0]),
                        None,
                        (5000, Interpolation::Linear),
                    ),
                    RepeatMode::Forever,
                ),
            ]),
        }
    }
}

impl Component for Clip {
    fn draw(&mut self, frame: &mut Frame) {
        Clear.render(frame.area(), frame.buffer_mut());
        let area = frame.area().inner_centered(33, 2);
        let main_text = Text::from(vec![
            Line::from("| S Y M P I L |").bold(),
            Line::from("Coming soon...").italic(),
        ]);
        //render_menu(f, state);
        frame.render_widget(main_text.light_green().centered(), area);
        let link = Hyperlink::new("https://github.com/orhun/ratzilla".red());
        frame.render_widget(link, area.offset(Offset { x: 0, y: 4 }));
        frame.render_effect(&mut self.intro_effect, area, Duration::from_millis(40));
    }

    fn handle_events(&mut self, key_event: KeyEvent) -> Option<bool> {
        // match key_event.code {
        //     KeyCode::Char('c') if key_event.ctrl => {
        //         let clip = self.clone();
        //         tokio::spawn({
        //             let text = self.text.borrow().clone();
        //             async move {
        //                 clip.set_clipboard(&text).await;
        //             }
        //         });
        //     }
        //     KeyCode::Char('v') if key_event.ctrl => {
        //         if let Ok(mut text) = self.text.try_borrow_mut() {
        //             let clip = self.clone();
        //             tokio::spawn(async move {
        //                 let clipboard_text = self.get_clipboard().await;
        //                 *text = clipboard_text;
        //             });
        //         }
        //     }
        //     _ => {}
        // }
        None
    }
}
impl Clip {
    pub fn new() -> Self {
        Self::default()
    }
    async fn set_clipboard(&self, text: &str) {
        let window = web_sys::window().unwrap();
        let nav = window.navigator().clipboard();
        let promise = nav.write_text(text);
        wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();
    }

    async fn get_clipboard(&self) -> String {
        let window = web_sys::window().unwrap();
        let nav = window.navigator().clipboard();
        let promise = nav.read_text();
        let result = wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();
        result.as_string().unwrap_or_default()
    }
}
