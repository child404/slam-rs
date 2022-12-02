/// Runs main app with UI based on dmenu
use crate::{cli::cmd, config, screen, ui::UI};
use std::path::{Path, PathBuf};

pub enum Error {
    ScreenError(screen::Error),
    ConfigError(config::Error),
    CmdError(cmd::Error),
    InternalError,
}

impl From<cmd::Error> for Error {
    fn from(error: cmd::Error) -> Self {
        Self::CmdError(error)
    }
}

impl From<config::Error> for Error {
    fn from(error: config::Error) -> Self {
        Self::ConfigError(error)
    }
}

impl From<screen::Error> for Error {
    fn from(error: screen::Error) -> Self {
        Self::ScreenError(error)
    }
}

pub fn run(config_path: &Path, dmenu_path: Option<PathBuf>) -> Result<(), Error> {
    let mut ui = UI::new(config_path, dmenu_path)?;
    loop {
        ui.start()?;
    }
}

pub fn apply_layout(layout_path: &Path) {
    unimplemented!();
}
