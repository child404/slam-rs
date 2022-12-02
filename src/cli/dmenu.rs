use std::{path::PathBuf, process};

use super::cmd::{self, Cmd, CmdResult};

pub struct Dmenu {
    cmd: Cmd,
}

impl Default for Dmenu {
    fn default() -> Self {
        Self::new(None, None)
    }
}

impl Dmenu {
    pub fn new(bin_path: Option<PathBuf>, args: Option<&[String]>) -> Self {
        Self {
            cmd: Cmd::new(
                bin_path,
                args.unwrap_or(&["-l 5".to_string(), "-g 1".to_string(), "-p".to_string()]),
                "dmenu",
            ),
        }
    }

    fn to_cmd(&self, message: &Message) -> String {
        format!(
            "printf \"{}\" | {} \"{}\"",
            message.prompt.join("\n"),
            self.cmd,
            message.content
        )
    }

    pub fn run(&self, message: Message) -> CmdResult<()> {
        cmd::run(&self.to_cmd(&message))
    }

    pub fn run_until_output_not_matched(&self, message: Message) -> CmdResult<String> {
        loop {
            let result = self.run_and_fetch_output(&message, true);
            if let Err(cmd::Error::InvalidOutput) = result {
                continue;
            }
            return result;
        }
    }

    pub fn run_and_fetch_output(
        &self,
        message: &Message,
        validate_output: bool,
    ) -> CmdResult<String> {
        // Loop until user won't choose one of the existing options or exit
        match cmd::run_and_fetch_output(&self.to_cmd(message)) {
            Err(cmd::Error::EmptyOutput) => process::exit(0),
            Ok(output) => {
                if !validate_output || message.contains(&output) {
                    Ok(output)
                } else {
                    Err(cmd::Error::InvalidOutput)
                }
            }
            other_error => other_error,
        }
    }
}

pub struct Message {
    prompt: Vec<String>,
    content: String,
}

impl Message {
    pub fn new(prompt: &[String], content: &str) -> Self {
        Self {
            prompt: prompt.to_vec(),
            content: content.to_string(),
        }
    }

    pub fn contains(&self, choice: &String) -> bool {
        self.prompt.contains(choice)
    }
}
