use crate::cmd_context::GlobalOptions;
use anyhow::Result;
use colored::Colorize;
use proof_lib::lint::load_config_for_path;
use std::path::PathBuf;

pub(crate) fn run_with_globals(globals: &GlobalOptions) -> Result<()> {
    run(globals.config())
}

fn run(config_override: &Option<PathBuf>) -> Result<()> {
    let root = std::env::current_dir()?;
    let cfg = load_config_for_path(&root, config_override)?;
    if cfg.davinci.is_empty() {
        println!(
            "No DaVinci entries registered. Use `proof pin md://... --id name` to pin a figure."
        );
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
