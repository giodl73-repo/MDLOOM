use anyhow::{bail, Result};
use std::path::PathBuf;

use crate::cmd_context::GlobalOptions;

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

pub(crate) fn run_index_with_globals(args: Args, globals: &GlobalOptions) -> Result<()> {
    run_index(apply_global_output(args, globals))
}

pub(crate) fn run_index(args: Args) -> Result<()> {
    run_crop_page("index", args)
}

pub(crate) fn run_toc_with_globals(args: Args, globals: &GlobalOptions) -> Result<()> {
    run_toc(apply_global_output(args, globals))
}

pub(crate) fn run_toc(mut args: Args) -> Result<()> {
    if args.page.title.is_none() {
        args.page.title = Some("Table of Contents".to_string());
    }
    run_crop_page("index", args)
}

pub(crate) fn run_catalog_with_globals(args: Args, globals: &GlobalOptions) -> Result<()> {
    run_catalog(apply_global_output(args, globals))
}

pub(crate) fn run_catalog(args: Args) -> Result<()> {
    run_crop_page("catalog", args)
}

fn apply_global_output(mut args: Args, globals: &GlobalOptions) -> Args {
    if args.page.output.is_none() {
        args.page.output = globals.output().clone();
    }
    args
}

fn run_crop_page(command: &str, args: Args) -> Result<()> {
    let crop_bin = args.crop_bin.clone();
    crate::cmd_crop::run_crop(crop_bin, build_crop_page_args(command, args)?)
}

fn build_crop_page_args(command: &str, args: Args) -> Result<Vec<String>> {
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

    Ok(crop_args)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn globals(output: Option<PathBuf>) -> GlobalOptions {
        GlobalOptions::new(None, "text".to_string(), false, false, output)
    }

    fn args(page: CorpusPageArgs) -> Args {
        Args {
            crop_bin: PathBuf::from("crop"),
            page,
        }
    }

    #[test]
    fn index_args_map_to_crop_index() {
        let crop_args = build_crop_page_args(
            "index",
            args(CorpusPageArgs {
                root: Some(PathBuf::from("docs")),
                view: None,
                title: Some("Guide Index".to_string()),
                extensions: vec!["md".to_string()],
                exclude_dirs: vec!["target".to_string()],
                output: Some(PathBuf::from("INDEX.md")),
            }),
        )
        .unwrap();

        assert_eq!(
            crop_args,
            vec![
                "index",
                "--root",
                "docs",
                "--title",
                "Guide Index",
                "--extension",
                "md",
                "--exclude-dir",
                "target",
                "--output",
                "INDEX.md"
            ]
        );
    }

    #[test]
    fn catalog_args_map_to_crop_catalog_view() {
        let crop_args = build_crop_page_args(
            "catalog",
            args(CorpusPageArgs {
                root: None,
                view: Some(PathBuf::from("ready.json")),
                title: None,
                extensions: vec![],
                exclude_dirs: vec![],
                output: Some(PathBuf::from("CATALOG.md")),
            }),
        )
        .unwrap();

        assert_eq!(
            crop_args,
            vec!["catalog", "--view", "ready.json", "--output", "CATALOG.md"]
        );
    }

    #[test]
    fn global_output_is_used_when_page_output_missing() {
        let crop_args = build_crop_page_args(
            "index",
            apply_global_output(
                args(CorpusPageArgs {
                    root: Some(PathBuf::from("docs")),
                    view: None,
                    title: None,
                    extensions: vec![],
                    exclude_dirs: vec![],
                    output: None,
                }),
                &globals(Some(PathBuf::from("GLOBAL.md"))),
            ),
        )
        .unwrap();

        assert_eq!(
            crop_args,
            vec!["index", "--root", "docs", "--output", "GLOBAL.md"]
        );
    }

    #[test]
    fn page_output_overrides_global_output() {
        let crop_args = build_crop_page_args(
            "catalog",
            apply_global_output(
                args(CorpusPageArgs {
                    root: None,
                    view: Some(PathBuf::from("ready.json")),
                    title: None,
                    extensions: vec![],
                    exclude_dirs: vec![],
                    output: Some(PathBuf::from("LOCAL.md")),
                }),
                &globals(Some(PathBuf::from("GLOBAL.md"))),
            ),
        )
        .unwrap();

        assert_eq!(
            crop_args,
            vec!["catalog", "--view", "ready.json", "--output", "LOCAL.md"]
        );
    }

    #[test]
    fn toc_sets_default_title_when_missing() {
        let mut toc_args = args(CorpusPageArgs {
            root: Some(PathBuf::from("docs")),
            view: None,
            title: None,
            extensions: vec![],
            exclude_dirs: vec![],
            output: None,
        });
        if toc_args.page.title.is_none() {
            toc_args.page.title = Some("Table of Contents".to_string());
        }

        let crop_args = build_crop_page_args("index", toc_args).unwrap();

        assert_eq!(
            crop_args,
            vec!["index", "--root", "docs", "--title", "Table of Contents"]
        );
    }

    #[test]
    fn index_rejects_root_and_view() {
        let err = build_crop_page_args(
            "index",
            args(CorpusPageArgs {
                root: Some(PathBuf::from("docs")),
                view: Some(PathBuf::from("view.json")),
                title: None,
                extensions: vec![],
                exclude_dirs: vec![],
                output: None,
            }),
        )
        .unwrap_err();

        assert!(err.to_string().contains("either --root or --view"));
    }
}
