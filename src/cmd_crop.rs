use crate::cmd_context::GlobalOptions;
use anyhow::{bail, Context, Result};
use clap::Subcommand;
use proof_lib::crop_side_info;
use proof_lib::lint::load_config_for_path as load_config;
use serde::Serialize;
use std::collections::BTreeSet;
use std::io::{self, Write};
use std::path::{Component, Path, PathBuf};
use std::process::{self, Command};

const DEFAULT_VIEW_OUTPUT: &str = ".crop\\views\\proof-view.json";
const DEFAULT_VIEW_DIR: &str = ".crop\\views";

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
    /// List CROP view recipes in a view store
    ListViews(ListViewsArgs),
    /// Validate one CROP view recipe or every recipe in a view store
    InspectViews(InspectViewsArgs),
    /// Inspect CROP views and sync compiler side-info in one preflight step
    Prepare(PrepareArgs),
    /// Write a crop.view.v1 recipe from PROOF root/config/tag settings
    View(ViewArgs),
    /// Run a persisted crop.view.v1 recipe as a JSON context pack
    RunView(RunViewArgs),
    /// Generate local link side-info
    Links(SideInfoArgs),
    /// Render local link audit rows from CROP link side-info
    LinkList(LinkListArgs),
    /// Generate backlink and orphan side-info
    Backlinks(SideInfoArgs),
    /// Render inbound links for one target from CROP backlinks side-info
    BacklinkList(BacklinkListArgs),
    /// Generate frontmatter inventory side-info
    Frontmatter(SideInfoArgs),
    /// Render source metadata rows from CROP frontmatter side-info
    FrontmatterList(FrontmatterListArgs),
    /// Generate heading inventory side-info
    Headings(SideInfoArgs),
    /// Render headings for one source from CROP heading side-info
    HeadingList(HeadingListArgs),
    /// Report PROOF generated artifact manifest health through CROP
    Artifacts(ArtifactsArgs),
    /// Generate side-info JSON files under .proof\side-info for PROOF compiler use
    Sync(SyncArgs),
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
    /// Restrict scanned files to one or more extensions, e.g. --extension md
    #[arg(long = "extension")]
    extensions: Vec<String>,
    /// Exclude directories by basename while scanning docs
    #[arg(long = "exclude-dir")]
    exclude_dirs: Vec<String>,
    /// Relay CROP strict mode: render first, then fail on corpus issues
    #[arg(long)]
    strict: bool,
    /// Limit --strict to selected issue classes: broken-links, orphan-pages, duplicate-anchors
    #[arg(long = "strict-on")]
    strict_on: Vec<String>,
    /// Output format: markdown or json
    #[arg(long)]
    format: Option<String>,
    /// Optional output path. Defaults to CROP stdout
    #[arg(long)]
    output: Option<PathBuf>,
}

pub(crate) struct CropStatusRequest {
    pub(crate) root: Option<PathBuf>,
    pub(crate) view: Option<PathBuf>,
    pub(crate) title: Option<String>,
    pub(crate) extensions: Vec<String>,
    pub(crate) exclude_dirs: Vec<String>,
    pub(crate) strict: bool,
    pub(crate) strict_on: Vec<String>,
    pub(crate) format: String,
    pub(crate) output: Option<PathBuf>,
}

#[derive(clap::Args)]
struct ListViewsArgs {
    /// View store directory to list. Defaults to .crop\views
    #[arg(long, default_value = DEFAULT_VIEW_DIR)]
    dir: PathBuf,
    /// Optional JSON output path. Defaults to CROP stdout
    #[arg(long)]
    output: Option<PathBuf>,
}

#[derive(clap::Args)]
struct InspectViewsArgs {
    /// Single crop.view.v1 recipe to inspect
    #[arg(long)]
    file: Option<PathBuf>,
    /// View store directory. Defaults to .crop\views
    #[arg(long, default_value = ".crop\\views")]
    dir: PathBuf,
    /// Exit non-zero when any view recipe fails inspection
    #[arg(long)]
    strict: bool,
    /// Override the inspected view task for a one-off searchable run
    #[arg(long)]
    query: Option<String>,
    /// Restrict inspected view ingest to one or more file extensions, e.g. --extension md
    #[arg(long = "extension")]
    extensions: Vec<String>,
    /// Exclude directories by basename while inspecting one view
    #[arg(long = "exclude-dir")]
    exclude_dirs: Vec<String>,
    /// Optional JSON output path. Defaults to CROP stdout
    #[arg(long)]
    output: Option<PathBuf>,
}

#[derive(clap::Args)]
struct PrepareArgs {
    /// View store directory to inspect before syncing side-info
    #[arg(long = "dir", default_value = DEFAULT_VIEW_DIR)]
    dir: PathBuf,
    /// crop.view.v1 recipe to sync into .proof\side-info
    #[arg(long, default_value = ".crop\\views\\proof-guides.json")]
    view: PathBuf,
    /// Directory where links/backlinks/frontmatter/headings JSON files are written
    #[arg(long, default_value = ".proof\\side-info")]
    output_dir: PathBuf,
}

#[derive(clap::Args)]
struct ViewArgs {
    /// Documentation/source root for the CROP view
    #[arg(long, default_value = ".")]
    root: PathBuf,
    /// Output crop.view.v1 recipe path
    #[arg(long)]
    output: Option<PathBuf>,
    /// View name
    #[arg(long, default_value = "proof-view")]
    name: String,
    /// Context-crop task prompt for crop view --file
    #[arg(long)]
    task: Option<String>,
    /// Context-crop token budget
    #[arg(long, default_value_t = 12000)]
    token_budget: usize,
    /// Seed unit index for crop expansion
    #[arg(long, default_value_t = 0)]
    seed: usize,
    /// Restrict view files to one or more extensions, e.g. --extension md
    #[arg(long = "extension")]
    extensions: Vec<String>,
    /// Exclude directories by basename
    #[arg(long = "exclude-dir")]
    exclude_dirs: Vec<String>,
    /// Add a frontmatter tags predicate (repeatable)
    #[arg(long = "tag")]
    tags: Vec<String>,
    /// Add a frontmatter ops predicate (repeatable)
    #[arg(long = "op")]
    ops: Vec<String>,
    /// Add a frontmatter content_tags predicate (repeatable)
    #[arg(long = "content-tag")]
    content_tags: Vec<String>,
    /// Add a raw crop.view.v1 frontmatter_query clause, e.g. status eq 'ready'
    #[arg(long = "frontmatter-query")]
    frontmatter_query: Option<String>,
}

