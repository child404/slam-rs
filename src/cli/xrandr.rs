use crate::screen::OutputModes;

use super::cmd::{self, Cmd, CmdResult};
use regex::Regex;
use std::collections::HashMap;

pub struct Xrandr {
    pub cmd: Cmd,
}

impl Default for Xrandr {
    fn default() -> Self {
        Self::new(None)
    }
}

fn parse_screen_output(line: &str) -> Option<String> {
    line.split_whitespace().take(1).next().map(str::to_string)
}

impl Xrandr {
    pub fn new(args: Option<&[String]>) -> Self {
        Self {
            cmd: Cmd::new(None, args.unwrap_or_default(), "xrandr"),
        }
    }

    pub fn get_output_modes(&self) -> CmdResult<HashMap<String, OutputModes>> {
        let screens_regexp =
            Regex::new(r"(.+) connected\n(?:[\da-zA-Z]+x[\da-zA-Z]+ [\da-zA-Z]+\.[\da-zA-Z]+\n)+")
                .expect("Hardcoded regexp.");
        let screen_options = cmd::run_and_fetch_output(
            &(self.cmd.to_string() + " | grep -Ev \"disconnected|Screen\" | awk '{print $1, $2}' | awk -F'[/+* ]' '{print $1\" \"$2}'")
        )?;
        Ok(HashMap::from_iter(
            screens_regexp
                .captures_iter(&screen_options)
                .map(|captures| {
                    let [modes, output_name] = &[&captures[0], &captures[1]];
                    (
                        output_name.to_string(),
                        modes
                            .parse()
                            .expect("Correct display options as it already matched regexp."),
                    )
                }),
        ))
    }

    pub fn count_connected_outputs(&self) -> CmdResult<usize> {
        Ok(
            cmd::run_and_fetch_output(&format!("{} | grep \" connected\"", self.cmd))?
                .split('\n')
                .count(),
        )
    }

    pub fn list_connected_outputs(&self) -> CmdResult<Vec<String>> {
        Ok(
            cmd::run_and_fetch_output(&format!("{} | grep \" connected\"", self.cmd))?
                .split('\n')
                .flat_map(parse_screen_output)
                .collect(),
        )
    }

    pub fn list_disconnected_outputs(&self) -> CmdResult<Vec<String>> {
        Ok(
            cmd::run_and_fetch_output(&format!("{} | grep \" disconnected\"", self.cmd))?
                .split('\n')
                .flat_map(parse_screen_output)
                .collect(),
        )
    }

    pub fn run_with_args(&self, args: &[String]) -> CmdResult<()> {
        cmd::run(&format!("{} {}", self.cmd, args.join(" ")))
    }
}
