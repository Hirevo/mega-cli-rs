use std::process::ExitCode;
use std::time::Duration;

use async_read_progress::TokioAsyncReadProgressExt;
use clap::Parser;
use color_eyre::eyre::{Context, ContextCompat};
use indicatif::ProgressBar;
use tokio::fs::File;
use tokio_util::compat::TokioAsyncReadCompatExt;

use crate::config::Config;
use crate::utils;
use crate::utils::terminal::USER_ATTENDED;
use crate::Result;

#[derive(Debug, Clone, PartialEq, Parser)]
#[command(author, rename_all = "kebab-case")]
pub struct Opts {
    /// The shared MEGA link from which to list nodes
    #[arg(long, short)]
    link: Option<String>,
    /// The password to use to decrypt the shared link, if such is used
    #[arg(long, short)]
    password: Option<String>,
    /// Path (eg. `/Root/folder`) or handle (eg. `H:gZlB3JxS`) to the MEGA file to compare with
    #[arg(long)]
    remote: String,
    /// Path to the local file to compare with
    #[arg(long)]
    local: String,
}

impl Opts {
    pub fn may_need_user_session(&self) -> bool {
        self.link.is_none() && self.password.is_none()
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

        let nodes = match (opts.link.as_deref(), opts.password.as_deref()) {
            (None, None) => mega
                .fetch_own_nodes()
                .await
                .context("could net fetch own MEGA nodes")?,
            (Some(link), None) => mega
                .fetch_public_nodes(link)
                .await
                .context("could net fetch password-protected MEGA nodes")?,
            (Some(link), Some(password)) => mega
                .fetch_protected_nodes(link, password)
                .await
                .context("could net fetch password-protected MEGA nodes")?,
            (None, Some(_)) => {
                todo!()
            }
        };

        if let Some(bar) = maybe_bar {
            bar.finish_and_clear();
        }

        nodes
    };

    let node = if opts.remote.starts_with("H:") {
        nodes
            .get_node_by_handle(&opts.remote[2..])
            .context("could not find node (by handle)")?
    } else {
        nodes
            .get_node_by_path(&opts.remote)
            .context("could not find node (by path)")?
    };

    let (remote_condensed_mac, key, iv) = {
        let condensed_mac = node.condensed_mac().unwrap();
        let key = node.aes_key();
        let iv = node.aes_iv().unwrap();
        (condensed_mac, key, iv)
    };

    let local_condensed_mac = {
        let file = File::open(&opts.local)
            .await
            .context("could not open local file")?;

        let size = file
            .metadata()
            .await
            .context("could not get file metadata")?
            .len();

        let bar = ProgressBar::new(size);
        bar.set_style(utils::terminal::standard_progress_style());
        bar.set_message(format!("computing MAC for `{0}`...", opts.local));

        let reader = {
            let bar = bar.clone();
            file.report_progress(Duration::from_millis(100), move |bytes_read| {
                bar.set_position(bytes_read as u64);
            })
        };

        let condensed_mac = mega::compute_condensed_mac(reader.compat(), size, key, iv)
            .await
            .context("could not compute local condensed MAC")?;

        bar.finish_and_clear();
        condensed_mac
    };

    if local_condensed_mac == *remote_condensed_mac {
        crate::success!(to: std::io::stdout(), "OK ! (the MACs are identical)")?;
        Ok(ExitCode::SUCCESS)
    } else {
        crate::error!(to: std::io::stdout(), "FAILED ! (the MACs differ)")?;
        Ok(ExitCode::FAILURE)
    }
}
