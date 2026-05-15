use crate::cmd_context::GlobalOptions;
use anyhow::{bail, Context, Result};
use clap::Subcommand;
use std::path::PathBuf;
use std::process::{self, Command};

#[derive(clap::Args)]
pub(crate) struct Args {
    /// CROP executable to invoke
    #[arg(long, global = true, default_value = "crop")]
    crop_bin: PathBuf,

    #[command(subcommand)]
    command: CropCommand,
}

#[derive(Subcommand)]
enum CropCommand {
    /// Generate a CROP corpus status page for a root or named view
    Status(StatusArgs),
    /// Validate CROP view recipes in a view store
    InspectViews(InspectViewsArgs),
    /// Generate local link side-info
    Links(SideInfoArgs),
    /// Generate backlink and orphan side-info
    Backlinks(SideInfoArgs),
    /// Generate frontmatter inventory side-info
    Frontmatter(SideInfoArgs),
    /// Generate heading inventory side-info
    Headings(SideInfoArgs),
    /// Report PROOF generated artifact manifest health through CROP
    Artifacts(ArtifactsArgs),
}

#[derive(clap::Args)]
struct StatusArgs {
    /// Documentation/source root to scan
    #[arg(long)]
    root: Option<PathBuf>,
    /// crop.view.v1 recipe to scan
    #[arg(long)]
    view: Option<PathBuf>,
    /// Status page title
    #[arg(long)]
    title: Option<String>,
    /// Restrict scanned files to one or more extensions, e.g. -e md
    #[arg(long = "extension")]
    extensions: Vec<String>,
    /// Exclude directories by basename while scanning docs
    #[arg(long = "exclude-dir")]
    exclude_dirs: Vec<String>,
    /// Relay CROP strict mode: render first, then fail on corpus issues
    #[arg(long)]
    strict: bool,
    /// Optional Markdown output path. Defaults to CROP stdout
    #[arg(long)]
    output: Option<PathBuf>,
}

#[derive(clap::Args)]
struct InspectViewsArgs {
    /// View store directory. Defaults to .crop\views
    #[arg(long, default_value = ".crop\\views")]
    dir: PathBuf,
    /// Exit non-zero when any view recipe fails inspection
    #[arg(long)]
    strict: bool,
}

#[derive(clap::Args)]
struct SideInfoArgs {
    /// Root directory or file to analyze
    #[arg(long)]
    root: Option<PathBuf>,
    /// crop.view.v1 recipe to analyze
    #[arg(long)]
    view: Option<PathBuf>,
    /// Restrict analyzed files to one or more extensions, e.g. --extension md
    #[arg(long = "extension")]
    extensions: Vec<String>,
    /// Exclude directories by basename while analyzing
    #[arg(long = "exclude-dir")]
    exclude_dirs: Vec<String>,
    /// Optional output path. Defaults to CROP stdout
    #[arg(long)]
    output: Option<PathBuf>,
}

#[derive(clap::Args)]
struct ArtifactsArgs {
    /// PROOF repository root. CROP reads .proof\artifacts.json under this root
    #[arg(long)]
    root: Option<PathBuf>,
    /// Explicit PROOF artifact manifest path
    #[arg(long)]
    manifest: Option<PathBuf>,
    /// Optional output path. Defaults to CROP stdout
    #[arg(long)]
    output: Option<PathBuf>,
}

pub(crate) fn run_with_globals(args: Args, globals: &GlobalOptions) -> Result<()> {
    match args.command {
        CropCommand::Status(status) => run_status(args.crop_bin, status),
        CropCommand::InspectViews(inspect) => run_inspect_views(args.crop_bin, inspect),
        CropCommand::Links(side_info) => run_side_info(args.crop_bin, "links", side_info, globals),
        CropCommand::Backlinks(side_info) => {
            run_side_info(args.crop_bin, "backlinks", side_info, globals)
        }
        CropCommand::Frontmatter(side_info) => {
            run_side_info(args.crop_bin, "frontmatter", side_info, globals)
        }
        CropCommand::Headings(side_info) => {
            run_side_info(args.crop_bin, "headings", side_info, globals)
        }
        CropCommand::Artifacts(artifacts) => run_artifacts(args.crop_bin, artifacts, globals),
    }
}

fn run_status(crop_bin: PathBuf, args: StatusArgs) -> Result<()> {
    run_crop(crop_bin, build_status_args(args)?)
}

