/// UI based on dmenu
use crate::{
    cli::{
        cmd::CmdResult,
        dmenu::{Dmenu, Message},
        xrandr::Xrandr,
    },
    config::{self, LayoutConfig},
    screen::{Layout, Orientation, Output, Position, State},
};
use std::{
    collections::HashMap,
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
    xrandr: Xrandr,
    config: LayoutConfig,
}

impl UI {
    // FIXME: needs refactoring
    fn create_layout(&mut self) -> CmdResult<()> {
        let mut screen_options = self.xrandr.get_output_modes()?;
        let outputs_connected = screen_options.keys().cloned().collect::<Vec<String>>();
        if screen_options.len() == 1 {
            return self.dmenu.run(Message::new(
                &[],
                "You don't have any external monitors connected.",
            ));
        }
        let mut relative_outputs = HashMap::new();
        let mut is_primary_selected = false;
        let mut layout = Layout::new();
        layout.name = self
            .dmenu
            .run_and_fetch_output(&Message::new(&[], "Give the name to your layout:"), false)?;
        // TODO: show user all outputs except that one which is selected
        // TODO: instead of removing output, add next logic:
        //      1) when user specifies the monitor, the monitor still there, but with `check`
        //      2) if user selects checked output - ask does he want to override selected settings
        //      3) if yes - override settings, otherwise, continue
        loop {
            let mut output = Output::new();
            output.name = self.dmenu.run_until_output_not_matched(Message::new(
                &screen_options.keys().cloned().collect::<Vec<String>>(),
                "What screen to connect?",
            ))?;
            output.mode.resolution = self
                .dmenu
                .run_until_output_not_matched(Message::new(
                    &screen_options[&output.name].resolutions(),
                    "Choose resolution:",
                ))?
                .into();
            output.mode.rate = self
                .dmenu
                .run_until_output_not_matched(Message::new(
                    &screen_options[&output.name].rates(),
                    "Choose rate:",
                ))?
                .into();
            output.orientation = self
                .dmenu
                .run_until_output_not_matched(Message::new(
                    &Orientation::list(),
                    "Choose orientation:",
                ))?
                .into();

            let outputs_not_selected = outputs_connected
                .iter()
                .filter(|output_name| output_name != &&output.name)
                .map(|s| s.to_string())
                .collect::<Vec<String>>();

            // TODO: handle situation when the duplicated/relative screen is not connected in the end
            // TODO: handle issue when only no outputs left for relative position
            let position = self.dmenu.run_until_output_not_matched(Message::new(
                &Position::list(),
                "Choose position:",
            ))?;
            let relative_screen = if position.as_str() != "Center" {
                Some(
                    self.dmenu.run_until_output_not_matched(Message::new(
                        &outputs_not_selected
                            .iter()
                            .filter(|output_name| {
                                if let Some(relative_output_name) =
                                    relative_outputs.get(&output_name.to_string())
                                {
                                    relative_output_name == &output.name
                                } else {
                                    true
                                }
                            })
                            .map(|s| s.to_string())
                            .collect::<Vec<String>>(),
                        "Choose relative screen:",
                    ))?,
                )
            } else {
                None
            };
            if let Some(output_name) = relative_screen.clone() {
                relative_outputs.insert(output.name.clone(), output_name);
            }
            output.position = Position::from(&position, relative_screen);

            // TODO: if user chose duplicated - skip position
            //                     disconnected - skip all steps
            // TODO: start some parts if only connected/duplicates was choosen
            let state = self
                .dmenu
                .run_until_output_not_matched(Message::new(&State::list(), "Choose state:"))?;
            let duplicated_screen = if state.as_str() == "Duplicated" {
                Some(self.dmenu.run_until_output_not_matched(Message::new(
                    &outputs_not_selected,
                    "Choose duplicated screen:",
                ))?)
            } else {
                None
            };
            output.state = State::from(&state, duplicated_screen);

            output.is_primary = if !is_primary_selected {
                self.ask_with_confirmation(&format!(
                    "Make screen {} primary? (only once)",
                    &output.name
                ))?
            } else {
                false
            };
            if output.is_primary {
                is_primary_selected = true;
            }

            screen_options.remove(&output.name);
            layout.add(&output);

            if screen_options.is_empty() || !self.ask_with_confirmation("Add one more screen?")? {
                break;
            }
        }
        // TODO: handle if no outputs specified
        self.config.add(&layout);
        if self.ask_with_confirmation("Apply new layout?")? {
            self.config.apply(&layout.name)
        }
        println!("You choose {} monitor", layout.name);
        unimplemented!("Chain of commands/choices to create new layout.");
    }

    fn remove_layout(&mut self) -> CmdResult<()> {
        let layout_name = self.choose_layout()?;
        self.config.remove(&layout_name);
        Ok(())
    }

    fn ask_and_create_layout_if_yes(&mut self) -> CmdResult<()> {
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

    fn choose_layout(&mut self) -> CmdResult<String> {
        if self.config.is_empty() {
            self.ask_and_create_layout_if_yes()?;
            Ok(String::new())
        } else {
            let layout_names = self.config.layout_names();
            self.dmenu
                .run_until_output_not_matched(Message::new(&layout_names, "Choose layout:"))
        }
    }

    fn apply_layout(&mut self) -> CmdResult<()> {
        let layout_name = self.choose_layout()?;
        self.config.apply(&layout_name);
        Ok(())
    }

    pub fn new(config_path: &Path, dmenu_path: Option<PathBuf>) -> Result<Self, config::Error> {
        Ok(Self {
            dmenu: Dmenu::new(dmenu_path, None),
            xrandr: Xrandr::default(),
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
