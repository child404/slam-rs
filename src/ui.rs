/// UI based on dmenu
use crate::{
    cli::{
        cmd::CmdResult,
        dmenu::{Dmenu, Message},
        xrandr::Xrandr,
    },
    config::{self, LayoutConfig, CHECK_SIGN},
    exit_err,
    screen::{Layout, Orientation, Output, Position, State},
    vec_from_enum,
};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    process,
};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

const PRIMARY_NOT_SELECTED: bool = false;
const PRIMARY_SELECTED: bool = true;

#[derive(EnumIter)]
pub enum StartOption {
    ApplyLayout,
    RemoveLayout,
    NewLayout,
    Exit,
}

impl ToString for StartOption {
    fn to_string(&self) -> String {
        match self {
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
            "New Layout" => Self::NewLayout,
            "Remove Layout" => Self::RemoveLayout,
            "Exit" => Self::Exit,
            "Apply Layout" => Self::ApplyLayout,
            other => {
                exit_err!("Unexpected start option: {}", other);
            }
        }
    }
}

pub struct UserInterface {
    dmenu: Dmenu,
    xrandr: Xrandr,
    config: LayoutConfig,
}

// TODO: add LayoutManager struct which will create/remove/apply layouts
impl UserInterface {
    pub fn new(config_path: &Path, dmenu_path: Option<PathBuf>) -> Result<Self, config::Error> {
        Ok(Self {
            dmenu: Dmenu::new(dmenu_path, None),
            xrandr: Xrandr::default(),
            config: LayoutConfig::try_from_toml(config_path)?,
        })
    }

    fn select_layout_name(&self, layout: &mut Layout) -> CmdResult<()> {
        layout.name = self
            .dmenu
            .run_and_fetch_output(
                &Message::new(
                    &self.config.layout_names(),
                    "What is the name of a new layout? (created are listed below)",
                ),
                false,
            )?
            .replace(CHECK_SIGN, "");
        Ok(())
    }

    fn select_output_name(&self, output: &mut Output, output_names: &[String]) -> CmdResult<()> {
        output.name = self.select_from_list(output_names, "What screen to connect?")?;
        Ok(())
    }

    fn select_state(&self, output: &mut Output, other_outputs: &[String]) -> CmdResult<()> {
        let state = self
            .dmenu
            .run_until_output_not_matched(Message::new(&vec_from_enum!(State), "Choose state:"))?;
        let duplicated_screen = if &state == "Duplicated" {
            Some(self.select_from_list(other_outputs, "Choose duplicated screen:")?)
        } else {
            None
        };
        output.state = State::from(&state, duplicated_screen);
        Ok(())
    }

    fn select_resolution(&self, output: &mut Output, resolutions: &[String]) -> CmdResult<()> {
        output.mode.resolution = self
            .select_from_list(resolutions, "Choose resolution:")?
            .into();
        Ok(())
    }

    fn select_rate(&self, output: &mut Output, rates: &[String]) -> CmdResult<()> {
        output.mode.rate = self.select_from_list(rates, "Choose rate:")?.into();
        Ok(())
    }

    fn select_from_list(&self, options: &[String], message: &str) -> CmdResult<String> {
        self.dmenu
            .run_until_output_not_matched(Message::new(options, message))
    }

    fn select_orientation(&self, output: &mut Output) -> CmdResult<()> {
        output.orientation = self
            .select_from_list(&vec_from_enum!(Orientation), "Choose orientation:")?
            .into();
        Ok(())
    }

    fn select_position(
        &self,
        output: &mut Output,
        other_outputs: &[String],
        relative_outputs: &mut HashMap<String, String>,
    ) -> CmdResult<()> {
        // filter outputs that already were placed relatively to the current output
        let outputs_for_relative_position = other_outputs
            .iter()
            .filter(|output_name| {
                if let Some(relative_output_name) = relative_outputs.get(&output_name.to_string()) {
                    relative_output_name == &output.name
                } else {
                    true
                }
            })
            .cloned()
            .collect::<Vec<String>>();
        let positions = if !outputs_for_relative_position.is_empty() {
            vec_from_enum!(Position)
        } else {
            vec![Position::Center.to_string()]
        };
        let position = self.select_from_list(&positions, "Choose position:")?;
        let relative_screen = if &position != "Center" {
            Some(self.dmenu.run_until_output_not_matched(Message::new(
                &outputs_for_relative_position,
                "Choose relative screen:",
            ))?)
        } else {
            None
        };
        if let Some(output_name) = relative_screen.clone() {
            relative_outputs.insert(output.name.clone(), output_name);
        }
        output.position = Position::from(&position, relative_screen);
        Ok(())
    }

    fn layout_name_should_not_be_empty(&self) -> CmdResult<()> {
        self.dmenu.run_and_fetch_output(
            &Message::new(
                &[],
                "Layout name should not be empty string (press eny key to continue)",
            ),
            false,
        )?;
        Ok(())
    }