#[derive(clap::Args)]
struct RunViewArgs {
    /// crop.view.v1 recipe to run
    #[arg(long)]
    file: PathBuf,
    /// Override the view task for a one-off searchable run
    #[arg(long)]
    query: Option<String>,
    /// Restrict view ingest to one or more file extensions, e.g. --extension md
    #[arg(long = "extension")]
    extensions: Vec<String>,
    /// Exclude directories by basename while running the view
    #[arg(long = "exclude-dir")]
    exclude_dirs: Vec<String>,
    /// Emit prefix-cache-aware JSON for the view pack, e.g. generic
    #[arg(long = "prefix-cache")]
    prefix_cache: Option<String>,
    /// Optional JSON output path. Defaults to CROP stdout
    #[arg(long)]
    output: Option<PathBuf>,
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
struct BacklinkListArgs {
    /// Target source/page to render inbound links for
    #[arg(long)]
    target: String,
    /// CROP backlinks JSON report to consume
    #[arg(
        long = "side-info",
        default_value = ".proof\\side-info\\backlinks.json"
    )]
    side_info: PathBuf,
    /// Render format: list, table, or count
    #[arg(long, default_value = "list")]
    format: String,
    /// Optional output path. Defaults to stdout
    #[arg(long)]
    output: Option<PathBuf>,
}

#[derive(clap::Args)]
struct LinkListArgs {
    /// CROP links JSON report to consume
    #[arg(long = "side-info", default_value = ".proof\\side-info\\links.json")]
    side_info: PathBuf,
    /// Optional source/page to render outbound links for
    #[arg(long)]
    source: Option<String>,
    /// Link status to include: all, ok, or broken
    #[arg(long, default_value = "all")]
    status: String,
    /// Render format: list, table, or count
    #[arg(long, default_value = "list")]
    format: String,
    /// Optional output path. Defaults to stdout
    #[arg(long)]
    output: Option<PathBuf>,
}

#[derive(clap::Args)]
struct HeadingListArgs {
    /// Source/page to render headings for
    #[arg(long)]
    source: String,
    /// CROP headings JSON report to consume
    #[arg(long = "side-info", default_value = ".proof\\side-info\\headings.json")]
    side_info: PathBuf,
    /// Render format: list, table, or count
    #[arg(long, default_value = "list")]
    format: String,
    /// Optional output path. Defaults to stdout
    #[arg(long)]
    output: Option<PathBuf>,
}

#[derive(clap::Args)]
struct FrontmatterListArgs {
    /// CROP frontmatter JSON report to consume
    #[arg(
        long = "side-info",
        default_value = ".proof\\side-info\\frontmatter.json"
    )]
    side_info: PathBuf,
    /// Frontmatter field to filter or render, e.g. tags or status
    #[arg(long)]
    field: Option<String>,
    /// Field value to match
    #[arg(long)]
    value: Option<String>,
    /// Match mode when --value is set: has or eq
    #[arg(long = "op", default_value = "has")]
    op: String,
    /// Render format: list, table, or count
    #[arg(long, default_value = "list")]
    format: String,
    /// Optional output path. Defaults to stdout
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

#[derive(clap::Args)]
struct SyncArgs {
    /// Root directory or file to analyze
    #[arg(long)]
    root: Option<PathBuf>,
    /// crop.view.v1 recipe to analyze
    #[arg(long)]
    view: Option<PathBuf>,
    /// Directory where links/backlinks/frontmatter/headings JSON files are written
    #[arg(long, default_value = ".proof\\side-info")]
    output_dir: PathBuf,
    /// Restrict analyzed files to one or more extensions, e.g. --extension md
    #[arg(long = "extension")]
    extensions: Vec<String>,
    /// Exclude directories by basename while analyzing
    #[arg(long = "exclude-dir")]
    exclude_dirs: Vec<String>,
}

pub(crate) fn run_with_globals(args: Args, globals: &GlobalOptions) -> Result<()> {
    match args.command {
        CropCommand::Status(status) => run_status(args.crop_bin, status, globals),
        CropCommand::ListViews(list) => run_list_views(args.crop_bin, list, globals),
        CropCommand::InspectViews(inspect) => run_inspect_views(args.crop_bin, inspect, globals),
        CropCommand::Prepare(prepare) => run_prepare(args.crop_bin, prepare, globals),
        CropCommand::View(view) => run_view(view, globals),
        CropCommand::RunView(run_view) => run_run_view(args.crop_bin, run_view, globals),
        CropCommand::Links(side_info) => run_side_info(args.crop_bin, "links", side_info, globals),
        CropCommand::LinkList(link_list) => run_link_list(link_list, globals),
        CropCommand::Backlinks(side_info) => {
            run_side_info(args.crop_bin, "backlinks", side_info, globals)
        }
        CropCommand::BacklinkList(backlink_list) => run_backlink_list(backlink_list, globals),
        CropCommand::Frontmatter(side_info) => {
            run_side_info(args.crop_bin, "frontmatter", side_info, globals)
        }
        CropCommand::FrontmatterList(frontmatter_list) => {
            run_frontmatter_list(frontmatter_list, globals)
        }
        CropCommand::Headings(side_info) => {
            run_side_info(args.crop_bin, "headings", side_info, globals)
        }
        CropCommand::HeadingList(heading_list) => run_heading_list(heading_list, globals),
        CropCommand::Artifacts(artifacts) => run_artifacts(args.crop_bin, artifacts, globals),
        CropCommand::Sync(sync) => run_sync(args.crop_bin, sync, globals),
    }
}

fn run_status(crop_bin: PathBuf, mut args: StatusArgs, globals: &GlobalOptions) -> Result<()> {
    apply_global_output(&mut args.output, globals);
    normalize_global_text_format(&mut args.format, globals);
    apply_global_report_format(&mut args.format, globals);
    run_crop(crop_bin, build_status_args(args)?)
}

fn build_status_args(args: StatusArgs) -> Result<Vec<String>> {
    build_status_request_args(CropStatusRequest {
        root: args.root,
        view: args.view,
        title: args.title,
        extensions: args.extensions,
        exclude_dirs: args.exclude_dirs,
        strict: args.strict,
        strict_on: args.strict_on,
        format: args.format.unwrap_or_else(|| "markdown".to_string()),
        output: args.output,
    })
}

