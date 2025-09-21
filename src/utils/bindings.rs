use crate::app::Mode;
use crate::utils::error::{Error, Result};
use config::{Config, File};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use derive_deref::Deref;
use derive_deref::DerefMut;
use ratatui::style::{Color, Modifier, Style};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_with_macros::skip_serializing_none;
use std::collections::HashMap;
use std::path::Path;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const APP_NAME: &str = env!("CARGO_PKG_NAME");
use std::path::PathBuf;

use super::action::Action;
use super::Args;

#[derive(Clone, Debug, Deref, DerefMut)]
pub struct KeyBindings(pub HashMap<Mode, HashMap<Vec<KeyEvent>, Action>>);

impl Default for KeyBindings {
    fn default() -> Self {
        KeyBindings(HashMap::from([
            // Global Bindings
            (
                Mode::Global,
                HashMap::from([
                    (
                        vec![KeyEvent {
                            code: KeyCode::Char('q'),
                            modifiers: KeyModifiers::empty(),
                            kind: KeyEventKind::Press,
                            state: KeyEventState::NONE,
                        }],
                        Action::ToggleShowQuit,
                    ),
                    (
                        vec![KeyEvent {
                            code: KeyCode::Char(':'),
                            modifiers: KeyModifiers::empty(),
                            kind: KeyEventKind::Press,
                            state: KeyEventState::NONE,
                        }],
                        Action::ToggleLog,
                    ),
                    (
                        vec![KeyEvent {
                            code: KeyCode::Char(' '),
                            modifiers: KeyModifiers::CONTROL,
                            kind: KeyEventKind::Press,
                            state: KeyEventState::NONE,
                        }],
                        Action::ToggleShowHelp,
                    ),
                    (
                        vec![KeyEvent {
                            code: KeyCode::Char('x'),
                            modifiers: KeyModifiers::empty(),
                            kind: KeyEventKind::Press,
                            state: KeyEventState::NONE,
                        }],
                        Action::ClosePopup,
                    ),
                    (
                        vec![KeyEvent {
                            code: KeyCode::Tab,
                            modifiers: KeyModifiers::empty(),
                            kind: KeyEventKind::Press,
                            state: KeyEventState::NONE,
                        }],
                        Action::ToggleNav,
                    ),
                    (
                        vec![KeyEvent {
                            code: KeyCode::Char('h'),
                            modifiers: KeyModifiers::empty(),
                            kind: KeyEventKind::Press,
                            state: KeyEventState::NONE,
                        }],
                        Action::PreviousView,
                    ),
                    (
                        vec![KeyEvent {
                            code: KeyCode::Char('l'),
                            modifiers: KeyModifiers::empty(),
                            kind: KeyEventKind::Press,
                            state: KeyEventState::NONE,
                        }],
                        Action::NextView,
                    ),
                    (
                        vec![KeyEvent {
                            code: KeyCode::Char(' '),
                            modifiers: KeyModifiers::empty(),
                            kind: KeyEventKind::Press,
                            state: KeyEventState::NONE,
                        }],
                        Action::ToggleShowHelp,
                    ),
                    (
                        vec![KeyEvent {
                            code: KeyCode::Char('k'),
                            modifiers: KeyModifiers::empty(),
                            kind: KeyEventKind::Press,
                            state: KeyEventState::NONE,
                        }],
                        Action::Back,
                    ),
                    (
                        vec![KeyEvent {
                            code: KeyCode::Char('j'),
                            modifiers: KeyModifiers::empty(),
                            kind: KeyEventKind::Press,
                            state: KeyEventState::NONE,
                        }],
                        Action::Forward,
                    ),
                ]),
            ),
            // Login Bindings
            // Settings Bindings
            (
                Mode::Settings,
                HashMap::from([(
                    vec![KeyEvent {
                        code: KeyCode::Backspace,
                        modifiers: KeyModifiers::empty(),
                        kind: KeyEventKind::Press,
                        state: KeyEventState::NONE,
                    }],
                    Action::Home,
                )]),
            ),
            // Chat Bindings
            (
                Mode::Chat,
                HashMap::from([
                    (
                        vec![KeyEvent {
                            code: KeyCode::Char('/'),
                            modifiers: KeyModifiers::empty(),
                            kind: KeyEventKind::Press,
                            state: KeyEventState::NONE,
                        }],
                        Action::EnterInput,
                    ),
                    (
                        vec![KeyEvent {
                            code: KeyCode::Char('c'),
                            modifiers: KeyModifiers::empty(),
                            kind: KeyEventKind::Press,
                            state: KeyEventState::NONE,
                        }],
                        Action::ToggleChats,
                    ),
                    (
                        vec![KeyEvent {
                            code: KeyCode::Char('u'),
                            modifiers: KeyModifiers::empty(),
                            kind: KeyEventKind::Press,
                            state: KeyEventState::NONE,
                        }],
                        Action::ToggleUsers,
                    ),
                    (
                        vec![KeyEvent {
                            code: KeyCode::Char('k'),
                            modifiers: KeyModifiers::empty(),
                            kind: KeyEventKind::Press,
                            state: KeyEventState::NONE,
                        }],
                        Action::Back,
                    ),
                    (
                        vec![KeyEvent {
                            code: KeyCode::Char('j'),
                            modifiers: KeyModifiers::empty(),
                            kind: KeyEventKind::Press,
                            state: KeyEventState::NONE,
                        }],
                        Action::Forward,
                    ),
                ]),
            ),
            (Mode::Map, HashMap::from([])),
            (
                Mode::Filebrowser,
                HashMap::from([
                    (
                        vec![KeyEvent {
                            code: KeyCode::Tab,
                            modifiers: KeyModifiers::empty(),
                            kind: KeyEventKind::Press,
                            state: KeyEventState::NONE,
                        }],
                        Action::ToggleSidebar,
                    ),
                    (
                        vec![KeyEvent {
                            code: KeyCode::Enter,
                            modifiers: KeyModifiers::empty(),
                            kind: KeyEventKind::Press,
                            state: KeyEventState::NONE,
                        }],
                        Action::SelectOption,
                    ),
                ]),
            ),
            (
                Mode::MusicPlayer,
                HashMap::from([
                    (
                        vec![KeyEvent {
                            code: KeyCode::Tab,
                            modifiers: KeyModifiers::empty(),
                            kind: KeyEventKind::Press,
                            state: KeyEventState::NONE,
                        }],
                        Action::ToggleSidebar,
                    ),
                    (
                        vec![KeyEvent {
                            code: KeyCode::Char(' '),
                            modifiers: KeyModifiers::empty(),
                            kind: KeyEventKind::Press,
                            state: KeyEventState::NONE,
                        }],
                        Action::PausePlay,
                    ),
                    (
                        vec![KeyEvent {
                            code: KeyCode::Char('o'),
                            modifiers: KeyModifiers::empty(),
                            kind: KeyEventKind::Press,
                            state: KeyEventState::NONE,
                        }],
                        Action::OpenFilepicker,
                    ),
                    (
                        vec![KeyEvent {
                            code: KeyCode::Enter,
                            modifiers: KeyModifiers::empty(),
                            kind: KeyEventKind::Press,
                            state: KeyEventState::NONE,
                        }],
                        Action::SelectOption,
                    ),
                ]),
            ),
            (
                Mode::Blog,
                HashMap::from([
                    (
                        vec![KeyEvent {
                            code: KeyCode::Tab,
                            modifiers: KeyModifiers::empty(),
                            kind: KeyEventKind::Press,
                            state: KeyEventState::NONE,
                        }],
                        Action::ToggleSidebar,
                    ),
                    (
                        vec![KeyEvent {
                            code: KeyCode::Char('f'),
                            modifiers: KeyModifiers::empty(),
                            kind: KeyEventKind::Press,
                            state: KeyEventState::NONE,
                        }],
                        Action::OpenFilepicker,
                    ),
                    (
                        vec![KeyEvent {
                            code: KeyCode::Enter,
                            modifiers: KeyModifiers::empty(),
                            kind: KeyEventKind::Press,
                            state: KeyEventState::NONE,
                        }],
                        Action::SelectOption,
                    ),
                ]),
            ),
            (
                Mode::Login,
                HashMap::from([
                    (
                        vec![KeyEvent {
                            code: KeyCode::BackTab,
                            modifiers: KeyModifiers::SHIFT,
                            kind: KeyEventKind::Press,
                            state: KeyEventState::NONE,
                        }],
                        Action::Back,
                    ),
                    (
                        vec![KeyEvent {
                            code: KeyCode::Tab,
                            modifiers: KeyModifiers::empty(),
                            kind: KeyEventKind::Press,
                            state: KeyEventState::NONE,
                        }],
                        Action::Forward,
                    ),
                ]),
            ),
            // Homepage bindings
            (
                Mode::Home,
                HashMap::from([(
                    vec![KeyEvent {
                        code: KeyCode::Char('/'),
                        modifiers: KeyModifiers::empty(),
                        kind: KeyEventKind::Press,
                        state: KeyEventState::NONE,
                    }],
                    Action::EnterInsert,
                )]),
            ),
        ]))
    }
}

