use std::{collections::HashMap, fmt::Display, time::Duration};

use chrono::{DateTime, Utc};
use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, MouseEvent};
use log::error;
use ratatui::{
    prelude::*,
    widgets::{
        canvas::{Canvas, Circle, Map, MapResolution, Rectangle},
        ListState, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget,
        StatefulWidgetRef, Widget, Wrap, *,
    },
};
use tokio::sync::mpsc::UnboundedSender;
use tracing::trace;
use tui_input::{backend::crossterm::EventHandler, Input};
use tui_markdown::from_str;

use super::{components::Post, Component, Frame, InputMode, ScrollState, StatefulList};
use crate::{
    app::{App, Mode},
    utils::{action::Action, key_event_to_string, AppConfiguration, Ctx, Error, FileEntry},
};

pub struct Blog {
    config: Option<AppConfiguration>,
    sidebar: bool,
    mode: InputMode,
    input: Input,
    files: StatefulList<FileEntry>,
    open_file: usize,
    action_tx: Option<UnboundedSender<Action>>,
    keymap: HashMap<KeyEvent, Action>,
    content: Post,
    area: Rect,
}

impl Blog {
    pub fn new() -> Self {
        Self {
            config: None,
            sidebar: true,
            mode: InputMode::Select,
            content: Post::default(),
            action_tx: None,
            area: Rect::default(),
            open_file: 0,
            input: Input::default(),
            keymap: HashMap::new(),
            files: StatefulList::default(),
        }
    }

    fn generate_posts(&mut self) -> Result<()> {
        let conf = match self.config.clone() {
            Some(c) => c,
            None => return Err(Error::Configuration("Configuration not set".to_string()).into()),
        };
        let post_path = conf.config.app_data_path + "/posts/";
        let post_path = shellexpand::tilde(&post_path).to_string();
        log::info!("Loading items.");
        // Open all markdown files in the posts directory
        let markdowns = std::fs::read_dir(post_path.clone())
            .map_err(|e| Error::Configuration(format!("Failed to read posts directory: {}", e)))?;
        let mut files = Vec::new();
        for entry in markdowns.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("md") {
                let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
                let file = FileEntry {
                    name: file_name.to_string(),
                    path: path.clone(),
                    ctime: Some(
                        entry
                            .metadata()
                            .and_then(|m| m.created())
                            .map(DateTime::<Utc>::from)?,
                    ),
                    size: entry
                        .metadata()
                        .map_err(|e| {
                            Error::Configuration(format!("Failed to read metadata: {}", e))
                        })?
                        .len(),
                    is_dir: false,
                };
                files.push(file);
            }
        }
        self.files = StatefulList::with_items(files.clone());
        self.files.state.select(Some(0));
        // Set markdown from first file
        let markdown = if let Some(first_file) = files.first() {
            let file_path = format!("{}/{}", post_path, first_file.name);
            std::fs::read_to_string(file_path)
                .map_err(|e| Error::Configuration(format!("Failed to read post file: {}", e)))?
        } else {
            "# No posts found".to_string()
        };
        let filea = files.first().cloned().unwrap_or_default();
        let title = filea.name.replace(".md", "");
        let state = ScrollState::new(markdown.lines().count());
        self.content = Post::new(
            markdown.clone(),
            title.clone(),
            filea.ctime.unwrap_or_default(),
            state,
        );
        Ok(())
    }

    pub fn keymap(mut self, keymap: HashMap<KeyEvent, Action>) -> Self {
        self.keymap = keymap;
        self
    }
}

