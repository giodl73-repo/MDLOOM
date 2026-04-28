use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use anyhow::Context;
use proof_lib::davinci::check_daVinci;
use proof_lib::draft::build_draft_plan;
use proof_lib::fix::{serialize_json, serialize_rich, FixOptions};
use proof_lib::compile::{compile_file, derive_output_path, ViolationSeverity};
use proof_lib::tree::dirtree::{DirtreeOptions, SortOrder, generate as dirtree_generate, verify_paths as dirtree_verify};
use proof_lib::tree::schema::{FieldMap, generate_org, generate_taxonomy, generate_dependency, generate_outline};
use proof_lib::spec_gen;
use proof_lib::layout::{self, Align, Direction, LayoutConfig, extract_content_lines};
use proof_lib::{Confidence, Diagnostic, FixPlan, GlintConfig, Runner, Severity};
use std::path::{Path, PathBuf};
use std::process;

#[derive(Parser)]
#[command(
    name = "proof",
    version,
    about = "A fast, schema-driven markdown and ASCII art linter with AI-assisted fix pipeline",
    long_about = None,
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    /// Files or directories to lint (default: current directory)
    paths: Vec<PathBuf>,

    /// Schema config file (default: proof.toml in target directory)
    #[arg(short, long, global = true)]
    config: Option<PathBuf>,

    /// Output format: text (default) | json | rich | github
    #[arg(short = 'f', long, default_value = "text", global = true)]
    format: String,

    /// Show only errors (suppress warnings)
    #[arg(short = 'e', long, global = true)]
    errors_only: bool,

    /// Exit with code 0 even if issues found
    #[arg(long, global = true)]
    no_fail: bool,

    /// Write output to file instead of stdout
    #[arg(short = 'o', long, global = true)]
    output: Option<PathBuf>,
}

