use strum::Display;

#[derive(Default, Display, Copy, Clone, PartialEq, Eq)]
pub enum InputMode {
    #[default]
    Normal,
    Insert,
    OptionInput,
    InsertUser,
    InsertPass,
    Processing,
    Submit,
    Select,
    Cancel,
}

impl InputMode {
    pub const CONFIRM: &'static [Self] = &[Self::Submit, Self::Cancel];
}