impl Component for Blog {
    fn register_config_handler(&mut self, config: AppConfiguration) -> Result<()> {
        self.config = Some(config.clone());
        self.generate_posts()?;
        self.content.register_config_handler(config)?;
        Ok(())
    }
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.content.register_action_handler(tx.clone())?;
        self.action_tx = Some(tx);
        Ok(())
    }

    fn register_layout_handler(&mut self, area: Rect) -> Result<()> {
        self.area = area;

        let vertical = Layout::horizontal(match self.sidebar {
            true => vec![Constraint::Percentage(18), Constraint::Fill(1)],
            false => vec![Constraint::Fill(1)],
        })
        .split(self.area);
        let areas = vertical.to_vec();
        self.area = areas[0];
        self.content
            .register_layout_handler(areas[areas.len() - 1])?;

        Ok(())
    }

    fn handle_mouse_events(&mut self, mouse: MouseEvent) -> Result<Option<Action>> {
        let tx = self.action_tx.clone().unwrap();
        Ok(None)
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        let action = match self.mode {
            InputMode::Select => match key.code {
                KeyCode::Esc | KeyCode::Backspace => Action::ToggleSidebar,
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
            },
            _ => return Ok(None),
        };
        Ok(Some(action))
    }

    fn update(&mut self, action: Action, ctx: &Ctx) -> Result<Option<Action>> {
        self.content.update(action.clone(), ctx)?;
        match action {
            Action::SelectOption => {
                if self.mode == InputMode::Select {
                    match self.files.state.selected() {
                        Some(index) => {
                            if let Some(selected) = self.files.items.get(index) {
                                // If an item is selected, we can render the content area with the post
                                let file_path = selected.path.clone();
                                let markdown =
                                    std::fs::read_to_string(&file_path).map_err(|e| {
                                        Error::Configuration(format!(
                                            "Failed to read post file: {}",
                                            e
                                        ))
                                    })?;
                                let title = selected.name.replace(".md", "");
                                let ctime = selected.ctime.unwrap_or_default();
                                let mut view_size = self.area.height as usize;
                                let max = markdown.lines().count();
                                if max < view_size {
                                    view_size = 0;
                                } else {
                                    view_size = (view_size / 2) - view_size / 3;
                                    // Reserve one line for the scrollbar
                                }
                                let state = ScrollState::new(max - view_size);
                                self.content = Post::new(markdown, title, ctime, state);
                                self.open_file = index;
                                // Render the post content
                            } else {
                                error!("Selected index {} out of bounds", index);
                            }
                        }
                        None => {
                            // If no item is selected, we can just clear the content area
                        }
                    };
                }
            }
            Action::ToggleSidebar => {
                match self.mode {
                    InputMode::Select => self.mode = InputMode::Normal,
                    _ => self.mode = InputMode::Select,
                }
                self.sidebar = !self.sidebar;
                if !self.sidebar {
                    self.mode = InputMode::Normal;
                }
            }
            Action::Forward => {
                if self.mode == InputMode::Select {
                    self.content.scroll_top();
                    self.files.next();
                }
            }
            Action::Back => {
                if self.mode == InputMode::Select {
                    self.content.scroll_top();
                    self.files.previous();
                }
            }
            _ => (),
        }
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>) -> Result<()> {
        // Grab current state
        // Selection Block for post
        let posts = self
            .files
            .items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                ListItem::new(vec![Line::from(item.name.to_string())]).style(
                    match self.open_file == i {
                        true => Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(Color::Black)
                            .bg(Color::Cyan),

                        false => Style::default().fg(Color::White),
                    },
                )
            })
            .collect::<Vec<_>>();
        let options = List::new(posts).highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(Color::Yellow)
                .fg(Color::Black),
        );
        let option_block = Block::default()
            .style(match self.mode {
                InputMode::Select => Style::default().bg(Color::Black),
                _ => Style::default(),
            })
            .borders(Borders::RIGHT)
            .border_style(match self.mode {
                InputMode::Select => Style::default().fg(Color::Yellow),
                _ => Style::default(),
            });
        // Set up areas
        if self.sidebar {
            let posts = options.block(option_block);
            // send state to app
            f.render_stateful_widget(posts, self.area, &mut self.files.state);
        }
        self.content.draw(f)?;
        Ok(())
    }
}
