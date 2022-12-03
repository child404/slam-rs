use crate::exit_err;
use std::{
    fmt::{self, Display},
    io,
    path::PathBuf,
    process::{Command, Stdio},
    str::{self, Utf8Error},
};
use which::which;

#[derive(Debug)]
pub enum Error {
    Utf8(Utf8Error),
    Io(io::Error),
    InvalidOutput,
    EmptyOutput,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Utf8(error) => write!(f, "Unable to decode Utf-8 command output: {}", error),
            Self::Io(error) => write!(f, "Failed to run the command: {}", error),
            Self::InvalidOutput => write!(f, "Output didn't match given options."),
            Self::EmptyOutput => write!(f, "Expected output, found empty value."),
        }
    }
}

impl From<Utf8Error> for Error {
    fn from(error: Utf8Error) -> Self {
        Error::Utf8(error)
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::Io(error)
    }
}

pub fn find_executable(name: &str) -> PathBuf {
    which(name).unwrap_or_else(|_| {
        exit_err!(
            "Cannot find {} in PATH! Please, install {} or add it to PATH if installed.",
            name,
            name
        )
    })
}

pub struct Cmd {
    pub bin_path: PathBuf,
    pub args: Vec<String>,
}

impl Display for Cmd {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?} {}", self.bin_path, self.args.join(" "))
    }
}

impl Cmd {
    pub fn new(bin_path: Option<PathBuf>, args: &[String], bin_name: &str) -> Self {
        Self {
            bin_path: bin_path.unwrap_or_else(|| find_executable(bin_name)),
            args: args.to_vec(),
        }
    }
}

pub type CmdResult<T> = Result<T, Error>;

pub fn run(command: &str) -> CmdResult<()> {
    let mut child = Command::new("bash").arg("-c").arg(command).spawn()?;
    child.wait()?;
    Ok(())
}

pub fn run_and_fetch_output(command: &str) -> CmdResult<String> {
    let child = Command::new("bash")
        .arg("-c")
        .arg(command)
        .stdout(Stdio::piped())
        .spawn()?;
    let output = child.wait_with_output()?;
    let output = str::from_utf8(&output.stdout)?;
    if !output.is_empty() {
        Ok(output.trim().to_string())
    } else {
        Err(Error::EmptyOutput)
    }
}
