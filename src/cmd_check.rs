use crate::cmd_context::GlobalOptions;
use crate::cmd_paths::check_paths_or_cwd;
use anyhow::Result;
use colored::Colorize;
use proof_lib::davinci::check_daVinci;
use proof_lib::fix::{serialize_json, serialize_rich};
use proof_lib::frontmatter::FrontmatterFilter;
use proof_lib::lint::{lint_paths, load_config_for_path as load_config};
use proof_lib::{Diagnostic, Severity};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process;

#[derive(clap::Args)]
pub(crate) struct Args {
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
    /// Only check source files with this frontmatter tag (repeatable)
    #[arg(long = "tag")]
    tags: Vec<String>,
    /// Only check source files with this operation tag (repeatable)
    #[arg(long = "op")]
    ops: Vec<String>,
    /// Only check source files with this content tag (repeatable)
    #[arg(long = "content-tag")]
    content_tags: Vec<String>,
}

#[derive(Clone, Copy, Default)]
struct Flags {
    da_vinci: bool,
    show_by_code: bool,
    deduplicate: bool,
    detect_unused: bool,
}

impl Args {
    fn take_paths(&mut self) -> Vec<PathBuf> {
        std::mem::take(&mut self.paths)
    }

    fn flags(&self) -> Flags {
        Flags {
            da_vinci: self.da_vinci,
            show_by_code: self.by_code,
            deduplicate: self.deduplicate,
            detect_unused: self.unused,
        }
    }

    fn tag_filter(&self) -> FrontmatterFilter {
        FrontmatterFilter {
            tags: self.tags.clone(),
            ops: self.ops.clone(),
            content: self.content_tags.clone(),
        }
    }
}

struct Options<'a> {
    config_override: &'a Option<PathBuf>,
    format: &'a str,
    errors_only: bool,
    no_fail: bool,
    output: &'a Option<PathBuf>,
    flags: Flags,
    tag_filter: FrontmatterFilter,
}

impl<'a> Options<'a> {
    fn from_globals(
        flags: Flags,
        tag_filter: FrontmatterFilter,
        globals: &'a GlobalOptions,
    ) -> Self {
        Self {
            config_override: globals.config(),
            format: globals.format(),
            errors_only: globals.errors_only(),
            no_fail: globals.no_fail(),
            output: globals.output(),
            flags,
            tag_filter,
        }
    }
}

fn run_with_globals(
    paths: Vec<PathBuf>,
    flags: Flags,
    tag_filter: FrontmatterFilter,
    globals: &GlobalOptions,
) -> Result<()> {
    run(paths, Options::from_globals(flags, tag_filter, globals))
}

pub(crate) fn run_command(
    mut args: Args,
    top_level_paths: &[PathBuf],
    globals: &GlobalOptions,
) -> Result<()> {
    let flags = args.flags();
    let tag_filter = args.tag_filter();
    let paths = check_paths_or_cwd(args.take_paths(), top_level_paths)?;
    run_with_globals(paths, flags, tag_filter, globals)
}

pub(crate) fn run_default(top_level_paths: &[PathBuf], globals: &GlobalOptions) -> Result<()> {
    let paths = check_paths_or_cwd(Vec::new(), top_level_paths)?;
    run_with_globals(
        paths,
        Flags::default(),
        FrontmatterFilter::default(),
        globals,
    )
}

