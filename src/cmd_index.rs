use anyhow::{bail, Result};
use std::path::PathBuf;

use crate::cmd_context::GlobalOptions;

#[derive(clap::Args)]
pub(crate) struct Args {
    /// MDCROP executable to invoke for corpus indexing
    #[arg(long, global = true, default_value = "mdcrop")]
    mdcrop_bin: PathBuf,

    #[command(flatten)]
    page: CorpusPageArgs,
}

#[derive(clap::Args)]
struct CorpusPageArgs {
    /// Root directory or file to index/catalog
    #[arg(long)]
    root: Option<PathBuf>,
    /// mdcrop.view.v1 recipe to index/catalog
    #[arg(long)]
    view: Option<PathBuf>,
    /// Page title. Defaults to MDCROP's root/view-derived title
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
    reject_non_markdown_global_format("index", globals)?;
    run_index(apply_global_output(args, globals))
}

pub(crate) fn run_index(args: Args) -> Result<()> {
    run_mdcrop_page("index", args)
}

pub(crate) fn run_toc_with_globals(args: Args, globals: &GlobalOptions) -> Result<()> {
    reject_non_markdown_global_format("toc", globals)?;
    run_toc(apply_global_output(args, globals))
}

pub(crate) fn run_toc(mut args: Args) -> Result<()> {
    if args.page.title.is_none() {
        args.page.title = Some("Table of Contents".to_string());
    }
    run_mdcrop_page("index", args)
}

pub(crate) fn run_catalog_with_globals(args: Args, globals: &GlobalOptions) -> Result<()> {
    reject_non_markdown_global_format("catalog", globals)?;
    run_catalog(apply_global_output(args, globals))
}

pub(crate) fn run_catalog(args: Args) -> Result<()> {
    run_mdcrop_page("catalog", args)
}

fn apply_global_output(mut args: Args, globals: &GlobalOptions) -> Args {
    if args.page.output.is_none() {
        args.page.output = globals.output().clone();
    }
    args
}

fn reject_non_markdown_global_format(command: &str, globals: &GlobalOptions) -> Result<()> {
    match globals.format() {
        "text" | "markdown" => Ok(()),
        other => bail!(
            "mdloom {} is Markdown-only; use text/markdown output format, got {:?}",
            command,
            other
        ),
    }
}

fn run_mdcrop_page(command: &str, args: Args) -> Result<()> {
    let mdcrop_bin = args.mdcrop_bin.clone();
    crate::cmd_mdcrop::run_mdcrop(mdcrop_bin, build_mdcrop_page_args(command, args)?)
}

fn build_mdcrop_page_args(command: &str, args: Args) -> Result<Vec<String>> {
    let page = args.page;
    if page.root.is_some() && page.view.is_some() {
        bail!(
            "mdloom {} accepts either --root or --view, not both",
            command
        );
    }
    if page.root.is_none() && page.view.is_none() {
        bail!("mdloom {} requires --root or --view", command);
    }

    let mut mdcrop_args = vec![command.to_string()];
    if let Some(root) = page.root {
        mdcrop_args.push("--root".to_string());
        mdcrop_args.push(root.display().to_string());
    }
    if let Some(view) = page.view {
        mdcrop_args.push("--view".to_string());
        mdcrop_args.push(view.display().to_string());
    }
    if let Some(title) = page.title {
        mdcrop_args.push("--title".to_string());
        mdcrop_args.push(title);
    }
    for extension in page.extensions {
        mdcrop_args.push("--extension".to_string());
        mdcrop_args.push(extension);
    }
    for exclude_dir in page.exclude_dirs {
        mdcrop_args.push("--exclude-dir".to_string());
        mdcrop_args.push(exclude_dir);
    }
    if let Some(output) = page.output {
        mdcrop_args.push("--output".to_string());
        mdcrop_args.push(output.display().to_string());
    }

    Ok(mdcrop_args)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn globals(output: Option<PathBuf>) -> GlobalOptions {
        GlobalOptions::new(None, "text".to_string(), false, false, output)
    }

    fn globals_with_format(format: &str) -> GlobalOptions {
        GlobalOptions::new(None, format.to_string(), false, false, None)
    }

    fn args(page: CorpusPageArgs) -> Args {
        Args {
            mdcrop_bin: PathBuf::from("mdcrop"),
            page,
        }
    }

    #[test]
    fn index_args_map_to_mdcrop_index() {
        let mdcrop_args = build_mdcrop_page_args(
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
            mdcrop_args,
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
    fn catalog_args_map_to_mdcrop_catalog_view() {
        let mdcrop_args = build_mdcrop_page_args(
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
            mdcrop_args,
            vec!["catalog", "--view", "ready.json", "--output", "CATALOG.md"]
        );
    }

    #[test]
    fn global_output_is_used_when_page_output_missing() {
        let mdcrop_args = build_mdcrop_page_args(
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
            mdcrop_args,
            vec!["index", "--root", "docs", "--output", "GLOBAL.md"]
        );
    }

    #[test]
    fn page_output_overrides_global_output() {
        let mdcrop_args = build_mdcrop_page_args(
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
            mdcrop_args,
            vec!["catalog", "--view", "ready.json", "--output", "LOCAL.md"]
        );
    }

    #[test]
    fn page_commands_reject_non_markdown_global_format() {
        let err =
            reject_non_markdown_global_format("index", &globals_with_format("json")).unwrap_err();

        assert!(err.to_string().contains("Markdown-only"));
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

        let mdcrop_args = build_mdcrop_page_args("index", toc_args).unwrap();

        assert_eq!(
            mdcrop_args,
            vec!["index", "--root", "docs", "--title", "Table of Contents"]
        );
    }

    #[test]
    fn index_rejects_root_and_view() {
        let err = build_mdcrop_page_args(
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

    #[test]
    fn index_requires_root_or_view() {
        let err = build_mdcrop_page_args(
            "index",
            args(CorpusPageArgs {
                root: None,
                view: None,
                title: None,
                extensions: vec![],
                exclude_dirs: vec![],
                output: None,
            }),
        )
        .unwrap_err();

        assert!(err.to_string().contains("requires --root or --view"));
    }
}
