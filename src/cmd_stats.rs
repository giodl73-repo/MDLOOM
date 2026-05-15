use crate::cmd_context::GlobalOptions;
use crate::cmd_paths::paths_or_cwd;
use anyhow::Result;
use proof_lib::frontmatter::{FrontmatterFilter, FrontmatterTagCounts};
use proof_lib::lint::lint_paths;
use proof_lib::Severity;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

#[derive(clap::Args)]
pub(crate) struct Args {
    paths: Vec<PathBuf>,
    /// Break down by directory
    #[arg(long)]
    by_directory: bool,
    /// Break down by error code
    #[arg(long)]
    by_code: bool,
    /// Break down source frontmatter tags, ops, and content tags
    #[arg(long)]
    by_tag: bool,
    /// Only include source files with this frontmatter tag (repeatable)
    #[arg(long = "tag")]
    tags: Vec<String>,
    /// Only include source files with this operation tag (repeatable)
    #[arg(long = "op")]
    ops: Vec<String>,
    /// Only include source files with this content tag (repeatable)
    #[arg(long = "content-tag")]
    content_tags: Vec<String>,
}

pub(crate) fn run_with_globals(args: Args, globals: &GlobalOptions) -> Result<()> {
    run(args, globals.config())
}

fn run(args: Args, config_override: &Option<PathBuf>) -> Result<()> {
    let Args {
        paths,
        by_directory,
        by_code,
        by_tag,
        tags,
        ops,
        content_tags,
    } = args;
    let paths = paths_or_cwd(paths)?;
    let lint_summary = lint_paths(&paths, config_override)?;
    let tag_filter = FrontmatterFilter {
        tags,
        ops,
        content: content_tags,
    };
    let selected_files = if tag_filter.is_empty() {
        lint_summary.files.clone()
    } else {
        lint_summary
            .files
            .iter()
            .filter(|path| tag_filter.matches_path(path))
            .cloned()
            .collect()
    };
    let selected_file_set: BTreeSet<PathBuf> = selected_files.iter().cloned().collect();
    let all_diags = if tag_filter.is_empty() {
        lint_summary.diagnostics
    } else {
        lint_summary
            .diagnostics
            .into_iter()
            .filter(|diag| selected_file_set.contains(&diag.file))
            .collect()
    };

    let errors = all_diags
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .count();
    let warnings = all_diags
        .iter()
        .filter(|d| d.severity == Severity::Warning)
        .count();

    println!("files:    {}", selected_files.len());
    println!("errors:   {}", errors);
    println!("warnings: {}", warnings);

    if by_code {
        println!("\nBy error code:");
        let mut by_code_map: BTreeMap<&str, usize> = BTreeMap::new();
        for d in &all_diags {
            *by_code_map.entry(d.code).or_default() += 1;
        }
        for (code, count) in &by_code_map {
            println!("  {:30} {}", code, count);
        }
    }

    if by_directory {
        println!("\nBy directory:");
        let mut by_dir: BTreeMap<String, usize> = BTreeMap::new();
        for d in &all_diags {
            let dir = d
                .file
                .parent()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| ".".to_string());
            *by_dir.entry(dir).or_default() += 1;
        }
        for (dir, count) in &by_dir {
            println!("  {:50} {}", dir, count);
        }
    }

    if by_tag {
        let counts = FrontmatterTagCounts::from_files(&selected_files);
        println!("\nBy tag:");
        print_counts(&counts.tags);
        println!("\nBy op:");
        print_counts(&counts.ops);
        println!("\nBy content tag:");
        print_counts(&counts.content);
        println!(
            "\nfrontmatter files: {} ({} with tags)",
            counts.files_with_frontmatter, counts.files_with_tags
        );
    }

    Ok(())
}

fn print_counts(counts: &BTreeMap<String, usize>) {
    if counts.is_empty() {
        println!("  (none)");
        return;
    }

    for (tag, count) in counts {
        println!("  {:30} {}", tag, count);
    }
}
