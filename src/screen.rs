use crate::exit_err;
use itertools::Itertools;
use regex::Regex;
use serde_derive::{Deserialize, Serialize};
use std::{
    cmp::{Eq, Ord, Ordering, PartialEq},
    collections::HashMap,
    fmt,
    hash::Hash,
    str::FromStr,
};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

pub type Outputs = HashMap<String, Output>;

#[derive(Debug)]
pub enum Error {
    InvalidResolution(String),
    InvalidRate(String),
    InvalidPosition(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidPosition(position) => write!(f, "Invalid position: {}", position),
            Self::InvalidResolution(resolution) => write!(f, "Invalid resolution: {}", resolution),
            Self::InvalidRate(rate) => write!(f, "Invalid refresh rate: {}", rate),
        }
    }
}

pub trait ToXrandrArg {
    fn to_xrandr_arg(&self) -> String;
}

#[derive(Debug, Deserialize, Serialize, Clone, Hash, PartialEq, Eq, Copy, Default)]
pub struct Resolution {
    height: u16,
    width: u16,
}

impl PartialOrd for Resolution {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Resolution {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.height != other.height {
            self.height.cmp(&other.height)
        } else {
            self.width.cmp(&other.width)
        }
    }
}

impl From<String> for Resolution {
    fn from(resolution: String) -> Self {
        resolution
            .as_str()
            .parse()
            .expect("Use From only when user selects pre-defined resolutions (correct by default).")
    }
}

impl FromStr for Resolution {
    fn from_str(resolution: &str) -> Result<Self, Self::Err> {
        match resolution
            .split('x')
            .take(2)
            .flat_map(|x| x.parse())
            .collect::<Vec<u16>>()[..]
        {
            [height, width] => Ok(Self { height, width }),
            _ => Err(Self::Err::InvalidResolution(resolution.to_string())),
        }
    }

    type Err = Error;
}

impl ToString for Resolution {
    fn to_string(&self) -> String {
        format!("{}x{}", self.height, self.width)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Hash, PartialEq, Eq, Copy, Default)]
pub struct Rate {
    value: u16,
}

impl PartialOrd for Rate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Rate {
    fn cmp(&self, other: &Self) -> Ordering {
        self.value.cmp(&other.value)
    }
}

impl From<String> for Rate {
    fn from(rate: String) -> Self {
        rate.as_str()
            .parse()
            .expect("Use From only when user selects pre-defined rates (correct by default).")
    }
}

impl FromStr for Rate {
    fn from_str(rate: &str) -> Result<Self, Self::Err> {
        rate.parse::<f64>().map_or_else(
            |_| Err(Self::Err::InvalidRate(rate.to_string())),
            |rate| {
                Ok(Self {
                    value: rate.round() as u16,
                })
            },
        )
    }

    type Err = Error;
}

impl ToString for Rate {
    fn to_string(&self) -> String {
        format!("{}", self.value)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Default, EnumIter)]
pub enum Position {
    #[default]
    Center,
    LeftOf(String),
    RightOf(String),
    Above(String),
    Below(String),
}

impl Position {
    pub fn list() -> Vec<String> {
        Self::iter().map(|s| s.to_string()).collect()
    }

    pub fn from(position: &str, relative_screen: Option<String>) -> Self {
        match position {
            "Center" => Self::Center,
            other => {
                let relative_screen = relative_screen.expect("Relative screen should be specified");
                match other {
                    "Left of" => Self::LeftOf(relative_screen),
                    "Right of" => Self::RightOf(relative_screen),
                    "Above" => Self::Above(relative_screen),
                    "Below" => Self::Below(relative_screen),
                    _ => exit_err!("Unexpected position: {}", position),
                }
            }
        }
    }
}

impl ToString for Position {
    fn to_string(&self) -> String {
        match self {
            Self::Center => "Center",
            Self::LeftOf(_) => "Left of",
            Self::RightOf(_) => "Right of",
            Self::Below(_) => "Below",
            Self::Above(_) => "Above",
        }
        .to_string()
    }
}

impl ToXrandrArg for Position {
    fn to_xrandr_arg(&self) -> String {
        match self {
            Self::Center => "".to_string(),
            Self::LeftOf(output) => format!("--left-of {}", output),
            Self::RightOf(output) => format!("--right-of {}", output),
            Self::Below(output) => format!("--below {}", output),
            Self::Above(output) => format!("--above {}", output),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Default, EnumIter)]
pub enum State {
    #[default]
    Connected,
    Duplicated(String),
    Disconnected,
}

impl State {
    pub fn list() -> Vec<String> {
        Self::iter().map(|s| s.to_string()).collect()
    }

