mod app;
mod pages;
mod utils;

use crate::app::App;
use crate::utils::{Action, BackendType, MultiBackendBuilder};
use ratzilla::backend::cursor::CursorShape;
use ratzilla::backend::dom::DomBackendOptions;
use ratzilla::backend::webgl2::WebGl2BackendOptions;
use ratzilla::WebRenderer;
use std::{cell::RefCell, io, rc::Rc};
use tokio::sync::mpsc;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const APP_NAME: &str = env!("CARGO_PKG_NAME");
const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");

fn main() -> io::Result<()> {
    let dom_options = DomBackendOptions::new(None, CursorShape::SteadyUnderScore);

    let webgl2_options = WebGl2BackendOptions::new()
        .cursor_shape(CursorShape::SteadyUnderScore)
        .enable_console_debug_api()
        .enable_mouse_selection();

    let terminal = MultiBackendBuilder::with_fallback(BackendType::Dom)
        .dom_options(dom_options)
        .webgl2_options(webgl2_options)
        .build_terminal()?;

    let app = Rc::new(RefCell::new(App::new()));
    // Register Handler for Events
    let (action_tx, mut action_rx) = mpsc::unbounded_channel::<Action>();
    terminal.on_key_event({
        let event_state = app.clone();
        move |key_event| {
            let mut state = event_state.borrow_mut();
            state.handle_events(key_event);
        }
    });
    app.borrow_mut().register_action_handler(action_tx).unwrap();
    // Run the application
    terminal.draw_web({
        let render_state = app.clone();
        move |frame| {
            App::run(&mut render_state.borrow_mut(), frame, &mut action_rx)
                .expect("Failed to run app");
        }
    });

    Ok(())
}
