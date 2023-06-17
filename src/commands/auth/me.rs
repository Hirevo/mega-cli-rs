use std::process::ExitCode;
use std::time::Duration;

use clap::Parser;
use color_eyre::eyre::Context;
use indicatif::ProgressBar;

use crate::config::Config;
use crate::utils;
use crate::utils::terminal::USER_ATTENDED;
use crate::Result;

#[derive(Debug, Clone, PartialEq, Parser)]
#[command(author, rename_all = "kebab-case")]
pub struct Opts {}

impl Opts {
    pub fn may_need_user_session(&self) -> bool {
        true
    }
}

pub async fn handle(_: Config, mega: &mut mega::Client, _: Opts) -> Result<ExitCode> {
    let maybe_bar = USER_ATTENDED.then(|| {
        let bar = ProgressBar::new_spinner();
        bar.set_style(utils::terminal::spinner_style());
        bar.set_message("fetching current user info...");
        bar.enable_steady_tick(Duration::from_millis(75));
        bar
    });

    let user = mega
        .get_current_user_info()
        .await
        .context("could not fetch current user info")?;

    if let Some(bar) = maybe_bar {
        bar.finish_and_clear();
    }

    crate::info!(to: std::io::stdout(), "id = `{0}`", user.id)?;
    crate::info!(to: std::io::stdout(), "email = `{0}`", user.email)?;
    crate::info!(to: std::io::stdout(), "first_name = `{0}`", user.first_name)?;
    crate::info!(to: std::io::stdout(), "last_name = `{0}`", user.last_name)?;
    crate::info!(
        to: std::io::stdout(),
        "birth_date = `{0:?}`",
        user.birth_date,
    )?;
    crate::info!(
        to: std::io::stdout(),
        "country_code = `{0}`",
        user.country_code,
    )?;

    Ok(ExitCode::SUCCESS)
}
