use crate::cmd_context::GlobalOptions;
use anyhow::Result;
use colored::Colorize;
use proof_lib::frontmatter::FrontmatterTagCounts;
use proof_lib::lint::load_config_for_path;
use proof_lib::GlintConfig;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(clap::Args)]
pub(crate) struct Args {
    /// Directory to inspect (default: current directory)
    #[arg(default_value = ".")]
    dir: PathBuf,
}

pub(crate) fn run_with_globals(args: Args, globals: &GlobalOptions) -> Result<()> {
    run(args, globals.config())
}

fn run(args: Args, config_override: &Option<PathBuf>) -> Result<()> {
    let dir = args.dir;
    let root = if dir.is_absolute() {
        dir.clone()
    } else {
        std::env::current_dir()?.join(dir)
    };

    println!(
        "{} — {}",
        "proof status".bold(),
        root.display().to_string().cyan()
    );
    println!();

    let mut source_count = 0usize;
    let mut compiled_count = 0usize;
    let mut stale_count = 0usize;
    let mut last_compile: Option<SystemTime> = None;
    let mut source_files = Vec::new();

    for entry in walkdir::WalkDir::new(&root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        if name.ends_with(".source.md") {
            source_count += 1;
            source_files.push(path.to_path_buf());
            let output = path.with_file_name(
                name.strip_suffix(".source.md").unwrap_or(name).to_string() + ".md",
            );
            if output.exists() {
                compiled_count += 1;
                if let (Ok(src_meta), Ok(out_meta)) = (path.metadata(), output.metadata()) {
                    if let (Ok(src_mod), Ok(out_mod)) = (src_meta.modified(), out_meta.modified()) {
                        if src_mod > out_mod {
                            stale_count += 1;
                        }
                        if last_compile.map_or(true, |lc| out_mod > lc) {
                            last_compile = Some(out_mod);
                        }
                    }
                }
            } else {
                stale_count += 1;
            }
        }
    }

    let stale_label = if stale_count == 0 {
        "0".green().to_string()
    } else {
        format!("{}", stale_count).yellow().to_string()
    };

    println!("  {:<16} {}", "Sources".dimmed(), source_count);
    println!("  {:<16} {}", "Compiled".dimmed(), compiled_count);
    println!("  {:<16} {}", "Stale".dimmed(), stale_label);
    let tag_counts = FrontmatterTagCounts::from_files(&source_files);
    println!(
        "  {:<16} {} files, {} tags",
        "Frontmatter".dimmed(),
        tag_counts.files_with_frontmatter,
        tag_counts.tags.len() + tag_counts.ops.len() + tag_counts.content.len()
    );

    if let Some(ts) = last_compile {
        let secs = ts.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
        let now_secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let age = now_secs.saturating_sub(secs);
        let age_str = if age < 60 {
            format!("{} sec ago", age)
        } else if age < 3600 {
            format!("{} min ago", age / 60)
        } else if age < 86400 {
            format!("{} hr ago", age / 3600)
        } else {
            format!("{} days ago", age / 86400)
        };
        println!("  {:<16} {}", "Last compile".dimmed(), age_str);
    } else {
        println!("  {:<16} {}", "Last compile".dimmed(), "never".dimmed());
    }

    let cache_file = root.join(".proof/last-check.json");
    if cache_file.exists() {
        if let Ok(content) = std::fs::read_to_string(&cache_file) {
            let errors: Option<u64> = extract_json_u64(&content, "errors");
            let warnings: Option<u64> = extract_json_u64(&content, "warnings");
            let files: Option<u64> = extract_json_u64(&content, "files_checked");
            if let Some(e) = errors {
                let err_label = if e == 0 {
                    "0".green().to_string()
                } else {
                    format!("{}", e).red().to_string()
                };
                println!(
                    "  {:<16} {} errors, {} warnings (last check, {} files)",
                    "Diagnostics".dimmed(),
                    err_label,
                    warnings.unwrap_or(0),
                    files.unwrap_or(0)
                );
            }
        }
    }

    let cfg = if config_override.is_some() {
        load_config_for_path(&root, config_override)?
    } else {
        GlintConfig::load_or_default(&root)
    };
    let schema_count = cfg.section_schemas.len();
    let target_count = cfg.compile.len();
    let root_flag = if cfg.files.root {
        "root=true"
    } else {
        "root=false"
    };
    println!(
        "  {:<16} proof.toml ({}, {} schemas, {} compile targets)",
        "Config".dimmed(),
        root_flag,
        schema_count,
        target_count
    );

    println!();
    Ok(())
}

fn extract_json_u64(json: &str, key: &str) -> Option<u64> {
    let search = format!("\"{}\":", key);
    let pos = json.find(&search)?;
    let after = json[pos + search.len()..].trim_start();
    let end = after
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(after.len());
    after[..end].parse().ok()
}
