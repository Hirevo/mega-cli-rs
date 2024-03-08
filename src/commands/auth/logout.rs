use std::process::ExitCode;
use std::sync::Arc;
use std::time::Duration;

use clap::Parser;
use color_eyre::eyre::{Context, ContextCompat};
use indicatif::ProgressBar;

use crate::config::{Config, CONFIG_NAME};
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

pub async fn handle(mut config: Config, mega: &mut Arc<mega::Client>, _: Opts) -> Result<ExitCode> {
    let maybe_bar = USER_ATTENDED.then(|| {
        let bar = ProgressBar::new_spinner();
        bar.set_style(utils::terminal::spinner_style());
        bar.set_message("logging out from MEGA...");
        bar.enable_steady_tick(Duration::from_millis(75));
        bar
    });

    let mega = Arc::get_mut(mega).context("could not mutably borrow MEGA client")?;
    mega.logout().await.context("could not log out from MEGA")?;

    if let Some(bar) = maybe_bar {
        bar.finish_and_clear();
    }

    match config {
        Config::V1(ref mut config) => {
            config.auth.session = None;
        }
    }

    confy::store(CONFIG_NAME, None, config).context("could not save configuration")?;

    crate::success!(to: std::io::stdout(), "successfully logged out from MEGA !")?;

    Ok(ExitCode::SUCCESS)
}