fn run(paths: Vec<PathBuf>, options: Options<'_>) -> Result<()> {
    let mut all_diags: Vec<Diagnostic> = Vec::new();
    let mut files_checked = 0usize;

    // DaVinci root = the directory containing proof.toml (the proof root).
    // Run once, not per-file, using the config's location as the URI root.
    if options.flags.da_vinci {
        let proof_root = options
            .config_override
            .as_deref()
            .and_then(Path::parent)
            .map(Path::to_path_buf)
            .unwrap_or_else(|| {
                paths
                    .first()
                    .map(|p| {
                        if p.is_dir() {
                            p.clone()
                        } else {
                            p.parent().unwrap_or(p).to_path_buf()
                        }
                    })
                    .unwrap_or_else(|| std::env::current_dir().unwrap())
            });
        let cfg = load_config(&proof_root, options.config_override)?;
        if !cfg.davinci.is_empty() {
            let dv_diags = check_daVinci(&cfg, &proof_root);
            if dv_diags.is_empty() {
                eprintln!(
                    "{} all {} DaVinci invariants satisfied",
                    "✓".green(),
                    cfg.davinci.len()
                );
            }
            all_diags.extend(dv_diags);
        }
    }

    let lint_summary = lint_paths(&paths, options.config_override)?;
    if options.tag_filter.is_empty() {
        files_checked += lint_summary.files_checked;
        all_diags.extend(lint_summary.diagnostics);
    } else {
        let selected_files: BTreeSet<PathBuf> = lint_summary
            .files
            .iter()
            .filter(|path| options.tag_filter.matches_path(path))
            .cloned()
            .collect();
        files_checked += selected_files.len();
        all_diags.extend(
            lint_summary
                .diagnostics
                .into_iter()
                .filter(|diag| selected_files.contains(&diag.file)),
        );
    }

    // Corpus-level scan for orphaned figures (--unused). Runs once across all
    // input paths so a figure under one directory can still be considered used
    // when referenced from a sibling.
    if options.flags.detect_unused {
        for path in &paths {
            let scan_root = if path.is_dir() {
                path.clone()
            } else {
                path.parent().unwrap_or(path).to_path_buf()
            };
            all_diags.extend(proof_lib::unused::unused_diagnostics(&scan_root));
        }
    }

    if options.errors_only {
        all_diags.retain(|d| d.severity == Severity::Error);
    }

    all_diags.sort_by(|a, b| {
        a.file
            .cmp(&b.file)
            .then(a.span.line.cmp(&b.span.line))
            .then(a.span.col.cmp(&b.span.col))
    });

    let error_count = all_diags
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .count();
    let warn_count = all_diags
        .iter()
        .filter(|d| d.severity == Severity::Warning)
        .count();

    let out = if options.flags.deduplicate && options.format == "text" {
        format_deduplicated(&all_diags)
    } else {
        format_output(&all_diags, options.format)?
    };

    if let Some(out_path) = options.output {
        std::fs::write(out_path, &out)?;
        eprintln!("Output written to {}", out_path.display());
    } else {
        print!("{}", out);
    }

    if !all_diags.is_empty() && options.format == "text" {
        eprintln!();
    }

    let status = if error_count > 0 {
        "FAIL".red().bold()
    } else {
        "OK".green().bold()
    };
    eprintln!(
        "{} — {} files checked, {} error{}, {} warning{}",
        status,
        files_checked,
        error_count,
        if error_count == 1 { "" } else { "s" },
        warn_count,
        if warn_count == 1 { "" } else { "s" },
    );

    if options.flags.show_by_code && !all_diags.is_empty() {
        use std::collections::BTreeMap;
        let mut by_code: BTreeMap<&str, (usize, usize)> = BTreeMap::new(); // (errors, warnings)
        for d in &all_diags {
            let entry = by_code.entry(d.code).or_default();
            if d.severity == Severity::Error {
                entry.0 += 1;
            } else {
                entry.1 += 1;
            }
        }
        eprintln!();
        for (code, (errs, warns)) in &by_code {
            let parts: Vec<String> = [
                if *errs > 0 {
                    format!("{} error{}", errs, if *errs == 1 { "" } else { "s" })
                } else {
                    String::new()
                },
                if *warns > 0 {
                    format!("{} warning{}", warns, if *warns == 1 { "" } else { "s" })
                } else {
                    String::new()
                },
            ]
            .iter()
            .filter(|s| !s.is_empty())
            .cloned()
            .collect();
            eprintln!("  {:<30} {}", code, parts.join(", "));
        }
    }

    // Write .proof/last-check.json so `proof status` can show cached results.
    write_last_check_cache(&paths, files_checked, error_count, warn_count);

    if !options.no_fail && error_count > 0 {
        process::exit(1);
    }
    Ok(())
}

fn write_last_check_cache(paths: &[PathBuf], files_checked: usize, errors: usize, warnings: usize) {
    let root = paths
        .first()
        .map(|p| {
            if p.is_dir() {
                p.clone()
            } else {
                p.parent().unwrap_or(p).to_path_buf()
            }
        })
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
    let cache_dir = root.join(".proof");
    if std::fs::create_dir_all(&cache_dir).is_ok() {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let json = format!(
            "{{\"files_checked\":{},\"errors\":{},\"warnings\":{},\"timestamp_secs\":{}}}",
            files_checked, errors, warnings, ts
        );
        let _ = std::fs::write(cache_dir.join("last-check.json"), json);
    }
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
                    level,
                    d.file.display(),
                    d.span.line,
                    d.span.col,
                    d.code,
                    d.message
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
        let parent = d
            .file
            .parent()
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
