use anyhow::Result;
use colored::Colorize;
use proof_lib::backfill::{self, BackfillOptions};
use std::path::PathBuf;

#[derive(clap::Args)]
pub(crate) struct Args {
    /// Markdown files or directories to backfill
    paths: Vec<PathBuf>,
    /// Write generated .source.md files under DIR
    #[arg(long, default_value = "proof-source")]
    output_source: PathBuf,
    /// Write extraction and round-trip report
    #[arg(long, default_value = "backfill-report.json")]
    report: PathBuf,
    /// Prefer exact source mirroring over semantic extraction
    #[arg(long)]
    literal_first: bool,
    /// Extract high-confidence markdown tables into sidecar data files
    #[arg(long)]
    extract_tables: bool,
    /// Compile generated sources and compare to originals
    #[arg(long)]
    check_roundtrip: bool,
}

pub(crate) fn run(args: Args) -> Result<()> {
    let paths = if args.paths.is_empty() {
        vec![std::env::current_dir()?]
    } else {
        args.paths
    };

    let report = backfill::run(BackfillOptions {
        paths,
        output_source: args.output_source,
        report: args.report.clone(),
        literal_first: args.literal_first,
        extract_tables: args.extract_tables,
        check_roundtrip: args.check_roundtrip,
    })?;

    eprintln!(
        "{} generated {} source file{} from {} markdown file{}",
        "backfill".cyan().bold(),
        report.summary.files_generated,
        if report.summary.files_generated == 1 {
            ""
        } else {
            "s"
        },
        report.summary.files_scanned,
        if report.summary.files_scanned == 1 {
            ""
        } else {
            "s"
        },
    );
    if report.summary.roundtrip_checked > 0 {
        eprintln!(
            "  round-trip: {} passed, {} failed",
            report.summary.roundtrip_passed, report.summary.roundtrip_failed
        );
    }
    eprintln!("  report: {}", args.report.display());

    Ok(())
}
