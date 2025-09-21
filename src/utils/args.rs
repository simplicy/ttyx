use std::path::PathBuf;

use clap::Parser;

use crate::{utils::version, APP_NAME};

#[derive(Parser, Debug, Clone)]
#[command(author, version = version(), about)]
pub struct Args {
    #[arg(
        short,
        long,
        value_name = "FLOAT",
        help = "Number of ticks per second",
        default_value_t = 15.0
    )]
    pub tick_rate: f64,

    #[arg(
        short,
        long,
        value_name = "BOOL",
        help = "Enable web mode",
        default_value_t = false
    )]
    pub web: bool,

    #[arg(
        short,
        long,
        value_name = "FLOAT",
        help = "Frame rate, i.e. number of frames per second",
        default_value_t = 30.0
    )]
    pub frame_rate: f64,

    #[arg(
        short,
        long,
        value_name = "Username",
        help = "Username for the application",
        default_value = None
    )]
    pub username: Option<String>,

    #[arg(
        short,
        long,
        value_name = "password",
        help = "Password for the application",
        default_value = None
    )]
    pub password: Option<String>,

    #[arg(
        short,
        long,
        value_name = "PATH",
        help = "Path to the configuration file", 
        default_value = format!("~/.{}/config.toml",APP_NAME)
    )]
    #[arg(value_hint = clap::ValueHint::FilePath)]
    pub config_file: String,

    #[arg(short,
        long,
        value_name = "PATH", 
        help = "Path to the application data directory",
        default_value = format!("~/.{}",APP_NAME)
    )]
    pub app_data_path: String,

    #[arg(
        short,
        long,
        value_name = "LOG_LEVEL",
        help = "Set the log level for the application",
        default_value = "info"
    )]
    pub log_level: String,
}
