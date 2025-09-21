use std::time::Duration;

mod controls;
mod filepicker;
mod filestats;
mod fps;
mod help;
mod loader;
mod log;
mod menu;
mod mouselist;
mod navigation;
mod popup;
mod post;
mod quit;
mod toast;
mod wave;
pub use controls::*;
pub use filepicker::*;
pub use filestats::*;
pub use fps::*;
pub use help::*;
pub use loader::*;
pub use log::*;
pub use menu::*;
pub use mouselist::*;
pub use navigation::*;
pub use popup::*;
pub use post::*;
pub use quit::*;
pub use toast::*;
pub use wave::*;

use crate::utils::action::Action;

pub struct Modal {
    pub title: Option<String>,
    pub content: String,
    pub subaction: Option<Action>,
    pub duration: Duration,
}
