use anyhow::{bail, Result};
use std::path::PathBuf;

#[derive(clap::Args)]
pub(crate) struct Args {
    /// CROP executable to invoke for corpus indexing
    #[arg(long, global = true, default_value = "crop")]
    crop_bin: PathBuf,

    #[command(flatten)]
    page: CorpusPageArgs,
}

#[derive(clap::Args)]
struct CorpusPageArgs {
    /// Root directory or file to index/catalog
    #[arg(long)]
    root: Option<PathBuf>,
    /// crop.view.v1 recipe to index/catalog
    #[arg(long)]
    view: Option<PathBuf>,
    /// Page title. Defaults to CROP's root/view-derived title
    #[arg(long)]
    title: Option<String>,
    /// Restrict files to one or more extensions, e.g. --extension md
    #[arg(long = "extension")]
    extensions: Vec<String>,
    /// Exclude directories by basename
    #[arg(long = "exclude-dir")]
    exclude_dirs: Vec<String>,
    /// Optional Markdown output path. Defaults to stdout
    #[arg(long)]
    output: Option<PathBuf>,
}

pub(crate) fn run_index(args: Args) -> Result<()> {
    run_crop_page("index", args)
}

pub(crate) fn run_toc(mut args: Args) -> Result<()> {
    if args.page.title.is_none() {
        args.page.title = Some("Table of Contents".to_string());
    }
    run_crop_page("index", args)
}

pub(crate) fn run_catalog(args: Args) -> Result<()> {
    run_crop_page("catalog", args)
}

fn run_crop_page(command: &str, args: Args) -> Result<()> {
    let page = args.page;
    if page.root.is_some() && page.view.is_some() {
        bail!(
            "proof {} accepts either --root or --view, not both",
            command
        );
    }

    let mut crop_args = vec![command.to_string()];
    if let Some(root) = page.root {
        crop_args.push("--root".to_string());
        crop_args.push(root.display().to_string());
    }
    if let Some(view) = page.view {
        crop_args.push("--view".to_string());
        crop_args.push(view.display().to_string());
    }
    if let Some(title) = page.title {
        crop_args.push("--title".to_string());
        crop_args.push(title);
    }
    for extension in page.extensions {
        crop_args.push("--extension".to_string());
        crop_args.push(extension);
    }
    for exclude_dir in page.exclude_dirs {
        crop_args.push("--exclude-dir".to_string());
        crop_args.push(exclude_dir);
    }
    if let Some(output) = page.output {
        crop_args.push("--output".to_string());
        crop_args.push(output.display().to_string());
    }

    crate::cmd_crop::run_crop(args.crop_bin, crop_args)
}