pub(crate) fn build_status_request_args(args: CropStatusRequest) -> Result<Vec<String>> {
    if args.root.is_some() && args.view.is_some() {
        bail!("proof crop status accepts either --root or --view, not both");
    }
    if args.root.is_none() && args.view.is_none() {
        bail!("proof crop status requires --root or --view");
    }
    if !args.strict && !args.strict_on.is_empty() {
        bail!("proof crop status --strict-on requires --strict");
    }
    for strict_on in &args.strict_on {
        validate_status_strict_policy(strict_on)?;
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
    for strict_on in args.strict_on {
        crop_args.push("--strict-on".to_string());
        crop_args.push(strict_on);
    }
    crop_args.push("--format".to_string());
    crop_args.push(crop_report_format_value(&args.format)?);
    if let Some(output) = args.output {
        crop_args.push("--output".to_string());
        crop_args.push(output.display().to_string());
    }

    Ok(crop_args)
}

fn validate_status_strict_policy(policy: &str) -> Result<()> {
    match policy {
        "broken-links" | "orphan-pages" | "duplicate-anchors" => Ok(()),
        other => bail!(
            "proof crop status --strict-on must be broken-links, orphan-pages, or duplicate-anchors, got {:?}",
            other
        ),
    }
}

fn run_list_views(
    crop_bin: PathBuf,
    mut args: ListViewsArgs,
    globals: &GlobalOptions,
) -> Result<()> {
    reject_non_json_artifact_global_format("list-views", globals)?;
    apply_global_output(&mut args.output, globals);
    let output = args.output.clone();
    let crop_args = build_list_views_args(args);
    if let Some(output) = output {
        run_crop_to_output(crop_bin, crop_args, output)
    } else {
        run_crop(crop_bin, crop_args)
    }
}

fn build_list_views_args(args: ListViewsArgs) -> Vec<String> {
    vec![
        "view".to_string(),
        "--list".to_string(),
        "--dir".to_string(),
        args.dir.display().to_string(),
    ]
}

fn run_inspect_views(
    crop_bin: PathBuf,
    mut args: InspectViewsArgs,
    globals: &GlobalOptions,
) -> Result<()> {
    reject_non_json_inspect_format(globals)?;
    apply_global_output(&mut args.output, globals);
    let output = args.output.clone();
    let crop_args = build_inspect_views_args(args)?;
    if let Some(output) = output {
        run_crop_to_output(crop_bin, crop_args, output)
    } else {
        run_crop(crop_bin, crop_args)
    }
}

fn build_inspect_views_args(args: InspectViewsArgs) -> Result<Vec<String>> {
    if args.file.is_some() && args.strict {
        bail!("proof crop inspect-views --strict requires store inspection with --dir");
    }
    if args.file.is_some() && args.dir != PathBuf::from(DEFAULT_VIEW_DIR) {
        bail!("proof crop inspect-views accepts either --file or --dir, not both");
    }
    if args.file.is_none()
        && (args.query.is_some() || !args.extensions.is_empty() || !args.exclude_dirs.is_empty())
    {
        bail!("proof crop inspect-views --query, --extension, and --exclude-dir require --file");
    }

    let mut crop_args = vec!["view".to_string(), "--inspect".to_string()];
    if let Some(file) = args.file {
        crop_args.push("--file".to_string());
        crop_args.push(file.display().to_string());
    } else {
        crop_args.push("--dir".to_string());
        crop_args.push(args.dir.display().to_string());
    }
    if args.strict {
        crop_args.push("--strict".to_string());
    }
    if let Some(query) = args.query {
        crop_args.push("--query".to_string());
        crop_args.push(query);
    }
    for extension in args.extensions {
        crop_args.push("--extension".to_string());
        crop_args.push(extension);
    }
    for exclude_dir in args.exclude_dirs {
        crop_args.push("--exclude-dir".to_string());
        crop_args.push(exclude_dir);
    }

    Ok(crop_args)
}

fn reject_non_json_inspect_format(globals: &GlobalOptions) -> Result<()> {
    match globals.format() {
        "text" | "json" => Ok(()),
        other => bail!(
            "proof crop inspect-views emits JSON; use text/json output format, got {:?}",
            other
        ),
    }
}

fn reject_non_json_artifact_global_format(command: &str, globals: &GlobalOptions) -> Result<()> {
    match globals.format() {
        "text" | "json" => Ok(()),
        other => bail!(
            "proof crop {} writes JSON artifacts; use text/json output format, got {:?}",
            command,
            other
        ),
    }
}

fn reject_global_output_for_output_dir(command: &str, globals: &GlobalOptions) -> Result<()> {
    if globals.output().is_some() {
        bail!(
            "proof crop {} writes multiple artifacts; use --output-dir instead of global -o/--output",
            command
        );
    }
    Ok(())
}

fn run_prepare(crop_bin: PathBuf, args: PrepareArgs, globals: &GlobalOptions) -> Result<()> {
    reject_non_json_artifact_global_format("prepare", globals)?;
    reject_global_output_for_output_dir("prepare", globals)?;
    let command_args = build_prepare_args(args)?;
    for crop_args in command_args {
        run_crop(crop_bin.clone(), crop_args)?;
    }
    Ok(())
}

fn build_prepare_args(args: PrepareArgs) -> Result<Vec<Vec<String>>> {
    let view = args.view;
    let mut commands = vec![build_inspect_views_args(InspectViewsArgs {
        file: None,
        dir: args.dir,
        strict: true,
        query: None,
        extensions: Vec::new(),
        exclude_dirs: Vec::new(),
        output: None,
    })?];
    commands.push(build_inspect_views_args(InspectViewsArgs {
        file: Some(view.clone()),
        dir: PathBuf::from(".crop\\views"),
        strict: false,
        query: None,
        extensions: Vec::new(),
        exclude_dirs: Vec::new(),
        output: None,
    })?);
    commands.extend(build_sync_args(SyncArgs {
        root: None,
        view: Some(view),
        output_dir: args.output_dir,
        extensions: Vec::new(),
        exclude_dirs: Vec::new(),
    })?);
    Ok(commands)
}

fn run_view(args: ViewArgs, globals: &GlobalOptions) -> Result<()> {
    reject_non_json_artifact_global_format("view", globals)?;
    let output = view_output_path(&args, globals);
    let recipe = build_view_recipe(&args, globals)?;
    if let Some(parent) = output.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating {}", parent.display()))?;
    }
    let json = serde_json::to_string_pretty(&recipe)?;
    std::fs::write(&output, json).with_context(|| format!("writing {}", output.display()))?;
    println!("wrote {}", output.display());
    Ok(())
}

#[derive(Debug, Serialize, PartialEq, Eq)]
struct CropViewRecipe {
    schema_version: &'static str,
    name: String,
    root: String,
    task: String,
    token_budget: usize,
    seed: usize,
    include_extensions: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    exclude_dirs: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    frontmatter_query: Option<String>,
}

fn build_view_recipe(args: &ViewArgs, globals: &GlobalOptions) -> Result<CropViewRecipe> {
    let output = view_output_path(args, globals);
    build_view_recipe_for_output(args, globals, &output)
}

fn build_view_recipe_for_output(
    args: &ViewArgs,
    globals: &GlobalOptions,
    output: &Path,
) -> Result<CropViewRecipe> {
    let config = load_config(&args.root, globals.config())?;
    let include_extensions = if args.extensions.is_empty() {
        extensions_from_include_patterns(&config.files.include)
    } else {
        normalize_extensions(&args.extensions)
    };
    let exclude_dirs = if args.exclude_dirs.is_empty() {
        exclude_dirs_from_patterns(&config.files.exclude)
    } else {
        dedupe_strings(&args.exclude_dirs)
    };

    Ok(CropViewRecipe {
        schema_version: "crop.view.v1",
        name: args.name.clone(),
        root: view_root_for_output(&args.root, output)?,
        task: args
            .task
            .clone()
            .unwrap_or_else(|| format!("{} corpus", args.name)),
        token_budget: args.token_budget,
        seed: args.seed,
        include_extensions,
        exclude_dirs,
        frontmatter_query: build_frontmatter_query(args)?,
    })
}

