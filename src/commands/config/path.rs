use std::io::Write;
use std::process::ExitCode;

use clap::Parser;

use crate::config::{Config, CONFIG_NAME};
use crate::utils::terminal::USER_ATTENDED;
use crate::Result;

#[derive(Debug, Clone, PartialEq, Parser)]
#[command(author, rename_all = "kebab-case")]
pub struct Opts {}

impl Opts {
    pub fn may_need_user_session(&self) -> bool {
        false
    }
}

pub async fn handle(_: Config, _: &mut mega::Client, _: Opts) -> Result<ExitCode> {
    let path = confy::get_configuration_file_path(CONFIG_NAME, None)?;

    if *USER_ATTENDED {
        crate::success!(to: std::io::stdout(), "path = `{0}`", path.display())?;
    } else {
        writeln!(std::io::stdout(), "{0}", path.display())?;
    }

    Ok(ExitCode::SUCCESS)
}
