use std::pin::pin;
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

pub async fn handle(_: Config, mega: &mega::Client, _: Opts) -> Result<ExitCode> {
    let mut nodes = {
        let maybe_bar = USER_ATTENDED.then(|| {
            let bar = ProgressBar::new_spinner();
            bar.set_style(utils::terminal::spinner_style());
            bar.set_message("fetching MEGA nodes...");
            bar.enable_steady_tick(Duration::from_millis(75));
            bar
        });

        let nodes = mega
            .fetch_own_nodes()
            .await
            .context("could net fetch own MEGA nodes")?;

        if let Some(bar) = maybe_bar {
            bar.finish_and_clear();
        }

        nodes
    };

    let ctrl_c = tokio::signal::ctrl_c();
    let mut ctrl_c = pin!(ctrl_c);

    loop {
        let maybe_bar = USER_ATTENDED.then(|| {
            let bar = ProgressBar::new_spinner();
            bar.set_style(utils::terminal::spinner_style());
            bar.set_message(format!("waiting for new events (CTRL-C to terminate)..."));
            bar.enable_steady_tick(Duration::from_millis(75));
            bar
        });

        let events = tokio::select! {
            biased;
            _ = &mut ctrl_c => {
                if let Some(bar) = maybe_bar {
                    bar.finish_and_clear();
                }

                return Ok(ExitCode::SUCCESS);
            },
            events = mega.wait_events(&nodes) => events?,
        };

        if let Some(bar) = maybe_bar {
            bar.finish_and_clear();
        }

        for event in events.events() {
            match event {
                mega::Event::NodeCreated { nodes } => {
                    for node in nodes {
                        crate::info!(to: std::io::stdout(), from: "mega:follow", "created node: (H:{0}) `{1}`", node.handle(), node.name())?;
                    }
                }
                mega::Event::NodeUpdated { attrs } => {
                    if let Some(node) = nodes.get_node_by_handle(attrs.handle()) {
                        crate::info!(to: std::io::stdout(), from: "mega:follow", "updated node: (H:{0}) `{1}`", node.handle(), node.name())?;
                    }
                }
                mega::Event::NodeDeleted { handle } => {
                    if let Some(node) = nodes.get_node_by_handle(handle) {
                        crate::info!(to: std::io::stdout(), from: "mega:follow", "deleted node: (H:{0}) `{1}`", node.handle(), node.name())?;
                    }
                }
            }
        }

        nodes.apply_events(events)?;
    }

    // Ok(ExitCode::SUCCESS)
}
