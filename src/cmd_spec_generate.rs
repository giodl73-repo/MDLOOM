use crate::cmd_context::GlobalOptions;
use anyhow::{Context, Result};
use colored::Colorize;
use proof_lib::lint::load_config_for_path as load_config;
use proof_lib::spec_gen;
use std::path::PathBuf;

#[derive(clap::Args)]
pub(crate) struct Args {
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
    /// Use the configured AI CLI (proof.toml [ai]) to generate richer invariants.
    /// Falls back to static analysis when not set. Configure in proof.toml:
    ///   [ai]
    ///   command = "claude"
    ///   args    = ["-p", "{prompt}"]
    #[arg(long)]
    ai: bool,
}

pub(crate) fn run_with_globals(args: Args, globals: &GlobalOptions) -> Result<()> {
    run(args, globals.config())
}

fn run(args: Args, config_override: &Option<PathBuf>) -> Result<()> {
    let Args {
        uri,
        id: id_override,
        protection,
        root: root_override,
        output,
        ai: use_ai,
    } = args;
    let config = load_config(
        &std::env::current_dir().unwrap_or_default(),
        config_override,
    )?;
    let root = root_override.unwrap_or_else(|| std::env::current_dir().unwrap());

    // Resolve the URI
    let parsed =
        mdpath::parse(&uri).map_err(|e| anyhow::anyhow!("invalid md:// URI {:?}: {}", uri, e))?;
    let element = mdpath::resolve(&parsed, &root)
        .map_err(|e| anyhow::anyhow!("cannot resolve {:?}: {}", uri, e))?;

    // Derive ID from URI if not provided
    let id = id_override.unwrap_or_else(|| {
        // Use the last path segment minus extension, falling back to "figure"
        parsed
            .path
            .split('/')
            .last()
            .unwrap_or("figure")
            .trim_end_matches(".md")
            .replace(['-', '_'], "-")
            .to_string()
    });

    eprintln!(
        "{} Analyzing {} ({} lines)...",
        "→".cyan(),
        uri.dimmed(),
        element.content.lines().count(),
    );

    // AI path: call configured CLI and stream response directly
    if use_ai {
        let ai_cfg = &config.ai;
        eprintln!(
            "{} Calling {:?} for AI-assisted invariant suggestions...",
            "→".cyan(),
            ai_cfg.command
        );
        let prompt = proof_lib::ai::spec_generate_prompt(&uri, &element.content);
        let response = proof_lib::ai::call_ai(&prompt, ai_cfg)
            .with_context(|| format!(
                "AI CLI {:?} failed. Check [ai] in proof.toml or run without --ai for static analysis.",
                ai_cfg.command
            ))?;
        eprintln!(
            "{} AI response received — paste the block below into proof.toml",
            "✓".green()
        );
        eprintln!();
        match output {
            Some(path) => {
                std::fs::write(&path, &response)?;
                eprintln!("{} written to {}", "✓".green(), path.display());
            }
            None => print!("{}", response),
        }
        return Ok(());
    }

    let spec = spec_gen::generate(&element.content, element.label.as_deref(), &uri, &id);

    // Override protection from CLI
    let mut spec = spec;
    spec.protection = protection;

    let toml_out = spec_gen::format_toml(&spec);

    // Print summary to stderr, TOML to stdout (or file)
    eprintln!(
        "{} {} invariant{} suggested for {:?}",
        "✓".green(),
        spec.invariants.len(),
        if spec.invariants.len() == 1 { "" } else { "s" },
        spec.id,
    );
    for inv in &spec.invariants {
        eprintln!(
            "  {} [{}] {}",
            match inv.confidence {
                spec_gen::SuggestionConfidence::High => "●".green().to_string(),
                spec_gen::SuggestionConfidence::Medium => "◐".yellow().to_string(),
                spec_gen::SuggestionConfidence::Low => "○".dimmed().to_string(),
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
