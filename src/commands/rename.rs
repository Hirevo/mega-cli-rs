use std::process::ExitCode;
use std::time::Duration;

use clap::Parser;
use color_eyre::eyre::{Context, ContextCompat};
use indicatif::ProgressBar;

use crate::config::Config;
use crate::utils;
use crate::utils::terminal::USER_ATTENDED;
use crate::Result;

#[derive(Debug, Clone, PartialEq, Parser)]
#[command(author, rename_all = "kebab-case")]
pub struct Opts {
    /// Path (eg. `/Root/folder/file.txt) or handle (eg. `H:gZlB3JxS`) in MEGA of the node to rename
    path: String,
    /// New name for the node
    name: String,
}

impl Opts {
    pub fn may_need_user_session(&self) -> bool {
        true
    }
}

pub async fn handle(_: Config, mega: &mega::Client, opts: Opts) -> Result<ExitCode> {
    let nodes = {
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

    let node = if opts.path.starts_with("H:") {
        nodes
            .get_node_by_handle(&opts.path[2..])
            .context("could not find node (by handle)")?
    } else {
        nodes
            .get_node_by_path(&opts.path)
            .context("could not find node (by path)")?
    };

    let maybe_bar = USER_ATTENDED.then(|| {
        let bar = ProgressBar::new_spinner();
        bar.set_style(utils::terminal::spinner_style());
        bar.set_message(format!(
            "renaming `{0}` to `{1}`...",
            node.name(),
            opts.name,
        ));
        bar.enable_steady_tick(Duration::from_millis(75));
        bar
    });

    mega.rename_node(node, &opts.name)
        .await
        .context("could not rename node within MEGA")?;

    if let Some(bar) = maybe_bar {
        bar.finish_with_message(format!("renamed `{0}` to `{1}` !", node.name(), opts.name));
    }

    Ok(ExitCode::SUCCESS)
}
