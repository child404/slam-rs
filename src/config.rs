use crate::{
    cli::{cmd::CmdResult, xrandr::Xrandr},
    exit_err,
    screen::Layout,
};
use serde_derive::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt, fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

pub const CHECK_SIGN: &str = " ✓";

pub type Layouts = HashMap<String, Layout>;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    TomlDe(toml::de::Error),
    TomlSer(toml::ser::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "Failed to read config file: {}", error),
            Self::TomlDe(error) => write!(f, "Invalid layout config structure: {}", error),
            Self::TomlSer(error) => write!(f, "Error serializing layout config: {}", error),
        }
    }
}

impl From<toml::ser::Error> for Error {
    fn from(error: toml::ser::Error) -> Self {
        Self::TomlSer(error)
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<toml::de::Error> for Error {
    fn from(error: toml::de::Error) -> Self {
        Self::TomlDe(error)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LayoutConfig {
    #[serde(skip_serializing, skip_deserializing)]
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
        self.layouts
            .iter()
            .map(|(layout_name, layout)| {
                if layout.is_current {
                    format!("{}{}", layout_name, CHECK_SIGN)
                } else {
                    layout_name.to_string()
                }
            })
            .collect()
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
                    let mut config = toml::from_str::<Self>(&content)?;
                    config.file = config_path.to_path_buf();
                    config
                } else {
                    Self::new(config_path)
                })
            },
        )
    }

    pub fn is_empty(&self) -> bool {
        self.layouts.is_empty()
    }

    pub fn apply(&mut self, layout_name: &str, xrandr: &Xrandr) -> CmdResult<()> {
        if let Some(layout) = self.layouts.get(layout_name) {
            xrandr.run_with_args(&layout.get_xrandr_args())?;
            self._mark_layout_as_current(layout_name)
                .unwrap_or_else(|error| exit_err!("{}", error));
        }
        Ok(())
    }

    fn _mark_layout_as_current(&mut self, layout_name: &str) -> Result<(), Error> {
        for (name, mut layout) in self.layouts.iter_mut() {
            layout.is_current = *name == *layout_name;
        }
        self._overwrite_config()
    }

    pub fn remove(&mut self, layout_name: &str) -> Result<(), Error> {
        self.layouts.remove(layout_name);
        self._overwrite_config()
    }

    pub fn add(&mut self, layout: &Layout) -> Result<(), Error> {
        self.layouts.insert(layout.name.clone(), layout.clone());
        self._overwrite_config()
    }

    fn _overwrite_config(&self) -> Result<(), Error> {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&self.file)
            .expect("File created on init, or existed before otherwise");
        file.write_all(toml::Value::try_from(&self)?.to_string().as_bytes())?;
        Ok(())
    }
}
