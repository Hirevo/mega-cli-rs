use std::process::ExitCode;

use tokio::process::Command;

use clap::Parser;
use color_eyre::eyre::{Context, ContextCompat};
use dialoguer::theme::ColorfulTheme;
use dialoguer::Input;

use crate::config::{Config, CONFIG_NAME};
use crate::utils::terminal::USER_ATTENDED;
use crate::Result;

#[derive(Debug, Clone, PartialEq, Parser)]
#[command(author, rename_all = "kebab-case")]
pub struct Opts {
    /// Command to use to open the text editor
    #[arg(long, env)]
    editor: Option<String>,
}

impl Opts {
    pub fn may_need_user_session(&self) -> bool {
        false
    }
}

pub async fn handle(_: Config, _: &mega::Client, opts: Opts) -> Result<ExitCode> {
    let path = confy::get_configuration_file_path(CONFIG_NAME, None)?;

    let maybe_editor = (opts.editor.map(Ok))
        .or_else(|| {
            USER_ATTENDED.then(|| {
                let theme = ColorfulTheme::default();
                let editor = Input::with_theme(&theme)
                    .with_prompt("Enter the editor command to use")
                    .interact_text();
                editor
            })
        })
        .transpose()?;

    let Some(editor) = maybe_editor else {
        crate::error!(to: std::io::stderr(), from: "mega:config", "could not determine the editor to use")?;
        crate::error!(to: std::io::stderr(), from: "mega:config", "please specify an editor via CLI arguments or using the `EDITOR` environment variable")?;
        return Ok(ExitCode::FAILURE);
    };

    let words =
        shell_words::split(&editor).context("could not split EDITOR command into shell words")?;

    let (command, arguments) = words
        .split_first()
        .context("no words in EDITOR shell command")?;

    let status = Command::new(command)
        .args(arguments)
        .arg(path)
        .status()
        .await
        .context("could not spawn EDITOR process and wait for it to complete")?;

    let code = status
        .success()
        .then_some(ExitCode::SUCCESS)
        .unwrap_or(ExitCode::FAILURE);

    if status.success() {
        crate::success!(
            to: std::io::stdout(),
            "configuration successfully edited ({status}) !"
        )?;
    } else {
        crate::error!(
            to: std::io::stdout(),
            "error when editing configuration ({status}) !"
        )?;
    }

    Ok(code)
}