#[derive(Subcommand)]
enum TreeAction {
    /// Generate a dirtree from the filesystem
    Generate {
        /// Tree kind: dirtree | org | taxonomy | dependency | outline (default: dirtree)
        #[arg(long, default_value = "dirtree")]
        kind: String,
        /// Source URI for schema-driven kinds (md://path#section:table:0 or md://path.json)
        /// Not needed for dirtree (uses --root instead)
        source: Option<String>,
        // ── dirtree options ──────────────────────────
        /// Root directory to walk (dirtree only)
        #[arg(long, default_value = ".")]
        root: PathBuf,
        /// Max depth to recurse (dirtree only)
        #[arg(long)]
        max_depth: Option<usize>,
        /// Glob patterns to exclude, comma-separated (dirtree only)
        #[arg(long)]
        exclude: Option<String>,
        /// Sort order: name | ext | size | mtime
        #[arg(long, default_value = "name")]
        sort: String,
        /// Directories before files (dirtree only, default: true)
        #[arg(long, default_value = "true")]
        dirs_first: bool,
        // ── schema-driven field mapping ───────────────
        /// Column/field name for the node label (auto-detected if omitted)
        #[arg(long)]
        name: Option<String>,
        /// Column/field name for the parent reference (auto-detected if omitted)
        #[arg(long)]
        parent: Option<String>,
        /// Column/field name for display text (auto-detected if omitted)
        #[arg(long)]
        label: Option<String>,
        /// Source data format: table | json (default: table)
        #[arg(long, default_value = "table")]
        format: String,
        /// Value that marks a root node (default: —, -, null, empty)
        #[arg(long)]
        root_marker: Option<String>,
        // ── shared ────────────────────────────────────
        /// Indent width per level (default: 4)
        #[arg(long, default_value = "4")]
        indent_width: usize,
        /// Don't wrap output in a dirtree/tree fence
        #[arg(long)]
        no_fence: bool,
        /// Root directory for md:// resolution (default: cwd)
        #[arg(long)]
        resolve_root: Option<PathBuf>,
        /// Write output to file (default: stdout)
        #[arg(short = 'o', long)]
        output: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum Command {
    /// Lint files and report diagnostics (default)
    Check {
        paths: Vec<PathBuf>,
        /// Also validate all pinned DaVinci figures against their invariants
        #[arg(long = "daVinci")]
        da_vinci: bool,
        /// Show error/warning counts grouped by diagnostic code
        #[arg(long)]
        by_code: bool,
        /// Group identical diagnostics by (code, directory) — at corpus scale,
        /// renders "50x CODE in dir/*.md" instead of 50 individual lines.
        /// Singletons still print normally; groups of 2+ collapse.
        #[arg(long)]
        deduplicate: bool,
        /// Also report `.md` figures that no `.source.md` references via
        /// `proof:include` / `proof:layout` / `source=md://...`. Emitted as
        /// `unused_figure` warnings — useful for pruning orphaned drafts.
        #[arg(long)]
        unused: bool,
    },
    /// Apply a fix plan generated by AI
    Fix {
        /// Fix plan JSON file
        #[arg(long, required = true)]
        plan: PathBuf,
        /// Show diff without writing any files
        #[arg(long)]
        dry_run: bool,
        /// Only apply fixes at or above this confidence: high | medium | low
        #[arg(long, default_value = "high")]
        min_confidence: String,
        /// Skip re-running check after applying fixes
        #[arg(long)]
        no_verify: bool,
        /// Skip signal-loss check (allow fixes that remove non-whitespace content)
        /// Use only when you've confirmed the removed content is preserved elsewhere
        #[arg(long)]
        no_signal_check: bool,
    },
    /// Generate a pre-populated draft fix plan — AI fills in decisions inline
    Draft {
        paths: Vec<PathBuf>,
        /// Output file for the draft plan (default: draft-plan.json)
        #[arg(short = 'o', long, default_value = "draft-plan.json")]
        output: PathBuf,
    },
    /// Pin a figure as a DaVinci — registers it in proof.toml with invariants
    Pin {
        /// The md:// URI to pin
        uri: String,
        /// Stable identifier for this pin
        #[arg(long, required = true)]
        id: String,
        /// Human description
        #[arg(long, default_value = "")]
        description: String,
        /// Template name for base invariants
        #[arg(long)]
        template: Option<String>,
        /// Protection tier: warn | error | lock
        #[arg(long, default_value = "warn")]
        protection: String,
        /// Root directory (default: current directory)
        #[arg(long)]
        root: Option<PathBuf>,
        /// Config file to update (default: proof.toml in current directory)
        #[arg(long)]
        config_file: Option<PathBuf>,
    },
    /// List all pinned DaVinci figures
    PinList,
    /// Resolve an md:// URI — print the element content, label, and line range
    Resolve {
        /// The md:// URI to resolve
        uri: String,
        /// Root directory (default: current directory, or where proof.toml lives)
        #[arg(short, long)]
        root: Option<PathBuf>,
        /// Output format: text (default) | json
        #[arg(short = 'f', long, default_value = "text")]
        format: String,
    },
    /// List every .source.md file referencing an md:// URI (reverse dependency lookup).
    /// File-only and heading-only queries match all references at or below that scope.
    Depends {
        /// The md:// URI to look up
        uri: String,
        /// Root directory to scan (default: current directory, or where proof.toml lives)
        #[arg(short, long)]
        root: Option<PathBuf>,
        /// Output format: text (default) | json
        #[arg(long, default_value = "text")]
        format: String,
    },
    /// Print the effective config for a path
    Config,
    /// Write a proof.toml to the current directory
    Init,
    /// Summary statistics (error/warning counts by directory and code)
    Stats {
        paths: Vec<PathBuf>,
        /// Break down by directory
        #[arg(long)]
        by_directory: bool,
        /// Break down by error code
        #[arg(long)]
        by_code: bool,
    },
    /// Compile source documents — resolve proof: directives and write output
    Compile {
        /// Source files or directories (default: current directory)
        paths: Vec<PathBuf>,
        /// Explicit output path (only valid for single-file compile)
        #[arg(short = 'o', long)]
        output: Option<PathBuf>,
        /// Output directory for all compiled files (overrides per-file placement)
        #[arg(long)]
        output_dir: Option<PathBuf>,
        /// Validate without writing any output files
        #[arg(long)]
        check: bool,
        /// Watch for changes and recompile automatically
        #[arg(long)]
        watch: bool,
        /// Delete output file when compile produces errors (default: leave stale output in place)
        #[arg(long)]
        delete_on_error: bool,
        /// Show running count instead of one line per file (useful for 50+ source files)
        #[arg(long)]
        progress: bool,
        /// Root directory for md:// URI resolution (default: proof.toml location or cwd)
        #[arg(long)]
        root: Option<PathBuf>,
    },
    /// Generate or validate ASCII tree diagrams
    Tree {
        #[command(subcommand)]
        action: TreeAction,
    },
    /// Analyze a figure and generate suggested DaVinci invariants
    SpecGenerate {
        /// The md:// URI of the figure to analyze
        uri: String,
        /// Stable ID for the [[davinci]] entry (default: derived from URI)
        #[arg(long)]
        id: Option<String>,
        /// Protection tier: warn | error | lock (default: error)
        #[arg(long, default_value = "error")]
        protection: String,
        /// Root directory for URI resolution (default: cwd)
        #[arg(long)]
        root: Option<PathBuf>,
        /// Write output to file instead of stdout
        #[arg(short = 'o', long)]
        output: Option<PathBuf>,
    },
    /// Compose N figures side-by-side into a single ASCII art collage
    Layout {
        /// Source figures: md:// URIs or file paths
        sources: Vec<String>,
        /// Spaces between frames (default: 3)
        #[arg(long, default_value = "3")]
        gap: usize,
        /// Vertical alignment: top | center | bottom (default: top)
        #[arg(long, default_value = "top")]
        align: String,
        /// Labels above each frame (one per source, space-separated)
        #[arg(long, num_args = 0..)]
        labels: Vec<String>,
        /// Number of frames per row before wrapping (default: all)
        #[arg(long)]
        cols: Option<usize>,
        /// Max output width in columns (default: 120)
        #[arg(long, default_value = "120")]
        width: usize,
        /// Composition direction: horizontal | vertical (or h | v)
        #[arg(long, default_value = "horizontal")]
        direction: String,
        /// Add a border box around each frame
        #[arg(long)]
        border: bool,
        /// Write output to file (default: stdout)
        #[arg(short = 'o', long)]
        output: Option<PathBuf>,
        /// Root directory for md:// URI resolution (default: current directory)
        #[arg(long)]
        root: Option<PathBuf>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command.as_ref() {
        Some(Command::Draft { .. }) => {}  // handled below
        _ => {}
    }
    match cli.command {
        Some(Command::Fix { plan, dry_run, min_confidence, no_verify, no_signal_check }) => {
            return cmd_fix(plan, dry_run, min_confidence, no_verify, no_signal_check, &cli.config);
        }
        Some(Command::Draft { paths, output }) => {
            let paths = if paths.is_empty() { vec![std::env::current_dir()?] } else { paths };
            return cmd_draft(paths, output, &cli.config);
        }
        Some(Command::Resolve { uri, root, format }) => {
            return cmd_resolve(uri, root, format);
        }
        Some(Command::Depends { uri, root, format }) => {
            return cmd_depends(uri, root, format);
        }
        Some(Command::Pin { uri, id, description, template, protection, root, config_file }) => {
            return cmd_pin(uri, id, description, template, protection, root, config_file);
        }
        Some(Command::PinList) => {
            return cmd_pin_list(&cli.config);
        }
        Some(Command::Init) => {
            return cmd_init();
        }
        Some(Command::Config) => {
            println!("(use proof.toml in your project directory — see `proof init`)");
            return Ok(());
        }
        Some(Command::Stats { paths, by_directory, by_code }) => {
            let paths = if paths.is_empty() { vec![std::env::current_dir()?] } else { paths };
            return cmd_stats(paths, by_directory, by_code, &cli.config);
        }
        Some(Command::Tree { action }) => {
            return cmd_tree(action);
        }
        Some(Command::SpecGenerate { uri, id, protection, root, output }) => {
            return cmd_spec_generate(uri, id, protection, root, output);
        }
        Some(Command::Compile { paths, output, output_dir, check, watch, delete_on_error, progress, root }) => {
            let paths = if paths.is_empty() { vec![std::env::current_dir()?] } else { paths };
            if watch {
                return cmd_compile_watch(paths, output_dir, root, &cli.config);
            }
            return cmd_compile(paths, output, output_dir, check, delete_on_error, progress, root, &cli.config);
        }
        Some(Command::Layout {
            sources, gap, align, labels, cols, width, direction, border, output, root,
        }) => {
            return cmd_layout(
                sources, gap, align, labels, cols, width, direction, border, output, root,
            );
        }
        _ => {}
    }

    // Default: check
    let mut show_by_code = false;
    let mut deduplicate = false;
    let mut detect_unused = false;
    let (paths, da_vinci) = match &cli.command {
        Some(Command::Check { paths, da_vinci, by_code, deduplicate: dedup, unused: u }) => {
            show_by_code = *by_code;
            deduplicate = *dedup;
            detect_unused = *u;
            (if paths.is_empty() { vec![] } else { paths.clone() }, *da_vinci)
        }
        _ => (vec![], false),
    };
    let paths = if paths.is_empty() && !cli.paths.is_empty() {
        cli.paths.clone()
    } else if paths.is_empty() {
        vec![std::env::current_dir()?]
    } else {
        paths
    };

    cmd_check(paths, &cli, da_vinci, show_by_code, deduplicate, detect_unused)
}

// ─────────────────────────────────────────────────────────
// check
// ─────────────────────────────────────────────────────────

fn cmd_check(paths: Vec<PathBuf>, cli: &Cli, da_vinci: bool, show_by_code: bool, deduplicate: bool, detect_unused: bool) -> Result<()> {
    let mut all_diags: Vec<Diagnostic> = Vec::new();
    let mut files_checked = 0usize;

    // DaVinci root = the directory containing proof.toml (the proof root)
    // Run once, not per-file, using the config's location as the URI root
    if da_vinci {
        let proof_root = cli.config.as_deref()
            .and_then(|p| std::path::Path::new(p).parent())
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| paths.first()
                .map(|p| if p.is_dir() { p.clone() } else { p.parent().unwrap_or(p).to_path_buf() })
                .unwrap_or_else(|| std::env::current_dir().unwrap())
            );
        let cfg = load_config(&proof_root, &cli.config);
        if !cfg.davinci.is_empty() {
            let dv_diags = check_daVinci(&cfg, &proof_root);
            if dv_diags.is_empty() {
                eprintln!("{} all {} DaVinci invariants satisfied",
                    "✓".green(), cfg.davinci.len());
            }
            all_diags.extend(dv_diags);
        }
    }

    for path in &paths {
        let cfg = load_config(path, &cli.config);
        let dir = if path.is_dir() { path.clone() } else { path.parent().unwrap_or(path).to_path_buf() };
        let runner = Runner::new(&dir, cfg)?;

        if path.is_file() {
            files_checked += 1;
            all_diags.extend(runner.lint_file(path));
        } else {
            files_checked += count_files(path);
            all_diags.extend(runner.run());
        }
    }

    // Corpus-level scan for orphaned figures (--unused). Runs once across all
    // input paths so a figure under one directory can still be considered used
    // when referenced from a sibling.
    if detect_unused {
        for path in &paths {
            let scan_root = if path.is_dir() {
                path.clone()
            } else {
                path.parent().unwrap_or(path).to_path_buf()
            };
            all_diags.extend(proof_lib::unused::unused_diagnostics(&scan_root));
        }
    }

    if cli.errors_only {
        all_diags.retain(|d| d.severity == Severity::Error);
    }

    all_diags.sort_by(|a, b| {
        a.file.cmp(&b.file)
            .then(a.span.line.cmp(&b.span.line))
            .then(a.span.col.cmp(&b.span.col))
    });

    let error_count = all_diags.iter().filter(|d| d.severity == Severity::Error).count();
    let warn_count = all_diags.iter().filter(|d| d.severity == Severity::Warning).count();

    let out = if deduplicate && cli.format == "text" {
        format_deduplicated(&all_diags)
    } else {
        format_output(&all_diags, &cli.format)?
    };

    if let Some(ref out_path) = cli.output {
        std::fs::write(out_path, &out)?;
        eprintln!("Output written to {}", out_path.display());
    } else {
        print!("{}", out);
    }

    if !all_diags.is_empty() && cli.format == "text" {
        eprintln!();
    }

    let status = if error_count > 0 { "FAIL".red().bold() } else { "OK".green().bold() };
    eprintln!(
        "{} — {} files checked, {} error{}, {} warning{}",
        status, files_checked,
        error_count, if error_count == 1 { "" } else { "s" },
        warn_count,  if warn_count == 1  { "" } else { "s" },
    );

    if show_by_code && !all_diags.is_empty() {
        use std::collections::BTreeMap;
        let mut by_code: BTreeMap<&str, (usize, usize)> = BTreeMap::new(); // (errors, warnings)
        for d in &all_diags {
            let entry = by_code.entry(d.code).or_default();
            if d.severity == Severity::Error { entry.0 += 1; } else { entry.1 += 1; }
        }
        eprintln!();
        for (code, (errs, warns)) in &by_code {
            let parts: Vec<String> = [
                if *errs > 0 { format!("{} error{}", errs, if *errs == 1 { "" } else { "s" }) } else { String::new() },
                if *warns > 0 { format!("{} warning{}", warns, if *warns == 1 { "" } else { "s" }) } else { String::new() },
            ].iter().filter(|s| !s.is_empty()).cloned().collect();
            eprintln!("  {:<30} {}", code, parts.join(", "));
        }
    }

    if !cli.no_fail && error_count > 0 {
        process::exit(1);
    }
    Ok(())
}

fn format_output(diags: &[Diagnostic], format: &str) -> Result<String> {
    match format {
        "json" => Ok(serialize_json(diags)?),
        "rich" => Ok(serialize_rich(diags)?),
        "github" => {
            let mut out = String::new();
            for d in diags {
                let level = match d.severity {
                    Severity::Error => "error",
                    Severity::Warning => "warning",
                    Severity::Info => "notice",
                };
                out.push_str(&format!(
                    "::{} file={},line={},col={}::[{}] {}\n",
                    level, d.file.display(), d.span.line, d.span.col, d.code, d.message
                ));
            }
            Ok(out)
        }
        _ => {
            // text (default)
            let mut out = String::new();
            for d in diags {
                let sev = match d.severity {
                    Severity::Error => "error".red().bold().to_string(),
                    Severity::Warning => "warning".yellow().bold().to_string(),
                    Severity::Info => "info".blue().to_string(),
                };
                out.push_str(&format!(
                    "{}:{}: {} [{}]: {}\n",
                    d.file.display().to_string().cyan(),
                    d.span.to_string().white(),
                    sev,
                    d.code.dimmed(),
                    d.message
                ));
                if let Some(ref note) = d.note {
                    out.push_str(&format!("  {} {}\n", "note:".dimmed(), note));
                }
            }
            Ok(out)
        }
    }
}

/// Group identical diagnostics (same code + parent directory) into one summary
/// line each. Singletons render normally. Groups of N >= 2 render as
/// `Nx CODE [severity]: message — in <dir>/*.md`.
///
/// This is the --deduplicate text renderer. Default text rendering is unchanged.
fn format_deduplicated(diags: &[Diagnostic]) -> String {
    use std::collections::BTreeMap;

    // Key: (code, parent dir as displayable string, severity).
    // Value: (count, first diagnostic seen for that key — used for sample message).
    type Key = (&'static str, String, Severity);
    let mut groups: BTreeMap<Key, (usize, &Diagnostic)> = BTreeMap::new();
    let mut order: Vec<Key> = Vec::new();

    for d in diags {
        let parent = d.file.parent()
            .map(|p| p.display().to_string())
            .unwrap_or_default();
        let key: Key = (d.code, parent, d.severity.clone());
        let entry = groups.entry(key.clone()).or_insert_with(|| {
            order.push(key.clone());
            (0, d)
        });
        entry.0 += 1;
    }

    let mut out = String::new();
    for key in &order {
        let (count, sample) = &groups[key];
        let (code, parent, severity) = key;
        let sev = match severity {
            Severity::Error => "error".red().bold().to_string(),
            Severity::Warning => "warning".yellow().bold().to_string(),
            Severity::Info => "info".blue().to_string(),
        };
        if *count == 1 {
            // Render exactly like text format for singletons.
            out.push_str(&format!(
                "{}:{}: {} [{}]: {}\n",
                sample.file.display().to_string().cyan(),
                sample.span.to_string().white(),
                sev,
                code.dimmed(),
                sample.message
            ));
            if let Some(ref note) = sample.note {
                out.push_str(&format!("  {} {}\n", "note:".dimmed(), note));
            }
        } else {
            let location = if parent.is_empty() {
                "*.md".to_string()
            } else {
                format!("{}/*.md", parent)
            };
            out.push_str(&format!(
                "{} {} [{}]: {} {} {}\n",
                format!("{}x", count).bold(),
                sev,
                code.dimmed(),
                sample.message,
                "in".dimmed(),
                location.cyan(),
            ));
        }
    }
    out
}

// ─────────────────────────────────────────────────────────
// fix
// ─────────────────────────────────────────────────────────

fn cmd_fix(
    plan_path: PathBuf,
    dry_run: bool,
    min_confidence_str: String,
    no_verify: bool,
    no_signal_check: bool,
    config_override: &Option<PathBuf>,
) -> Result<()> {
    let min_confidence = match min_confidence_str.as_str() {
        "high" => Confidence::High,
        "medium" => Confidence::Medium,
        "low" => Confidence::Low,
        other => {
            eprintln!("proof: unknown confidence level {:?} — use high, medium, or low", other);
            process::exit(2);
        }
    };

    // Accept both FixPlan and DraftPlan (draft → fix via to_fix_plan())
    let plan = load_plan(&plan_path)?;
    let root = std::env::current_dir()?;

    eprintln!(
        "{} {} fixes from {} (min confidence: {}, dry-run: {})",
        if dry_run { "Previewing" } else { "Applying" },
        plan.fixes.len(),
        plan_path.display(),
        min_confidence,
        dry_run,
    );

    let opts = FixOptions { dry_run, min_confidence, check_signal: !no_signal_check };
    let result = plan.apply(&opts, &root)?;

    eprintln!();
    for skip in &result.skipped {
        eprintln!("{} [{}] {}", "SKIP".yellow(), skip.id, skip.reason);
    }

    eprintln!();
    if dry_run {
        eprintln!("{} — {} fixes previewed, {} skipped (no files written)",
            "DRY RUN".cyan().bold(), result.applied.len(), result.skipped.len());
    } else {
        eprintln!(
            "{} — {} fixes applied to {} files, {} skipped",
            "DONE".green().bold(), result.applied.len(), result.files_modified, result.skipped.len()
        );

        // Re-run check unless suppressed
        if !no_verify && result.files_modified > 0 {
            eprintln!("\n{} verifying fixes…", "→".cyan());
            let args = vec![std::env::current_dir()?.to_string_lossy().to_string()];
            let verify_cfg = GlintConfig::load_or_default(&root);
            let runner = Runner::new(&root, verify_cfg)?;
            let diags = runner.run();
            let errors = diags.iter().filter(|d| d.severity == Severity::Error).count();
            if errors == 0 {
                eprintln!("{} zero errors remaining", "✓".green());
            } else {
                eprintln!(
                    "{} {} error{} remain after fix — review manually",
                    "!".yellow(), errors, if errors == 1 { "" } else { "s" }
                );
                let _ = args; // suppress unused warning
                process::exit(1);
            }
        }
    }

    Ok(())
}

// ─────────────────────────────────────────────────────────
// stats
// ─────────────────────────────────────────────────────────

fn cmd_stats(
    paths: Vec<PathBuf>,
    by_directory: bool,
    by_code: bool,
    _config_override: &Option<PathBuf>,
) -> Result<()> {
    let mut all_diags: Vec<Diagnostic> = Vec::new();
    let mut files_checked = 0usize;

    for path in &paths {
        let cfg = load_config(&path, &None);
        let dir = if path.is_dir() { path.clone() } else { path.parent().unwrap_or(&path).to_path_buf() };
        let runner = Runner::new(&dir, cfg)?;
        if path.is_file() {
            files_checked += 1;
            all_diags.extend(runner.lint_file(path));
        } else {
            files_checked += count_files(path);
            all_diags.extend(runner.run());
        }
    }

    let errors = all_diags.iter().filter(|d| d.severity == Severity::Error).count();
    let warnings = all_diags.iter().filter(|d| d.severity == Severity::Warning).count();

    println!("files:    {}", files_checked);
    println!("errors:   {}", errors);
    println!("warnings: {}", warnings);

    if by_code {
        println!("\nBy error code:");
        let mut by_code_map: std::collections::BTreeMap<&str, usize> = std::collections::BTreeMap::new();
        for d in &all_diags {
            *by_code_map.entry(d.code).or_default() += 1;
        }
        for (code, count) in &by_code_map {
            println!("  {:30} {}", code, count);
        }
    }

    if by_directory {
        println!("\nBy directory:");
        let mut by_dir: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
        for d in &all_diags {
            let dir = d.file.parent()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| ".".to_string());
            *by_dir.entry(dir).or_default() += 1;
        }
        for (dir, count) in &by_dir {
            println!("  {:50} {}", dir, count);
        }
    }

    Ok(())
}

// ─────────────────────────────────────────────────────────
// ─────────────────────────────────────────────────────────
// resolve
// ─────────────────────────────────────────────────────────

fn cmd_pin(
    uri: String,
    id: String,
    description: String,
    template: Option<String>,
    protection: String,
    root: Option<PathBuf>,
    config_file: Option<PathBuf>,
) -> Result<()> {
    let root = root.unwrap_or_else(|| std::env::current_dir().unwrap());
    let config_path = config_file.unwrap_or_else(|| root.join("proof.toml"));

    // Resolve the URI first to verify it works and get metadata
    let parsed = mdpath::parse(&uri)
        .map_err(|e| anyhow::anyhow!("invalid md:// URI: {}", e))?;
    let element = mdpath::resolve(&parsed, &root)
        .map_err(|e| anyhow::anyhow!("cannot resolve URI: {}", e))?;

    // Use the named form of the URI (strings over numbers)
    let stable_uri = element.uri.clone();

    // Build TOML snippet to append
    let desc = if description.is_empty() {
        element.label.as_deref().unwrap_or("").to_string()
    } else {
        description
    };

    let template_line = template
        .as_deref()
        .map(|t| format!("\ntemplate = {:?}", t))
        .unwrap_or_default();

    let toml_snippet = format!(
        "\n[[davinci]]\nid = {:?}\nuri = {:?}\ndescription = {:?}\nprotection = {:?}{}\n",
        id, stable_uri, desc, protection, template_line
    );

    // Append to proof.toml
    let existing = std::fs::read_to_string(&config_path).unwrap_or_default();
    if existing.contains(&format!("id = {:?}", id)) {
        eprintln!("{} DaVinci '{}' already exists in {} — update it manually",
            "warn:".yellow(), id, config_path.display());
        return Ok(());
    }
    std::fs::write(&config_path, format!("{}{}", existing, toml_snippet))?;

    eprintln!("{} Pinned {} as '{}'", "✓".green().bold(), stable_uri.cyan(), id);
    eprintln!("  Kind: {}", element.kind.as_deref().unwrap_or("figure"));
    eprintln!("  Lines: {}–{}", element.line_start, element.line_end);
    if let Some(label) = &element.label {
        eprintln!("  Label: {}", label);
    }
    eprintln!();
    eprintln!("Add invariants to {}", config_path.display());
    eprintln!("Then run: proof check --daVinci .");

    Ok(())
}

fn cmd_pin_list(config_override: &Option<PathBuf>) -> Result<()> {
    let root = std::env::current_dir()?;
    let cfg = load_config(&root, config_override);
    if cfg.davinci.is_empty() {
        println!("No DaVinci entries registered. Use `proof pin md://... --id name` to pin a figure.");
        return Ok(());
    }
    println!("{} DaVinci entries:", cfg.davinci.len());
    for entry in &cfg.davinci {
        let inv_count = entry.invariants.len();
        println!(
            "  {} [{}] {} — {} invariant{}",
            entry.id.cyan().bold(),
            entry.protection,
            entry.uri,
            inv_count,
            if inv_count == 1 { "" } else { "s" }
        );
        if !entry.description.is_empty() {
            println!("    {}", entry.description.dimmed());
        }
    }
    Ok(())
}

fn cmd_resolve(uri: String, root: Option<PathBuf>, format: String) -> Result<()> {
    let root = root.unwrap_or_else(|| std::env::current_dir().unwrap());

    let parsed = mdpath::parse(&uri)
        .map_err(|e| anyhow::anyhow!("invalid md:// URI: {}", e))?;

    let element = mdpath::resolve(&parsed, &root)
        .map_err(|e| anyhow::anyhow!("resolve failed: {}", e))?;

    match format.as_str() {
        "json" => {
            let json = serde_json::json!({
                "uri": element.uri,
                "file": element.file.display().to_string(),
                "line_start": element.line_start,
                "line_end": element.line_end,
                "element_type": format!("{:?}", element.element_type),
                "kind": element.kind,
                "label": element.label,
                "section_heading": element.section_heading,
                "content": element.content,
            });
            println!("{}", serde_json::to_string_pretty(&json)?);
        }
        _ => {
            // text format
            println!("{}", element.uri.cyan());
            if let Some(h) = &element.section_heading {
                println!("  section:  {}", h);
            }
            if let Some(label) = &element.label {
                println!("  label:    {}", label);
            }
            if let Some(kind) = &element.kind {
                println!("  kind:     {}", kind);
            }
            println!("  lines:    {}–{}", element.line_start, element.line_end);
            println!("  file:     {}", element.file.display());
            println!();
            println!("{}", element.content);
        }
    }

    Ok(())
}

// ─────────────────────────────────────────────────────────
// depends — reverse dependency lookup for md:// URIs
// ─────────────────────────────────────────────────────────

fn cmd_depends(uri: String, root: Option<PathBuf>, format: String) -> Result<()> {
    let scan_root = root
        .or_else(|| find_proof_root_for_cwd())
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    let deps = proof_lib::depends::find_dependents(&uri, &scan_root);

    match format.as_str() {
        "json" => {
            let arr: Vec<_> = deps.iter().map(|d| {
                serde_json::json!({
                    "file": d.source_file.display().to_string(),
                    "line": d.line,
                    "uri": d.uri,
                })
            }).collect();
            println!("{}", serde_json::to_string_pretty(&serde_json::json!({
                "query": uri,
                "root": scan_root.display().to_string(),
                "count": deps.len(),
                "references": arr,
            }))?);
        }
        _ => {
            if deps.is_empty() {
                println!("No references to {} found under {}",
                    uri.cyan(),
                    scan_root.display());
                return Ok(());
            }
            println!("{} reference{} to {}:",
                deps.len(),
                if deps.len() == 1 { "" } else { "s" },
                uri.cyan().bold());
            for d in &deps {
                let rel = d.source_file
                    .strip_prefix(&scan_root)
                    .unwrap_or(&d.source_file);
                println!("  {}:{}  {}",
                    rel.display(),
                    d.line.to_string().yellow(),
                    d.uri.dimmed());
            }
        }
    }
    Ok(())
}

/// Walk up from cwd looking for proof.toml so `proof depends` works from any subdir.
fn find_proof_root_for_cwd() -> Option<PathBuf> {
    let mut dir = std::env::current_dir().ok()?;
    loop {
        if dir.join("proof.toml").exists() { return Some(dir); }
        match dir.parent() {
            Some(p) => dir = p.to_path_buf(),
            None => return None,
        }
    }
}

// ─────────────────────────────────────────────────────────
// tree
// ─────────────────────────────────────────────────────────

fn cmd_tree(action: TreeAction) -> Result<()> {
    match action {
        TreeAction::Generate {
            kind, source, root, max_depth, exclude, sort, dirs_first,
            name, parent, label, format, root_marker,
            indent_width, no_fence, resolve_root, output,
        } => {
            let result = match kind.as_str() {
                "dirtree" => {
                    let sort_order = match sort.as_str() {
                        "ext"   => SortOrder::Ext,
                        "size"  => SortOrder::Size,
                        "mtime" => SortOrder::Mtime,
                        _       => SortOrder::Name,
                    };
                    let exclude_patterns = exclude
                        .map(|s| s.split(',').map(|p| p.trim().to_string()).collect())
                        .unwrap_or_default();
                    let opts = DirtreeOptions {
                        root,
                        max_depth,
                        exclude: exclude_patterns,
                        dirs_first,
                        sort: sort_order,
                        wrap_fence: !no_fence,
                        indent_width,
                    };
                    dirtree_generate(&opts)?
                }
                other => {
                    // Schema-driven: resolve source URI or file
                    let src_uri = source.ok_or_else(|| {
                        anyhow::anyhow!(
                            "proof tree generate --kind {} requires a source URI argument\n\
                             Example: proof tree generate --kind org md://docs/team.md#:table:0",
                            other
                        )
                    })?;

                    let resolve_from = resolve_root.unwrap_or_else(|| std::env::current_dir().unwrap());
                    let content = resolve_source(&src_uri, &resolve_from)?;

                    let mut field_map = FieldMap {
                        name,
                        parent,
                        label,
                        root_marker,
                        ..Default::default()
                    };

                    let body = match other {
                        "org" => generate_org(&content, &format, &mut field_map, indent_width)?,
                        "taxonomy" => generate_taxonomy(&content, &format, &mut field_map, indent_width)?,
                        "dependency" => generate_dependency(&content, &format, &mut field_map, indent_width)?,
                        "outline" => generate_outline(&content, indent_width)?,
                        unknown => anyhow::bail!(
                            "unknown tree kind {:?} — use dirtree, org, taxonomy, dependency, or outline",
                            unknown
                        ),
                    };

                    if no_fence {
                        body
                    } else {
                        format!("```{}\n{}\n```", other, body)
                    }
                }
            };

            match output {
                Some(path) => {
                    std::fs::write(&path, &result)?;
                    eprintln!("{} {} tree written to {}", "✓".green(), kind, path.display());
                }
                None => println!("{}", result),
            }
        }
    }
    Ok(())
}

/// Resolve a source — md:// URI via mdpath, or plain file path.
fn resolve_source(src: &str, root: &std::path::Path) -> Result<String> {
    if src.starts_with("md://") {
        let parsed = mdpath::parse(src)
            .map_err(|e| anyhow::anyhow!("invalid md:// URI {:?}: {}", src, e))?;
        let element = mdpath::resolve(&parsed, root)
            .map_err(|e| anyhow::anyhow!("cannot resolve {:?}: {}", src, e))?;
        Ok(element.content)
    } else {
        // Plain file path
        let path = root.join(src);
        std::fs::read_to_string(&path)
            .with_context(|| format!("reading source file: {}", path.display()))
    }
}

// ─────────────────────────────────────────────────────────
// spec generate
// ─────────────────────────────────────────────────────────

fn cmd_spec_generate(
    uri: String,
    id_override: Option<String>,
    protection: String,
    root_override: Option<PathBuf>,
    output: Option<PathBuf>,
) -> Result<()> {
    let root = root_override.unwrap_or_else(|| std::env::current_dir().unwrap());

    // Resolve the URI
    let parsed = mdpath::parse(&uri)
        .map_err(|e| anyhow::anyhow!("invalid md:// URI {:?}: {}", uri, e))?;
    let element = mdpath::resolve(&parsed, &root)
        .map_err(|e| anyhow::anyhow!("cannot resolve {:?}: {}", uri, e))?;

    // Derive ID from URI if not provided
    let id = id_override.unwrap_or_else(|| {
        // Use the last path segment minus extension, falling back to "figure"
        parsed.path
            .split('/')
            .last()
            .unwrap_or("figure")
            .trim_end_matches(".md")
            .replace(['-', '_'], "-")
            .to_string()
    });

    eprintln!("{} Analyzing {} ({} lines)...",
        "→".cyan(),
        uri.dimmed(),
        element.content.lines().count(),
    );

    let spec = spec_gen::generate(
        &element.content,
        element.label.as_deref(),
        &uri,
        &id,
    );

    // Override protection from CLI
    let mut spec = spec;
    spec.protection = protection;

    let toml_out = spec_gen::format_toml(&spec);

    // Print summary to stderr, TOML to stdout (or file)
    eprintln!("{} {} invariant{} suggested for {:?}",
        "✓".green(),
        spec.invariants.len(),
        if spec.invariants.len() == 1 { "" } else { "s" },
        spec.id,
    );
    for inv in &spec.invariants {
        eprintln!("  {} [{}] {}",
            match inv.confidence {
                spec_gen::SuggestionConfidence::High   => "●".green().to_string(),
                spec_gen::SuggestionConfidence::Medium => "◐".yellow().to_string(),
                spec_gen::SuggestionConfidence::Low    => "○".dimmed().to_string(),
            },
            inv.confidence.label(),
            inv.rule,
        );
    }
    eprintln!();
    eprintln!("Paste the output below into your proof.toml, then run:");
    eprintln!("  proof check --daVinci .");
    eprintln!();

    match output {
        Some(path) => {
            std::fs::write(&path, &toml_out)?;
            eprintln!("{} written to {}", "✓".green(), path.display());
        }
        None => print!("{}", toml_out),
    }

    Ok(())
}

// ─────────────────────────────────────────────────────────
// compile
// ─────────────────────────────────────────────────────────

fn cmd_compile(
    paths: Vec<PathBuf>,
    output_override: Option<PathBuf>,
    output_dir: Option<PathBuf>,
    check_only: bool,
    delete_on_error: bool,
    progress: bool,
    root_override: Option<PathBuf>,
    config_override: &Option<PathBuf>,
) -> Result<()> {
    if output_override.is_some() && output_dir.is_some() {
        eprintln!("{} -o and --output-dir are mutually exclusive", "error:".red());
        process::exit(2);
    }

    let root = root_override.unwrap_or_else(|| std::env::current_dir().unwrap());
    let config = load_config(&root, config_override);

    // Build a list of (source_path, output_dir) pairs.
    // When using [[compile]] targets from proof.toml (and no explicit paths/output-dir),
    // route each source file to the correct target's output_dir.
    let using_defaults = paths.iter().any(|p| p == &std::env::current_dir().unwrap());
    let has_multi_targets = config.compile.len() > 1;

    let source_dir_pairs: Vec<(PathBuf, Option<PathBuf>)> = if !config.compile.is_empty()
        && using_defaults
        && output_dir.is_none()
        && output_override.is_none()
    {
        // Per-target routing from proof.toml
        let mut pairs = Vec::new();
        for target in &config.compile {
            let src_dir = target.source_dir.as_ref()
                .map(|s| root.join(s))
                .unwrap_or_else(|| root.clone());
            let out = target.output_dir.as_ref().map(|d| root.join(d));
            if let Some(ref dir) = out { let _ = std::fs::create_dir_all(dir); }
            if src_dir.is_dir() {
                for entry in walkdir::WalkDir::new(&src_dir)
                    .into_iter().filter_map(|e| e.ok())
                    .filter(|e| e.file_type().is_file())
                {
                    let p = entry.path().to_path_buf();
                    if p.to_str().map(|s| s.ends_with(".source.md")).unwrap_or(false) {
                        pairs.push((p, out.clone()));
                    }
                }
            } else if src_dir.is_file() {
                pairs.push((src_dir, out));
            }
        }
        pairs
    } else {
        // Explicit paths or single output_dir override
        let resolved_out = output_dir
            .or_else(|| config.compile.first()
                .and_then(|t| t.output_dir.as_ref())
                .map(|d| root.join(d)));
        if let Some(ref dir) = resolved_out { let _ = std::fs::create_dir_all(dir); }
        let mut pairs = Vec::new();
        for path in &paths {
            if path.is_file() {
                pairs.push((path.clone(), resolved_out.clone()));
            } else {
                for entry in walkdir::WalkDir::new(path)
                    .into_iter().filter_map(|e| e.ok())
                    .filter(|e| e.file_type().is_file())
                {
                    let p = entry.path().to_path_buf();
                    if p.to_str().map(|s| s.ends_with(".source.md")).unwrap_or(false) {
                        pairs.push((p, resolved_out.clone()));
                    }
                }
            }
        }
        pairs
    };

    let source_files: Vec<PathBuf> = source_dir_pairs.iter().map(|(p, _)| p.clone()).collect();

    if source_files.is_empty() {
        eprintln!("{} no .source.md files found", "proof compile:".yellow());
        return Ok(());
    }

    if output_override.is_some() && source_files.len() > 1 {
        eprintln!("{} -o can only be used with a single source file", "error:".red());
        process::exit(2);
    }

    let mut total_errors = 0usize;
    let mut total_warnings = 0usize;
    let mut compiled = 0usize;

    for (source_path, target_out_dir) in &source_dir_pairs {
        let output_path = if let Some(ref out) = output_override {
            out.clone()
        } else if let Some(ref dir) = target_out_dir {
            // Derive filename, then place it in the output directory
            if let Some(derived) = derive_output_path(source_path) {
                let filename = derived.file_name().expect("derived path has filename");
                dir.join(filename)
            } else {
                eprintln!("{} {} has no .source.md suffix — skipping",
                    "skip:".yellow(), source_path.display());
                continue;
            }
        } else if let Some(p) = derive_output_path(source_path) {
            p
        } else {
            eprintln!("{} {} has no .source.md suffix — use -o to specify output",
                "skip:".yellow(), source_path.display());
            continue;
        };

        if check_only {
            eprintln!("  {} {} → {} (check only)",
                "→".cyan(), source_path.display(), output_path.display());
        }

        let result = compile_file(source_path, &output_path, &root, &config)?;

        // Report violations
        for v in &result.violations {
            let sev = match v.severity {
                ViolationSeverity::Error   => { total_errors += 1; "error".red().bold().to_string() }
                ViolationSeverity::Warning => { total_warnings += 1; "warning".yellow().bold().to_string() }
            };
            eprintln!(
                "{}:{}:{}: {} [{}]: {}",
                source_path.display().to_string().cyan(),
                v.source_line, 1,
                sev, v.code, v.message
            );
            if let Some(ref id) = v.figure_id {
                eprintln!("    figure: {}", id);
            }
            if !v.uri.is_empty() {
                eprintln!("    uri:    {}", v.uri);
            }
        }

        // F119: --delete-on-error removes stale output when compile fails
        if !result.written && delete_on_error && output_path.exists() {
            let _ = std::fs::remove_file(&output_path);
            eprintln!("{} deleted stale output: {}", "→".yellow(), output_path.display());
        }

        if result.written {
            compiled += 1;
            if !progress {
                eprintln!("{} {} → {}  ({} directive{})",
                    "✓".green(),
                    source_path.display().to_string().cyan(),
                    output_path.display(),
                    result.directives_resolved,
                    if result.directives_resolved == 1 { "" } else { "s" },
                );
            } else {
                eprint!("\r  compiling {}/{}…  ", compiled, source_files.len());
            }
        } else if !result.violations.iter().any(|v| v.severity == ViolationSeverity::Error) {
            if !check_only {
                // Copy source to output unchanged
                std::fs::copy(source_path, &output_path)?;
                compiled += 1;
                if progress { eprint!("\r  compiling {}/{}…  ", compiled, source_files.len()); }
            }
        }
    }

    if progress { eprintln!(); } // clear progress line
    eprintln!();
    if total_errors > 0 {
        eprintln!("{} — {} compiled, {} error{}, {} warning{}",
            "FAIL".red().bold(), compiled,
            total_errors, if total_errors == 1 { "" } else { "s" },
            total_warnings, if total_warnings == 1 { "" } else { "s" },
        );
        process::exit(1);
    } else {
        eprintln!("{} — {} compiled, {} warning{}",
            "OK".green().bold(), compiled,
            total_warnings, if total_warnings == 1 { "" } else { "s" },
        );
    }
    Ok(())
}

// ─────────────────────────────────────────────────────────
// compile --watch
// ─────────────────────────────────────────────────────────

fn cmd_compile_watch(
    paths: Vec<PathBuf>,
    output_dir_override: Option<PathBuf>,
    root_override: Option<PathBuf>,
    config_override: &Option<PathBuf>,
) -> Result<()> {
    use notify::{RecommendedWatcher, RecursiveMode, Watcher, Event, EventKind};
    use std::collections::{HashMap, HashSet};
    use std::sync::mpsc;
    use std::time::{Duration, Instant};

    let root = root_override.unwrap_or_else(|| std::env::current_dir().unwrap());
    let config = load_config(&root, config_override);

    // Build watch targets from [[compile]] entries or CLI paths
    // Each target is (source_dir, output_dir)
    let using_default_paths = paths.iter().any(|p| p == &std::env::current_dir().unwrap());

    let watch_targets: Vec<(PathBuf, Option<PathBuf>)> = if !config.compile.is_empty() && using_default_paths {
        // Use all [[compile]] targets from proof.toml
        config.compile.iter().map(|t| {
            let src = t.source_dir.as_ref().map(|s| root.join(s))
                .unwrap_or_else(|| root.clone());
            let out = t.output_dir.as_ref()
                .map(|d| root.join(d))
                .or_else(|| output_dir_override.clone());
            (src, out)
        }).collect()
    } else {
        // CLI paths + optional output_dir override
        let out = output_dir_override
            .or_else(|| config.compile.first()
                .and_then(|t| t.output_dir.as_ref())
                .map(|d| root.join(d)));
        paths.into_iter().map(|p| (p, out.clone())).collect()
    };

    // For watch, flatten to just watch_paths; output_dir isn't used here
    let output_dir: Option<PathBuf> = None; // unused in watch — each target carries its own

    if let Some(ref dir) = output_dir {
        std::fs::create_dir_all(dir)?;
    }

    eprintln!("{} watching for changes (Ctrl-C to stop)", "proof compile --watch:".cyan().bold());
    for (src, out) in &watch_targets {
        if let Some(out) = out {
            eprintln!("  {} → {}", src.display().to_string().dimmed(), out.display().to_string().dimmed());
            std::fs::create_dir_all(out)?;
        } else {
            eprintln!("  {} (output next to source)", src.display().to_string().dimmed());
        }
    }
    eprintln!();

    // Reverse-dependency index. dep_to_sources[F] = every source file whose
    // last successful compile pulled F in via an md:// URI. When F changes,
    // every source listed under it gets recompiled.
    let mut dep_to_sources: HashMap<PathBuf, HashSet<PathBuf>> = HashMap::new();
    let mut watched_deps: HashSet<PathBuf> = HashSet::new();

    let (tx, rx) = mpsc::channel::<Result<Event, notify::Error>>();
    let mut watcher = RecommendedWatcher::new(tx, notify::Config::default())?;

    for (src_dir, _) in &watch_targets {
        if src_dir.exists() {
            watcher.watch(src_dir, RecursiveMode::Recursive)?;
        }
    }

    // Initial compile pass for all targets — collect dependencies as we go.
    for (src_dir, out_dir) in &watch_targets {
        let sources = compile_watch_pass(&[src_dir.clone()], out_dir, &root, &config)?;
        for source_path in &sources {
            update_deps_for_source(
                source_path, &root, &mut dep_to_sources,
                &mut watched_deps, &mut watcher,
            );
        }
    }
    if !watched_deps.is_empty() {
        eprintln!("{} watching {} md:// dependency file{}",
            "→".cyan(), watched_deps.len(),
            if watched_deps.len() == 1 { "" } else { "s" });
    }

    // Build a lookup: source path prefix → output_dir
    let target_map: Vec<(PathBuf, Option<PathBuf>)> = watch_targets.clone();

    let debounce = Duration::from_millis(100);
    let mut pending_sources: HashSet<PathBuf> = HashSet::new();
    let mut last_event = Instant::now();

    loop {
        match rx.recv_timeout(Duration::from_millis(50)) {
            Ok(Ok(event)) => {
                if matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
                    for path in event.paths {
                        let is_source = path.to_str()
                            .map(|s| s.ends_with(".source.md"))
                            .unwrap_or(false);
                        if is_source {
                            pending_sources.insert(path);
                            last_event = Instant::now();
                        } else {
                            // Non-source file: check the reverse dep index
                            let key = std::fs::canonicalize(&path).unwrap_or(path);
                            if let Some(dependents) = dep_to_sources.get(&key) {
                                for dep_src in dependents {
                                    pending_sources.insert(dep_src.clone());
                                }
                                last_event = Instant::now();
                            }
                        }
                    }
                }
            }
            Ok(Err(e)) => eprintln!("{} watcher error: {}", "warn:".yellow(), e),
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }

        if !pending_sources.is_empty() && last_event.elapsed() >= debounce {
            let changed: Vec<PathBuf> = pending_sources.drain().collect();
            for source_path in &changed {
                // Find the matching target's output_dir
                let out_dir = target_map.iter()
                    .find(|(src, _)| source_path.starts_with(src))
                    .and_then(|(_, out)| out.clone());
                compile_one_watch(source_path, &out_dir, &root, &config);
                update_deps_for_source(
                    source_path, &root, &mut dep_to_sources,
                    &mut watched_deps, &mut watcher,
                );
            }
        }
    }

    Ok(())
}

/// Re-scan the source file for md:// URIs, resolve each to a filesystem path,
/// refresh `dep_to_sources` for this source, and add newly discovered deps to
/// the watcher (keeping `watched_deps` as the dedupe set). Existing dep
/// entries that no longer apply to this source are pruned.
fn update_deps_for_source<W: notify::Watcher>(
    source_path: &Path,
    root: &Path,
    dep_to_sources: &mut std::collections::HashMap<PathBuf, std::collections::HashSet<PathBuf>>,
    watched_deps: &mut std::collections::HashSet<PathBuf>,
    watcher: &mut W,
) {
    use notify::RecursiveMode;
    let canonical_source = std::fs::canonicalize(source_path)
        .unwrap_or_else(|_| source_path.to_path_buf());

    let new_deps = scan_md_uri_deps(source_path, root);

    // Prune stale entries: anything in dep_to_sources that pointed to this
    // source but isn't in the fresh set anymore.
    let stale: Vec<PathBuf> = dep_to_sources.iter()
        .filter(|(dep, srcs)| srcs.contains(&canonical_source) && !new_deps.contains(*dep))
        .map(|(dep, _)| dep.clone())
        .collect();
    for dep in stale {
        if let Some(srcs) = dep_to_sources.get_mut(&dep) {
            srcs.remove(&canonical_source);
            if srcs.is_empty() {
                dep_to_sources.remove(&dep);
            }
        }
    }

    // Insert / update for current deps and watch each one.
    for dep in &new_deps {
        dep_to_sources.entry(dep.clone())
            .or_insert_with(std::collections::HashSet::new)
            .insert(canonical_source.clone());

        if !watched_deps.contains(dep) && dep.exists() {
            // Watch the file's parent directory non-recursively so we get
            // notified about edits without explicit recursive coverage. (notify
            // on Windows is happier watching directories than individual files
            // for cross-editor compatibility — many editors atomic-rename.)
            let watch_target = dep.parent().unwrap_or(dep);
            match watcher.watch(watch_target, RecursiveMode::NonRecursive) {
                Ok(_) => { watched_deps.insert(dep.clone()); }
                Err(_) => {
                    // Already watched (parent matches an existing recursive
                    // watch on a source dir), or transient permission issue —
                    // record as watched anyway so we don't retry on every
                    // recompile.
                    watched_deps.insert(dep.clone());
                }
            }
        }
    }
}

/// Scan a `.source.md` file for `md://` URIs and resolve each one to its
/// filesystem path via mdpath. Failed resolutions are silently skipped — the
/// compiler will surface the error on the next compile pass with proper
/// diagnostics; for the watcher we just want the paths we CAN resolve.
fn scan_md_uri_deps(source_path: &Path, root: &Path) -> std::collections::HashSet<PathBuf> {
    use std::collections::HashSet;
    let mut deps: HashSet<PathBuf> = HashSet::new();
    let content = match std::fs::read_to_string(source_path) {
        Ok(c) => c,
        Err(_) => return deps,
    };

    // Find every md:// literal in the source. Each URI runs from `md://` up to
    // (but not including) the first whitespace, quote, backtick, or `>` —
    // robust enough for all directive arg styles (`source=md://...`,
    // bare-line bodies, `[[davinci]] uri = "md://..."`, etc.).
    let mut idx = 0;
    while let Some(pos) = content[idx..].find("md://") {
        let start = idx + pos;
        let rest = &content[start..];
        let end_off = rest.find(|c: char| {
            c.is_whitespace() || c == '"' || c == '`' || c == '>' || c == '<'
        }).unwrap_or(rest.len());
        let uri = &rest[..end_off];
        idx = start + end_off.max(1);

        let parsed = match mdpath::parse(uri) {
            Ok(p) => p,
            Err(_) => continue,
        };
        if let Ok(element) = mdpath::resolve(&parsed, root) {
            let canonical = std::fs::canonicalize(&element.file)
                .unwrap_or(element.file);
            // Don't add the source file itself — that's covered by
            // `.source.md` event handling and would cause feedback loops.
            if canonical != source_path {
                deps.insert(canonical);
            }
        }
    }
    deps
}

fn compile_watch_pass(
    watch_paths: &[PathBuf],
    output_dir: &Option<PathBuf>,
    root: &std::path::Path,
    config: &proof_lib::GlintConfig,
) -> Result<Vec<PathBuf>> {
    let mut sources: Vec<PathBuf> = Vec::new();
    for watch_path in watch_paths {
        for entry in walkdir::WalkDir::new(watch_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let p = entry.path().to_path_buf();
            if p.to_str().map(|s| s.ends_with(".source.md")).unwrap_or(false) {
                compile_one_watch(&p, output_dir, root, config);
                sources.push(p);
            }
        }
    }
    eprintln!("{} initial compile: {} files", "→".cyan(), sources.len());
    Ok(sources)
}

fn compile_one_watch(
    source_path: &PathBuf,
    output_dir: &Option<PathBuf>,
    root: &std::path::Path,
    config: &proof_lib::GlintConfig,
) {
    let output_path = if let Some(dir) = output_dir {
        if let Some(derived) = derive_output_path(source_path) {
            let filename = derived.file_name().expect("has filename");
            dir.join(filename)
        } else {
            return;
        }
    } else if let Some(p) = derive_output_path(source_path) {
        p
    } else {
        return;
    };

    let ts = chrono_or_time();
    match compile_file(source_path, &output_path, root, config) {
        Ok(result) => {
            let errors: Vec<_> = result.violations.iter()
                .filter(|v| v.severity == ViolationSeverity::Error)
                .collect();
            if errors.is_empty() {
                eprintln!("{} {} {} → {}  {}",
                    ts.dimmed(),
                    "✓".green(),
                    source_path.file_name().unwrap_or_default().to_string_lossy().cyan(),
                    output_path.file_name().unwrap_or_default().to_string_lossy(),
                    format!("({} directives)", result.directives_resolved).dimmed(),
                );
            } else {
                // File was NOT written — make this very visible
                eprintln!("{} {} {} — {} error{} (output NOT updated)",
                    ts.dimmed(),
                    "✗".red().bold(),
                    source_path.file_name().unwrap_or_default().to_string_lossy().red().bold(),
                    errors.len(),
                    if errors.len() == 1 { "" } else { "s" },
                );
                for e in &errors {
                    eprintln!("  {}:{} {} [{}]: {}",
                        source_path.display().to_string().dimmed(),
                        e.source_line,
                        "error".red(),
                        e.code,
                        e.message,
                    );
                    if !e.uri.is_empty() {
                        eprintln!("    uri: {}", e.uri.dimmed());
                    }
                }
                eprintln!("  {} fix the errors above, then save to recompile", "→".yellow());
            }
        }
        Err(e) => {
            eprintln!("{} {} {} — compile failed: {}",
                ts.dimmed(),
                "✗".red().bold(),
                source_path.file_name().unwrap_or_default().to_string_lossy().red().bold(),
                e,
            );
            eprintln!("  {} output NOT updated", "→".yellow());
        }
    }
}

fn chrono_or_time() -> String {
    // Simple HH:MM:SS timestamp without pulling in chrono
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let h = (secs / 3600) % 24;
    let m = (secs / 60) % 60;
    let s = secs % 60;
    format!("{:02}:{:02}:{:02}", h, m, s)
}

// ─────────────────────────────────────────────────────────
// layout
// ─────────────────────────────────────────────────────────

fn cmd_layout(
    sources: Vec<String>,
    gap: usize,
    align_str: String,
    labels: Vec<String>,
    cols: Option<usize>,
    width: usize,
    direction_str: String,
    border: bool,
    output: Option<PathBuf>,
    root: Option<PathBuf>,
) -> Result<()> {
    if sources.is_empty() {
        eprintln!("{} no sources provided — pass md:// URIs or file paths", "error:".red());
        process::exit(2);
    }

    let align = Align::parse(&align_str)?;
    let direction = Direction::parse(&direction_str)?;
    let root = root.unwrap_or_else(|| std::env::current_dir().unwrap());

    let config = LayoutConfig { gap, align, labels, cols, width, direction, border };

    // Resolve each source to content lines
    let mut figures: Vec<Vec<String>> = Vec::new();
    for source in &sources {
        let content = if source.starts_with("md://") {
            // Resolve via mdpath
            let parsed = mdpath::parse(source)
                .map_err(|e| anyhow::anyhow!("invalid md:// URI {:?}: {}", source, e))?;
            let element = mdpath::resolve(&parsed, &root)
                .map_err(|e| anyhow::anyhow!("cannot resolve {:?}: {}", source, e))?;
            element.content
        } else {
            // File path — read and use whole file content
            let path = root.join(source);
            std::fs::read_to_string(&path)
                .with_context(|| format!("reading figure file: {}", path.display()))?
        };
        figures.push(extract_content_lines(&content));
    }

    let result = layout::layout(figures, &config);

    match output {
        Some(path) => {
            std::fs::write(&path, &result)?;
            eprintln!("{} layout written to {}", "✓".green(), path.display());
        }
        None => println!("{}", result),
    }

    Ok(())
}

// ─────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────

fn cmd_draft(paths: Vec<PathBuf>, output: PathBuf, config_override: &Option<PathBuf>) -> Result<()> {
    // Collect all diagnostics (same as check)
    let mut all_diags: Vec<Diagnostic> = Vec::new();
    let root = paths.first().map(|p| if p.is_dir() { p.clone() } else { p.parent().unwrap_or(p).to_path_buf() })
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    for path in &paths {
        let cfg = load_config(path, config_override);
        let dir = if path.is_dir() { path.clone() } else { path.parent().unwrap_or(path).to_path_buf() };
        let runner = Runner::new(&dir, cfg)?;
        if path.is_file() {
            all_diags.extend(runner.lint_file(path));
        } else {
            all_diags.extend(runner.run());
        }
    }

    let error_count = all_diags.iter().filter(|d| d.severity == Severity::Error).count();
    let warn_count = all_diags.iter().filter(|d| d.severity == Severity::Warning).count();

    // Build the draft plan
    let draft = build_draft_plan(&all_diags, &root)?;

    let json = serde_json::to_string_pretty(&draft)?;
    std::fs::write(&output, &json)?;

    eprintln!(
        "{} — {} errors, {} warnings across {} groups ({} auto-fixable, {} need review)",
        "draft".cyan().bold(),
        error_count, warn_count,
        draft.summary.total_groups,
        draft.summary.auto_fixable,
        draft.summary.needs_review,
    );
    eprintln!("Draft plan written to {}", output.display().to_string().cyan());
    eprintln!();
    eprintln!("Next steps:");
    eprintln!("  1. Open {} — AI fills in `decision` and `new_string` for non-auto groups", output.display());
    eprintln!("  2. proof fix --plan {} --dry-run", output.display());
    eprintln!("  3. proof fix --plan {}", output.display());

    Ok(())
}

/// Load a plan file — accepts both FixPlan and DraftPlan formats.
/// DraftPlan is automatically converted to FixPlan via to_fix_plan().
fn load_plan(path: &std::path::Path) -> anyhow::Result<FixPlan> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("reading plan file: {}", path.display()))?;

    // Try FixPlan first (has "schema_version" + "fixes" array)
    if let Ok(plan) = serde_json::from_str::<FixPlan>(&content) {
        return Ok(plan);
    }

    // Try DraftPlan (has "schema_version" + "groups" array)
    if let Ok(draft) = serde_json::from_str::<proof_lib::draft::DraftPlan>(&content) {
        eprintln!("{} converting draft plan to fix plan (auto+annotated groups only)",
            "info:".cyan());
        return Ok(draft.to_fix_plan());
    }

    anyhow::bail!("cannot parse {} as FixPlan or DraftPlan", path.display())
}

fn load_config(path: &std::path::Path, override_path: &Option<PathBuf>) -> GlintConfig {
    if let Some(ref cfg_path) = override_path {
        match GlintConfig::load(cfg_path) {
            Ok(cfg) => return cfg,
            Err(e) => eprintln!("proof: warning: {}", e),
        }
    }
    let dir = if path.is_dir() { path.to_path_buf() } else { path.parent().unwrap_or(path).to_path_buf() };
    GlintConfig::load_or_default(&dir)
}

fn cmd_init() -> Result<()> {
    let path = std::path::Path::new("proof.toml");
    if path.exists() {
        eprintln!("{} proof.toml already exists", "warning:".yellow());
        return Ok(());
    }
    let content = include_str!("../schemas/default.toml");
    std::fs::write(path, content)?;
    println!("{} proof.toml created", "OK".green().bold());
    Ok(())
}

fn count_files(dir: &std::path::Path) -> usize {
    walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().is_file()
                && e.path().extension().map(|x| x == "md").unwrap_or(false)
        })
        .count()
}
