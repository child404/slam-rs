use super::cmd::{self, Cmd, CmdResult};

pub struct Xrandr {
    cmd: Cmd,
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

    pub fn list_screens(&self) -> CmdResult<Vec<String>> {
        Ok(
            cmd::run_and_fetch_output(&format!("{} | grep \" connected\"", self.cmd))?
                .split('\n')
                .flat_map(parse_screen_output)
                .collect(),
        )
    }
}
