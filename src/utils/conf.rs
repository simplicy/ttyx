//! Handles getting the data for the app configuration from the config file.
//! The config file is named config and is in the root of the project in TOML format.
//! This config file can determin and change many factors of the application such as the port and host of the server and database and db type.

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
use super::{Args, KeyBindings, Styles};

/// The server configuration for the API
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub log_level: String,
    pub app_data_path: String,
    pub popup_timeout: i64,
    pub username: Option<String>,
    pub password: Option<String>,
}
/// The default configuration for the server
impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            app_data_path: "~/.".to_owned() + APP_NAME,
            log_level: "info".to_string(),
            popup_timeout: 5,
            username: None,
            password: None,
        }
    }
}

/// The database configuration for the API
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub db_type: String,
}
/// The default configuration for the database
impl Default for DatabaseConfig {
    fn default() -> Self {
        DatabaseConfig {
            db_type: "memory".to_string(),
        }
    }
}

/// The configuration for the application
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct AppConfiguration {
    pub config: AppConfig,
    pub databaseconfig: DatabaseConfig,
    pub keybindings: KeyBindings,
    pub styles: Styles,
}

/// Implementation for the AppConfiguration
impl AppConfiguration {
    /// Initialize the application with the passed arguements
    pub fn init(app_args: Args) -> Result<Self> {
        // Get config pagth from CLI args (will use default if not changed)
        let conf_path = app_args.config_file.clone();
        let conf_path = shellexpand::tilde(&conf_path).to_string();
        // Get the config
        let mut cfg = match Self::load_config(conf_path) {
            Ok(cfg) => cfg,
            Err(e) => {
                log::error!("{:?}", e);
                AppConfiguration::default()
            }
        };
        let default_config = AppConfiguration::default();
        for (mode, default_bindings) in default_config.keybindings.iter() {
            let user_bindings = cfg.keybindings.entry(*mode).or_default();
            for (key, cmd) in default_bindings.iter() {
                user_bindings
                    .entry(key.clone())
                    .or_insert_with(|| cmd.clone());
            }
        }
        for (mode, default_styles) in default_config.styles.iter() {
            let user_styles = cfg.styles.entry(*mode).or_default();
            for (style_key, style) in default_styles.iter() {
                user_styles
                    .entry(style_key.clone())
                    .or_insert_with(|| *style);
            }
        }
        // Reconfigure app with ovverides from args
        cfg.configure(app_args);
        // Return the configuration
        Ok(cfg)
    }

    /// Configure the app with the args passed
    /// Only override the values that are not still default
    fn configure(&mut self, args: Args) {
        self.config.log_level = match args.log_level.is_empty() {
            // Use default / config file
            true => self.config.log_level.to_string(),
            // Use arg passed
            false => args.log_level,
        };
        self.config.username = match args.username.is_some() {
            // Use default / config file
            true => self.config.username.clone(),
            false => args.username.clone(),
        };
        self.config.password = match args.password.is_some() {
            // Use default / config file
            true => self.config.password.clone(),
            false => args.password.clone(),
        };
        self.config.app_data_path = match args.app_data_path.is_empty() {
            // Use default / config file
            true => self.config.app_data_path.to_string(),
            // Use arg passed
            false => args.app_data_path,
        };
    }

    pub fn update(config: AppConfig, config_path: &str) -> Result<()> {
        let path_str = config_path.to_owned() + "/config.toml";
        if config.app_data_path.is_empty() {
            log::error!("Config file path is empty");
            return Err(Error::InvalidAppDataPath);
        }
        log::info!("Updating config file at {}", path_str);
        let toml = match toml::to_string(&config) {
            Ok(toml) => toml,
            Err(e) => {
                log::error!("{:?}", e);
                "".to_owned()
            }
        };
        let path_str = shellexpand::tilde(&path_str).to_string();
        let path = Path::new(&path_str);
        // Write the toml to the file
        match std::fs::write(path, toml) {
            Ok(_) => {
                log::info!("Config file updated at {}", path_str);
                Ok(())
            }
            Err(e) => {
                log::error!("{:?}", e);
                Err(Error::CreatingConfig)
            }
        }
    }

    /// Generate the config file from the path provided
    fn create_config(path: PathBuf) -> Result<AppConfiguration> {
        //Create Default Config
        let default_config = AppConfiguration::default();
        //Write to a toml string
        let toml = toml::to_string(&default_config)?;
        //create directories
        match std::fs::create_dir_all(path.parent().unwrap()) {
            Ok(_) => {}
            Err(_) => log::error!("{:?}", Error::CreatingConfig),
        };
        // Create the file
        std::fs::File::create(path.clone())?;
        // Write the toml to the file
        std::fs::write(path, toml)?;
        //Return the config to be used in the app
        Ok(default_config)
    }

    /// Get the app config
    pub fn load_config(conf_path: String) -> Result<AppConfiguration> {
        let path = Path::new(&conf_path);
        //Check if the file exists
        match path.exists() {
            true => {
                // Build a config with the loaded file
                let build = Config::builder()
                    .add_source(File::with_name(&conf_path).required(true))
                    .build()?;
                // Deserialize the config into the AppConfiguration struct
                let cfg = match build.try_deserialize().unwrap() {
                    Some(cfg) => cfg,
                    None => return Err(Error::DeserializingConfig),
                };
                // return the path
                Ok(cfg)
            }
            false => {
                log::info!("Creating config file at {}", path.display());
                Self::create_config(path.to_path_buf())
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::utils::parse_key_sequence;

    use super::*;
    #[test]
    fn test_config() -> Result<()> {
        let c = AppConfiguration::default();
        assert_eq!(
            c.keybindings
                .get(&Mode::Global)
                .unwrap()
                .get(&parse_key_sequence("<q>")?)
                .unwrap(),
            &Action::ToggleShowQuit
        );
        Ok(())
    }
}
