pub mod action;
mod args;
mod bindings;
mod conf;
mod ctx;
mod directory;
mod error;
mod inputmode;
mod styles;
pub use args::*;
pub use bindings::*;
pub use conf::*;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
pub use ctx::*;
use derive_deref::{Deref, DerefMut};
pub use directory::*;
pub use error::*;
pub use inputmode::*;
use ratatui::style::{Color, Modifier, Style};
pub use styles::*;

use crate::backend::BackendType;
use color_eyre::eyre::Result as ColorResult;
use directories::ProjectDirs;
use lazy_static::lazy_static;
use std::{collections::HashMap, path::PathBuf};
use tracing::error;
use tracing_error::ErrorLayer;
use tracing_subscriber::{
    self, prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt, Layer,
};
use wasm_bindgen::JsValue;

use crate::{app::Mode, VERSION};

lazy_static! {
    pub static ref APP_NAME: String = env!("CARGO_PKG_NAME").to_uppercase().to_string();
    pub static ref DATA_FOLDER: Option<PathBuf> = std::env::var("DATA").ok().map(PathBuf::from);
    pub static ref LOG_ENV: String = "LOG_LEVEL".to_string();
    pub static ref LOG_FILE: String = format!("{}.log", env!("CARGO_PKG_NAME"));
}

/// Inject HTML footer with backend switching links
pub(crate) fn inject_backend_footer(current_backend: BackendType) -> Result<(), JsValue> {
    let window = web_sys::window().ok_or("No window")?;
    let document = window.document().ok_or("No document")?;

    // Remove existing footer if present
    if let Some(existing) = document.get_element_by_id("ratzilla-backend-footer") {
        existing.remove();
    }

    // Create footer element
    let footer = document.create_element("div")?;
    footer.set_id("ratzilla-backend-footer");

    // Set footer styles
    footer.set_attribute(
        "style",
        "position: fixed; bottom: 0; left: 0; right: 0; \
         background: rgba(0,0,0,0.8); color: white; \
         padding: 8px 16px; font-family: monospace; font-size: 12px; \
         display: flex; justify-content: center; gap: 16px; \
         border-top: 1px solid #333; z-index: 1000;",
    )?;

    // Get current URL without backend param - use relative URL to avoid protocol issues
    let location = window.location();
    let base_url = location.pathname().unwrap_or_default();

    let backends = [BackendType::Dom, BackendType::Canvas];
    let mut links = Vec::new();

    for backend in backends {
        let is_current = backend == current_backend;
        let style = if is_current {
            "color: #4ade80; font-weight: bold; text-decoration: none;"
        } else {
            "color: #94a3b8; text-decoration: none; cursor: pointer;"
        };

        let link = if is_current {
            format!("<span style=\"{}\">● {backend}</span>", style,)
        } else {
            format!(
                "<a href=\"{}?backend={}\" style=\"{}\">{backend}</a>",
                base_url,
                backend.as_str(),
                style,
            )
        };

        links.push(link);
    }

    let footer_html = format!(
        "<span style=\"color: #64748b;\">Backend:</span> {} | \
         <span style=\"color: #64748b;\">FPS:</span> \
         <span id=\"ratzilla-fps\" style=\"color: #4ade80; font-weight: bold;\">--</span>",
        links.join(" | ")
    );

    footer.set_inner_html(&footer_html);

    // Append to body
    let body = document.body().ok_or("No body")?;
    body.append_child(&footer)?;

    Ok(())
}

pub fn initialize_panic_handler() -> ColorResult<()> {
    let (panic_hook, eyre_hook) = color_eyre::config::HookBuilder::default()
        .panic_section(format!(
            "This is a bug. Consider reporting it at {}",
            env!("CARGO_PKG_REPOSITORY")
        ))
        .capture_span_trace_by_default(false)
        .display_location_section(false)
        .display_env_section(false)
        .into_hooks();
    eyre_hook.install()?;
    std::panic::set_hook(Box::new(move |panic_info| {
        if let Ok(mut t) = crate::tui::Tui::new() {
            if let Err(r) = t.exit() {
                error!("Unable to exit Terminal: {:?}", r);
            }
        }

        #[cfg(not(debug_assertions))]
        {
            use human_panic::{handle_dump, print_msg, Metadata};
            let meta = Metadata {
                version: env!("CARGO_PKG_VERSION").into(),
                name: env!("CARGO_PKG_NAME").into(),
                authors: env!("CARGO_PKG_AUTHORS").replace(':', ", ").into(),
                homepage: env!("CARGO_PKG_HOMEPAGE").into(),
            };

            let file_path = handle_dump(&meta, panic_info);
            // prints human-panic message
            print_msg(file_path, &meta)
                .expect("human-panic: printing error message to console failed");
            eprintln!("{}", panic_hook.panic_report(panic_info)); // prints color-eyre stack trace to stderr
        }
        let msg = format!("{}", panic_hook.panic_report(panic_info));
        log::error!("Error: {}", strip_ansi_escapes::strip_str(msg));

        #[cfg(debug_assertions)]
        {
            // Better Panic stacktrace that is only enabled when debugging.
            better_panic::Settings::auto()
                .most_recent_first(false)
                .lineno_suffix(true)
                .verbosity(better_panic::Verbosity::Full)
                .create_panic_handler()(panic_info);
        }

        std::process::exit(libc::EXIT_FAILURE);
    }));
    Ok(())
}

pub fn initialize_logging(directory: PathBuf) -> ColorResult<()> {
    std::fs::create_dir_all(directory.clone())?;
    let log_path = directory.join(LOG_FILE.clone());
    let log_file = std::fs::File::create(log_path)?;
    std::env::set_var(
        "RUST_LOG",
        std::env::var("RUST_LOG")
            .or_else(|_| std::env::var(LOG_ENV.clone()))
            .unwrap_or_else(|_| format!("{}=info", env!("CARGO_CRATE_NAME"))),
    );
    let file_subscriber = tracing_subscriber::fmt::layer()
        .with_file(true)
        .with_line_number(true)
        .with_writer(log_file)
        .with_target(false)
        .with_ansi(false)
        .with_filter(tracing_subscriber::filter::EnvFilter::from_default_env());
    tracing_subscriber::registry()
        .with(file_subscriber)
        .with(ErrorLayer::default())
        .init();
    Ok(())
}

/// Similar to the `std::dbg!` macro, but generates `tracing` events rather
/// than printing to stdout.
///
/// By default, the verbosity level for the generated events is `DEBUG`, but
/// this can be customized.
#[macro_export]
macro_rules! trace_dbg {
    (target: $target:expr, level: $level:expr, $ex:expr) => {{
        match $ex {
            value => {
                tracing::event!(target: $target, $level, ?value, stringify!($ex));
                value
            }
        }
    }};
    (level: $level:expr, $ex:expr) => {
        trace_dbg!(target: module_path!(), level: $level, $ex)
    };
    (target: $target:expr, $ex:expr) => {
        trace_dbg!(target: $target, level: tracing::Level::DEBUG, $ex)
    };
    ($ex:expr) => {
        trace_dbg!(level: tracing::Level::DEBUG, $ex)
    };
}

pub fn version() -> String {
    let author = clap::crate_authors!();
    let commit_hash = VERSION;
    // let current_exe_path = PathBuf::from(clap::crate_name!()).display().to_string();
    format!(
        "\
{commit_hash}

Authors: {author}

        "
    )
}
