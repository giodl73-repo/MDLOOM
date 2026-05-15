use crate::cmd_context::GlobalOptions;
use crate::cmd_paths::paths_or_cwd;
use anyhow::Result;
use colored::Colorize;
use proof_lib::draft::build_draft_plan;
use proof_lib::lint::lint_paths;
use proof_lib::Severity;
use std::path::PathBuf;

#[derive(clap::Args)]
pub(crate) struct Args {
    paths: Vec<PathBuf>,
    /// Output file for the draft plan (default: draft-plan.json)
    #[arg(short = 'o', long, default_value = "draft-plan.json")]
    output: PathBuf,
}

pub(crate) fn run_with_globals(args: Args, globals: &GlobalOptions) -> Result<()> {
    run(args, globals.config())
}

fn run(args: Args, config_override: &Option<PathBuf>) -> Result<()> {
    let paths = paths_or_cwd(args.paths)?;
    let output = args.output;
    let root = paths
        .first()
        .map(|p| {
            if p.is_dir() {
                p.clone()
            } else {
                p.parent().unwrap_or(p).to_path_buf()
            }
        })
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    let lint_summary = lint_paths(&paths, config_override)?;
    let all_diags = lint_summary.diagnostics;

    let error_count = all_diags
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .count();
    let warn_count = all_diags
        .iter()
        .filter(|d| d.severity == Severity::Warning)
        .count();

    let draft = build_draft_plan(&all_diags, &root)?;

    let json = serde_json::to_string_pretty(&draft)?;
    std::fs::write(&output, &json)?;

    eprintln!(
        "{} — {} errors, {} warnings across {} groups ({} auto-fixable, {} need review)",
        "draft".cyan().bold(),
        error_count,
        warn_count,
        draft.summary.total_groups,
        draft.summary.auto_fixable,
        draft.summary.needs_review,
    );
    eprintln!(
        "Draft plan written to {}",
        output.display().to_string().cyan()
    );
    eprintln!();
    eprintln!("Next steps:");
    eprintln!(
        "  1. Open {} — AI fills in `decision` and `new_string` for non-auto groups",
        output.display()
    );
    eprintln!("  2. proof fix --plan {} --dry-run", output.display());
    eprintln!("  3. proof fix --plan {}", output.display());

    Ok(())
}
