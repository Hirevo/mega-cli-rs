use std::process::ExitCode;
use std::time::Duration;

use clap::Parser;
use color_eyre::eyre::{Context, ContextCompat};
use indicatif::ProgressBar;
use text_trees::{FormatCharacters, StringTreeNode, TreeFormatting};

use crate::config::Config;
use crate::utils;
use crate::utils::terminal::USER_ATTENDED;
use crate::Result;

#[derive(Debug, Clone, PartialEq, Parser)]
#[command(author, rename_all = "kebab-case")]
pub struct Opts {
    /// Show node handles (eg. `H:gZlB3JxS`)
    #[arg(long)]
    show_handles: bool,
    /// The shared MEGA link from which to list nodes.
    #[arg(long, short)]
    link: Option<String>,
    /// The password to use to decrypt the shared link, if such is used.
    #[arg(long, short)]
    password: Option<String>,
    /// Path (eg. `/Root/folder`) or handle (eg. `H:gZlB3JxS`) to the MEGA folder to list from
    path: Option<String>,
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

    let formatting = TreeFormatting::dir_tree(FormatCharacters::box_chars());

    if let Some(path) = opts.path {
        let node = if path.starts_with("H:") {
            nodes
                .get_node_by_handle(&path[2..])
                .context("could not find node (by handle)")?
        } else {
            nodes
                .get_node_by_path(&path)
                .context("could not find node (by path)")?
        };

        let tree = construct_tree_node(&nodes, node, opts.show_handles);
        tree.write_with_format(&mut std::io::stdout(), &formatting)?;
    } else {
        let (mut folders, mut files): (Vec<_>, Vec<_>) =
            nodes.roots().partition(|node| node.kind().is_folder());

        folders.sort_unstable_by_key(|node| node.name());
        files.sort_unstable_by_key(|node| node.name());

        for root in folders.into_iter().chain(files) {
            let tree = construct_tree_node(&nodes, root, opts.show_handles);
            tree.write_with_format(&mut std::io::stdout(), &formatting)?;
        }
    }

    Ok(ExitCode::SUCCESS)
}

fn construct_tree_node(
    nodes: &mega::Nodes,
    node: &mega::Node,
    show_handles: bool,
) -> StringTreeNode {
    let (mut folders, mut files): (Vec<_>, Vec<_>) = node
        .children()
        .iter()
        .filter_map(|hash| nodes.get_node_by_handle(hash))
        .partition(|node| node.kind().is_folder());

    folders.sort_unstable_by_key(|node| node.name());
    files.sort_unstable_by_key(|node| node.name());

    let label = format_node_label(node, show_handles);
    let children = std::iter::empty()
        .chain(folders)
        .chain(files)
        .map(|node| construct_tree_node(nodes, node, show_handles));

    StringTreeNode::with_child_nodes(label, children)
}

fn format_node_label(node: &mega::Node, show_handles: bool) -> String {
    if node.kind().is_file() {
        if show_handles {
            format!("(H:{0}) {1}", node.handle(), node.name())
        } else {
            format!("{0}", node.name())
        }
    } else {
        if show_handles {
            format!("(H:{0}) {1}/", node.handle(), node.name())
        } else {
            format!("{0}/", node.name())
        }
    }
}
