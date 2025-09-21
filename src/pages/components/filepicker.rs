use std::{collections::HashMap, fmt::Display, path::PathBuf, time::Duration};

use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use log::error;
use ratatui::{layout::Flex, prelude::*, widgets::*};
use tokio::sync::mpsc::UnboundedSender;
use tracing::trace;
use tui_input::{backend::crossterm::EventHandler, Input};

use crate::pages::{Component, Frame, InputMode, StatefulList};
use crate::utils::AppConfiguration;
use crate::{
    app::Mode,
    utils::{action::Action, key_event_to_string, Ctx, DirectorySearch, FileEntry},
    APP_NAME,
};

pub struct Filepicker {
    pub hidden: bool,
    pub directory: PathBuf,
    pub files: StatefulList<FileEntry>,
    pub action_tx: Option<UnboundedSender<Action>>,
    pub keymap: HashMap<KeyEvent, Action>,
    restrictions: Vec<String>,
    mouse: Option<MouseEvent>,
    config: Option<AppConfiguration>,
    show: bool,
    popup: bool,
    area: Rect,
    areas: Vec<Rect>,
}

impl Filepicker {
    pub fn new(popup: bool, restrict: Option<Vec<String>>) -> Self {
        let path = shellexpand::tilde(&"~/".to_string()).to_string().into();
        let restrictions = restrict.unwrap_or_else(|| vec![]);
        let files = StatefulList::with_items(DirectorySearch::open_directory(
            &path,
            false,
            Some(&restrictions.clone()),
        ));
        Self {
            files,
            config: None,
            directory: path,
            restrictions,
            hidden: false,
            popup,
            show: !popup,
            action_tx: None,
            keymap: HashMap::new(),
            mouse: None,
            area: Rect::default(),
            areas: Vec::new(),
        }
    }

    pub fn keymap(mut self, keymap: HashMap<KeyEvent, Action>) -> Self {
        self.keymap = keymap;
        self
    }

    /// helper function to create a centered rect using up certain percentage of the available rect `r`
    fn popup_area(area: Rect) -> Rect {
        let vertical = Layout::vertical([Constraint::Percentage(50)])
            .flex(Flex::Center)
            .vertical_margin(2);
        let horizontal = Layout::horizontal([Constraint::Percentage(20)]).flex(Flex::Center);
        let [area] = horizontal.areas(area);
        let [area] = vertical.areas(area);
        area
    }

    fn load_files(&mut self, path: PathBuf) -> Result<()> {
        self.files = StatefulList::with_items(DirectorySearch::open_directory(
            &path.clone(),
            self.hidden,
            Some(&self.restrictions),
        ));
        self.directory = path.into();

        self.files.state.select(Some(0));

        Ok(())
    }

    fn load_selected(&mut self) -> Result<()> {
        let tx = self.action_tx.clone().unwrap();
        let index = self.files.state.selected().unwrap_or(0);
        match self.files.items.get(index).is_some_and(|file| file.is_dir) {
            true => {
                let path = self.files.items.get(index).unwrap().path.clone();
                self.load_files(path)?;
            }
            false => {
                tx.send(Action::Toast(
                    "Error".to_string(),
                    "Selected item is not a directory".to_string(),
                ));
                log::info!(
                    "Selected file: {}",
                    self.files
                        .items
                        .get(index)
                        .map_or("None".to_string(), |f| f.name.clone())
                );
            }
        }
        Ok(())
    }

    pub fn register_popup_layout_handler(&mut self, area: Rect) -> Result<()> {
        self.area = Self::popup_area(area);
        // Get Areas
        let layout = Layout::vertical([Constraint::Min(1)]);
        let [bottom_area] = layout.areas(self.area);
        let block = Block::bordered();
        let content_area = block.inner(bottom_area);

        self.areas = vec![bottom_area, content_area];
        Ok(())
    }
}