fn view_output_path(args: &ViewArgs, globals: &GlobalOptions) -> PathBuf {
    args.output
        .clone()
        .or_else(|| globals.output().clone())
        .unwrap_or_else(|| PathBuf::from(DEFAULT_VIEW_OUTPUT))
}

fn run_run_view(crop_bin: PathBuf, mut args: RunViewArgs, globals: &GlobalOptions) -> Result<()> {
    reject_non_json_artifact_global_format("run-view", globals)?;
    apply_global_output(&mut args.output, globals);
    let output = args.output.clone();
    let crop_args = build_run_view_args(args)?;
    if let Some(output) = output {
        run_crop_to_output(crop_bin, crop_args, output)
    } else {
        run_crop(crop_bin, crop_args)
    }
}

fn build_run_view_args(args: RunViewArgs) -> Result<Vec<String>> {
    let mut crop_args = vec![
        "view".to_string(),
        "--file".to_string(),
        args.file.display().to_string(),
    ];
    if let Some(query) = args.query {
        crop_args.push("--query".to_string());
        crop_args.push(query);
    }
    for extension in args.extensions {
        crop_args.push("--extension".to_string());
        crop_args.push(extension);
    }
    for exclude_dir in args.exclude_dirs {
        crop_args.push("--exclude-dir".to_string());
        crop_args.push(exclude_dir);
    }
    if let Some(prefix_cache) = args.prefix_cache {
        validate_prefix_cache(&prefix_cache)?;
        crop_args.push("--prefix-cache".to_string());
        crop_args.push(prefix_cache);
    }
    Ok(crop_args)
}

fn validate_prefix_cache(prefix_cache: &str) -> Result<()> {
    match prefix_cache {
        "generic" => Ok(()),
        other => bail!(
            "proof crop run-view --prefix-cache must be generic, got {:?}",
            other
        ),
    }
}

fn view_root_for_output(root: &Path, output: &Path) -> Result<String> {
    let cwd = std::env::current_dir()?;
    let target = absolute_lexical(root, &cwd);
    let base = output
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map(|parent| absolute_lexical(parent, &cwd))
        .unwrap_or(cwd);
    Ok(relative_path(&target, &base)
        .unwrap_or(target)
        .display()
        .to_string())
}

fn absolute_lexical(path: &Path, cwd: &Path) -> PathBuf {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    };
    normalize_lexical(&absolute)
}

fn normalize_lexical(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

fn relative_path(target: &Path, base: &Path) -> Option<PathBuf> {
    let target_components: Vec<_> = target.components().collect();
    let base_components: Vec<_> = base.components().collect();
    let mut common = 0usize;
    while common < target_components.len()
        && common < base_components.len()
        && component_eq(target_components[common], base_components[common])
    {
        common += 1;
    }
    if common == 0 {
        return None;
    }

    let mut out = PathBuf::new();
    for component in &base_components[common..] {
        if matches!(component, Component::Normal(_)) {
            out.push("..");
        }
    }
    for component in &target_components[common..] {
        out.push(component.as_os_str());
    }
    if out.as_os_str().is_empty() {
        out.push(".");
    }
    Some(out)
}

fn component_eq(left: Component<'_>, right: Component<'_>) -> bool {
    left.as_os_str()
        .to_string_lossy()
        .eq_ignore_ascii_case(&right.as_os_str().to_string_lossy())
}

fn build_frontmatter_query(args: &ViewArgs) -> Result<Option<String>> {
    let mut clauses = Vec::new();
    if let Some(query) = args.frontmatter_query.as_deref().map(str::trim) {
        if !query.is_empty() {
            clauses.push(query.to_string());
        }
    }
    for tag in &args.tags {
        clauses.push(frontmatter_clause("tags", tag)?);
    }
    for op in &args.ops {
        clauses.push(frontmatter_clause("ops", op)?);
    }
    for content_tag in &args.content_tags {
        clauses.push(frontmatter_clause("content_tags", content_tag)?);
    }
    Ok((!clauses.is_empty()).then(|| clauses.join(" and ")))
}

fn frontmatter_clause(field: &str, value: &str) -> Result<String> {
    if value.contains('\'') {
        bail!(
            "frontmatter query value {:?} contains a single quote, which crop.view.v1 cannot encode safely",
            value
        );
    }
    Ok(format!("{} has '{}'", field, value))
}

fn extensions_from_include_patterns(patterns: &[String]) -> Vec<String> {
    let mut extensions = BTreeSet::new();
    for pattern in patterns {
        for part in pattern.split(['{', '}', ',']) {
            if let Some(extension) = extension_from_pattern(part) {
                extensions.insert(extension);
            }
        }
    }
    if extensions.is_empty() {
        extensions.insert("md".to_string());
    }
    extensions.into_iter().collect()
}

fn extension_from_pattern(pattern: &str) -> Option<String> {
    let trimmed = pattern.trim().trim_matches('"').trim_matches('\'');
    let (_, extension) = trimmed.rsplit_once('.')?;
    let extension = extension
        .trim_matches('*')
        .trim_matches('?')
        .trim_matches(']')
        .trim_matches(')')
        .trim();
    if extension.is_empty()
        || extension
            .chars()
            .any(|ch| !(ch.is_ascii_alphanumeric() || ch == '_' || ch == '-'))
    {
        None
    } else {
        Some(extension.trim_start_matches('.').to_string())
    }
}

fn normalize_extensions(extensions: &[String]) -> Vec<String> {
    let mut out = BTreeSet::new();
    for extension in extensions {
        let normalized = extension.trim().trim_start_matches('.');
        if !normalized.is_empty() {
            out.insert(normalized.to_string());
        }
    }
    out.into_iter().collect()
}

fn exclude_dirs_from_patterns(patterns: &[String]) -> Vec<String> {
    let mut dirs = BTreeSet::new();
    for pattern in patterns {
        if let Some(dir) = exclude_dir_from_pattern(pattern) {
            dirs.insert(dir);
        }
    }
    dirs.into_iter().collect()
}

fn exclude_dir_from_pattern(pattern: &str) -> Option<String> {
    let normalized = pattern.replace('\\', "/");
    let mut parts = normalized
        .split('/')
        .map(str::trim)
        .filter(|part| !part.is_empty() && *part != "**" && *part != "*");
    parts
        .find(|part| {
            !part.contains('*')
                && !part.contains('?')
                && !part.contains('.')
                && *part != "!"
                && *part != "."
        })
        .map(str::to_string)
}

fn dedupe_strings(values: &[String]) -> Vec<String> {
    values
        .iter()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn run_side_info(
    crop_bin: PathBuf,
    command: &str,
    args: SideInfoArgs,
    globals: &GlobalOptions,
) -> Result<()> {
    run_crop(crop_bin, build_side_info_args(command, args, globals)?)
}

fn run_backlink_list(mut args: BacklinkListArgs, globals: &GlobalOptions) -> Result<()> {
    reject_non_markdown_snippet_global_format("backlink-list", globals)?;
    apply_global_output(&mut args.output, globals);
    let rendered = crop_side_info::render_backlinks(&args.target, &args.side_info, &args.format)?;
    write_snippet(rendered, args.output)
}

fn run_link_list(mut args: LinkListArgs, globals: &GlobalOptions) -> Result<()> {
    reject_non_markdown_snippet_global_format("link-list", globals)?;
    apply_global_output(&mut args.output, globals);
    let status = link_status_filter(&args.status)?;
    let filter = crop_side_info::LinkFilter {
        source: args.source,
        status,
    };
    let rendered = crop_side_info::render_links(&args.side_info, &filter, &args.format)?;
    write_snippet(rendered, args.output)
}

fn link_status_filter(status: &str) -> Result<Option<String>> {
    match status {
        "all" => Ok(Some("all".to_string())),
        "ok" | "broken" => Ok(Some(status.to_string())),
        _ => bail!("link status must be 'all', 'ok', or 'broken'"),
    }
}

fn run_heading_list(mut args: HeadingListArgs, globals: &GlobalOptions) -> Result<()> {
    reject_non_markdown_snippet_global_format("heading-list", globals)?;
    apply_global_output(&mut args.output, globals);
    let rendered = crop_side_info::render_headings(&args.source, &args.side_info, &args.format)?;
    write_snippet(rendered, args.output)
}

fn run_frontmatter_list(mut args: FrontmatterListArgs, globals: &GlobalOptions) -> Result<()> {
    reject_non_markdown_snippet_global_format("frontmatter-list", globals)?;
    apply_global_output(&mut args.output, globals);
    let filter = crop_side_info::FrontmatterFilter {
        field: args.field,
        value: args.value,
        op: parse_frontmatter_match(&args.op)?,
    };
    let rendered = crop_side_info::render_frontmatter(&args.side_info, &filter, &args.format)?;
    write_snippet(rendered, args.output)
}

fn parse_frontmatter_match(op: &str) -> Result<crop_side_info::FrontmatterMatch> {
    match op {
        "has" => Ok(crop_side_info::FrontmatterMatch::Has),
        "eq" => Ok(crop_side_info::FrontmatterMatch::Eq),
        _ => bail!("frontmatter match op must be 'has' or 'eq'"),
    }
}

fn write_snippet(rendered: String, output: Option<PathBuf>) -> Result<()> {
    if let Some(output) = output {
        if let Some(parent) = output.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("creating {}", parent.display()))?;
        }
        std::fs::write(&output, rendered)
            .with_context(|| format!("writing {}", output.display()))?;
    } else {
        println!("{}", rendered);
    }
    Ok(())
}

