use std::collections::HashMap;

use derive_deref::{Deref, DerefMut};
use ratzilla::event::KeyEvent;
use ratzilla::ratatui::layout::{Constraint, Layout, Position};
use ratzilla::ratatui::style::{Modifier, Style, Stylize};
use ratzilla::ratatui::text::{Line, Span, Text};
use ratzilla::ratatui::widgets::{List, ListItem};
use ratzilla::ratatui::Frame;
use ratzilla::ratatui::{
    style::Color,
    widgets::{Block, Paragraph},
};
use ratzilla::{event::KeyCode, WebRenderer};
use tachyonfx::fx::RepeatMode;
use tachyonfx::{fx, Duration, Effect, EffectRenderer, Interpolation};

use crate::pages::Component;

pub enum InputMode {
    Normal,
    Editing,
}

/// App holds the state of the application
pub struct Login {
    /// Current value of the input box
    input: String,
    /// Position of cursor in the editor area.
    character_index: usize,
    /// Current input mode
    input_mode: InputMode,
    /// History of recorded messages
    messages: Vec<String>,
}

impl Component for Login {
    fn handle_events(&mut self, key_event: KeyEvent) -> Option<bool> {
        match self.input_mode {
            InputMode::Normal => {
                if let KeyCode::Char('e') = key_event.code {
                    self.input_mode = InputMode::Editing;
                }
                None
            }
            InputMode::Editing => {
                match key_event.code {
                    KeyCode::Enter => self.submit_message(),
                    KeyCode::Char(to_insert) => self.enter_char(to_insert),
                    KeyCode::Backspace => self.delete_char(),
                    KeyCode::Left => self.move_cursor_left(),
                    KeyCode::Right => self.move_cursor_right(),
                    KeyCode::Esc => self.input_mode = InputMode::Normal,
                    _ => {}
                }
                Some(true)
            }
        }
    }

    fn draw(&self, frame: &mut Frame) {
        let vertical = Layout::horizontal([
            Constraint::Min(1),
            Constraint::Length(50),
            Constraint::Min(1),
        ]);
        let [_, input_area, _] = vertical.areas(frame.area());
        let input = Layout::vertical([
            Constraint::Min(1),
            Constraint::Length(3),
            Constraint::Min(1),
        ]);
        let [_, input_area, _] = input.areas(input_area);

        let input = Paragraph::new(self.input.as_str())
            .style(match self.input_mode {
                InputMode::Normal => Style::default(),
                InputMode::Editing => Style::default().fg(Color::Yellow),
            })
            .block(Block::bordered().title("Email"));
        frame.render_widget(input, input_area);
        match self.input_mode {
            // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
            InputMode::Normal => {}

            // Make the cursor visible and ask ratatui to put it at the specified coordinates after
            // rendering
            InputMode::Editing => frame.set_cursor_position(Position::new(
                // Draw the cursor at the current position in the input field.
                // This position can be controlled via the left and right arrow key
                input_area.x + self.character_index as u16 + 1,
                // Move one line down, from the border to the input line
                input_area.y + 1,
            )),
        }
        // let mut effect = fx::sequence(&[
        //     fx::coalesce((3000, Interpolation::SineOut)),
        //     fx::sleep(1000),
        // ]);
        // frame.render_effect(&mut effect, input_area, Duration::from_millis(40));
    }
}

impl Login {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            input_mode: InputMode::Normal,
            messages: Vec::new(),
            character_index: 0,
        }
    }
    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.character_index.saturating_sub(1);
        self.character_index = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.character_index.saturating_add(1);
        self.character_index = self.clamp_cursor(cursor_moved_right);
    }

    fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.input.insert(index, new_char);
        self.move_cursor_right();
    }

    /// Returns the byte index based on the character position.
    ///
    /// Since each character in a string can contain multiple bytes, it's necessary to calculate
    /// the byte index based on the index of the character.
    fn byte_index(&self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.input.len())
    }

    fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.character_index != 0;
        if is_not_cursor_leftmost {
            // Method "remove" is not used on the saved text for deleting the selected char.
            // Reason: Using remove on String works on bytes instead of the chars.
            // Using remove would require special care because of char boundaries.

            let current_index = self.character_index;
            let from_left_to_current_index = current_index - 1;

            // Getting all characters before the selected character.
            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
            // Getting all characters after selected character.
            let after_char_to_delete = self.input.chars().skip(current_index);

            // Put all characters together except the selected one.
            // By leaving the selected one out, it is forgotten and therefore deleted.
            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.chars().count())
    }

    fn reset_cursor(&mut self) {
        self.character_index = 0;
    }

    fn submit_message(&mut self) {
        self.messages.push(self.input.clone());
        self.input.clear();
        self.reset_cursor();
    }
}