impl Serialize for KeyBindings {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut top = serializer.serialize_map(Some(self.0.len()))?;

        for (mode, bindings) in &self.0 {
            // For each Vec<KeyEvent>, produce a unique and human-readable string
            let serialized_bindings: HashMap<String, &Action> = bindings
                .iter()
                .map(|(key_seq, action)| {
                    let key_str = key_seq
                        .iter()
                        .map(|k| "<".to_owned() + &k.code.to_string() + ">")
                        .collect::<Vec<_>>()
                        .join(", ");
                    (key_str, action)
                })
                .collect();
            top.serialize_entry(mode, &serialized_bindings)?;
        }
        top.end()
    }
}

impl<'de> Deserialize<'de> for KeyBindings {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let parsed_map = HashMap::<Mode, HashMap<String, Action>>::deserialize(deserializer)?;

        let keybindings = parsed_map
            .into_iter()
            .map(|(mode, inner_map)| {
                let converted_inner_map = inner_map
                    .into_iter()
                    .map(|(key_str, cmd)| (parse_key_sequence(&key_str).unwrap(), cmd))
                    .collect();
                (mode, converted_inner_map)
            })
            .collect();

        Ok(KeyBindings(keybindings))
    }
}
fn parse_key_event(raw: &str) -> Result<KeyEvent> {
    let raw_lower = raw.to_ascii_lowercase();
    let (remaining, modifiers) = extract_modifiers(&raw_lower);
    parse_key_code_with_modifiers(remaining, modifiers)
}