fn reject_non_markdown_snippet_global_format(command: &str, globals: &GlobalOptions) -> Result<()> {
    match globals.format() {
        "text" | "markdown" | "list" | "table" | "count" => Ok(()),
        other => bail!(
            "proof crop {} renders Markdown snippets; use text/markdown global output format or list/table/count snippet format, got {:?}",
            command,
            other
        ),
    }
}

fn apply_global_output(output: &mut Option<PathBuf>, globals: &GlobalOptions) {
    if output.is_none() {
        *output = globals.output().clone();
    }
}

fn apply_global_report_format(format: &mut Option<String>, globals: &GlobalOptions) {
    if format.is_none() && globals.format() != "text" {
        *format = Some(globals.format().to_string());
    }
}

fn normalize_global_text_format(format: &mut Option<String>, globals: &GlobalOptions) {
    if format.as_deref() == Some("text") && globals.format() == "text" {
        *format = None;
    }
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
    if args.root.is_none() && args.view.is_none() {
        bail!("proof crop {} requires --root or --view", command);
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
    let output = args.output.or_else(|| globals.output().clone());
    if let Some(output) = output {
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
    if args.root.is_none() && args.manifest.is_none() {
        bail!("proof crop artifacts requires --root or --manifest");
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
    let output = args.output.or_else(|| globals.output().clone());
    if let Some(output) = output {
        crop_args.push("--output".to_string());
        crop_args.push(output.display().to_string());
    }

    Ok(crop_args)
}

fn run_sync(crop_bin: PathBuf, args: SyncArgs, globals: &GlobalOptions) -> Result<()> {
    reject_non_json_artifact_global_format("sync", globals)?;
    reject_global_output_for_output_dir("sync", globals)?;
    let command_args = build_sync_args(args)?;
    for crop_args in command_args {
        run_crop(crop_bin.clone(), crop_args)?;
    }
    Ok(())
}

fn build_sync_args(args: SyncArgs) -> Result<Vec<Vec<String>>> {
    if args.root.is_some() && args.view.is_some() {
        bail!("proof crop sync accepts either --root or --view, not both");
    }
    if args.root.is_none() && args.view.is_none() {
        bail!("proof crop sync requires --root or --view");
    }

    std::fs::create_dir_all(&args.output_dir)
        .with_context(|| format!("creating side-info directory {}", args.output_dir.display()))?;

    let mut all = Vec::new();
    for command in ["links", "backlinks", "frontmatter", "headings"] {
        let mut crop_args = vec![command.to_string()];
        if let Some(root) = &args.root {
            crop_args.push("--root".to_string());
            crop_args.push(root.display().to_string());
        }
        if let Some(view) = &args.view {
            crop_args.push("--view".to_string());
            crop_args.push(view.display().to_string());
        }
        for extension in &args.extensions {
            crop_args.push("--extension".to_string());
            crop_args.push(extension.clone());
        }
        for exclude_dir in &args.exclude_dirs {
            crop_args.push("--exclude-dir".to_string());
            crop_args.push(exclude_dir.clone());
        }
        crop_args.push("--format".to_string());
        crop_args.push("json".to_string());
        crop_args.push("--output".to_string());
        crop_args.push(
            args.output_dir
                .join(format!("{}.json", command))
                .display()
                .to_string(),
        );
        all.push(crop_args);
    }
    Ok(all)
}

fn crop_report_format(globals: &GlobalOptions) -> Result<String> {
    crop_report_format_value(match globals.format() {
        "text" => "json",
        other => other,
    })
}

fn crop_report_format_value(format: &str) -> Result<String> {
    match format {
        "json" | "markdown" => Ok(format.to_string()),
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

fn run_crop_to_output(crop_bin: PathBuf, args: Vec<String>, output: PathBuf) -> Result<()> {
    let child_output = Command::new(&crop_bin)
        .args(&args)
        .output()
        .with_context(|| {
            format!(
                "failed to invoke CROP executable '{}'; install crop or pass --crop-bin",
                crop_bin.display()
            )
        })?;

    if let Some(parent) = output.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating {}", parent.display()))?;
    }
    std::fs::write(&output, &child_output.stdout)
        .with_context(|| format!("writing {}", output.display()))?;
    io::stderr().write_all(&child_output.stderr)?;

    if let Some(code) = child_output.status.code() {
        if code != 0 {
            process::exit(code);
        }
    } else if !child_output.status.success() {
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

    fn globals_with_output(format: &str, output: PathBuf) -> GlobalOptions {
        GlobalOptions::new(None, format.to_string(), false, false, Some(output))
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
            strict_on: vec!["broken-links".to_string(), "duplicate-anchors".to_string()],
            format: Some("markdown".to_string()),
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
                "--strict-on",
                "broken-links",
                "--strict-on",
                "duplicate-anchors",
                "--format",
                "markdown",
                "--output",
                "STATUS.md"
            ]
        );
    }

    #[test]
    fn status_args_can_request_json_contract() {
        let args = build_status_args(StatusArgs {
            root: None,
            view: Some(PathBuf::from(".crop\\views\\ready.json")),
            title: None,
            extensions: vec![],
            exclude_dirs: vec![],
            strict: false,
            strict_on: vec![],
            format: Some("json".to_string()),
            output: Some(PathBuf::from("READY.status.json")),
        })
        .unwrap();

        assert_eq!(
            args,
            vec![
                "status",
                "--view",
                ".crop\\views\\ready.json",
                "--format",
                "json",
                "--output",
                "READY.status.json",
            ]
        );
    }

    #[test]
    fn status_uses_global_output_when_local_output_missing() {
        let mut status = StatusArgs {
            root: Some(PathBuf::from("docs")),
            view: None,
            title: None,
            extensions: vec![],
            exclude_dirs: vec![],
            strict: false,
            strict_on: vec![],
            format: Some("markdown".to_string()),
            output: None,
        };
        apply_global_output(
            &mut status.output,
            &globals_with_output("text", PathBuf::from("STATUS.md")),
        );

        let args = build_status_args(status).unwrap();

        assert_eq!(
            args,
            vec![
                "status",
                "--root",
                "docs",
                "--format",
                "markdown",
                "--output",
                "STATUS.md"
            ]
        );
    }

    #[test]
    fn status_uses_global_format_when_local_format_missing() {
        let mut status = StatusArgs {
            root: Some(PathBuf::from("docs")),
            view: None,
            title: None,
            extensions: vec![],
            exclude_dirs: vec![],
            strict: false,
            strict_on: vec![],
            format: None,
            output: None,
        };
        apply_global_report_format(&mut status.format, &globals("json"));

        let args = build_status_args(status).unwrap();

        assert_eq!(args, vec!["status", "--root", "docs", "--format", "json"]);
    }

    #[test]
    fn status_defaults_to_markdown_without_local_or_global_format() {
        let args = build_status_args(StatusArgs {
            root: Some(PathBuf::from("docs")),
            view: None,
            title: None,
            extensions: vec![],
            exclude_dirs: vec![],
            strict: false,
            strict_on: vec![],
            format: None,
            output: None,
        })
        .unwrap();

        assert_eq!(
            args,
            vec!["status", "--root", "docs", "--format", "markdown"]
        );
    }

    #[test]
    fn status_normalizes_global_text_default_to_markdown() {
        let mut format = Some("text".to_string());
        normalize_global_text_format(&mut format, &globals("text"));
        apply_global_report_format(&mut format, &globals("text"));

        let args = build_status_args(StatusArgs {
            root: Some(PathBuf::from("docs")),
            view: None,
            title: None,
            extensions: vec![],
            exclude_dirs: vec![],
            strict: false,
            strict_on: vec![],
            format,
            output: None,
        })
        .unwrap();

        assert_eq!(
            args,
            vec!["status", "--root", "docs", "--format", "markdown"]
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
            strict_on: vec![],
            format: Some("markdown".to_string()),
            output: None,
        })
        .unwrap_err();

        assert!(err.to_string().contains("either --root or --view"));
    }

    #[test]
    fn status_requires_root_or_view() {
        let err = build_status_args(StatusArgs {
            root: None,
            view: None,
            title: None,
            extensions: vec![],
            exclude_dirs: vec![],
            strict: false,
            strict_on: vec![],
            format: Some("markdown".to_string()),
            output: None,
        })
        .unwrap_err();

        assert!(err.to_string().contains("requires --root or --view"));
    }

    #[test]
    fn status_rejects_unsupported_report_format() {
        let err = build_status_args(StatusArgs {
            root: Some(PathBuf::from("docs")),
            view: None,
            title: None,
            extensions: vec![],
            exclude_dirs: vec![],
            strict: false,
            strict_on: vec![],
            format: Some("yaml".to_string()),
            output: None,
        })
        .unwrap_err();

        assert!(err.to_string().contains("json or markdown"));
    }

    #[test]
    fn status_rejects_strict_policy_without_strict() {
        let err = build_status_args(StatusArgs {
            root: Some(PathBuf::from("docs")),
            view: None,
            title: None,
            extensions: vec![],
            exclude_dirs: vec![],
            strict: false,
            strict_on: vec!["broken-links".to_string()],
            format: Some("markdown".to_string()),
            output: None,
        })
        .unwrap_err();

        assert!(err.to_string().contains("requires --strict"));
    }

    #[test]
    fn status_rejects_unknown_strict_policy() {
        let err = build_status_args(StatusArgs {
            root: Some(PathBuf::from("docs")),
            view: None,
            title: None,
            extensions: vec![],
            exclude_dirs: vec![],
            strict: true,
            strict_on: vec!["stale-artifacts".to_string()],
            format: Some("markdown".to_string()),
            output: None,
        })
        .unwrap_err();

        assert!(err.to_string().contains("broken-links"));
    }

    #[test]
    fn list_views_args_map_to_crop_view_list() {
        let args = build_list_views_args(ListViewsArgs {
            dir: PathBuf::from(".crop\\views"),
            output: None,
        });

        assert_eq!(args, vec!["view", "--list", "--dir", ".crop\\views"]);
    }

    #[test]
    fn inspect_views_args_map_to_crop_view_inspect() {
        let args = build_inspect_views_args(InspectViewsArgs {
            file: None,
            dir: PathBuf::from(".crop\\views"),
            strict: true,
            query: None,
            extensions: Vec::new(),
            exclude_dirs: Vec::new(),
            output: None,
        })
        .unwrap();

        assert_eq!(
            args,
            vec!["view", "--inspect", "--dir", ".crop\\views", "--strict"]
        );
    }

    #[test]
    fn inspect_views_args_can_inspect_single_view_file() {
        let args = build_inspect_views_args(InspectViewsArgs {
            file: Some(PathBuf::from(".crop\\views\\ready.json")),
            dir: PathBuf::from(".crop\\views"),
            strict: false,
            query: Some("refresh docs".to_string()),
            extensions: vec!["md".to_string()],
            exclude_dirs: vec!["target".to_string()],
            output: None,
        })
        .unwrap();

        assert_eq!(
            args,
            vec![
                "view",
                "--inspect",
                "--file",
                ".crop\\views\\ready.json",
                "--query",
                "refresh docs",
                "--extension",
                "md",
                "--exclude-dir",
                "target"
            ]
        );
    }

    #[test]
    fn inspect_views_rejects_store_filter_overrides() {
        let err = build_inspect_views_args(InspectViewsArgs {
            file: None,
            dir: PathBuf::from(".crop\\views"),
            strict: false,
            query: Some("refresh docs".to_string()),
            extensions: Vec::new(),
            exclude_dirs: Vec::new(),
            output: None,
        })
        .unwrap_err();

        assert!(err.to_string().contains("require --file"));
    }

    #[test]
    fn inspect_views_rejects_strict_single_file() {
        let err = build_inspect_views_args(InspectViewsArgs {
            file: Some(PathBuf::from(".crop\\views\\ready.json")),
            dir: PathBuf::from(".crop\\views"),
            strict: true,
            query: None,
            extensions: Vec::new(),
            exclude_dirs: Vec::new(),
            output: None,
        })
        .unwrap_err();

        assert!(err.to_string().contains("requires store inspection"));
    }

    #[test]
    fn inspect_views_rejects_file_and_custom_dir() {
        let err = build_inspect_views_args(InspectViewsArgs {
            file: Some(PathBuf::from(".crop\\views\\ready.json")),
            dir: PathBuf::from("other\\views"),
            strict: false,
            query: None,
            extensions: Vec::new(),
            exclude_dirs: Vec::new(),
            output: None,
        })
        .unwrap_err();

        assert!(err.to_string().contains("either --file or --dir"));
    }

    #[test]
    fn inspect_views_rejects_non_json_global_format() {
        let err = reject_non_json_inspect_format(&globals("markdown")).unwrap_err();

        assert!(err.to_string().contains("emits JSON"));
    }

    #[test]
    fn json_artifact_commands_reject_non_json_global_format() {
        let err = reject_non_json_artifact_global_format("view", &globals("markdown")).unwrap_err();

        assert!(err.to_string().contains("writes JSON artifacts"));
    }

    #[test]
    fn output_dir_commands_reject_global_output() {
        let err = reject_global_output_for_output_dir(
            "sync",
            &GlobalOptions::new(
                None,
                "text".to_string(),
                false,
                false,
                Some(PathBuf::from("SIDE_INFO.json")),
            ),
        )
        .unwrap_err();

        assert!(err.to_string().contains("--output-dir"));
    }

    #[test]
    fn prepare_args_inspect_views_then_sync_side_info() {
        let dir = tempfile::tempdir().unwrap();
        let output_dir = dir.path().join("side-info");
        let args = build_prepare_args(PrepareArgs {
            dir: PathBuf::from(".crop\\views"),
            view: PathBuf::from(".crop\\views\\proof-guides.json"),
            output_dir: output_dir.clone(),
        })
        .unwrap();

        assert_eq!(args.len(), 6);
        assert_eq!(
            args[0],
            vec!["view", "--inspect", "--dir", ".crop\\views", "--strict"]
        );
        assert_eq!(
            args[1],
            vec![
                "view",
                "--inspect",
                "--file",
                ".crop\\views\\proof-guides.json"
            ]
        );
        assert_eq!(args[2][0], "links");
        assert_eq!(args[3][0], "backlinks");
        assert_eq!(args[4][0], "frontmatter");
        assert_eq!(args[5][0], "headings");
        for crop_args in &args[2..] {
            assert!(crop_args.contains(&"--view".to_string()));
            assert!(crop_args.contains(&".crop\\views\\proof-guides.json".to_string()));
            assert!(crop_args.contains(&"--format".to_string()));
            assert!(crop_args.contains(&"json".to_string()));
        }
        assert!(args[3].contains(&output_dir.join("backlinks.json").display().to_string()));
    }

    #[test]
    fn view_recipe_maps_config_and_frontmatter_filters() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("proof.toml"),
            r#"
[files]
include = ["src/**/*.source.md", "docs/**/*.md"]
exclude = ["target/**", "node_modules/**"]
"#,
        )
        .unwrap();

        let recipe = build_view_recipe(
            &ViewArgs {
                root: dir.path().to_path_buf(),
                output: Some(PathBuf::from("ready.json")),
                name: "ready-guides".to_string(),
                task: None,
                token_budget: 8000,
                seed: 2,
                extensions: vec![],
                exclude_dirs: vec![],
                tags: vec!["guide".to_string()],
                ops: vec!["compile".to_string()],
                content_tags: vec!["markdown".to_string()],
                frontmatter_query: None,
            },
            &globals("text"),
        )
        .unwrap();

        assert_eq!(recipe.schema_version, "crop.view.v1");
        assert_eq!(recipe.name, "ready-guides");
        assert_eq!(recipe.task, "ready-guides corpus");
        assert_eq!(recipe.token_budget, 8000);
        assert_eq!(recipe.seed, 2);
        assert_eq!(recipe.include_extensions, vec!["md"]);
        assert_eq!(recipe.exclude_dirs, vec!["node_modules", "target"]);
        assert_eq!(
            recipe.frontmatter_query.as_deref(),
            Some("tags has 'guide' and ops has 'compile' and content_tags has 'markdown'")
        );
    }

    #[test]
    fn view_recipe_prefers_explicit_extensions_and_exclude_dirs() {
        let dir = tempfile::tempdir().unwrap();
        let recipe = build_view_recipe(
            &ViewArgs {
                root: dir.path().to_path_buf(),
                output: Some(PathBuf::from("view.json")),
                name: "all-docs".to_string(),
                task: Some("all docs".to_string()),
                token_budget: 12000,
                seed: 0,
                extensions: vec![".md".to_string(), "mdx".to_string(), "md".to_string()],
                exclude_dirs: vec!["target".to_string(), "target".to_string()],
                tags: vec![],
                ops: vec![],
                content_tags: vec![],
                frontmatter_query: None,
            },
            &globals("text"),
        )
        .unwrap();

        assert_eq!(recipe.include_extensions, vec!["md", "mdx"]);
        assert_eq!(recipe.exclude_dirs, vec!["target"]);
        assert_eq!(recipe.frontmatter_query, None);
    }

    #[test]
    fn view_recipe_uses_global_output_for_relative_root() {
        let dir = tempfile::tempdir().unwrap();
        let output = dir.path().join(".crop").join("views").join("ready.json");
        let recipe = build_view_recipe(
            &ViewArgs {
                root: dir.path().to_path_buf(),
                output: None,
                name: "ready".to_string(),
                task: None,
                token_budget: 12000,
                seed: 0,
                extensions: vec![],
                exclude_dirs: vec![],
                tags: vec![],
                ops: vec![],
                content_tags: vec![],
                frontmatter_query: None,
            },
            &globals_with_output("text", output),
        )
        .unwrap();

        assert_eq!(
            recipe.root,
            PathBuf::from("..").join("..").display().to_string()
        );
    }

    #[test]
    fn view_recipe_keeps_default_output_when_no_global_output() {
        let recipe = build_view_recipe(
            &ViewArgs {
                root: PathBuf::from("."),
                output: None,
                name: "default-view".to_string(),
                task: None,
                token_budget: 12000,
                seed: 0,
                extensions: vec![],
                exclude_dirs: vec![],
                tags: vec![],
                ops: vec![],
                content_tags: vec![],
                frontmatter_query: None,
            },
            &globals("text"),
        )
        .unwrap();

        assert_eq!(
            recipe.root,
            PathBuf::from("..").join("..").display().to_string()
        );
    }

    #[test]
    fn view_recipe_combines_raw_frontmatter_query_with_shorthands() {
        let dir = tempfile::tempdir().unwrap();
        let recipe = build_view_recipe(
            &ViewArgs {
                root: dir.path().to_path_buf(),
                output: Some(PathBuf::from("ready.json")),
                name: "ready".to_string(),
                task: None,
                token_budget: 12000,
                seed: 0,
                extensions: vec![],
                exclude_dirs: vec![],
                tags: vec!["guide".to_string()],
                ops: vec![],
                content_tags: vec![],
                frontmatter_query: Some("status eq 'ready'".to_string()),
            },
            &globals("text"),
        )
        .unwrap();

        assert_eq!(
            recipe.frontmatter_query.as_deref(),
            Some("status eq 'ready' and tags has 'guide'")
        );
    }

    #[test]
    fn view_recipe_rejects_unencodable_frontmatter_value() {
        let dir = tempfile::tempdir().unwrap();
        let err = build_view_recipe(
            &ViewArgs {
                root: dir.path().to_path_buf(),
                output: Some(PathBuf::from("view.json")),
                name: "bad".to_string(),
                task: None,
                token_budget: 12000,
                seed: 0,
                extensions: vec![],
                exclude_dirs: vec![],
                tags: vec!["author's".to_string()],
                ops: vec![],
                content_tags: vec![],
                frontmatter_query: None,
            },
            &globals("text"),
        )
        .unwrap_err();

        assert!(err.to_string().contains("single quote"));
    }

    #[test]
    fn run_view_args_map_to_crop_view_file() {
        let args = build_run_view_args(RunViewArgs {
            file: PathBuf::from(".crop\\views\\ready.json"),
            query: Some("refresh docs".to_string()),
            extensions: vec!["md".to_string()],
            exclude_dirs: vec!["target".to_string()],
            prefix_cache: Some("generic".to_string()),
            output: Some(PathBuf::from("pack.json")),
        })
        .unwrap();

        assert_eq!(
            args,
            vec![
                "view",
                "--file",
                ".crop\\views\\ready.json",
                "--query",
                "refresh docs",
                "--extension",
                "md",
                "--exclude-dir",
                "target",
                "--prefix-cache",
                "generic"
            ]
        );
    }

    #[test]
    fn run_view_rejects_unknown_prefix_cache() {
        let err = build_run_view_args(RunViewArgs {
            file: PathBuf::from(".crop\\views\\ready.json"),
            query: None,
            extensions: Vec::new(),
            exclude_dirs: Vec::new(),
            prefix_cache: Some("specialized".to_string()),
            output: None,
        })
        .unwrap_err();

        assert!(err.to_string().contains("must be generic"));
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
    fn side_info_uses_global_output_when_local_output_missing() {
        let args = build_side_info_args(
            "links",
            SideInfoArgs {
                root: Some(PathBuf::from("docs")),
                view: None,
                extensions: vec![],
                exclude_dirs: vec![],
                output: None,
            },
            &globals_with_output("text", PathBuf::from("links.json")),
        )
        .unwrap();

        assert_eq!(
            args,
            vec![
                "links",
                "--root",
                "docs",
                "--format",
                "json",
                "--output",
                "links.json"
            ]
        );
    }

    #[test]
    fn snippet_commands_reject_non_markdown_global_format() {
        let err =
            reject_non_markdown_snippet_global_format("link-list", &globals("json")).unwrap_err();

        assert!(err.to_string().contains("Markdown snippets"));
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
    fn side_info_requires_root_or_view() {
        let err = build_side_info_args(
            "links",
            SideInfoArgs {
                root: None,
                view: None,
                extensions: vec![],
                exclude_dirs: vec![],
                output: None,
            },
            &globals("text"),
        )
        .unwrap_err();

        assert!(err.to_string().contains("requires --root or --view"));
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

    #[test]
    fn artifacts_uses_global_output_when_local_output_missing() {
        let args = build_artifacts_args(
            ArtifactsArgs {
                root: Some(PathBuf::from(".")),
                manifest: None,
                output: None,
            },
            &globals_with_output("markdown", PathBuf::from("ARTIFACTS.md")),
        )
        .unwrap();

        assert_eq!(
            args,
            vec![
                "artifacts",
                "--root",
                ".",
                "--format",
                "markdown",
                "--output",
                "ARTIFACTS.md"
            ]
        );
    }

    #[test]
    fn artifacts_requires_root_or_manifest() {
        let err = build_artifacts_args(
            ArtifactsArgs {
                root: None,
                manifest: None,
                output: None,
            },
            &globals("text"),
        )
        .unwrap_err();

        assert!(err.to_string().contains("requires --root or --manifest"));
    }

    #[test]
    fn sync_args_generate_all_side_info_commands() {
        let dir = tempfile::tempdir().unwrap();
        let output_dir = dir.path().join("side-info");
        let args = build_sync_args(SyncArgs {
            root: None,
            view: Some(PathBuf::from("ready.json")),
            output_dir: output_dir.clone(),
            extensions: vec!["md".to_string()],
            exclude_dirs: vec!["target".to_string()],
        })
        .unwrap();

        assert_eq!(args.len(), 4);
        assert_eq!(args[0][0], "links");
        assert_eq!(args[1][0], "backlinks");
        assert_eq!(args[2][0], "frontmatter");
        assert_eq!(args[3][0], "headings");
        for crop_args in &args {
            assert!(crop_args.contains(&"--view".to_string()));
            assert!(crop_args.contains(&"ready.json".to_string()));
            assert!(crop_args.contains(&"--format".to_string()));
            assert!(crop_args.contains(&"json".to_string()));
            assert!(crop_args.contains(&"--extension".to_string()));
            assert!(crop_args.contains(&"md".to_string()));
            assert!(crop_args.contains(&"--exclude-dir".to_string()));
            assert!(crop_args.contains(&"target".to_string()));
        }
        assert!(args[1].contains(&output_dir.join("backlinks.json").display().to_string()));
    }

    #[test]
    fn sync_requires_root_or_view() {
        let dir = tempfile::tempdir().unwrap();
        let err = build_sync_args(SyncArgs {
            root: None,
            view: None,
            output_dir: dir.path().join("side-info"),
            extensions: vec![],
            exclude_dirs: vec![],
        })
        .unwrap_err();

        assert!(err.to_string().contains("requires --root or --view"));
    }
}
