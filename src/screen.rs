use std::{fmt, str::FromStr};

use serde_derive::{Deserialize, Serialize};

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

#[derive(Debug, Deserialize, Serialize, Clone, Hash, PartialEq, Eq, Copy, Default)]
pub struct Resolution {
    height: u16,
    width: u16,
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

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub enum Position {
    #[default]
    Center,
    LeftOf(String),
    RightOf(String),
    Above(String),
    Below(String),
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub enum State {
    #[default]
    Disconnected,
    Duplicated(String),
    Connected,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub enum Orientation {
    #[default]
    Normal,
    Inverted,
    Left,
    Right,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Mode {
    pub resolution: Resolution,
    pub rate: Rate,
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

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Layout {
    pub name: String,
    pub outputs: Vec<Output>,
}

impl Layout {
    pub fn apply(&self) {
        unimplemented!();
    }
}
