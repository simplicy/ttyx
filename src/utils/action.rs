use std::{fmt, string::ToString};

use crossterm::event::MouseEvent;
use serde::{
    de::{self, Deserializer, Visitor},
    Deserialize, Serialize,
};
use strum::Display;

use crate::app::Mode;

#[derive(Debug, Hash, Clone, PartialEq, Eq, Serialize, Display, Deserialize)]
pub enum Action {
    Tick,
    Render,
    Resize(u16, u16),
    Mouse(MouseEvent),
    ToggleNav,
    Suspend,
    Resume,
    Quit,
    Refresh,
    Error(String),
    Back,
    Forward,

    ChangeMode(Mode),
    NextView,
    PreviousView,
    PausePlay,

    Settings,
    Home,
    SelectOption,
    OpenFile,
    SelectItem,
    ClosePopup,
    CloseToast,

    ToggleShowHelp,
    ToggleShowQuit,
    ToggleUsers,
    ToggleChats,
    ToggleLog,
    ToggleSidebar,
    OpenFilepicker,
    ScrollUp,
    ScrollDown,

    ScheduleIncrement,
    ScheduleDecrement,
    Increment(usize),
    Decrement(usize),
    CompleteInput(String),
    Login,
    Register,
    Toast(String, String),
    Popup(String, String),
    EnterNormal,
    EnterInput,
    LoggedIn,
    LoggedOut,
    EnterInsert,
    EnterProcessing,
    Cycle,
    Update,
}
