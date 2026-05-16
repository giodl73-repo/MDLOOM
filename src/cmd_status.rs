use crate::cmd_context::GlobalOptions;
use anyhow::{bail, Result};
use colored::Colorize;
use proof_lib::frontmatter::FrontmatterTagCounts;
use proof_lib::lint::load_config_for_path;
use proof_lib::GlintConfig;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(clap::Args)]
pub(crate) struct Args {
    /// Directory to inspect (default: current directory)
    #[arg(default_value = ".")]
    dir: PathBuf,
    /// Delegate corpus health to CROP status instead of the local PROOF summary
    #[arg(long)]
    crop: bool,
    /// CROP executable to invoke with --crop
    #[arg(long, default_value = "crop")]
    crop_bin: PathBuf,
    /// crop.view.v1 recipe to scan with --crop
    #[arg(long)]
    view: Option<PathBuf>,
    /// Relay CROP strict mode with --crop
    #[arg(long)]
    strict: bool,
    /// Limit --strict to selected CROP issue classes
    #[arg(long = "strict-on")]
    strict_on: Vec<String>,
    /// CROP status output format with --crop: markdown or json
    #[arg(long = "crop-format")]
    crop_format: Option<String>,
    /// Restrict CROP status to one or more extensions
    #[arg(long = "extension")]
    extensions: Vec<String>,
    /// Exclude directories by basename in CROP status
    #[arg(long = "exclude-dir")]
    exclude_dirs: Vec<String>,
}

pub(crate) fn run_with_globals(args: Args, globals: &GlobalOptions) -> Result<()> {
    if args.crop {
        return run_crop_status(args, globals);
    }
    reject_crop_only_options(&args)?;
    run(args, globals.config())
}

fn reject_crop_only_options(args: &Args) -> Result<()> {
    if args.view.is_some()
        || args.strict
        || !args.strict_on.is_empty()
        || !args.extensions.is_empty()
        || !args.exclude_dirs.is_empty()
        || args.crop_format.is_some()
        || args.crop_bin != Path::new("crop")
    {
        bail!("proof status CROP options require --crop");
    }
    Ok(())
}

fn run_crop_status(args: Args, globals: &GlobalOptions) -> Result<()> {
    let crop_bin = args.crop_bin.clone();
    crate::cmd_crop::run_crop(crop_bin, build_crop_status_args(args, globals)?)
}

fn build_crop_status_args(args: Args, globals: &GlobalOptions) -> Result<Vec<String>> {
    if args.view.is_some() && args.dir != Path::new(".") {
        bail!("proof status --crop accepts either a positional directory or --view, not both");
    }
    let root = if args.view.is_some() {
        None
    } else if args.dir.is_absolute() {
        Some(args.dir)
    } else {
        Some(std::env::current_dir()?.join(args.dir))
    };

    crate::cmd_crop::build_status_request_args(crate::cmd_crop::CropStatusRequest {
        root,
        view: args.view,
        title: None,
        extensions: args.extensions,
        exclude_dirs: args.exclude_dirs,
        strict: args.strict,
        strict_on: args.strict_on,
        format: crop_status_format(args.crop_format, globals),
        output: globals.output().clone(),
    })
}

fn crop_status_format(crop_format: Option<String>, globals: &GlobalOptions) -> String {
    crop_format.unwrap_or_else(|| {
        if globals.format() == "text" {
            "markdown".to_string()
        } else {
            globals.format().to_string()
        }
    })
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
                        if last_compile.is_none_or(|lc| out_mod > lc) {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn globals(output: Option<PathBuf>) -> GlobalOptions {
        GlobalOptions::new(None, "text".to_string(), false, false, output)
    }

    fn globals_with_format(format: &str) -> GlobalOptions {
        GlobalOptions::new(None, format.to_string(), false, false, None)
    }

    #[test]
    fn crop_status_args_use_root_by_default() {
        let dir = tempfile::tempdir().unwrap();
        let args = build_crop_status_args(
            Args {
                dir: dir.path().to_path_buf(),
                crop: true,
                crop_bin: PathBuf::from("crop"),
                view: None,
                strict: true,
                strict_on: vec!["broken-links".to_string()],
                crop_format: Some("json".to_string()),
                extensions: vec!["md".to_string()],
                exclude_dirs: vec!["target".to_string()],
            },
            &globals(Some(PathBuf::from("STATUS.json"))),
        )
        .unwrap();

        assert_eq!(
            args,
            vec![
                "status".to_string(),
                "--root".to_string(),
                dir.path().display().to_string(),
                "--extension".to_string(),
                "md".to_string(),
                "--exclude-dir".to_string(),
                "target".to_string(),
                "--strict".to_string(),
                "--strict-on".to_string(),
                "broken-links".to_string(),
                "--format".to_string(),
                "json".to_string(),
                "--output".to_string(),
                "STATUS.json".to_string(),
            ]
        );
    }

    #[test]
    fn crop_status_args_can_use_view() {
        let args = build_crop_status_args(
            Args {
                dir: PathBuf::from("."),
                crop: true,
                crop_bin: PathBuf::from("crop"),
                view: Some(PathBuf::from(".crop\\views\\ready.json")),
                strict: false,
                strict_on: vec![],
                crop_format: None,
                extensions: vec![],
                exclude_dirs: vec![],
            },
            &globals(None),
        )
        .unwrap();

        assert_eq!(
            args,
            vec![
                "status".to_string(),
                "--view".to_string(),
                ".crop\\views\\ready.json".to_string(),
                "--format".to_string(),
                "markdown".to_string(),
            ]
        );
    }

    #[test]
    fn crop_status_rejects_dir_with_view() {
        let err = build_crop_status_args(
            Args {
                dir: PathBuf::from("docs"),
                crop: true,
                crop_bin: PathBuf::from("crop"),
                view: Some(PathBuf::from(".crop\\views\\ready.json")),
                strict: false,
                strict_on: vec![],
                crop_format: None,
                extensions: vec![],
                exclude_dirs: vec![],
            },
            &globals(None),
        )
        .unwrap_err();

        assert!(err
            .to_string()
            .contains("either a positional directory or --view"));
    }

    #[test]
    fn crop_status_args_use_global_format_when_crop_format_missing() {
        let args = build_crop_status_args(
            Args {
                dir: PathBuf::from("."),
                crop: true,
                crop_bin: PathBuf::from("crop"),
                view: Some(PathBuf::from(".crop\\views\\ready.json")),
                strict: false,
                strict_on: vec![],
                crop_format: None,
                extensions: vec![],
                exclude_dirs: vec![],
            },
            &globals_with_format("json"),
        )
        .unwrap();

        assert_eq!(
            args,
            vec![
                "status".to_string(),
                "--view".to_string(),
                ".crop\\views\\ready.json".to_string(),
                "--format".to_string(),
                "json".to_string(),
            ]
        );
    }

    #[test]
    fn local_status_rejects_crop_only_options() {
        let err = reject_crop_only_options(&Args {
            dir: PathBuf::from("."),
            crop: false,
            crop_bin: PathBuf::from("crop"),
            view: Some(PathBuf::from("ready.json")),
            strict: false,
            strict_on: vec![],
            crop_format: None,
            extensions: vec![],
            exclude_dirs: vec![],
        })
        .unwrap_err();

        assert!(err.to_string().contains("require --crop"));
    }
}