fn build_status_args(args: StatusArgs) -> Result<Vec<String>> {
    if args.root.is_some() && args.view.is_some() {
        bail!("proof crop status accepts either --root or --view, not both");
    }

    let mut crop_args = vec!["status".to_string()];
    if let Some(root) = args.root {
        crop_args.push("--root".to_string());
        crop_args.push(root.display().to_string());
    }
    if let Some(view) = args.view {
        crop_args.push("--view".to_string());
        crop_args.push(view.display().to_string());
    }
    if let Some(title) = args.title {
        crop_args.push("--title".to_string());
        crop_args.push(title);
    }
    for extension in args.extensions {
        crop_args.push("--extension".to_string());
        crop_args.push(extension);
    }
    for exclude_dir in args.exclude_dirs {
        crop_args.push("--exclude-dir".to_string());
        crop_args.push(exclude_dir);
    }
    if args.strict {
        crop_args.push("--strict".to_string());
    }
    if let Some(output) = args.output {
        crop_args.push("--output".to_string());
        crop_args.push(output.display().to_string());
    }

    Ok(crop_args)
}

fn run_inspect_views(crop_bin: PathBuf, args: InspectViewsArgs) -> Result<()> {
    run_crop(crop_bin, build_inspect_views_args(args))
}

fn build_inspect_views_args(args: InspectViewsArgs) -> Vec<String> {
    let mut crop_args = vec![
        "view".to_string(),
        "--inspect".to_string(),
        "--dir".to_string(),
        args.dir.display().to_string(),
    ];
    if args.strict {
        crop_args.push("--strict".to_string());
    }

    crop_args
}

fn run_side_info(
    crop_bin: PathBuf,
    command: &str,
    args: SideInfoArgs,
    globals: &GlobalOptions,
) -> Result<()> {
    run_crop(crop_bin, build_side_info_args(command, args, globals)?)
}

fn build_side_info_args(
    command: &str,
    args: SideInfoArgs,
    globals: &GlobalOptions,
) -> Result<Vec<String>> {
    if args.root.is_some() && args.view.is_some() {
        bail!(
            "proof crop {} accepts either --root or --view, not both",
            command
        );
    }

    let mut crop_args = vec![command.to_string()];
    if let Some(root) = args.root {
        crop_args.push("--root".to_string());
        crop_args.push(root.display().to_string());
    }
    if let Some(view) = args.view {
        crop_args.push("--view".to_string());
        crop_args.push(view.display().to_string());
    }
    for extension in args.extensions {
        crop_args.push("--extension".to_string());
        crop_args.push(extension);
    }
    for exclude_dir in args.exclude_dirs {
        crop_args.push("--exclude-dir".to_string());
        crop_args.push(exclude_dir);
    }
    crop_args.push("--format".to_string());
    crop_args.push(crop_report_format(globals)?);
    if let Some(output) = args.output {
        crop_args.push("--output".to_string());
        crop_args.push(output.display().to_string());
    }

    Ok(crop_args)
}

fn run_artifacts(crop_bin: PathBuf, args: ArtifactsArgs, globals: &GlobalOptions) -> Result<()> {
    run_crop(crop_bin, build_artifacts_args(args, globals)?)
}

fn build_artifacts_args(args: ArtifactsArgs, globals: &GlobalOptions) -> Result<Vec<String>> {
    if args.root.is_some() && args.manifest.is_some() {
        bail!("proof crop artifacts accepts either --root or --manifest, not both");
    }

    let mut crop_args = vec!["artifacts".to_string()];
    if let Some(root) = args.root {
        crop_args.push("--root".to_string());
        crop_args.push(root.display().to_string());
    }
    if let Some(manifest) = args.manifest {
        crop_args.push("--manifest".to_string());
        crop_args.push(manifest.display().to_string());
    }
    crop_args.push("--format".to_string());
    crop_args.push(crop_report_format(globals)?);
    if let Some(output) = args.output {
        crop_args.push("--output".to_string());
        crop_args.push(output.display().to_string());
    }

    Ok(crop_args)
}

fn crop_report_format(globals: &GlobalOptions) -> Result<String> {
    match globals.format() {
        "text" => Ok("json".to_string()),
        "json" | "markdown" => Ok(globals.format().to_string()),
        other => bail!(
            "proof crop report format must be json or markdown, got {:?}",
            other
        ),
    }
}

