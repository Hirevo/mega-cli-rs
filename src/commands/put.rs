use std::path::PathBuf;
use std::process::ExitCode;
use std::time::{Duration, UNIX_EPOCH};

use tokio::fs::File;

use async_read_progress::TokioAsyncReadProgressExt;
use chrono::{TimeZone, Utc};
use clap::Parser;
use color_eyre::eyre::{Context, ContextCompat};
use indicatif::ProgressBar;
use tokio_util::compat::TokioAsyncReadCompatExt;

use crate::config::Config;
use crate::utils;
use crate::utils::terminal::USER_ATTENDED;
use crate::Result;

#[derive(Debug, Clone, PartialEq, Parser)]
#[command(author, rename_all = "kebab-case")]
pub struct Opts {
    /// Path of the input file
    input_file: PathBuf,
    /// Path (eg. `/Root/folder/file.txt) in MEGA to upload the file to
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

    // TODO: allow a folder's handle to be used as well.

    let (parent_path, file_name) = opts
        .path
        .rsplit_once('/')
        .context("empty parent MEGA path")?;

    let file_name = if file_name.is_empty() {
        opts.input_file
            .file_name()
            .context("could not get output file name")?
            .to_str()
            .context("file name is not valid UTF-8")?
    } else {
        file_name
    };

    let full_path = format!("{parent_path}/{file_name}");

    let parent_node = nodes
        .get_node_by_path(parent_path)
        .context("could not find parent node (by path)")?;

    let file = File::open(&opts.input_file)
        .await
        .context("could not open output file")?;
    let metadata = file.metadata().await?;

    let last_modified = {
        let date = metadata
            .modified()
            .context("could not get last modified date for input file")?
            .duration_since(UNIX_EPOCH)
            .context("could not get last modified date for input file")?;

        let seconds = i64::try_from(date.as_secs())
            .context("could not convert timestamp seconds from `u64` to `i64`")?;

        let date = Utc
            .timestamp_opt(seconds, date.subsec_nanos())
            .single()
            .context("could not convert last modification timestamp to `DateTime<Utc>`")?;

        mega::LastModified::Set(date)
    };

    let (reader, mut writer) = sluice::pipe::pipe();

    if *USER_ATTENDED {
        let bar = ProgressBar::new(metadata.len());
        bar.set_style(utils::terminal::standard_progress_style());
        bar.set_message(format!(
            "uploading `{0}` into `{1}`...",
            opts.input_file.display(),
            full_path,
        ));

        let file = {
            let bar = bar.clone();
            file.report_progress(Duration::from_millis(100), move |bytes_read| {
                bar.set_position(bytes_read as u64);
            })
        };

        futures::try_join!(
            async move {
                mega.upload_node(
                    parent_node,
                    file_name,
                    metadata.len(),
                    reader,
                    last_modified,
                )
                .await
                .context("could not upload MEGA node")
            },
            async move {
                futures::io::copy(file.compat(), &mut writer)
                    .await
                    .context("error during `io::copy` operation")
            },
        )?;

        bar.finish_with_message(format!(
            "`{0}` uploaded into `{1}` !",
            opts.input_file.display(),
            full_path,
        ));
    } else {
        futures::try_join!(
            async move {
                mega.upload_node(
                    parent_node,
                    file_name,
                    metadata.len(),
                    reader,
                    last_modified,
                )
                .await
                .context("could not upload MEGA node")
            },
            async move {
                futures::io::copy(file.compat(), &mut writer)
                    .await
                    .context("error during `io::copy` operation")
            },
        )?;
    }

    Ok(ExitCode::SUCCESS)
}
