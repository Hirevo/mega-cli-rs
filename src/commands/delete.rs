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
    /// Whether to move the file to the Rubbish Bin, instead of hard-deleting
    #[arg(long)]
    soft: bool,
    /// Path (eg. `/Root/folder/file.txt`) or handle (eg. `H:gZlB3JxS`) to the MEGA node to delete
    path: String,
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

    if opts.soft {
        let maybe_bar = USER_ATTENDED.then(|| {
            let bar = ProgressBar::new_spinner();
            bar.set_style(utils::terminal::spinner_style());
            bar.set_message(format!("moving `{0}` to the Rubbish Bin...", node.name()));
            bar.enable_steady_tick(Duration::from_millis(75));
            bar
        });

        let rubbish_bin = nodes.rubbish_bin().context("could not find Rubbish Bin")?;
        mega.move_node(node, rubbish_bin)
            .await
            .context("could not move node to the Rubbish Bin")?;

        if let Some(bar) = maybe_bar {
            bar.finish_with_message(format!("moved `{0}` to the Rubbish Bin !", node.name()));
        }
    } else {
        let maybe_bar = USER_ATTENDED.then(|| {
            let bar = ProgressBar::new_spinner();
            bar.set_style(utils::terminal::spinner_style());
            bar.set_message(format!("deleting `{0}`...", node.name()));
            bar.enable_steady_tick(Duration::from_millis(75));
            bar
        });

        mega.delete_node(node)
            .await
            .context("could not delete MEGA node")?;

        if let Some(bar) = maybe_bar {
            bar.finish_with_message(format!("deleted `{0}` !", node.name()));
        }
    }

    Ok(ExitCode::SUCCESS)
}
