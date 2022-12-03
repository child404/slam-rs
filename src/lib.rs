pub mod app;
pub mod cli;
pub mod config;
pub mod daemon;
pub mod screen;
pub mod ui;

use clap::Parser;
use std::path::PathBuf;

const PATH_TO_CONFIG: &str = ".config/slam_rs/config.toml";

#[macro_export]
macro_rules! exit_err {
    () => {
        process::exit(1);
    };
    ($($arg:tt)*) => {{
        eprintln!($($arg)*);
        std::process::exit(1);
    }};
}

#[macro_export]
macro_rules! vec_from_enum {
    ($t:ty) => {{
        <$t>::iter()
            .map(|option| option.to_string())
            .collect::<Vec<String>>()
    }};
}

pub fn find_config_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| exit_err!("Cannot find home dir"))
        .join(PATH_TO_CONFIG)
}

// TODO: add validation of config and layout paths via clap(validator = ...)
// TODO: add daemon save delay
// and add forbid_empty_values = true
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    // Path to config.toml file
    #[arg(short, long, value_name = "FILE", value_hint = clap::ValueHint::FilePath, required = false)]
    pub config: Option<PathBuf>,

    // Apply layout in /path/to/layout.toml file
    #[arg(short, long, value_name = "FILE", value_hint = clap::ValueHint::FilePath, exclusive = true, required = false)]
    pub layout: Option<PathBuf>,

    // Run the daemon to auto-detect layout
    #[arg(short, long, exclusive = true, required = false)]
    pub daemon: bool,

    // Path to dmenu executable
    #[arg(short = 'e', value_name = "BIN", value_hint = clap::ValueHint::ExecutablePath, required = false)]
    pub dmenu: Option<PathBuf>,
}
