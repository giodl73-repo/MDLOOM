use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use glint_lib::{Diagnostic, GlintConfig, Runner, Severity};
use std::path::PathBuf;
use std::process;

#[derive(Parser)]
#[command(
    name = "glint",
    version,
    about = "A fast, schema-driven markdown and ASCII art linter",
    long_about = "glint validates markdown files against a schema — \
                  checking ASCII art box alignment, required section structure, \
                  and custom content rules."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    /// Files or directories to lint (default: current directory)
    #[arg(global = true)]
    paths: Vec<PathBuf>,

    /// Schema config file (default: glint.toml in target directory)
    #[arg(short, long, global = true)]
    config: Option<PathBuf>,

    /// Output format: text (default), json, github
    #[arg(short = 'f', long, default_value = "text", global = true)]
    format: String,

    /// Show only errors (suppress warnings)
    #[arg(short = 'e', long, global = true)]
    errors_only: bool,

    /// Exit with code 0 even if issues found (useful for CI reporting)
    #[arg(long, global = true)]
    no_fail: bool,
}

#[derive(Subcommand)]
enum Command {
    /// Lint files (default when no subcommand given)
    Check {
        /// Files or directories to lint
        paths: Vec<PathBuf>,
    },
    /// Print the effective config and exit
    Config,
    /// Initialize a glint.toml schema in the current directory
    Init,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let paths: Vec<PathBuf> = if cli.paths.is_empty() {
        vec![std::env::current_dir()?]
    } else {
        cli.paths.clone()
    };

    match &cli.command {
        Some(Command::Config) => {
            println!("(config display not yet implemented — check glint.toml directly)");
            return Ok(());
        }
        Some(Command::Init) => {
            return write_init_config();
        }
        _ => {}
    }

    let mut all_diags: Vec<Diagnostic> = Vec::new();
    let mut files_checked = 0usize;

    for path in &paths {
        let cfg = if let Some(ref cfg_path) = cli.config {
            GlintConfig::load(cfg_path)?
        } else {
            let dir = if path.is_dir() {
                path.clone()
            } else {
                path.parent().unwrap_or(path).to_path_buf()
            };
            GlintConfig::load_or_default(&dir)
        };

        let dir = if path.is_dir() { path.clone() } else { path.parent().unwrap_or(path).to_path_buf() };
        let runner = Runner::new(&dir, cfg)?;

        let diags = if path.is_file() {
            files_checked += 1;
            runner.lint_file(path)
        } else {
            let collected = count_files(path);
            files_checked += collected;
            runner.run()
        };

        all_diags.extend(diags);
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

    match cli.format.as_str() {
        "json" => print_json(&all_diags),
        "github" => print_github(&all_diags),
        _ => print_text(&all_diags),
    }

    if !all_diags.is_empty() {
        eprintln!();
    }

    let summary_status = if error_count > 0 {
        "FAIL".red().bold()
    } else {
        "OK".green().bold()
    };
    eprintln!(
        "{} — {} files checked, {} error{}, {} warning{}",
        summary_status,
        files_checked,
        error_count,
        if error_count == 1 { "" } else { "s" },
        warn_count,
        if warn_count == 1 { "" } else { "s" },
    );

    if !cli.no_fail && error_count > 0 {
        process::exit(1);
    }

    Ok(())
}

fn print_text(diags: &[Diagnostic]) {
    for d in diags {
        let severity_label = match d.severity {
            Severity::Error => "error".red().bold(),
            Severity::Warning => "warning".yellow().bold(),
            Severity::Info => "info".blue(),
        };
        let code = format!("[{}]", d.code).dimmed();
        println!(
            "{}:{}: {} {}: {}",
            d.file.display().to_string().cyan(),
            d.span.to_string().white(),
            severity_label,
            code,
            d.message
        );
        if let Some(ref note) = d.note {
            println!("  {} {}", "note:".dimmed(), note);
        }
    }
}

fn print_github(diags: &[Diagnostic]) {
    for d in diags {
        let level = match d.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Info => "notice",
        };
        println!(
            "::{} file={},line={},col={}::[{}] {}",
            level,
            d.file.display(),
            d.span.line,
            d.span.col,
            d.code,
            d.message
        );
    }
}

fn print_json(diags: &[Diagnostic]) {
    println!("[");
    for (i, d) in diags.iter().enumerate() {
        let comma = if i < diags.len() - 1 { "," } else { "" };
        println!(
            r#"  {{"file":"{file}","line":{line},"col":{col},"severity":"{sev}","code":"{code}","message":"{msg}"}}{comma}"#,
            file = d.file.display().to_string().replace('\\', "/"),
            line = d.span.line,
            col = d.span.col,
            sev = d.severity,
            code = d.code,
            msg = d.message.replace('"', "\\\""),
            comma = comma,
        );
    }
    println!("]");
}

fn write_init_config() -> Result<()> {
    let path = std::path::Path::new("glint.toml");
    if path.exists() {
        eprintln!("{} glint.toml already exists — not overwriting", "warning:".yellow());
        return Ok(());
    }
    let content = include_str!("../schemas/default.toml");
    std::fs::write(path, content)?;
    println!("{} glint.toml created — edit to customize your schema", "OK".green().bold());
    Ok(())
}

fn count_files(dir: &std::path::Path) -> usize {
    walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .count()
}
