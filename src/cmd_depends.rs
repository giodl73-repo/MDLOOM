use anyhow::Result;
use colored::Colorize;
use std::path::PathBuf;

#[derive(clap::Args)]
pub(crate) struct Args {
    /// The md:// URI to look up
    uri: String,
    /// Root directory to scan (default: current directory, or where proof.toml lives)
    #[arg(short, long)]
    root: Option<PathBuf>,
    /// Output format: text (default) | json
    #[arg(long, default_value = "text")]
    format: String,
}

pub(crate) fn run(args: Args) -> Result<()> {
    let Args { uri, root, format } = args;
    let scan_root = root
        .or_else(find_proof_root_for_cwd)
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    let deps = proof_lib::depends::find_dependents(&uri, &scan_root);

    match format.as_str() {
        "json" => {
            let arr: Vec<_> = deps
                .iter()
                .map(|d| {
                    serde_json::json!({
                        "file": d.source_file.display().to_string(),
                        "line": d.line,
                        "uri": d.uri,
                    })
                })
                .collect();
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "query": uri,
                    "root": scan_root.display().to_string(),
                    "count": deps.len(),
                    "references": arr,
                }))?
            );
        }
        _ => {
            if deps.is_empty() {
                println!(
                    "No references to {} found under {}",
                    uri.cyan(),
                    scan_root.display()
                );
                return Ok(());
            }
            println!(
                "{} reference{} to {}:",
                deps.len(),
                if deps.len() == 1 { "" } else { "s" },
                uri.cyan().bold()
            );
            for d in &deps {
                let rel = d
                    .source_file
                    .strip_prefix(&scan_root)
                    .unwrap_or(&d.source_file);
                println!(
                    "  {}:{}  {}",
                    rel.display(),
                    d.line.to_string().yellow(),
                    d.uri.dimmed()
                );
            }
        }
    }
    Ok(())
}

/// Walk up from cwd looking for proof.toml so `proof depends` works from any subdir.
fn find_proof_root_for_cwd() -> Option<PathBuf> {
    let mut dir = std::env::current_dir().ok()?;
    loop {
        if dir.join("proof.toml").exists() {
            return Some(dir);
        }
        match dir.parent() {
            Some(p) => dir = p.to_path_buf(),
            None => return None,
        }
    }
}
