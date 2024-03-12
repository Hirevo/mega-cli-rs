use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::Arc;
use std::time::Duration;

use tokio::fs::{self, File};

use async_read_progress::AsyncReadProgressExt;
use clap::Parser;
use color_eyre::eyre::{bail, Context, ContextCompat};
use filetime::FileTime;
use futures::future::Either;
use indicatif::{MultiProgress, ProgressBar};
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

use crate::config::Config;
use crate::utils::terminal::USER_ATTENDED;
use crate::Result;
use crate::{utils, Error};

#[derive(Debug, Clone, PartialEq, Parser)]
#[command(author, rename_all = "kebab-case")]
pub struct Opts {
    /// Path to output file
    #[arg(long, short)]
    output: Option<PathBuf>,
    /// Whether to download all nodes
    #[arg(long, short)]
    all: bool,
    /// The shared MEGA link from which to download nodes
    #[arg(long, short)]
    link: Option<String>,
    /// The password to use to decrypt the shared link, if such is used
    #[arg(long, short)]
    password: Option<String>,
    /// The maximum number of parallel file downloads
    #[arg(long, short = 'P', default_value = "4")]
    parallel: usize,
    /// Path (eg. `/Root/folder/file.txt`) or handle (eg. `H:gZlB3JxS`) to the MEGA node to download
    path: Option<String>,
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

        Arc::new(nodes)
    };

    if opts.all {
        return download_all_nodes(mega, &nodes, opts).await;
    }

    let Some(path) = opts.path.as_ref() else {
        bail!("missing target path");
    };

    let root = if path.starts_with("H:") {
        nodes
            .get_node_by_handle(&path[2..])
            .context("could not find node (by handle)")?
    } else {
        nodes
            .get_node_by_path(&path)
            .context("could not find node (by path)")?
    };

    if root.kind().is_file() {
        download_file(mega, &nodes, root, opts).await?;
    } else {
        download_folder(mega, &nodes, root, opts).await?;
    }

    Ok(ExitCode::SUCCESS)
}

/// Returns whether the local file is identical to the remote file.
async fn is_file_already_downloaded(
    maybe_bar: Option<&ProgressBar>,
    node: &mega::Node,
    local_path: &Path,
) -> Result<bool, Error> {
    if !local_path.exists() {
        return Ok(false);
    }

    if !local_path.is_file() {
        // TODO: should we do anything fancier here ?
        return Ok(false);
    }

    let Some(remote_mac) = node.condensed_mac() else {
        return Ok(false);
    };

    let file = File::open(local_path).await?;
    let size = file.metadata().await?.len();

    if size != node.size() {
        return Ok(false);
    }

    let reader = match maybe_bar.cloned() {
        Some(bar) => {
            bar.set_style(utils::terminal::standard_progress_style());
            bar.set_message(format!("checking `{0}`...", local_path.display()));

            bar.set_position(0);
            bar.set_length(size);
            bar.reset();

            Either::Left(file.compat().report_progress(
                Duration::from_millis(100),
                move |bytes_read| {
                    bar.set_position(bytes_read as u64);
                },
            ))
        }
        None => Either::Right(file.compat()),
    };

    let local_mac = mega::compute_condensed_mac(
        reader,
        size,
        node.aes_key(),
        node.aes_iv().unwrap_or(&[0u8; 8]),
    )
    .await?;

    Ok(&local_mac == remote_mac)
}

