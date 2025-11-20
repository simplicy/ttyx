pub mod components;
pub mod notfound;
use ratzilla::{
    event::KeyEvent,
    ratatui::{layout::Rect, Frame},
};

pub trait Component {
    // #[allow(unused_variables)]
    // fn register_layout_handler(&mut self, area: Rect) -> Result<()> {
    //     Ok(())
    // }
    // fn unfocus(&mut self) -> Result<()> {
    //     Ok(())
    // }
    //
    // fn focus(&mut self) -> Result<()> {
    //     Ok(())
    // }
    //
    // fn is_focused(&self) -> bool {
    //     true
    // }

    #[allow(unused_variables)]
    fn handle_events(&mut self, key: KeyEvent) -> Option<bool> {
        None
    }
    // #[allow(unused_variables)]
    // fn handle_mouse_events(mouse: MouseEvent) -> Result<Option<Action>> {
    //     Ok(None)
    // }
    // #[allow(unused_variables)]
    // fn update(&mut self, action: Action, ctx: &Ctx) -> Result<Option<Action>> {
    //     Ok(None)
    // }
    fn draw(&self, f: &mut Frame<'_>) {}
}
