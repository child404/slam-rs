/// UI based on dmenu
use crate::{
    cli::{
        cmd::CmdResult,
        dmenu::{Dmenu, Message},
    },
    config::{self, LayoutConfig},
};
use std::{
    path::{Path, PathBuf},
    process,
};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(EnumIter)]
pub enum StartOption {
    AutoDetect,
    DisconnectAll,
    ApplyLayout,
    RemoveLayout,
    NewLayout,
    Exit,
}

impl StartOption {
    pub fn list() -> Vec<String> {
        Self::iter().map(|v| v.to_string()).collect()
    }
}

impl ToString for StartOption {
    fn to_string(&self) -> String {
        match self {
            Self::AutoDetect => "Auto-Detect",
            Self::DisconnectAll => "Disconnect All",
            Self::NewLayout => "New Layout",
            Self::RemoveLayout => "Remove Layout",
            Self::ApplyLayout => "Apply Layout",
            Self::Exit => "Exit",
        }
        .to_string()
    }
}

impl From<String> for StartOption {
    fn from(action: String) -> Self {
        match action.as_str() {
            "Auto-Detect" => Self::AutoDetect,
            "Disconnect All" => Self::DisconnectAll,
            "New Layout" => Self::NewLayout,
            "Remove Layout" => Self::RemoveLayout,
            "Exit" => Self::Exit,
            "Apply Layout" => Self::ApplyLayout,
            other => {
                crate::exit_err!("Unexpected start option: {}", other);
            }
        }
    }
}

pub struct UI {
    dmenu: Dmenu,
    config: LayoutConfig,
}

impl UI {
    fn create_layout(&self) -> CmdResult<()> {
        // self.dmenu.run_and_fetch_output(Message::new());
        unimplemented!("Chain of commands/choices to create new layout.");
    }

    fn remove_layout(&mut self) -> CmdResult<()> {
        self.config.remove(&self.choose_layout()?);
        Ok(())
    }

    fn ask_to_create_layout(&self) -> CmdResult<()> {
        if self.does_create_layout()? {
            self.create_layout()?;
        }
        Ok(())
    }

    fn ask_with_confirmation(&self, msg: &str) -> CmdResult<bool> {
        let answer = self.dmenu.run_until_output_not_matched(Message::new(
            &["No".to_string(), "Yes".to_string()],
            msg,
        ))?;
        Ok(answer == "Yes")
    }

    fn does_create_layout(&self) -> CmdResult<bool> {
        self.ask_with_confirmation("You don't have any layouts yet. Create one?")
    }

    fn choose_layout(&self) -> CmdResult<String> {
        if self.config.is_empty() {
            self.ask_to_create_layout()?;
            Ok(String::new())
        } else {
            let layout_names = self.config.layout_names();
            self.dmenu
                .run_until_output_not_matched(Message::new(&layout_names, "Choose layout:"))
        }
    }

    fn apply_layout(&self) -> CmdResult<()> {
        self.config.apply(&self.choose_layout()?);
        Ok(())
    }

    pub fn new(config_path: &Path, dmenu_path: Option<PathBuf>) -> Result<Self, config::Error> {
        Ok(Self {
            dmenu: Dmenu::new(dmenu_path, None),
            config: LayoutConfig::try_from_toml(config_path)?,
        })
    }

    pub fn start(&mut self) -> CmdResult<()> {
        match self.choose_start_option()? {
            StartOption::AutoDetect => {
                unimplemented!("Auto-Detect (xrandr?) monitors and apply layout automatically.")
            }
            StartOption::DisconnectAll => {
                unimplemented!("Apply layout with only one (internal?) monitor.")
            }
            StartOption::NewLayout => self.create_layout(),
            StartOption::ApplyLayout => self.apply_layout(),
            StartOption::RemoveLayout => self.remove_layout(),
            StartOption::Exit => process::exit(0),
        }
    }

    fn choose_start_option(&self) -> CmdResult<StartOption> {
        Ok(self
            .dmenu
            .run_until_output_not_matched(Message::new(&StartOption::list(), "Choose an option:"))?
            .into())
    }
}
