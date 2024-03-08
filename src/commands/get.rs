use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::Arc;
use std::time::Duration;

use tokio::fs::{self, File};

use async_read_progress::AsyncReadProgressExt;
use clap::Parser;
use color_eyre::eyre::{Context, ContextCompat};
use filetime::FileTime;
use futures::stream::{self, StreamExt};
use indicatif::{MultiProgress, ProgressBar};
use tokio_util::compat::TokioAsyncWriteCompatExt;

use crate::config::Config;
use crate::utils;
use crate::utils::terminal::USER_ATTENDED;
use crate::Result;

#[derive(Debug, Clone, PartialEq, Parser)]
#[command(author, rename_all = "kebab-case")]
pub struct Opts {
    /// Path to output file
    #[arg(long, short)]
    output: Option<PathBuf>,
    /// The shared MEGA link from which to download nodes.
    #[arg(long, short)]
    link: Option<String>,
    /// The password to use to decrypt the shared link, if such is used.
    #[arg(long, short)]
    password: Option<String>,
    /// The maximum number of parallel file downloads
    #[arg(long, short, default_value = "4")]
    parallel: usize,
    /// Path (eg. `/Root/folder/file.txt`) or handle (eg. `H:gZlB3JxS`) to the MEGA node to download
    path: String,
}

impl Opts {
    pub fn may_need_user_session(&self) -> bool {
        self.link.is_none() && self.password.is_none()
    }
}

pub async fn handle(_: Config, mega: &Arc<mega::Client>, opts: Opts) -> Result<ExitCode> {
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

    let root = if opts.path.starts_with("H:") {
        nodes
            .get_node_by_handle(&opts.path[2..])
            .context("could not find node (by handle)")?
    } else {
        nodes
            .get_node_by_path(&opts.path)
            .context("could not find node (by path)")?
    };

    if root.kind().is_file() {
        let output_path = opts
            .output
            .unwrap_or_else(|| Path::new(".").join(root.name()));

        // let root_path = utils::nodes::construct_full_path(&nodes, root);

        let maybe_multi = USER_ATTENDED.then(|| MultiProgress::new());
        download_file(maybe_multi.as_ref(), mega, &nodes, root, &output_path, None).await?;

        if let Some(multi) = maybe_multi.as_ref() {
            multi.clear()?;
        }

        crate::success!(
            to: std::io::stdout(),
            "downloaded `{0}` into `{1}` !",
            root.name(),
            output_path.display(),
        )?;
    } else {
        let output_path = opts.output.as_deref().unwrap_or_else(|| Path::new("."));

        let root_path = utils::nodes::construct_full_path(&nodes, root);

        let mut queue = Vec::default();
        queue.push(root);

        // Collect all the nodes to be downloaded.
        let involved_nodes: Vec<_> = std::iter::from_fn(|| loop {
            let node = queue.pop()?;

            if !node.kind().is_file() {
                for handle in node.children() {
                    let node = nodes.get_node_by_handle(handle).unwrap();
                    queue.push(node);
                }
            }

            // Since we'll be using `fs::create_dir_all`, we only need to consider the leaf nodes of the subtree.
            // A leaf node is either a file, or an empty folder.
            if node.kind().is_file() || node.children().is_empty() {
                break Some(node);
            }
        })
        .collect();

        let node_count =
            u64::try_from(involved_nodes.len()).context("could not cast `usize` to `u64`")?;

        let maybe_multi = USER_ATTENDED.then(|| MultiProgress::new());
        let maybe_overall = maybe_multi.as_ref().map(|multi| {
            let bar = multi.add(ProgressBar::new(node_count));
            bar.set_style(utils::terminal::discrete_progress_style());
            bar.set_message(format!("recursively downloading `{0}`...", root.name()));
            bar
        });

        let mut stream = stream::iter(involved_nodes.into_iter().map(|node| async {
            let relative_path = utils::nodes::construct_relative_path(&nodes, root, node);

            let output_path = output_path.join(&relative_path);

            if node.kind().is_file() {
                download_file(
                    maybe_multi.as_ref(),
                    mega,
                    &nodes,
                    node,
                    &output_path,
                    Some(&root_path),
                )
                .await?;
            } else {
                fs::create_dir_all(&output_path).await?;
            }

            Ok::<_, crate::Error>(())
        }))
        .buffer_unordered(opts.parallel);

        while let Some(_) = stream.next().await.transpose()? {
            if let Some(overall) = maybe_overall.as_ref() {
                overall.inc(1);
            }
        }

        if let Some(multi) = maybe_multi.as_ref() {
            multi.clear()?;
        }

        crate::success!(
            to: std::io::stdout(),
            "recursively downloaded `{0}` into `{1}` !",
            root.name(),
            output_path.join(root.name()).display(),
        )?;
    }

    Ok(ExitCode::SUCCESS)
}

async fn download_file(
    maybe_multi: Option<&MultiProgress>,
    mega: &mega::Client,
    nodes: &mega::Nodes,
    node: &mega::Node,
    output_path: &Path,
    root_path: Option<&str>,
) -> Result<()> {
    // create directories as needed before creating the file
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).await?;
    }

    let file = File::create(&output_path)
        .await
        .context("could not open (or create) output file")?;

    let (reader, writer) = sluice::pipe::pipe();

    let full_path = utils::nodes::construct_full_path(&nodes, node);

    // in case of a recursive download, only print the relative path from the downloaded folder.
    let displayed_path = root_path
        .and_then(|root_path| full_path.strip_prefix(root_path))
        .map(|path| path.trim_start_matches('/'))
        .unwrap_or_else(|| node.name());

    if let Some(multi) = maybe_multi {
        let bar = multi.add(ProgressBar::new(node.size()));
        bar.set_style(utils::terminal::standard_progress_style());
        bar.set_message(format!("downloading `{displayed_path}`..."));

        let reader = {
            let bar = bar.clone();
            reader.report_progress(Duration::from_millis(100), move |bytes_read| {
                bar.set_position(bytes_read as u64);
            })
        };

        futures::try_join!(
            async move {
                mega.download_node(node, writer)
                    .await
                    .context("could not download MEGA node")
            },
            async move {
                futures::io::copy(reader, &mut file.compat_write())
                    .await
                    .context("error during `io::copy` operation")
            },
        )?;

        multi.remove(&bar);
    } else {
        futures::try_join!(
            async move {
                mega.download_node(node, writer)
                    .await
                    .context("could not download MEGA node")
            },
            async move {
                futures::io::copy(reader, &mut file.compat_write())
                    .await
                    .context("error during `io::copy` operation")
            },
        )?;
    }

    // restore last modification date from MEGA
    if let Some(modified_at) = node.modified_at() {
        let mtime = FileTime::from_unix_time(
            modified_at.timestamp(),
            modified_at.timestamp_subsec_nanos(),
        );
        filetime::set_file_mtime(&output_path, mtime)
            .context("could not restore last modification date")?
    }

    Ok(())
}
