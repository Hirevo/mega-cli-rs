use std::process::ExitCode;

use clap::Subcommand;

pub mod edit;
pub mod path;

use crate::config::Config;
use crate::Result;

#[derive(Debug, Clone, PartialEq, Subcommand)]
#[command(author, rename_all = "kebab-case")]
pub enum Command {
    /// Display the path to the configuration file
    Path(path::Opts),
    /// Edit the configuration file with a text editor
    Edit(edit::Opts),
}

impl Command {
    pub fn may_need_user_session(&self) -> bool {
        match self {
            Command::Path(opts) => opts.may_need_user_session(),
            Command::Edit(opts) => opts.may_need_user_session(),
        }
    }
}

pub async fn handle(config: Config, mega: &mut mega::Client, opts: Command) -> Result<ExitCode> {
    match opts {
        Command::Path(opts) => path::handle(config, mega, opts).await,
        Command::Edit(opts) => edit::handle(config, mega, opts).await,
    }
}
