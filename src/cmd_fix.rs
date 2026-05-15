use crate::cmd_context::GlobalOptions;
use anyhow::{Context, Result};
use colored::Colorize;
use proof_lib::fix::FixOptions;
use proof_lib::lint::lint_paths;
use proof_lib::{Confidence, FixPlan, Severity};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::process;

#[derive(clap::Args)]
pub(crate) struct Args {
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
}

#[derive(Debug, Serialize)]
struct FixApplicationLog {
    schema_version: String,
    generated_by: String,
    plan_path: PathBuf,
    dry_run: bool,
    min_confidence: String,
    applied: usize,
    skipped: usize,
    files_modified: usize,
    modified_files: Vec<PathBuf>,
    verification: FixVerificationLog,
}

#[derive(Debug, Serialize)]
struct FixVerificationLog {
    status: String,
    errors: usize,
    warnings: usize,
    config: Option<PathBuf>,
    paths: Vec<PathBuf>,
}

pub(crate) fn run_with_globals(args: Args, globals: &GlobalOptions) -> Result<()> {
    let Args {
        plan: plan_path,
        dry_run,
        min_confidence: min_confidence_str,
        no_verify,
        no_signal_check,
    } = args;
    let min_confidence = match min_confidence_str.as_str() {
        "high" => Confidence::High,
        "medium" => Confidence::Medium,
        "low" => Confidence::Low,
        other => {
            eprintln!(
                "proof: unknown confidence level {:?} — use high, medium, or low",
                other
            );
            process::exit(2);
        }
    };

    // Accept both FixPlan and DraftPlan (draft -> fix via to_fix_plan()).
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

    let opts = FixOptions {
        dry_run,
        min_confidence: min_confidence.clone(),
        check_signal: !no_signal_check,
    };
    let result = plan.apply(&opts, &root)?;
    let mut verification = FixVerificationLog {
        status: "skipped".to_string(),
        errors: 0,
        warnings: 0,
        config: globals.config().clone(),
        paths: Vec::new(),
    };

    eprintln!();
    for skip in &result.skipped {
        eprintln!("{} [{}] {}", "SKIP".yellow(), skip.id, skip.reason);
    }

    eprintln!();
    if dry_run {
        eprintln!(
            "{} — {} fixes previewed, {} skipped (no files written)",
            "DRY RUN".cyan().bold(),
            result.applied.len(),
            result.skipped.len()
        );
    } else {
        eprintln!(
            "{} — {} fixes applied to {} files, {} skipped",
            "DONE".green().bold(),
            result.applied.len(),
            result.files_modified,
            result.skipped.len()
        );

        // Re-run check unless suppressed.
        if !no_verify && result.files_modified > 0 {
            eprintln!("\n{} verifying fixes…", "→".cyan());
            let verify_paths = result.modified_files.clone();
            let verify = lint_paths(&verify_paths, globals.config())?;
            let errors = verify
                .diagnostics
                .iter()
                .filter(|d| d.severity == Severity::Error)
                .count();
            let warnings = verify
                .diagnostics
                .iter()
                .filter(|d| d.severity == Severity::Warning)
                .count();
            verification = FixVerificationLog {
                status: if errors == 0 { "passed" } else { "failed" }.to_string(),
                errors,
                warnings,
                config: globals.config().clone(),
                paths: verify_paths,
            };
            if errors == 0 {
                eprintln!("{} zero errors remaining", "✓".green());
            } else {
                eprintln!(
                    "{} {} error{} remain after fix — review manually",
                    "!".yellow(),
                    errors,
                    if errors == 1 { "" } else { "s" }
                );
                write_last_fix_log(
                    &root,
                    FixApplicationLog::from_result(
                        &plan_path,
                        dry_run,
                        &min_confidence,
                        &result,
                        verification,
                    ),
                )?;
                process::exit(1);
            }
        }
    }

    write_last_fix_log(
        &root,
        FixApplicationLog::from_result(&plan_path, dry_run, &min_confidence, &result, verification),
    )?;

    Ok(())
}

impl FixApplicationLog {
    fn from_result(
        plan_path: &Path,
        dry_run: bool,
        min_confidence: &Confidence,
        result: &proof_lib::FixResult,
        verification: FixVerificationLog,
    ) -> Self {
        Self {
            schema_version: "1".to_string(),
            generated_by: "proof fix".to_string(),
            plan_path: plan_path.to_path_buf(),
            dry_run,
            min_confidence: min_confidence.to_string(),
            applied: result.applied.len(),
            skipped: result.skipped.len(),
            files_modified: result.files_modified,
            modified_files: result.modified_files.clone(),
            verification,
        }
    }
}

fn write_last_fix_log(root: &Path, log: FixApplicationLog) -> Result<()> {
    let dir = root.join(".proof");
    std::fs::create_dir_all(&dir).with_context(|| format!("creating {}", dir.display()))?;
    let path = dir.join("last-fix.json");
    let json = serde_json::to_string_pretty(&log)?;
    std::fs::write(&path, json).with_context(|| format!("writing {}", path.display()))?;
    Ok(())
}

/// Load a plan file — accepts both FixPlan and DraftPlan formats.
/// DraftPlan is automatically converted to FixPlan via to_fix_plan().
fn load_plan(path: &Path) -> Result<FixPlan> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("reading plan file: {}", path.display()))?;

    // Try FixPlan first (has "schema_version" + "fixes" array).
    if let Ok(plan) = serde_json::from_str::<FixPlan>(&content) {
        return Ok(plan);
    }

    // Try DraftPlan (has "schema_version" + "groups" array).
    if let Ok(draft) = serde_json::from_str::<proof_lib::draft::DraftPlan>(&content) {
        eprintln!(
            "{} converting draft plan to fix plan (auto+annotated groups only)",
            "info:".cyan()
        );
        return Ok(draft.to_fix_plan());
    }

    anyhow::bail!("cannot parse {} as FixPlan or DraftPlan", path.display())
}