impl Component for Filepicker {
    fn current_mode(&self) -> InputMode {
        InputMode::Normal
    }
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.action_tx = Some(tx);
        Ok(())
    }
    fn register_config_handler(&mut self, config: AppConfiguration) -> Result<()> {
        self.config = Some(config);
        Ok(())
    }

    fn register_layout_handler(&mut self, area: Rect) -> Result<()> {
        self.area = area;
        let layout = Layout::vertical([Constraint::Min(1)]);
        let [bottom_area] = layout.areas(self.area);
        let block = Block::bordered();
        let content_area = block.inner(bottom_area);

        self.areas = vec![bottom_area, content_area];
        Ok(())
    }

    fn handle_mouse_events(&mut self, mouse: MouseEvent) -> Result<Option<Action>> {
        trace!("Mouse event: {:?}", mouse);
        let tx = self.action_tx.clone().unwrap();
        self.areas.iter().enumerate().for_each(|(i, m)| {
            if let Some(mouse) = self.mouse {
                if mouse.kind == MouseEventKind::Up(MouseButton::Left)
                    && m.contains(Position::new(
                        self.mouse.unwrap().column,
                        self.mouse.unwrap().row,
                    ))
                {
                    // Handle click event
                    self.files.state.select(Some(i));
                    tx.send(Action::ChangeMode(Mode::ALL[i])).unwrap();
                }
            }
        });
        Ok(None)
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        let action = match key.code {
            KeyCode::Char('.') => {
                self.hidden = !self.hidden;
                self.load_files(self.directory.clone())?;
                Action::Update
            }
            KeyCode::Esc => {
                self.show = false;
                Action::Update
            }
            KeyCode::Enter => {
                self.load_selected()?;
                Action::Update
            }
            _ => return Ok(None),
        };
        Ok(Some(action))
    }

    fn update(&mut self, action: Action, ctx: &Ctx) -> Result<Option<Action>> {
        match action {
            Action::Forward => self.files.next(),
            Action::OpenFilepicker => {
                if self.popup {
                    self.show = !self.show;
                }
            }
            Action::Back => self.files.previous(),
            _ => (),
        }
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>) -> Result<()> {
        if self.popup && !self.show {
            return Ok(());
        }
        // Blocks for popup and button area
        let title = Block::default()
            .title_top(Line::from(self.directory.to_string_lossy().to_string()).left_aligned())
            .style(Style::default().bg(Color::Black).fg(Color::White));
        let status = Block::bordered()
            .borders(Borders::RIGHT)
            .border_style(Style::default().fg(Color::Yellow))
            .title_top(
                Line::from(match self.hidden {
                    false => "◎",
                    true => "◉",
                })
                .right_aligned(),
            )
            .style(Style::default().bg(Color::Black).fg(Color::White));

        let horizontal = Layout::horizontal([Constraint::Fill(1), Constraint::Length(3)]);
        let areas: [Rect; 2] = horizontal.areas(self.areas[0]);
        f.render_widget(Clear, self.area); //this clears out the background

        // Prep the widgets
        let text = self
            .files
            .items
            .iter()
            .filter(|x| {
                if self.hidden {
                    true
                } else if x.name != ".." {
                    !x.name.starts_with('.')
                } else {
                    true
                }
            })
            .enumerate()
            .map(|(i, file)| {
                ListItem::new(vec![Line::from(format!(
                    "{}{}",
                    if file.is_dir { "🖿 " } else { "  " },
                    file.name
                ))])
                .style(match file.name.starts_with('.') {
                    true => Style::default().fg(Color::DarkGray),
                    false => Style::default(),
                })
            })
            .collect::<Vec<_>>();

        let tasks = List::new(text).highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(Color::Yellow)
                .fg(Color::Black),
        );

        f.render_widget(title, areas[0]);
        f.render_widget(status, areas[1]);
        f.render_stateful_widget(tasks, self.areas[1], &mut self.files.state);
        Ok(())
    }
}
