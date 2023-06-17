use std::process::ExitCode;
use std::time::Duration;

use clap::{Args, Parser};
use color_eyre::eyre::Context;
use console::style;
use indicatif::ProgressBar;

pub mod commands;
pub mod config;
pub mod format;
pub mod serde_utils;
pub mod utils;

use crate::config::{Config, CONFIG_NAME};
use crate::utils::terminal::USER_ATTENDED;

pub type Error = color_eyre::Report;
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Clone, PartialEq, Args)]
#[command(rename_all = "kebab-case")]
pub struct GlobalOpts {
    /// Whether to skip logging in to MEGA.
    #[arg(long)]
    anonymous: bool,
}

#[derive(Debug, Clone, PartialEq, Parser)]
#[command(author, version, about, long_about = None, rename_all = "kebab-case")]
pub struct Opts {
    /// Global options.
    #[command(flatten)]
    global: GlobalOpts,
    /// Application subcommands.
    #[command(subcommand)]
    command: commands::Command,
}

#[tokio::main]
async fn main() -> ExitCode {
    match try_main().await {
        Ok(code) => code,
        Err(err) => {
            let errors: Vec<_> = err.chain().collect();

            let [error, causes @ .., last] = errors.as_slice() else {
                eprintln!("{0} `mega-cli` terminated due to an error", style("ERROR:").red());
                eprintln!();
                eprintln!("  {0} {err}", style('×').red());
                return ExitCode::FAILURE;
            };

            eprintln!(
                "{0} `mega-cli` terminated due to an error.",
                style("ERROR:").red()
            );
            eprintln!();
            eprintln!("  {0} {error}", style('×').red());
            for cause in causes {
                eprintln!("  {0} {cause}", style("├─▶").red());
            }
            eprintln!("  {0} {last}", style("╰─▶").red());

            ExitCode::FAILURE
        }
    }
}

async fn try_main() -> Result<ExitCode> {
    color_eyre::install()?;

    let opts = Opts::parse();
    let config: Config = confy::load(CONFIG_NAME, None)?;

    let mut mega = {
        let http_client = reqwest::Client::new();
        match &config {
            Config::V1(config) => mega::Client::builder()
                .origin(config.client.origin.clone())
                .timeout(config.client.timeout)
                .max_retries(config.client.max_retries)
                .min_retry_delay(config.client.min_retry_delay)
                .max_retry_delay(config.client.max_retry_delay)
                .https(config.client.https)
                .build(http_client)?,
        }
    };

    if opts.command.may_need_user_session() && !opts.global.anonymous {
        match config {
            Config::V1(ref config) => {
                if let Some(session) = config.auth.session.as_deref() {
                    let maybe_bar = USER_ATTENDED.then(|| {
                        let bar = ProgressBar::new_spinner();
                        bar.set_style(utils::terminal::spinner_style());
                        bar.set_message("resuming session with MEGA...");
                        bar.enable_steady_tick(Duration::from_millis(75));
                        bar
                    });

                    mega.resume_session(session)
                        .await
                        .context("could not resume session with MEGA")?;

                    if let Some(bar) = maybe_bar {
                        bar.finish_and_clear();
                    }
                }
            }
        };
    }

    let code = commands::handle(config, &mut mega, opts.command).await?;

    Ok(code)
}
