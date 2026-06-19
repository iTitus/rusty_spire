use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io::Write;
use std::path::Path;
use std::process::ExitStatus;

fn main() -> anyhow::Result<()> {
    let output_dir = Path::new("../sts2_data");

    std::process::Command::new("git")
        .current_dir(output_dir)
        .args(["status"])
        .status()?
        .exit_success()?;

    print!("Version: ");
    std::io::stdout().flush()?;
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let version = input.trim();
    if version.is_empty() {
        anyhow::bail!("empty version");
    }
    println!("{version:?}");

    std::process::Command::new("git")
        .current_dir(output_dir)
        .args(["add", "-A"])
        .status()?
        .exit_success()?;
    std::process::Command::new("git")
        .current_dir(output_dir)
        .args(["commit", "-a", "-m", version])
        .status()?
        .exit_success()?;
    std::process::Command::new("git")
        .current_dir(output_dir)
        .args(["push"])
        .status()?
        .exit_success()?;
    Ok(())
}

pub trait ExitStatusExt {
    fn exit_success(&self) -> Result<(), ExitStatusError>;
}

impl ExitStatusExt for ExitStatus {
    fn exit_success(&self) -> Result<(), ExitStatusError> {
        if self.success() {
            Ok(())
        } else {
            Err(ExitStatusError(self.code()))
        }
    }
}

#[derive(Debug)]
pub struct ExitStatusError(Option<i32>);

impl Display for ExitStatusError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "ExitStatusError: {:?}", self.0)
    }
}

impl Error for ExitStatusError {}
