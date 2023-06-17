use std::process::ExitCode;
use std::time::Duration;

use clap::Parser;
use color_eyre::eyre::Context;
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Input, Password};
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

pub async fn handle(mut config: Config, mega: &mut mega::Client, _: Opts) -> Result<ExitCode> {
    let theme = ColorfulTheme::default();

    let email: String = Input::with_theme(&theme)
        .with_prompt("Enter email address")
        .interact_text()?;

    let password: String = Password::with_theme(&theme)
        .with_prompt("Enter password")
        .interact()?;

    let maybe_bar = USER_ATTENDED.then(|| {
        let bar = ProgressBar::new_spinner();
        bar.set_style(utils::terminal::spinner_style());
        bar.set_message("logging in to MEGA...");
        bar.enable_steady_tick(Duration::from_millis(75));
        bar
    });

    let result = mega.login(&email, &password, None).await;

    if let Some(bar) = maybe_bar {
        bar.finish_and_clear();
    }

    if let Err(mega::Error::MegaError {
        code: mega::ErrorCode::EMFAREQUIRED,
    }) = result
    {
        let mfa: String = Input::with_theme(&theme)
            .with_prompt("Enter MFA code")
            .interact_text()?;

        let maybe_bar = USER_ATTENDED.then(|| {
            let bar = ProgressBar::new_spinner();
            bar.set_style(utils::terminal::spinner_style());
            bar.set_message("logging in to MEGA...");
            bar.enable_steady_tick(Duration::from_millis(75));
            bar
        });

        mega.login(&email, &password, Some(&mfa))
            .await
            .context("could not log in to MEGA")?;

        if let Some(bar) = maybe_bar {
            bar.finish_and_clear();
        }
    } else {
        result.context("could not log in to MEGA")?;
    }

    let session = mega
        .serialize_session()
        .await
        .context("could not serialize MEGA session")?;

    match config {
        Config::V1(ref mut config) => {
            config.auth.session = Some(session);
        }
    }

    confy::store(CONFIG_NAME, None, config).context("could not save configuration")?;

    crate::success!(to: std::io::stdout(), "successfully logged in to MEGA !")?;

    Ok(ExitCode::SUCCESS)
}