fn extract_modifiers(raw: &str) -> (&str, KeyModifiers) {
    let mut modifiers = KeyModifiers::empty();
    let mut current = raw;

    loop {
        match current {
            rest if rest.starts_with("ctrl-") => {
                modifiers.insert(KeyModifiers::CONTROL);
                current = &rest[5..];
            }
            rest if rest.starts_with("alt-") => {
                modifiers.insert(KeyModifiers::ALT);
                current = &rest[4..];
            }
            rest if rest.starts_with("shift-") => {
                modifiers.insert(KeyModifiers::SHIFT);
                current = &rest[6..];
            }
            _ => break, // break out of the loop if no known prefix is detected
        };
    }

    (current, modifiers)
}

fn parse_key_code_with_modifiers(raw: &str, mut modifiers: KeyModifiers) -> Result<KeyEvent> {
    let c = match raw {
        "esc" => KeyCode::Esc,
        "enter" => KeyCode::Enter,
        "left" => KeyCode::Left,
        "right" => KeyCode::Right,
        "up" => KeyCode::Up,
        "down" => KeyCode::Down,
        "home" => KeyCode::Home,
        "end" => KeyCode::End,
        "pageup" => KeyCode::PageUp,
        "pagedown" => KeyCode::PageDown,
        "backtab" => {
            modifiers.insert(KeyModifiers::SHIFT);
            KeyCode::BackTab
        }
        "back tab" => {
            modifiers.insert(KeyModifiers::SHIFT);
            KeyCode::BackTab
        }
        "backspace" => KeyCode::Backspace,
        "delete" => KeyCode::Delete,
        "insert" => KeyCode::Insert,
        "f1" => KeyCode::F(1),
        "f2" => KeyCode::F(2),
        "f3" => KeyCode::F(3),
        "f4" => KeyCode::F(4),
        "f5" => KeyCode::F(5),
        "f6" => KeyCode::F(6),
        "f7" => KeyCode::F(7),
        "f8" => KeyCode::F(8),
        "f9" => KeyCode::F(9),
        "f10" => KeyCode::F(10),
        "f11" => KeyCode::F(11),
        "f12" => KeyCode::F(12),
        "space" => KeyCode::Char(' '),
        "hyphen" => KeyCode::Char('-'),
        "minus" => KeyCode::Char('-'),
        "tab" => KeyCode::Tab,
        c if c.len() == 1 => {
            let mut c = c.chars().next().unwrap();
            if modifiers.contains(KeyModifiers::SHIFT) {
                c = c.to_ascii_uppercase();
            }
            KeyCode::Char(c)
        }
        _ => return Err(Error::InvalidKeyEvent(raw.to_string())),
    };
    Ok(KeyEvent::new(c, modifiers))
}

pub fn parse_key_sequence(raw: &str) -> Result<Vec<KeyEvent>> {
    if raw.chars().filter(|c| *c == '>').count() != raw.chars().filter(|c| *c == '<').count() {
        return Err(Error::InvalidKeyEvent(raw.to_string()));
    }
    let raw = if !raw.contains("><") {
        let raw = raw.strip_prefix('<').unwrap_or(raw);
        let raw = raw.strip_prefix('>').unwrap_or(raw);
        raw
    } else {
        raw
    };
    let sequences = raw
        .split("><")
        .map(|seq| {
            if let Some(s) = seq.strip_prefix('<') {
                s
            } else if let Some(s) = seq.strip_suffix('>') {
                s
            } else {
                seq
            }
        })
        .collect::<Vec<_>>();

    sequences.into_iter().map(parse_key_event).collect()
}

