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

pub(crate) fn run(args: Args) -> Result<()> {
    match args.command {
        CropCommand::Status(status) => run_status(args.crop_bin, status),
        CropCommand::InspectViews(inspect) => run_inspect_views(args.crop_bin, inspect),
    }
}

fn run_status(crop_bin: PathBuf, args: StatusArgs) -> Result<()> {
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

    run_crop(crop_bin, crop_args)
}

fn run_inspect_views(crop_bin: PathBuf, args: InspectViewsArgs) -> Result<()> {
    let mut crop_args = vec![
        "view".to_string(),
        "--inspect".to_string(),
        "--dir".to_string(),
        args.dir.display().to_string(),
    ];
    if args.strict {
        crop_args.push("--strict".to_string());
    }

    run_crop(crop_bin, crop_args)
}

fn run_crop(crop_bin: PathBuf, args: Vec<String>) -> Result<()> {
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