    pub fn from(state: &str, duplicated_screen: Option<String>) -> Self {
        match state {
            "Connected" => Self::Connected,
            "Disconnected" => Self::Disconnected,
            "Duplicated" => {
                Self::Duplicated(duplicated_screen.expect("Duplicated screen should be specified."))
            }
            _ => exit_err!("Unexpected state option: {}", state),
        }
    }
}

impl ToString for State {
    fn to_string(&self) -> String {
        match self {
            Self::Connected => "Connected",
            Self::Duplicated(_) => "Duplicated",
            Self::Disconnected => "Disconnected",
        }
        .to_string()
    }
}

impl ToXrandrArg for State {
    fn to_xrandr_arg(&self) -> String {
        match self {
            Self::Disconnected => "--off".to_string(),
            Self::Connected => "".to_string(),
            Self::Duplicated(screen) => format!("--same-as {}", screen),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Default, EnumIter)]
pub enum Orientation {
    #[default]
    Normal,
    Inverted,
    Left,
    Right,
}

impl Orientation {
    pub fn list() -> Vec<String> {
        Self::iter().map(|s| s.to_string()).collect()
    }
}

impl ToString for Orientation {
    fn to_string(&self) -> String {
        match self {
            Self::Normal => "Normal",
            Self::Inverted => "Inverted",
            Self::Left => "Left",
            Self::Right => "Right",
        }
        .to_string()
    }
}

impl ToXrandrArg for Orientation {
    fn to_xrandr_arg(&self) -> String {
        format!("--rotate {}", self.to_string().to_lowercase())
    }
}

impl From<String> for Orientation {
    fn from(orientation: String) -> Self {
        match orientation.as_str() {
            "Normal" => Self::Normal,
            "Inverted" => Self::Inverted,
            "Left" => Self::Left,
            "Right" => Self::Right,
            _ => exit_err!("Unknown orientatino: {}", orientation),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Mode {
    pub resolution: Resolution,
    pub rate: Rate,
}

impl ToXrandrArg for Mode {
    fn to_xrandr_arg(&self) -> String {
        format!(
            "--mode {} --rate {}",
            self.resolution.to_string(),
            self.rate.to_string()
        )
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Output {
    pub name: String,
    pub mode: Mode,
    pub is_primary: bool,
    pub state: State,
    pub position: Position,
    pub orientation: Orientation,
}

impl Output {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            mode: Mode {
                resolution: Resolution {
                    height: 0,
                    width: 0,
                },
                rate: Rate { value: 0 },
            },
            is_primary: false,
            state: State::Disconnected,
            position: Position::Center,
            orientation: Orientation::Normal,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Layout {
    pub name: String,
    pub outputs: Outputs,
}

impl Layout {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            outputs: Outputs::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.outputs.is_empty()
    }

    pub fn apply(&self) {
        unimplemented!();
    }

    pub fn add(&mut self, output: Output) {
        self.outputs.insert(output.name.clone(), output);
    }

    pub fn get(&self, output_name: &str) -> Option<&Output> {
        self.outputs.get(output_name)
    }
}

#[derive(Default)]
pub struct OutputModes {
    pub resolutions: Vec<Resolution>,
    pub rates: Vec<Rate>,
}

fn sort_and_filter_unique<T>(array: &mut [T]) -> Vec<T>
where
    T: Hash + Ord + Copy,
{
    array.sort_by(|a, b| b.cmp(a));
    array.iter().unique().copied().collect()
}

fn map_str<T>(t: &[T]) -> Vec<String>
where
    T: ToString,
{
    t.iter().map(T::to_string).collect()
}

impl OutputModes {
    pub fn is_empty(&self) -> bool {
        self.resolutions.is_empty() || self.rates.is_empty()
    }

    pub fn resolutions(&self) -> Vec<String> {
        map_str(&self.resolutions)
    }

    pub fn rates(&self) -> Vec<String> {
        map_str(&self.rates)
    }

    fn remove_duplicates(&mut self) {
        self.resolutions = sort_and_filter_unique(&mut self.resolutions);
        self.rates = sort_and_filter_unique(&mut self.rates);
    }

    fn add(&mut self, resolution: Resolution, rate: Rate) {
        self.resolutions.push(resolution);
        self.rates.push(rate);
    }
}

impl FromStr for OutputModes {
    fn from_str(screen_settings: &str) -> Result<Self, Self::Err> {
        let mut output_modes = Self::default();
        for mode in Regex::new(r"(\d+x\d+) (\d+\.\d+)\n")
            .unwrap()
            .captures_iter(screen_settings)
        {
            output_modes.add(mode[1].parse()?, mode[2].parse()?);
        }
        output_modes.remove_duplicates();
        Ok(output_modes)
    }

    type Err = Error;
}
