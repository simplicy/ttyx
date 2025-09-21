use std::{collections::HashMap, fmt::Display, time::Duration};

use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, MouseEvent};
use log::error;
use ratatui::{prelude::*, widgets::*};
use tokio::sync::mpsc::UnboundedSender;
use tracing::trace;
use tui_input::{backend::crossterm::EventHandler, Input};

use super::{Component, Frame, InputMode, StatefulList};
use crate::utils::{action::Action, key_event_to_string, AppConfiguration, Ctx};

#[derive(Default, Copy, Clone, PartialEq, Eq)]
pub enum SubMenuOption {
    #[default]
    BackgroundColor,
    ForegroundColor,
    FontSize,
    FontFamily,
    // Account related options
    Privacy,
    AccountColor,
    Language,
    Notifications,
    // Help related options
    Faq,
    Support,
}
impl Display for SubMenuOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SubMenuOption::Privacy => write!(f, "Privacy"),
            SubMenuOption::AccountColor => write!(f, "Account Color"),
            SubMenuOption::BackgroundColor => write!(f, "Background Color"),
            SubMenuOption::ForegroundColor => write!(f, "Foreground Color"),
            SubMenuOption::FontSize => write!(f, "Font Size"),
            SubMenuOption::FontFamily => write!(f, "Font Family"),
            SubMenuOption::Language => write!(f, "Language"),
            SubMenuOption::Notifications => write!(f, "Notifications"),
            SubMenuOption::Faq => write!(f, "FAQ"),
            SubMenuOption::Support => write!(f, "Support"),
        }
    }
}
impl SubMenuOption {
    const ALL: &'static [SubMenuOption] = &[
        Self::BackgroundColor,
        Self::ForegroundColor,
        Self::FontSize,
        Self::FontFamily,
        Self::Privacy,
        Self::AccountColor,
        Self::Language,
        Self::Notifications,
        Self::Faq,
        Self::Support,
    ];
    const THEME: &'static [SubMenuOption] = &[
        Self::BackgroundColor,
        Self::ForegroundColor,
        Self::FontSize,
        Self::FontFamily,
    ];
    const ACCOUNT: &'static [SubMenuOption] = &[
        Self::Privacy,
        Self::AccountColor,
        Self::Language,
        Self::Notifications,
    ];
    const HELP: &'static [SubMenuOption] = &[Self::Faq, Self::Support];
}

#[derive(Default)]
pub struct Settings {
    pub show: bool,
    pub mode: InputMode,
    pub input: Input,
    pub options: StatefulList<SubMenuOption>,
    pub selected_option: Option<SubMenuOption>,
    pub action_tx: Option<UnboundedSender<Action>>,
    pub keymap: HashMap<KeyEvent, Action>,
    config: AppConfiguration,
    area: Rect,
    areas: Vec<Rect>,
}

impl Settings {
    pub fn new(conf: AppConfiguration) -> Self {
        Self {
            config: conf,
            options: StatefulList::with_items(SubMenuOption::ALL.to_vec()),
            ..Default::default()
        }
    }
    fn draw_faq(&self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        Ok(())
    }
    fn draw_support(&self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        let layout = Layout::vertical([Constraint::Min(1)]);
        let [content_area] = layout.areas(area);
        // Make list of all config items
        let content_block =
            Paragraph::new("For support, please visit our website or contact us via email.").block(
                Block::default()
                    .borders(Borders::RIGHT)
                    .border_style(Style::default()),
            );
        f.render_widget(content_block, content_area);
        Ok(())
    }

    pub fn keymap(mut self, keymap: HashMap<KeyEvent, Action>) -> Self {
        self.keymap = keymap;
        self
    }
}

