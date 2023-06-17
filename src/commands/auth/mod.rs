use std::process::ExitCode;

use clap::Subcommand;

pub mod login;
pub mod logout;
pub mod me;

use crate::config::Config;
use crate::Result;

#[derive(Debug, Clone, PartialEq, Subcommand)]
#[command(author, rename_all = "kebab-case")]
pub enum Command {
    /// Create a new persistent session by logging in to MEGA
    Login(login::Opts),
    /// Log out of the current session with MEGA
    Logout(logout::Opts),
    /// Display information about the current session
    Me(me::Opts),
}

impl Command {
    pub fn may_need_user_session(&self) -> bool {
        match self {
            Command::Login(opts) => opts.may_need_user_session(),
            Command::Logout(opts) => opts.may_need_user_session(),
            Command::Me(opts) => opts.may_need_user_session(),
        }
    }
}

pub async fn handle(config: Config, mega: &mut mega::Client, opts: Command) -> Result<ExitCode> {
    match opts {
        Command::Login(opts) => login::handle(config, mega, opts).await,
        Command::Logout(opts) => logout::handle(config, mega, opts).await,
        Command::Me(opts) => me::handle(config, mega, opts).await,
    }
}