/// Performs the downloading of a remote MEGA file into a local one.
async fn perform_file_download(
    maybe_bar: Option<&ProgressBar>,
    mega: &mega::Client,
    node: &mega::Node,
    output_path: &Path,
) -> Result<()> {
    // create directories as needed before creating the file
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).await?;
    }

    let file = File::create(&output_path)
        .await
        .context("could not open (or create) output file")?;

    let (reader, writer) = sluice::pipe::pipe();

    if let Some(bar) = maybe_bar {
        bar.set_position(0);
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

/// Downloads a single file from MEGA, with progress reporting.
async fn download_file(
    mega: &Arc<mega::Client>,
    nodes: &Arc<mega::Nodes>,
    node: &mega::Node,
    opts: Opts,
) -> Result<()> {
    let output_path = opts
        .output
        .unwrap_or_else(|| Path::new(".").join(node.name()));

    let maybe_bar = USER_ATTENDED.then(|| ProgressBar::new(node.size()));

    let root_handle = Arc::new(node.handle().to_string());
    let output_path = Arc::new(output_path.to_path_buf());

    let future = || {
        let maybe_bar = maybe_bar.clone();
        let mega = Arc::clone(&mega);
        let nodes = Arc::clone(&nodes);
        let root_handle = Arc::clone(&root_handle);
        let output_path = Arc::clone(&output_path);
        async move {
            let root = nodes
                .get_node_by_handle(&root_handle)
                .context("could not get root node by handle")?;

            if is_file_already_downloaded(maybe_bar.as_ref(), root, &output_path).await? {
                return Ok(());
            }

            if let Some(bar) = maybe_bar.as_ref() {
                bar.set_style(utils::terminal::standard_progress_style());
                bar.set_message(format!(
                    "downloading `{0}` into `{1}`...",
                    root.name(),
                    output_path.display(),
                ));
                bar.set_position(0);
                bar.set_length(root.size());
                bar.reset();
            }

            perform_file_download(maybe_bar.as_ref(), &mega, root, output_path.as_path()).await?;

            Ok::<_, Error>(())
        }
    };

    tokio::spawn(future()).await??;

    if let Some(bar) = maybe_bar.as_ref() {
        bar.finish_and_clear();
    }

    crate::success!(
        to: std::io::stdout(),
        "downloaded `{0}` into `{1}` !",
        node.name(),
        output_path.display(),
    )?;

    Ok(())
}

#[derive(Debug, Clone, PartialEq)]
struct InvolvedNode {
    handle: String,
    remote_path: String,
    local_path: PathBuf,
}

/// Recursively downloads a folder from MEGA, with progress reporting.
async fn download_folder(
    mega: &Arc<mega::Client>,
    nodes: &Arc<mega::Nodes>,
    root: &mega::Node,
    opts: Opts,
) -> Result<()> {
    let output_path = opts
        .output
        .clone()
        .unwrap_or_else(|| Path::new(".").join(root.name()));

    let involved_nodes: Vec<_> = {
        let mut queue = VecDeque::default();
        queue.push_back(root);

        // TODO: maybe we should collect folders as well (to allow fetching empty folders) ?
        std::iter::from_fn(|| loop {
            let node = queue.pop_front()?;
            if node.kind().is_file() {
                let remote_path = utils::nodes::construct_relative_path(&nodes, root, node);
                let local_path = output_path.join(&remote_path[root.name().len() + 1..]);
                return Some(InvolvedNode {
                    handle: node.handle().to_string(),
                    remote_path,
                    local_path,
                });
            }

            for handle in node.children() {
                let Some(child) = nodes.get_node_by_handle(handle) else {
                    continue;
                };

                queue.push_back(child);
            }
        })
        .collect()
    };

    let node_count =
        u64::try_from(involved_nodes.len()).context("could not cast `usize` to `u64`")?;

    let maybe_multi = USER_ATTENDED.then(|| MultiProgress::new());
    let maybe_overall = maybe_multi.as_ref().map(|multi| {
        let bar = multi.add(ProgressBar::new(node_count));
        bar.set_style(utils::terminal::discrete_progress_style());
        bar.set_message(format!(
            "recursively downloading `{0}` into `{1}`...",
            root.name(),
            output_path.display(),
        ));
        bar
    });

    download_aggregate(
        opts,
        mega,
        nodes,
        involved_nodes,
        maybe_overall.clone(),
        maybe_multi.clone(),
    )
    .await?;

    if let Some(multi) = maybe_multi.as_ref() {
        multi.clear()?;
    }

    crate::success!(
        to: std::io::stdout(),
        "recursively downloaded `{0}` into `{1}` !",
        root.name(),
        output_path.display(),
    )?;

    Ok(())
}

/// Recursively downloads a folder from MEGA, with progress reporting.
async fn download_aggregate(
    opts: Opts,
    mega: &Arc<mega::Client>,
    nodes: &Arc<mega::Nodes>,
    involved_nodes: Vec<InvolvedNode>,
    maybe_overall: Option<ProgressBar>,
    maybe_multi: Option<MultiProgress>,
) -> Result<()> {
    let (tx, rx) = async_channel::bounded::<InvolvedNode>(opts.parallel);

    let tasks: Vec<_> = (0..opts.parallel)
        .map(|_| {
            let maybe_multi = maybe_multi.clone();
            let maybe_overall = maybe_overall.clone();
            let mega = Arc::clone(mega);
            let nodes = Arc::clone(nodes);
            let rx = rx.clone();
            tokio::spawn(async move {
                while let Ok(involved_node) = rx.recv().await {
                    let maybe_bar = maybe_multi
                        .as_ref()
                        .map(|multi| multi.add(ProgressBar::new(0)));

                    let involved_node = Arc::new(involved_node);

                    let future = || {
                        let maybe_bar = maybe_bar.clone();
                        let mega = Arc::clone(&mega);
                        let nodes = Arc::clone(&nodes);
                        let involved_node = Arc::clone(&involved_node);
                        async move {
                            let node = nodes
                                .get_node_by_handle(&involved_node.handle)
                                .context("could not get node by handle")?;

                            let already_downloaded = is_file_already_downloaded(
                                maybe_bar.as_ref(),
                                node,
                                &involved_node.local_path,
                            )
                            .await?;

                            if already_downloaded {
                                return Ok(());
                            }

                            if let Some(bar) = maybe_bar.as_ref() {
                                bar.set_style(utils::terminal::standard_progress_style());
                                bar.set_message(format!(
                                    "downloading `{0}`...",
                                    involved_node.remote_path
                                ));
                                bar.set_position(0);
                                bar.set_length(node.size());
                                bar.reset();
                            }

                            perform_file_download(
                                maybe_bar.as_ref(),
                                &mega,
                                node,
                                &involved_node.local_path,
                            )
                            .await?;

                            Ok::<_, Error>(())
                        }
                    };

                    tokio::spawn(future()).await??;

                    let Some(bar) = maybe_bar else {
                        continue;
                    };
                    let Some(multi) = maybe_multi.as_ref() else {
                        continue;
                    };
                    let Some(overall) = maybe_overall.as_ref() else {
                        continue;
                    };

                    multi.remove(&bar);
                    overall.inc(1);
                }

                Ok::<(), Error>(())
            })
        })
        .collect();

    drop(rx);
    for handle in involved_nodes {
        tx.send(handle).await?;
    }
    drop(tx);

    for task in tasks {
        task.await??;
    }

    Ok(())
}

pub async fn download_all_as_folder(
    opts: Opts,
    mega: &Arc<mega::Client>,
    nodes: &Arc<mega::Nodes>,
) -> Result<()> {
    let Some(output_path) = opts.output.clone() else {
        bail!("`-o|--output` required when downloading multiple root nodes");
    };

    let involved_nodes: Vec<_> = {
        let mut queue = VecDeque::default();
        queue.extend(nodes.roots());

        // TODO: maybe we should collect folders as well (to allow fetching empty folders) ?
        std::iter::from_fn(|| loop {
            let node = queue.pop_front()?;
            if node.kind().is_file() {
                let remote_path = utils::nodes::construct_full_path(nodes, node);
                let local_path = output_path.join(&remote_path);
                return Some(InvolvedNode {
                    handle: node.handle().to_string(),
                    remote_path,
                    local_path,
                });
            }

            for handle in node.children() {
                let Some(child) = nodes.get_node_by_handle(handle) else {
                    continue;
                };

                queue.push_back(child);
            }
        })
        .collect()
    };

    let node_count =
        u64::try_from(involved_nodes.len()).context("could not cast `usize` to `u64`")?;

    let maybe_multi = USER_ATTENDED.then(|| MultiProgress::new());
    let maybe_overall = maybe_multi.as_ref().map(|multi| {
        let bar = multi.add(ProgressBar::new(node_count));
        bar.set_style(utils::terminal::discrete_progress_style());
        bar.set_message(format!(
            "recursively downloading all nodes into `{}`...",
            output_path.display(),
        ));
        bar
    });

    download_aggregate(
        opts,
        mega,
        nodes,
        involved_nodes,
        maybe_overall.clone(),
        maybe_multi.clone(),
    )
    .await?;

    if let Some(multi) = maybe_multi.as_ref() {
        multi.clear()?;
    }

    crate::success!(
        to: std::io::stdout(),
        "recursively downloaded all nodes into `{}` !",
        output_path.display(),
    )?;

    Ok(())
}

/// Downloads all nodes from MEGA, with progress reporting.
async fn download_all_nodes(
    mega: &Arc<mega::Client>,
    nodes: &Arc<mega::Nodes>,
    opts: Opts,
) -> Result<ExitCode> {
    if opts.path.is_some() {
        bail!("target path supplied alongside `--all`");
    }

    let roots: Vec<&mega::Node> = nodes.roots().collect();

    match roots.as_slice() {
        [node] if node.kind().is_file() => {
            download_file(mega, nodes, node, opts).await?;
        }
        [node] => {
            download_folder(mega, nodes, node, opts).await?;
        }
        _ => {
            download_all_as_folder(opts, mega, nodes).await?;
        }
    }

    Ok(ExitCode::SUCCESS)
}