pub fn key_event_to_string(key_event: &KeyEvent) -> String {
    let char;
    let key_code = match key_event.code {
        KeyCode::Backspace => "backspace",
        KeyCode::Enter => "enter",
        KeyCode::Left => "left",
        KeyCode::Right => "right",
        KeyCode::Up => "up",
        KeyCode::Down => "down",
        KeyCode::Home => "home",
        KeyCode::End => "end",
        KeyCode::PageUp => "pageup",
        KeyCode::PageDown => "pagedown",
        KeyCode::Tab => "tab",
        KeyCode::BackTab => "backtab",
        KeyCode::Delete => "delete",
        KeyCode::Insert => "insert",
        KeyCode::F(c) => {
            char = format!("f({c})");
            &char
        }
        KeyCode::Char(' ') => "space",
        KeyCode::Char(c) => {
            char = c.to_string();
            &char
        }
        KeyCode::Esc => "esc",
        KeyCode::Null => "",
        KeyCode::CapsLock => "",
        KeyCode::Menu => "",
        KeyCode::ScrollLock => "",
        KeyCode::Media(_) => "",
        KeyCode::NumLock => "",
        KeyCode::PrintScreen => "",
        KeyCode::Pause => "",
        KeyCode::KeypadBegin => "",
        KeyCode::Modifier(_) => "",
    };

    let mut modifiers = Vec::with_capacity(3);

    if key_event.modifiers.intersects(KeyModifiers::CONTROL) {
        modifiers.push("ctrl");
    }

    if key_event.modifiers.intersects(KeyModifiers::SHIFT) {
        modifiers.push("shift");
    }

    if key_event.modifiers.intersects(KeyModifiers::ALT) {
        modifiers.push("alt");
    }

    let mut key = modifiers.join("-");

    if !key.is_empty() {
        key.push('-');
    }
    key.push_str(key_code);

    key
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::utils::key_event_to_string;

    use super::*;

    #[test]
    fn test_simple_keys() {
        assert_eq!(
            parse_key_event("a").unwrap(),
            KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty())
        );

        assert_eq!(
            parse_key_event("enter").unwrap(),
            KeyEvent::new(KeyCode::Enter, KeyModifiers::empty())
        );

        assert_eq!(
            parse_key_event("esc").unwrap(),
            KeyEvent::new(KeyCode::Esc, KeyModifiers::empty())
        );
    }

    #[test]
    fn test_with_modifiers() {
        assert_eq!(
            parse_key_event("ctrl-a").unwrap(),
            KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL)
        );

        assert_eq!(
            parse_key_event("alt-enter").unwrap(),
            KeyEvent::new(KeyCode::Enter, KeyModifiers::ALT)
        );

        assert_eq!(
            parse_key_event("shift-esc").unwrap(),
            KeyEvent::new(KeyCode::Esc, KeyModifiers::SHIFT)
        );
    }

    #[test]
    fn test_multiple_modifiers() {
        assert_eq!(
            parse_key_event("ctrl-alt-a").unwrap(),
            KeyEvent::new(
                KeyCode::Char('a'),
                KeyModifiers::CONTROL | KeyModifiers::ALT
            )
        );

        assert_eq!(
            parse_key_event("ctrl-shift-enter").unwrap(),
            KeyEvent::new(KeyCode::Enter, KeyModifiers::CONTROL | KeyModifiers::SHIFT)
        );
    }

    #[test]
    fn test_reverse_multiple_modifiers() {
        assert_eq!(
            key_event_to_string(&KeyEvent::new(
                KeyCode::Char('a'),
                KeyModifiers::CONTROL | KeyModifiers::ALT
            )),
            "ctrl-alt-a".to_string()
        );
    }

    #[test]
    fn test_invalid_keys() {
        assert!(parse_key_event("invalid-key").is_err());
        assert!(parse_key_event("ctrl-invalid-key").is_err());
    }

    #[test]
    fn test_case_insensitivity() {
        assert_eq!(
            parse_key_event("CTRL-a").unwrap(),
            KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL)
        );

        assert_eq!(
            parse_key_event("AlT-eNtEr").unwrap(),
            KeyEvent::new(KeyCode::Enter, KeyModifiers::ALT)
        );
    }
}