impl Component for Settings {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.action_tx = Some(tx);
        Ok(())
    }

    fn default_mode(&self) -> InputMode {
        InputMode::Normal
    }

    fn current_mode(&self) -> InputMode {
        self.mode
    }

    fn register_layout_handler(&mut self, area: Rect) -> Result<()> {
        self.area = area;
        let vertical = Layout::vertical([Constraint::Min(1)]);
        let [main_area] = vertical.areas(area);
        let longest_item = SubMenuOption::ALL
            .iter()
            .map(|setting| setting.to_string().len())
            .max()
            .unwrap_or(0);

        let main_layout = Layout::horizontal([
            Constraint::Length(longest_item as u16 + 3),
            Constraint::Min(1),
        ])
        .vertical_margin(0);
        let main_block = Block::default();
        let [option_area, content_area] = main_layout.areas(main_area);
        let option_area = main_block.inner(option_area);
        let content_area = main_block.inner(content_area);

        self.areas = vec![main_area, option_area, content_area];
        Ok(())
    }

    fn handle_mouse_events(&mut self, mouse: MouseEvent) -> Result<Option<Action>> {
        let tx = self.action_tx.clone().unwrap();

        Ok(None)
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        let action = match self.mode {
            InputMode::Normal => match key.code {
                KeyCode::Esc => Action::EnterNormal,
                KeyCode::Enter => Action::SelectOption,
                _ => {
                    self.input.handle_event(&crossterm::event::Event::Key(key));
                    Action::Update
                }
            },
            InputMode::OptionInput => {
                match key.code {
                    KeyCode::Esc => Action::EnterNormal,
                    KeyCode::Backspace => {
                        self.mode = InputMode::Normal;
                        Action::Update
                    }
                    KeyCode::Enter => Action::SelectOption,
                    _ => {
                        if let Some(action) = self.keymap.get(&key) {
                            trace!(
                                "Key event: {} -> Action: {:?}",
                                key_event_to_string(&key),
                                action
                            );
                            return Ok(Some(action.clone()));
                        }
                        // If no action is found, we can just return None
                        return Ok(None);
                    }
                }
            }
            _ => return Ok(None),
        };
        Ok(Some(action))
    }

    fn update(&mut self, action: Action, ctx: &Ctx) -> Result<Option<Action>> {
        match action {
            Action::EnterNormal => {
                self.mode = InputMode::Normal;
                self.selected_option = None;
            }
            Action::SelectOption => {
                self.selected_option = self
                    .options
                    .state
                    .selected()
                    .and_then(|i| self.options.items.get(i).cloned())
            }
            Action::Forward => {
                match self.selected_option {
                    Some(thing) => {
                        // self.submenulist.next()
                    }
                    _ => self.options.next(),
                }
            }
            Action::Back => match self.selected_option {
                Some(thing) => {
                    // self.submenulist.previous()
                }
                _ => self.options.previous(),
            },
            _ => (),
        }
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>) -> Result<()> {
        // Make list of all config items
        let text = self
            .options
            .items
            .iter()
            .enumerate()
            .map(|(i, mode)| ListItem::new(vec![Line::from(mode.to_string())]))
            .collect::<Vec<_>>();

        let options = List::new(text).highlight_style(match self.selected_option {
            Some(thing) => Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(Color::Cyan)
                .fg(Color::Black),
            _ => Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(Color::Yellow)
                .fg(Color::White),
        });

        let option_block = Block::default()
            .borders(Borders::RIGHT)
            .border_style(Style::default());

        let options = options.block(option_block);
        match self.options.state.selected() {
            Some(index) => {
                if let Some(selected) = self.options.items.get(index) {
                    match selected {
                        SubMenuOption::Support => self.draw_support(f, self.areas[2])?,
                        SubMenuOption::Faq => self.draw_faq(f, self.areas[2])?,
                        __ => {
                            // Render the content area with the selected option
                            let content_text = format!("You selected: {}", selected);
                            let content_paragraph = Paragraph::new(content_text)
                                .wrap(Wrap { trim: false })
                                .block(Block::default().borders(Borders::ALL).title("Details"));
                            f.render_widget(content_paragraph, self.areas[2]);
                        }
                    }
                } else {
                    error!("Selected index {} out of bounds", index);
                }
            }
            None => {
                // If no item is selected, we can just clear the content area
                f.render_widget(Clear, self.areas[2]);
            }
        };

        let main_block = Block::default();
        f.render_widget(main_block, self.areas[0]);
        f.render_stateful_widget(options, self.areas[1], &mut self.options.state);
        Ok(())
    }
}
