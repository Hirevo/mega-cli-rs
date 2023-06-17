use std::process::ExitCode;

use clap::Subcommand;

pub mod auth;
pub mod config;
pub mod delete;
pub mod follow;
pub mod get;
pub mod list;
pub mod mkdir;
pub mod put;
pub mod rename;
pub mod tree;

use crate::config::Config;
use crate::Result;

#[derive(Debug, Clone, PartialEq, Subcommand)]
#[command(author, rename_all = "kebab-case")]
pub enum Command {
    /// Manage authentication with MEGA
    #[command(subcommand)]
    Auth(auth::Command),
    /// Interact with the `mega-cli` configuration
    #[command(subcommand)]
    Config(config::Command),
    /// Download files from MEGA
    Get(get::Opts),
    /// Upload files to MEGA
    Put(put::Opts),
    /// List remote MEGA nodes
    List(list::Opts),
    /// Display remote MEGA nodes as a tree
    Tree(tree::Opts),
    /// Create folders within MEGA
    Mkdir(mkdir::Opts),
    /// Rename nodes within MEGA
    Rename(rename::Opts),
    /// Delete remote MEGA nodes
    Delete(delete::Opts),
    /// Display MEGA storage events as they happen
    Follow(follow::Opts),
}

impl Command {
    pub fn may_need_user_session(&self) -> bool {
        match self {
            Command::Auth(opts) => opts.may_need_user_session(),
            Command::Config(opts) => opts.may_need_user_session(),
            Command::Get(opts) => opts.may_need_user_session(),
            Command::Put(opts) => opts.may_need_user_session(),
            Command::List(opts) => opts.may_need_user_session(),
            Command::Tree(opts) => opts.may_need_user_session(),
            Command::Mkdir(opts) => opts.may_need_user_session(),
            Command::Rename(opts) => opts.may_need_user_session(),
            Command::Delete(opts) => opts.may_need_user_session(),
            Command::Follow(opts) => opts.may_need_user_session(),
        }
    }
}

pub async fn handle(config: Config, mega: &mut mega::Client, opts: Command) -> Result<ExitCode> {
    match opts {
        Command::Auth(opts) => auth::handle(config, mega, opts).await,
        Command::Config(opts) => config::handle(config, mega, opts).await,
        Command::Get(opts) => get::handle(config, mega, opts).await,
        Command::Put(opts) => put::handle(config, mega, opts).await,
        Command::List(opts) => list::handle(config, mega, opts).await,
        Command::Tree(opts) => tree::handle(config, mega, opts).await,
        Command::Mkdir(opts) => mkdir::handle(config, mega, opts).await,
        Command::Rename(opts) => rename::handle(config, mega, opts).await,
        Command::Delete(opts) => delete::handle(config, mega, opts).await,
        Command::Follow(opts) => follow::handle(config, mega, opts).await,
    }
}