    fn disconnect_other_monitors<'a>(
        &'a self,
        layout: &mut Layout,
        output_names: impl Iterator<Item = &'a String>,
    ) {
        for output_name in output_names {
            layout.add(Output {
                name: output_name.to_owned(),
                ..Output::new()
            });
        }
    }

    fn does_layout_exist_and_override(&self, layout_name: &str) -> CmdResult<bool> {
        Ok(!matches!(self.config.get(layout_name), None)
            && !self.does_override_existing_layout(layout_name)?)
    }

    fn create_layout(&mut self) -> CmdResult<()> {
        let mut output_modes = self.xrandr.get_output_modes()?;
        let outputs_connected = output_modes.keys().cloned().collect::<Vec<String>>();
        if output_modes.is_empty() {
            return self
                .dmenu
                .run(Message::new(&[], "You don't have any monitors connected."));
        }
        let mut relative_outputs = HashMap::new();
        let mut is_primary_selected = PRIMARY_NOT_SELECTED;
        let mut layout = Layout::new();

        self.select_layout_name(&mut layout)?;

        if layout.name.is_empty() {
            self.layout_name_should_not_be_empty()?;
            return self.create_layout();
        }

        if self.does_layout_exist_and_override(&layout.name)? {
            return self.create_layout();
        }

        loop {
            let mut output = Output::new();

            self.select_output_name(
                &mut output,
                &output_modes.keys().cloned().collect::<Vec<String>>(),
            )?;

            let other_outputs = outputs_connected
                .iter()
                .filter(|output_name| output_name != &&output.name)
                .cloned()
                .collect::<Vec<String>>();
            self.select_state(&mut output, &other_outputs)?;

            if !matches!(output.state, State::Disconnected) {
                let resolutions = output_modes[&output.name].resolutions();
                self.select_resolution(&mut output, &resolutions)?;
                let rates = output_modes[&output.name].rates();
                self.select_rate(&mut output, &rates)?;
                self.select_orientation(&mut output)?;
                if matches!(output.state, State::Connected) {
                    self.select_position(&mut output, &other_outputs, &mut relative_outputs)?;
                }
                output.is_primary =
                    !is_primary_selected && self.does_make_output_primary(&output.name)?;
                if output.is_primary {
                    is_primary_selected = PRIMARY_SELECTED;
                }
            }

            output_modes.remove(&output.name);
            layout.add(output);
            if output_modes.is_empty() || !self.does_add_another_screen()? {
                break;
            }
        }
        if layout.is_empty() {
            return Ok(());
        }
        self.disconnect_other_monitors(
            &mut layout,
            output_modes
                .keys()
                .chain(self.xrandr.list_disconnected_outputs()?.iter())
                .into_iter(),
        );
        self.config
            .add(&layout)
            .unwrap_or_else(|error| exit_err!("{}", error));
        if self.does_apply_new_layout()? {
            self.config.apply(&layout.name, &self.xrandr)?;
        }
        Ok(())
    }

    fn does_override_existing_layout(&self, layout_name: &str) -> CmdResult<bool> {
        self.ask_with_confirmation(&format!(
            "Do you really want to overwrite existing layout: `{}`?",
            layout_name
        ))
    }

    fn does_make_output_primary(&self, output_name: &str) -> CmdResult<bool> {
        self.ask_with_confirmation(&format!("Make screen {} primary? (only once)", output_name))
    }

    fn does_add_another_screen(&self) -> CmdResult<bool> {
        self.ask_with_confirmation("Add another screen?")
    }

    fn does_apply_new_layout(&self) -> CmdResult<bool> {
        self.ask_with_confirmation("Apply new layout?")
    }

    fn remove_layout(&mut self) -> CmdResult<()> {
        let layout_name = self.choose_layout()?;
        if self.ask_with_confirmation(&format!(
            "Do you really want to remove '{}' layout? This operation will be irreversible!",
            &layout_name
        ))? {
            self.config
                .remove(&layout_name)
                .unwrap_or_else(|error| exit_err!("{}", error));
        }
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
            Ok(self
                .dmenu
                .run_until_output_not_matched(Message::new(&layout_names, "Choose layout:"))?
                .replace(CHECK_SIGN, ""))
        }
    }

    fn apply_layout(&mut self) -> CmdResult<()> {
        let layout_name = self.choose_layout()?;
        self.config.apply(&layout_name, &self.xrandr)
    }

    pub fn start(&mut self) -> CmdResult<()> {
        match self.choose_start_option()? {
            StartOption::NewLayout => self.create_layout(),
            StartOption::ApplyLayout => {
                self.apply_layout()?;
                process::exit(0);
            }
            StartOption::RemoveLayout => self.remove_layout(),
            StartOption::Exit => process::exit(0),
        }
    }

    fn choose_start_option(&self) -> CmdResult<StartOption> {
        Ok(self
            .dmenu
            .run_until_output_not_matched(Message::new(
                &vec_from_enum!(StartOption),
                "Choose an option:",
            ))?
            .into())
    }
}
