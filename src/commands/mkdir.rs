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
    /// Create parent directories as needed (no errors if all folders already exists)
    #[arg(long, short)]
    parents: bool,
    /// Path (eg. `/Root/folder/subfolder) to the folder to create
    path: String,
}

impl Opts {
    pub fn may_need_user_session(&self) -> bool {
        true
    }
}

pub async fn handle(_: Config, mega: &mega::Client, opts: Opts) -> Result<ExitCode> {
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

    let path = opts.path.trim_end_matches('/');
    let steps = {
        let mut steps: Vec<(&str, &str)> =
            std::iter::successors(path.rsplit_once('/'), |(path, _)| path.rsplit_once('/'))
                .collect();

        if let Some((last, _)) = steps.last().copied() {
            if last == "" {
                steps.pop();
            }
        }
        steps.reverse();

        steps
    };

    if opts.parents {
        for (parent, folder_name) in steps {
            let full_path = format!("{parent}/{folder_name}");
            if nodes.get_node_by_path(&full_path).is_some() {
                continue;
            }

            let parent_node = nodes
                .get_node_by_path(parent)
                .context("could not find parent folder")?;

            let parent_handle = parent_node.handle().to_string();

            let maybe_bar = USER_ATTENDED.then(|| {
                let bar = ProgressBar::new_spinner();
                bar.set_style(utils::terminal::spinner_style());
                bar.set_message(format!("creating folder `{full_path}`..."));
                bar.enable_steady_tick(Duration::from_millis(75));
                bar
            });

            mega.create_folder(parent_node, folder_name)
                .await
                .context("could not create folder within MEGA")?;

            let mut is_applied = false;
            while !is_applied {
                let events = mega.wait_events(&nodes).await?;
                is_applied = events.events().iter().any(|event| match event {
                    mega::Event::NodeCreated { nodes } => nodes.iter().any(|node| {
                        node.name() == folder_name && node.parent() == Some(parent_handle.as_str())
                    }),
                    _ => false,
                });
                nodes.apply_events(events)?;
            }

            if let Some(bar) = maybe_bar {
                bar.finish_with_message(format!("created folder `{full_path}` !"));
            }
        }
    } else if let Some((parent, folder_name)) = steps.last().copied() {
        let full_path = format!("{parent}/{folder_name}");
        if nodes.get_node_by_path(&full_path).is_some() {
            crate::error!(to: std::io::stderr(), from: "mega:mkdir", "target folder already exists")?;
            return Ok(ExitCode::FAILURE);
        }

        let parent_node = nodes
            .get_node_by_path(parent)
            .context("could not find parent folder")?;

        let maybe_bar = USER_ATTENDED.then(|| {
            let bar = ProgressBar::new_spinner();
            bar.set_style(utils::terminal::spinner_style());
            bar.set_message(format!("creating folder `{full_path}`..."));
            bar.enable_steady_tick(Duration::from_millis(75));
            bar
        });

        mega.create_folder(parent_node, folder_name)
            .await
            .context("could not create folder within MEGA")?;

        if let Some(bar) = maybe_bar {
            bar.finish_with_message(format!("created folder `{full_path}` !"));
        }
    }

    Ok(ExitCode::SUCCESS)
}