pub(crate) fn run_crop(crop_bin: PathBuf, args: Vec<String>) -> Result<()> {
    let status = Command::new(&crop_bin)
        .args(&args)
        .status()
        .with_context(|| {
            format!(
                "failed to invoke CROP executable '{}'; install crop or pass --crop-bin",
                crop_bin.display()
            )
        })?;

    if let Some(code) = status.code() {
        if code != 0 {
            process::exit(code);
        }
    } else if !status.success() {
        process::exit(1);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn globals(format: &str) -> GlobalOptions {
        GlobalOptions::new(None, format.to_string(), false, false, None)
    }

    #[test]
    fn status_args_map_to_crop_status() {
        let args = build_status_args(StatusArgs {
            root: Some(PathBuf::from("docs")),
            view: None,
            title: Some("Docs".to_string()),
            extensions: vec!["md".to_string()],
            exclude_dirs: vec!["target".to_string()],
            strict: true,
            output: Some(PathBuf::from("STATUS.md")),
        })
        .unwrap();

        assert_eq!(
            args,
            vec![
                "status",
                "--root",
                "docs",
                "--title",
                "Docs",
                "--extension",
                "md",
                "--exclude-dir",
                "target",
                "--strict",
                "--output",
                "STATUS.md"
            ]
        );
    }

    #[test]
    fn status_rejects_root_and_view() {
        let err = build_status_args(StatusArgs {
            root: Some(PathBuf::from("docs")),
            view: Some(PathBuf::from("view.json")),
            title: None,
            extensions: vec![],
            exclude_dirs: vec![],
            strict: false,
            output: None,
        })
        .unwrap_err();

        assert!(err.to_string().contains("either --root or --view"));
    }

    #[test]
    fn inspect_views_args_map_to_crop_view_inspect() {
        let args = build_inspect_views_args(InspectViewsArgs {
            dir: PathBuf::from(".crop\\views"),
            strict: true,
        });

        assert_eq!(
            args,
            vec!["view", "--inspect", "--dir", ".crop\\views", "--strict"]
        );
    }

    #[test]
    fn side_info_defaults_text_global_format_to_json() {
        let args = build_side_info_args(
            "frontmatter",
            SideInfoArgs {
                root: None,
                view: Some(PathBuf::from("ready.json")),
                extensions: vec!["md".to_string()],
                exclude_dirs: vec![],
                output: Some(PathBuf::from("frontmatter.json")),
            },
            &globals("text"),
        )
        .unwrap();

        assert_eq!(
            args,
            vec![
                "frontmatter",
                "--view",
                "ready.json",
                "--extension",
                "md",
                "--format",
                "json",
                "--output",
                "frontmatter.json"
            ]
        );
    }

    #[test]
    fn side_info_relays_markdown_global_format() {
        let args = build_side_info_args(
            "links",
            SideInfoArgs {
                root: Some(PathBuf::from("docs")),
                view: None,
                extensions: vec![],
                exclude_dirs: vec!["target".to_string()],
                output: None,
            },
            &globals("markdown"),
        )
        .unwrap();

        assert_eq!(
            args,
            vec![
                "links",
                "--root",
                "docs",
                "--exclude-dir",
                "target",
                "--format",
                "markdown"
            ]
        );
    }

    #[test]
    fn side_info_rejects_unsupported_global_format() {
        let err = build_side_info_args(
            "links",
            SideInfoArgs {
                root: Some(PathBuf::from("docs")),
                view: None,
                extensions: vec![],
                exclude_dirs: vec![],
                output: None,
            },
            &globals("rich"),
        )
        .unwrap_err();

        assert!(err.to_string().contains("json or markdown"));
    }

    #[test]
    fn artifacts_args_map_to_crop_artifacts() {
        let args = build_artifacts_args(
            ArtifactsArgs {
                root: None,
                manifest: Some(PathBuf::from(".proof\\artifacts.json")),
                output: Some(PathBuf::from("ARTIFACTS.md")),
            },
            &globals("markdown"),
        )
        .unwrap();

        assert_eq!(
            args,
            vec![
                "artifacts",
                "--manifest",
                ".proof\\artifacts.json",
                "--format",
                "markdown",
                "--output",
                "ARTIFACTS.md"
            ]
        );
    }
}
