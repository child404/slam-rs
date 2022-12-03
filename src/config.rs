use crate::{cli::cmd::CmdResult, exit_err, screen::Layout};
use serde_derive::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt, fs, io,
    path::{Path, PathBuf},
};

pub type Layouts = HashMap<String, Layout>;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Toml(toml::de::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "Failed to read config file: {}", error),
            Self::Toml(error) => write!(f, "Invalid layout config structure: {}", error),
        }
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<toml::de::Error> for Error {
    fn from(error: toml::de::Error) -> Self {
        Self::Toml(error)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LayoutConfig {
    pub file: PathBuf,
    pub layouts: Layouts,
}

impl LayoutConfig {
    pub fn new(config_path: &Path) -> Self {
        Self {
            file: config_path.to_path_buf(),
            layouts: Layouts::new(),
        }
    }

    pub fn get(&self, layout_name: &str) -> Option<&Layout> {
        self.layouts.get(layout_name)
    }

    pub fn layout_names(&self) -> Vec<String> {
        self.layouts.keys().cloned().collect()
    }

    pub fn disconnect_all(&self) -> CmdResult<()> {
        unimplemented!()
    }

    fn _create_config_file(path: &Path) -> Result<(), io::Error> {
        fs::DirBuilder::new().recursive(true).create(
            path
                .parent()
                .unwrap_or_else(|| exit_err!(
                    "Incorrect path to config file. Expected file with parent directory, but the value was: {:?}",
                    path
                ))
        )?;
        fs::File::create(path)?;
        Ok(())
    }

    pub fn try_from_toml(config_path: &Path) -> Result<Self, Error> {
        fs::read_to_string(config_path).map_or_else(
            |error| {
                if error.kind() == io::ErrorKind::NotFound {
                    Self::_create_config_file(config_path)?;
                    Ok(Self::new(config_path))
                } else {
                    Err(error.into())
                }
            },
            |content| {
                Ok(if !content.is_empty() {
                    toml::from_str(&content)?
                } else {
                    Self::new(config_path)
                })
            },
        )
    }

    pub fn is_empty(&self) -> bool {
        self.layouts.is_empty()
    }

    pub fn apply(&self, layout_name: &str) {
        if let Some(layout) = self.layouts.get(layout_name) {
            layout.apply();
            self._mark_layout_as_current(layout_name);
        }
    }

    fn _mark_layout_as_current(&self, layout_name: &str) {
        unimplemented!();
    }

    pub fn remove(&mut self, layout_name: &str) {
        self.layouts.remove(layout_name);
        self._remove_from_toml(layout_name);
    }

    fn _remove_from_toml(&self, layout_name: &str) {
        unimplemented!();
    }

    pub fn add(&mut self, layout: &Layout) {
        self.layouts.insert(layout.name.clone(), layout.clone());
        self._add_to_toml(layout);
    }

    fn _add_to_toml(&self, layout: &Layout) {
        unimplemented!()
    }
}
