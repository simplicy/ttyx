mod app;
mod backend;
mod fps;
mod pages;

use crate::app::App;
use crate::backend::{BackendType, MultiBackendBuilder};
use ratzilla::backend::cursor::CursorShape;
use ratzilla::backend::dom::DomBackendOptions;
use ratzilla::backend::webgl2::WebGl2BackendOptions;
use ratzilla::WebRenderer;
use std::{cell::RefCell, io, rc::Rc};

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

    terminal.on_key_event({
        let event_state = app.clone();
        move |key_event| {
            let mut state = event_state.borrow_mut();
            state.handle_events(key_event);
        }
    });

    terminal.draw_web({
        let render_state = app.clone();
        move |frame| {
            let state = render_state.borrow();
            state.draw(frame);
        }
    });

    Ok(())
}
