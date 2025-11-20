use std::cell::RefCell;

use ratatui::{
    layout::Alignment,
    style::{Color, Stylize},
    widgets::{Block, BorderType, Paragraph},
    Frame,
};

use ratzilla::event::{KeyCode, KeyEvent};

use crate::pages::Component;

#[derive(Clone)]
pub struct Clip {
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
        }
    }
}

impl Component for Clip {
    fn draw(&self, frame: &mut Frame) {
        let block = Block::bordered()
            .title("Clipboard Example")
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded);

        if let Ok(text) = self.text.try_borrow() {
            let paragraph = Paragraph::new(text.to_string())
                .block(block)
                .fg(Color::White)
                .bg(Color::Black)
                .centered();

            frame.render_widget(paragraph, frame.area());
        }
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
