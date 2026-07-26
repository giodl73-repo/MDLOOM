use crate::cmd_context::GlobalOptions;
use crate::cmd_paths::paths_or_cwd;
use anyhow::Result;
use clap::ValueEnum;
use colored::Colorize;
use mdloom_lib::artifact::{self, ArtifactDiagnostic, ArtifactRecord, ArtifactStatus};
use mdloom_lib::compile::{compile_file, derive_output_path, ViolationSeverity};
use mdloom_lib::frontmatter::FrontmatterFilter;
use mdloom_lib::lint::load_config_for_path as load_config;
use std::path::{Path, PathBuf};
use std::process;

#[derive(clap::Args)]
pub(crate) struct Args {
    /// Source files or directories (default: current directory)
    paths: Vec<PathBuf>,
    /// Explicit output path (only valid for single-file compile)
    #[arg(short = 'o', long)]
    output: Option<PathBuf>,
    /// Output directory for all compiled files (overrides per-file placement)
    #[arg(long)]
    output_dir: Option<PathBuf>,
    /// Validate without writing any output files
    #[arg(long)]
    check: bool,
    /// Watch for changes and recompile automatically
    #[arg(long)]
    watch: bool,
    /// Delete output file when compile produces errors (default: leave stale output in place)
    #[arg(long)]
    delete_on_error: bool,
    /// Show running count instead of one line per file (useful for 50+ source files)
    #[arg(long)]
    progress: bool,
    /// Root directory for md:// URI resolution (default: mdloom.toml location or cwd)
    #[arg(long)]
    root: Option<PathBuf>,
    /// Output target format
    #[arg(long, value_enum, default_value_t = CompileTarget::Md)]
    target: CompileTarget,
    /// Only compile source files with this frontmatter tag (repeatable)
    #[arg(long = "tag")]
    tags: Vec<String>,
    /// Only compile source files with this operation tag (repeatable)
    #[arg(long = "op")]
    ops: Vec<String>,
    /// Only compile source files with this content tag (repeatable)
    #[arg(long = "content-tag")]
    content_tags: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum CompileTarget {
    Md,
    Html,
    Mdport,
    JsonReport,
    Site,
    Pdf,
    Docx,
    Pptx,
}

impl CompileTarget {
    fn as_str(self) -> &'static str {
        match self {
            CompileTarget::Md => "md",
            CompileTarget::Html => "html",
            CompileTarget::Mdport => "mdport",
            CompileTarget::JsonReport => "json-report",
            CompileTarget::Site => "site",
            CompileTarget::Pdf => "pdf",
            CompileTarget::Docx => "docx",
            CompileTarget::Pptx => "pptx",
        }
    }
}

pub(crate) fn run_with_globals(args: Args, globals: &GlobalOptions) -> Result<()> {
    run(args, globals.config())
}

fn run(args: Args, config_override: &Option<PathBuf>) -> Result<()> {
    let Args {
        paths,
        output,
        output_dir,
        check,
        watch,
        delete_on_error,
        progress,
        root,
        target,
        tags,
        ops,
        content_tags,
    } = args;
    let paths = paths_or_cwd(paths)?;
    if watch {
        if target != CompileTarget::Md {
            eprintln!(
                "{} --watch currently supports only --target md",
                "error:".red()
            );
            process::exit(2);
        }
        return run_watch(paths, output_dir, root, config_override);
    }

    run_once(
        paths,
        output,
        output_dir,
        check,
        delete_on_error,
        progress,
        root,
        target,
        FrontmatterFilter {
            tags,
            ops,
            content: content_tags,
        },
        config_override,
    )
}

fn run_once(
    paths: Vec<PathBuf>,
    output_override: Option<PathBuf>,
    output_dir: Option<PathBuf>,
    check_only: bool,
    delete_on_error: bool,
    progress: bool,
    root_override: Option<PathBuf>,
    target: CompileTarget,
    tag_filter: FrontmatterFilter,
    config_override: &Option<PathBuf>,
) -> Result<()> {
    if output_override.is_some() && output_dir.is_some() {
        eprintln!(
            "{} -o and --output-dir are mutually exclusive",
            "error:".red()
        );
        process::exit(2);
    }

    let root = root_override.unwrap_or_else(|| std::env::current_dir().unwrap());
    let config = load_config(&root, config_override)?;

    // Build a list of (source_path, output_dir) pairs.
    // When using [[compile]] targets from mdloom.toml (and no explicit paths/output-dir),
    // route each source file to the correct target's output_dir.
    let using_defaults = paths.iter().any(|p| p == &std::env::current_dir().unwrap());
    let _has_multi_targets = config.compile.len() > 1;

    let source_dir_pairs: Vec<(PathBuf, Option<PathBuf>)> = if !config.compile.is_empty()
        && using_defaults
        && output_dir.is_none()
        && output_override.is_none()
    {
        // Per-target routing from mdloom.toml
        let mut pairs = Vec::new();
        for target in &config.compile {
            let src_dir = target
                .source_dir
                .as_ref()
                .map(|s| root.join(s))
                .unwrap_or_else(|| root.clone());
            let out = target.output_dir.as_ref().map(|d| root.join(d));
            if let Some(ref dir) = out {
                let _ = std::fs::create_dir_all(dir);
            }
            if src_dir.is_dir() {
                for entry in walkdir::WalkDir::new(&src_dir)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .filter(|e| e.file_type().is_file())
                {
                    let p = entry.path().to_path_buf();
                    if p.to_str()
                        .map(|s| s.ends_with(".source.md"))
                        .unwrap_or(false)
                    {
                        pairs.push((p, out.clone()));
                    }
                }
            } else if src_dir.is_file() {
                pairs.push((src_dir, out));
            }
        }
        pairs
    } else {
        // Explicit paths or single output_dir override
        let resolved_out = output_dir.or_else(|| {
            config
                .compile
                .first()
                .and_then(|t| t.output_dir.as_ref())
                .map(|d| root.join(d))
        });
        if let Some(ref dir) = resolved_out {
            let _ = std::fs::create_dir_all(dir);
        }
        let mut pairs = Vec::new();
        for path in &paths {
            if path.is_file() {
                pairs.push((path.clone(), resolved_out.clone()));
            } else {
                for entry in walkdir::WalkDir::new(path)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .filter(|e| e.file_type().is_file())
                {
                    let p = entry.path().to_path_buf();
                    if p.to_str()
                        .map(|s| s.ends_with(".source.md"))
                        .unwrap_or(false)
                    {
                        pairs.push((p, resolved_out.clone()));
                    }
                }
            }
        }
        pairs
    };

    let source_dir_pairs: Vec<(PathBuf, Option<PathBuf>)> = if tag_filter.is_empty() {
        source_dir_pairs
    } else {
        source_dir_pairs
            .into_iter()
            .filter(|(source_path, _)| tag_filter.matches_path(source_path))
            .collect()
    };
    let source_files: Vec<PathBuf> = source_dir_pairs.iter().map(|(p, _)| p.clone()).collect();

    if source_files.is_empty() {
        eprintln!("{} no .source.md files found", "mdloom compile:".yellow());
        return Ok(());
    }

    if output_override.is_some() && source_files.len() > 1 {
        eprintln!(
            "{} -o can only be used with a single source file",
            "error:".red()
        );
        process::exit(2);
    }

    let mut total_errors = 0usize;
    let mut total_warnings = 0usize;
    let mut compiled = 0usize;
    let mut artifacts = Vec::new();

    for (source_path, target_out_dir) in &source_dir_pairs {
        let output_path = if let Some(ref out) = output_override {
            out.clone()
        } else if let Some(ref dir) = target_out_dir {
            // Derive filename, then place it in the output directory
            if let Some(derived) = derive_target_output_path(source_path, target) {
                let filename = derived.file_name().expect("derived path has filename");
                dir.join(filename)
            } else {
                eprintln!(
                    "{} {} has no .source.md suffix — skipping",
                    "skip:".yellow(),
                    source_path.display()
                );
                continue;
            }
        } else if let Some(p) = derive_target_output_path(source_path, target) {
            p
        } else {
            eprintln!(
                "{} {} has no .source.md suffix — use -o to specify output",
                "skip:".yellow(),
                source_path.display()
            );
            continue;
        };

        if check_only {
            eprintln!(
                "  {} {} → {} (check only)",
                "→".cyan(),
                source_path.display(),
                output_path.display()
            );
        }

        let result = compile_target_file(source_path, &output_path, &root, &config, target)?;

        let artifact_status = if result
            .violations
            .iter()
            .any(|v| v.severity == ViolationSeverity::Error)
        {
            ArtifactStatus::Error
        } else if result.from_cache {
            ArtifactStatus::Cached
        } else if !result.written {
            ArtifactStatus::UpToDate
        } else {
            ArtifactStatus::Written
        };

        if !check_only {
            artifacts.push(ArtifactRecord {
                source_path: source_path.clone(),
                output_path: output_path.clone(),
                target: target.as_str().to_string(),
                status: artifact_status,
                directives_resolved: result.directives_resolved,
                from_cache: result.from_cache,
                resolved_files: result.resolved_files.clone(),
                diagnostics: result
                    .violations
                    .iter()
                    .map(|v| ArtifactDiagnostic {
                        code: v.code.to_string(),
                        severity: match v.severity {
                            ViolationSeverity::Error => "error",
                            ViolationSeverity::Warning => "warning",
                        }
                        .to_string(),
                        line: v.source_line,
                        message: v.message.clone(),
                    })
                    .collect(),
            });
        }

        // Report violations
        for v in &result.violations {
            let sev = match v.severity {
                ViolationSeverity::Error => {
                    total_errors += 1;
                    "error".red().bold().to_string()
                }
                ViolationSeverity::Warning => {
                    total_warnings += 1;
                    "warning".yellow().bold().to_string()
                }
            };
            eprintln!(
                "{}:{}:{}: {} [{}]: {}",
                source_path.display().to_string().cyan(),
                v.source_line,
                1,
                sev,
                v.code,
                v.message
            );
            if let Some(ref id) = v.figure_id {
                eprintln!("    figure: {}", id);
            }
            if !v.uri.is_empty() {
                eprintln!("    uri:    {}", v.uri);
            }
        }

        // F119: --delete-on-error removes stale output when compile fails
        if !result.written && delete_on_error && output_path.exists() {
            let _ = std::fs::remove_file(&output_path);
            eprintln!(
                "{} deleted stale output: {}",
                "→".yellow(),
                output_path.display()
            );
        }

        if result.written {
            compiled += 1;
            if !progress {
                eprintln!(
                    "{} {} → {}  ({} directive{})",
                    "✓".green(),
                    source_path.display().to_string().cyan(),
                    output_path.display(),
                    result.directives_resolved,
                    if result.directives_resolved == 1 {
                        ""
                    } else {
                        "s"
                    },
                );
            } else {
                eprint!("\r  compiling {}/{}…  ", compiled, source_files.len());
            }
        } else if result.from_cache {
            compiled += 1;
            if progress {
                eprint!("\r  compiling {}/{}…  ", compiled, source_files.len());
            }
        } else if !result
            .violations
            .iter()
            .any(|v| v.severity == ViolationSeverity::Error)
        {
            if !check_only {
                // Copy source to output unchanged
                std::fs::copy(source_path, &output_path)?;
                compiled += 1;
                if progress {
                    eprint!("\r  compiling {}/{}…  ", compiled, source_files.len());
                }
            }
        }
    }

    if progress {
        eprintln!();
    } // clear progress line
    if !check_only {
        if target == CompileTarget::Site {
            write_static_site(&artifacts, &root)?;
        }
        let manifest = artifact::write_manifest(&root, artifacts)?;
        if !progress {
            eprintln!("  manifest: {}", manifest.display());
        }
    }
    eprintln!();
    if total_errors > 0 {
        eprintln!(
            "{} — {} compiled, {} error{}, {} warning{}",
            "FAIL".red().bold(),
            compiled,
            total_errors,
            if total_errors == 1 { "" } else { "s" },
            total_warnings,
            if total_warnings == 1 { "" } else { "s" },
        );
        process::exit(1);
    } else {
        eprintln!(
            "{} — {} compiled, {} warning{}",
            "OK".green().bold(),
            compiled,
            total_warnings,
            if total_warnings == 1 { "" } else { "s" },
        );
    }
    Ok(())
}

fn derive_target_output_path(source: &Path, target: CompileTarget) -> Option<PathBuf> {
    let mut output = derive_output_path(source)?;
    match target {
        CompileTarget::Md => {}
        CompileTarget::Html => {
            output.set_extension("html");
        }
        CompileTarget::Mdport => {
            output.set_extension("mdport.json");
        }
        CompileTarget::JsonReport => {
            output.set_extension("mdloom-report.json");
        }
        CompileTarget::Site => {
            output.set_extension("html");
        }
        CompileTarget::Pdf => {
            output.set_extension("pdf");
        }
        CompileTarget::Docx => {
            output.set_extension("docx");
        }
        CompileTarget::Pptx => {
            output.set_extension("pptx");
        }
    }
    Some(output)
}

fn compile_target_file(
    source_path: &Path,
    output_path: &Path,
    root: &Path,
    config: &mdloom_lib::MdloomConfig,
    target: CompileTarget,
) -> Result<mdloom_lib::compile::CompileResult> {
    match target {
        CompileTarget::Md => compile_file(source_path, output_path, root, config),
        CompileTarget::Html => compile_html_file(source_path, output_path, root, config),
        CompileTarget::Mdport => compile_mdport_file(source_path, output_path, root, config),
        CompileTarget::JsonReport => {
            compile_json_report_file(source_path, output_path, root, config)
        }
        CompileTarget::Site => compile_html_file(source_path, output_path, root, config),
        CompileTarget::Pdf => compile_pdf_file(source_path, output_path, root, config),
        CompileTarget::Docx => compile_docx_file(source_path, output_path, root, config),
        CompileTarget::Pptx => compile_pptx_file(source_path, output_path, root, config),
    }
}

fn compile_html_file(
    source_path: &Path,
    output_path: &Path,
    root: &Path,
    config: &mdloom_lib::MdloomConfig,
) -> Result<mdloom_lib::compile::CompileResult> {
    let temp_dir = unique_temp_dir()?;
    let markdown_path = temp_dir.join("compiled.md");
    let mut result = compile_file(source_path, &markdown_path, root, config)?;
    if result
        .violations
        .iter()
        .any(|v| v.severity == ViolationSeverity::Error)
    {
        let _ = std::fs::remove_dir_all(&temp_dir);
        result.output_path = output_path.to_path_buf();
        return Ok(result);
    }

    let markdown = std::fs::read_to_string(&markdown_path)?;
    let title = source_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("mdloom document");
    let html = mdloom_lib::publish::markdown_to_html_document(&markdown, title);
    let current = std::fs::read_to_string(output_path).unwrap_or_default();
    result.written = current != html;
    if result.written {
        let tmp = output_path.with_extension("mdloom_tmp");
        std::fs::write(&tmp, html)?;
        std::fs::rename(&tmp, output_path)?;
    }
    result.output_path = output_path.to_path_buf();
    let _ = std::fs::remove_dir_all(&temp_dir);
    Ok(result)
}

fn compile_mdport_file(
    source_path: &Path,
    output_path: &Path,
    root: &Path,
    config: &mdloom_lib::MdloomConfig,
) -> Result<mdloom_lib::compile::CompileResult> {
    let temp_dir = unique_temp_dir()?;
    let markdown_path = temp_dir.join("compiled.md");
    let mut result = compile_file(source_path, &markdown_path, root, config)?;
    if result
        .violations
        .iter()
        .any(|v| v.severity == ViolationSeverity::Error)
    {
        let _ = std::fs::remove_dir_all(&temp_dir);
        result.output_path = output_path.to_path_buf();
        return Ok(result);
    }

    let markdown = std::fs::read_to_string(&markdown_path)?;
    let title = source_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("mdloom document");
    let mdport = mdloom_lib::publish::markdown_to_mdport_document(
        &markdown,
        title,
        source_path,
        &result.resolved_files,
    );
    let current = std::fs::read_to_string(output_path).unwrap_or_default();
    result.written = current != mdport;
    if result.written {
        let tmp = output_path.with_extension("mdloom_tmp");
        std::fs::write(&tmp, mdport)?;
        std::fs::rename(&tmp, output_path)?;
    }
    result.output_path = output_path.to_path_buf();
    let _ = std::fs::remove_dir_all(&temp_dir);
    Ok(result)
}

fn compile_json_report_file(
    source_path: &Path,
    output_path: &Path,
    root: &Path,
    config: &mdloom_lib::MdloomConfig,
) -> Result<mdloom_lib::compile::CompileResult> {
    let temp_dir = unique_temp_dir()?;
    let markdown_path = temp_dir.join("compiled.md");
    let mut result = compile_file(source_path, &markdown_path, root, config)?;
    if result
        .violations
        .iter()
        .any(|v| v.severity == ViolationSeverity::Error)
    {
        let _ = std::fs::remove_dir_all(&temp_dir);
        result.output_path = output_path.to_path_buf();
        return Ok(result);
    }

    let markdown = std::fs::read_to_string(&markdown_path)?;
    let title = source_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("mdloom document");
    let frontmatter = mdloom_lib::frontmatter::read(source_path)?.unwrap_or_default();
    let diagnostics = result
        .violations
        .iter()
        .map(|violation| mdloom_lib::publish::JsonReportDiagnostic {
            code: violation.code.to_string(),
            severity: match violation.severity {
                ViolationSeverity::Error => "error",
                ViolationSeverity::Warning => "warning",
            }
            .to_string(),
            line: violation.source_line,
            message: violation.message.clone(),
        })
        .collect::<Vec<_>>();
    let report = mdloom_lib::publish::markdown_to_json_report_bundle(
        &markdown,
        title,
        source_path,
        output_path,
        &result.resolved_files,
        frontmatter,
        mdloom_lib::publish::JsonReportCompile {
            directives_resolved: result.directives_resolved,
            diagnostics_count: diagnostics.len(),
            diagnostics,
        },
    );
    let current = std::fs::read_to_string(output_path).unwrap_or_default();
    result.written = current != report;
    if result.written {
        let tmp = output_path.with_extension("mdloom_tmp");
        std::fs::write(&tmp, report)?;
        std::fs::rename(&tmp, output_path)?;
    }
    result.output_path = output_path.to_path_buf();
    let _ = std::fs::remove_dir_all(&temp_dir);
    Ok(result)
}

fn compile_pdf_file(
    source_path: &Path,
    output_path: &Path,
    root: &Path,
    config: &mdloom_lib::MdloomConfig,
) -> Result<mdloom_lib::compile::CompileResult> {
    let temp_dir = unique_temp_dir()?;
    let markdown_path = temp_dir.join("compiled.md");
    let mut result = compile_file(source_path, &markdown_path, root, config)?;
    if result
        .violations
        .iter()
        .any(|v| v.severity == ViolationSeverity::Error)
    {
        let _ = std::fs::remove_dir_all(&temp_dir);
        result.output_path = output_path.to_path_buf();
        return Ok(result);
    }

    let markdown = std::fs::read_to_string(&markdown_path)?;
    let title = source_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("mdloom document");
    let html = mdloom_lib::publish::markdown_to_html_document(&markdown, title);
    let pdf = mdloom_lib::publish::html_to_pdf_document(&html, title);
    let current = std::fs::read(output_path).unwrap_or_default();
    result.written = current != pdf;
    if result.written {
        let tmp = output_path.with_extension("mdloom_tmp");
        std::fs::write(&tmp, pdf)?;
        std::fs::rename(&tmp, output_path)?;
    }
    result.output_path = output_path.to_path_buf();
    let _ = std::fs::remove_dir_all(&temp_dir);
    Ok(result)
}

fn compile_docx_file(
    source_path: &Path,
    output_path: &Path,
    root: &Path,
    config: &mdloom_lib::MdloomConfig,
) -> Result<mdloom_lib::compile::CompileResult> {
    let temp_dir = unique_temp_dir()?;
    let markdown_path = temp_dir.join("compiled.md");
    let mut result = compile_file(source_path, &markdown_path, root, config)?;
    if result
        .violations
        .iter()
        .any(|v| v.severity == ViolationSeverity::Error)
    {
        let _ = std::fs::remove_dir_all(&temp_dir);
        result.output_path = output_path.to_path_buf();
        return Ok(result);
    }

    let markdown = std::fs::read_to_string(&markdown_path)?;
    let title = source_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("mdloom document");
    let docx = mdloom_lib::publish::markdown_to_docx_document(&markdown, title);
    let current = std::fs::read(output_path).unwrap_or_default();
    result.written = current != docx;
    if result.written {
        let tmp = output_path.with_extension("mdloom_tmp");
        std::fs::write(&tmp, docx)?;
        std::fs::rename(&tmp, output_path)?;
    }
    result.output_path = output_path.to_path_buf();
    let _ = std::fs::remove_dir_all(&temp_dir);
    Ok(result)
}

fn compile_pptx_file(
    source_path: &Path,
    output_path: &Path,
    root: &Path,
    config: &mdloom_lib::MdloomConfig,
) -> Result<mdloom_lib::compile::CompileResult> {
    if !source_path
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.ends_with(".slides.source.md"))
        .unwrap_or(false)
    {
        return Ok(mdloom_lib::compile::CompileResult {
            output_path: output_path.to_path_buf(),
            directives_resolved: 0,
            violations: vec![mdloom_lib::compile::CompileViolation {
                code: "PPTX-001",
                severity: ViolationSeverity::Error,
                uri: String::new(),
                figure_id: None,
                invariant: String::new(),
                message: "pptx target requires an explicit .slides.source.md source".to_string(),
                source_line: 1,
            }],
            from_cache: false,
            written: false,
            resolved_files: Vec::new(),
        });
    }

    let temp_dir = unique_temp_dir()?;
    let slides_markdown_path = temp_dir.join("compiled.slides.md");
    let mut result = compile_file(source_path, &slides_markdown_path, root, config)?;
    if result
        .violations
        .iter()
        .any(|v| v.severity == ViolationSeverity::Error)
    {
        let _ = std::fs::remove_dir_all(&temp_dir);
        result.output_path = output_path.to_path_buf();
        return Ok(result);
    }

    let source = std::fs::read_to_string(source_path)?;
    let title = source_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("mdloom deck");
    let pptx = match mdloom_lib::publish::slides_source_to_pptx_document(&source, title) {
        Ok(pptx) => pptx,
        Err(errors) => {
            result.violations.extend(errors.into_iter().map(|message| {
                mdloom_lib::compile::CompileViolation {
                    code: "PPTX-002",
                    severity: ViolationSeverity::Error,
                    uri: String::new(),
                    figure_id: None,
                    invariant: String::new(),
                    message,
                    source_line: 1,
                }
            }));
            result.output_path = output_path.to_path_buf();
            let _ = std::fs::remove_dir_all(&temp_dir);
            return Ok(result);
        }
    };
    let current = std::fs::read(output_path).unwrap_or_default();
    result.written = current != pptx;
    if result.written {
        let tmp = output_path.with_extension("mdloom_tmp");
        std::fs::write(&tmp, pptx)?;
        std::fs::rename(&tmp, output_path)?;
    }
    result.output_path = output_path.to_path_buf();
    let _ = std::fs::remove_dir_all(&temp_dir);
    Ok(result)
}

fn write_static_site(artifacts: &[ArtifactRecord], root: &Path) -> Result<()> {
    let site_root = site_root_for_artifacts(artifacts, root);
    let pages = artifacts
        .iter()
        .map(|artifact| {
            let title = std::fs::read_to_string(&artifact.output_path)
                .ok()
                .and_then(|html| mdloom_lib::publish::html_document_title(&html))
                .unwrap_or_else(|| title_from_path(&artifact.source_path));
            mdloom_lib::publish::SitePage {
                title,
                source_path: artifact.source_path.display().to_string(),
                output_path: artifact.output_path.display().to_string(),
                href: relative_href(&site_root, &artifact.output_path),
                status: artifact_status_name(&artifact.status).to_string(),
                diagnostics_count: artifact.diagnostics.len(),
            }
        })
        .collect::<Vec<_>>();
    mdloom_lib::publish::write_static_site(&site_root, pages)?;
    Ok(())
}

fn site_root_for_artifacts(artifacts: &[ArtifactRecord], root: &Path) -> PathBuf {
    let mut parents = artifacts
        .iter()
        .filter_map(|artifact| artifact.output_path.parent())
        .collect::<Vec<_>>();
    parents.dedup();
    match parents.as_slice() {
        [parent] => parent.to_path_buf(),
        _ => root.to_path_buf(),
    }
}

fn relative_href(site_root: &Path, output_path: &Path) -> String {
    output_path
        .strip_prefix(site_root)
        .unwrap_or(output_path)
        .display()
        .to_string()
        .replace('\\', "/")
}

fn title_from_path(path: &Path) -> String {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("Untitled")
        .trim_end_matches(".source")
        .to_string()
}

fn artifact_status_name(status: &ArtifactStatus) -> &'static str {
    match status {
        ArtifactStatus::Written => "written",
        ArtifactStatus::Cached => "cached",
        ArtifactStatus::UpToDate => "up_to_date",
        ArtifactStatus::Error => "error",
    }
}

fn unique_temp_dir() -> Result<PathBuf> {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("mdloom-compile-{}-{}", std::process::id(), nanos));
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn run_watch(
    paths: Vec<PathBuf>,
    output_dir_override: Option<PathBuf>,
    root_override: Option<PathBuf>,
    config_override: &Option<PathBuf>,
) -> Result<()> {
    use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
    use std::collections::{HashMap, HashSet};
    use std::sync::mpsc;
    use std::time::{Duration, Instant};

    let root = root_override.unwrap_or_else(|| std::env::current_dir().unwrap());
    let config = load_config(&root, config_override)?;

    // Build watch targets from [[compile]] entries or CLI paths
    // Each target is (source_dir, output_dir)
    let using_default_paths = paths.iter().any(|p| p == &std::env::current_dir().unwrap());

    let watch_targets: Vec<(PathBuf, Option<PathBuf>)> =
        if !config.compile.is_empty() && using_default_paths {
            // Use all [[compile]] targets from mdloom.toml
            config
                .compile
                .iter()
                .map(|t| {
                    let src = t
                        .source_dir
                        .as_ref()
                        .map(|s| root.join(s))
                        .unwrap_or_else(|| root.clone());
                    let out = t
                        .output_dir
                        .as_ref()
                        .map(|d| root.join(d))
                        .or_else(|| output_dir_override.clone());
                    (src, out)
                })
                .collect()
        } else {
            // CLI paths + optional output_dir override
            let out = output_dir_override.or_else(|| {
                config
                    .compile
                    .first()
                    .and_then(|t| t.output_dir.as_ref())
                    .map(|d| root.join(d))
            });
            paths.into_iter().map(|p| (p, out.clone())).collect()
        };

    // For watch, flatten to just watch_paths; output_dir isn't used here
    let output_dir: Option<PathBuf> = None; // unused in watch — each target carries its own

    if let Some(ref dir) = output_dir {
        std::fs::create_dir_all(dir)?;
    }

    eprintln!(
        "{} watching for changes (Ctrl-C to stop)",
        "mdloom compile --watch:".cyan().bold()
    );
    for (src, out) in &watch_targets {
        if let Some(out) = out {
            eprintln!(
                "  {} → {}",
                src.display().to_string().dimmed(),
                out.display().to_string().dimmed()
            );
            std::fs::create_dir_all(out)?;
        } else {
            eprintln!(
                "  {} (output next to source)",
                src.display().to_string().dimmed()
            );
        }
    }
    eprintln!();

    // Reverse-dependency index. dep_to_sources[F] = every source file whose
    // last successful compile pulled F in via an md:// URI. When F changes,
    // every source listed under it gets recompiled.
    let mut dep_to_sources: HashMap<PathBuf, HashSet<PathBuf>> = HashMap::new();
    let mut watched_deps: HashSet<PathBuf> = HashSet::new();

    let (tx, rx) = mpsc::channel::<Result<Event, notify::Error>>();
    let mut watcher = RecommendedWatcher::new(tx, notify::Config::default())?;

    for (src_dir, _) in &watch_targets {
        if src_dir.exists() {
            watcher.watch(src_dir, RecursiveMode::Recursive)?;
        }
    }

    // Initial compile pass for all targets — collect dependencies as we go.
    for (src_dir, out_dir) in &watch_targets {
        let sources = compile_watch_pass(&[src_dir.clone()], out_dir, &root, &config)?;
        for source_path in &sources {
            update_deps_for_source(
                source_path,
                &root,
                &mut dep_to_sources,
                &mut watched_deps,
                &mut watcher,
            );
        }
    }
    if !watched_deps.is_empty() {
        eprintln!(
            "{} watching {} md:// dependency file{}",
            "→".cyan(),
            watched_deps.len(),
            if watched_deps.len() == 1 { "" } else { "s" }
        );
    }

    // Build a lookup: source path prefix → output_dir
    let target_map: Vec<(PathBuf, Option<PathBuf>)> = watch_targets.clone();

    let debounce = Duration::from_millis(100);
    let mut pending_sources: HashSet<PathBuf> = HashSet::new();
    let mut last_event = Instant::now();

    loop {
        match rx.recv_timeout(Duration::from_millis(50)) {
            Ok(Ok(event)) => {
                if matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
                    for path in event.paths {
                        let is_source = path
                            .to_str()
                            .map(|s| s.ends_with(".source.md"))
                            .unwrap_or(false);
                        if is_source {
                            pending_sources.insert(path);
                            last_event = Instant::now();
                        } else {
                            // Non-source file: check the reverse dep index
                            let key = std::fs::canonicalize(&path).unwrap_or(path);
                            if let Some(dependents) = dep_to_sources.get(&key) {
                                for dep_src in dependents {
                                    pending_sources.insert(dep_src.clone());
                                }
                                last_event = Instant::now();
                            }
                        }
                    }
                }
            }
            Ok(Err(e)) => eprintln!("{} watcher error: {}", "warn:".yellow(), e),
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }

        if !pending_sources.is_empty() && last_event.elapsed() >= debounce {
            let changed: Vec<PathBuf> = pending_sources.drain().collect();
            for source_path in &changed {
                // Find the matching target's output_dir
                let out_dir = target_map
                    .iter()
                    .find(|(src, _)| source_path.starts_with(src))
                    .and_then(|(_, out)| out.clone());
                compile_one_watch(source_path, &out_dir, &root, &config);
                update_deps_for_source(
                    source_path,
                    &root,
                    &mut dep_to_sources,
                    &mut watched_deps,
                    &mut watcher,
                );
            }
        }
    }

    Ok(())
}

/// Re-scan the source file for md:// URIs, resolve each to a filesystem path,
/// refresh `dep_to_sources` for this source, and add newly discovered deps to
/// the watcher (keeping `watched_deps` as the dedupe set). Existing dep
/// entries that no longer apply to this source are pruned.
fn update_deps_for_source<W: notify::Watcher>(
    source_path: &Path,
    root: &Path,
    dep_to_sources: &mut std::collections::HashMap<PathBuf, std::collections::HashSet<PathBuf>>,
    watched_deps: &mut std::collections::HashSet<PathBuf>,
    watcher: &mut W,
) {
    use notify::RecursiveMode;
    let canonical_source =
        std::fs::canonicalize(source_path).unwrap_or_else(|_| source_path.to_path_buf());

    let new_deps = scan_md_uri_deps(source_path, root);

    // Prune stale entries: anything in dep_to_sources that pointed to this
    // source but isn't in the fresh set anymore.
    let stale: Vec<PathBuf> = dep_to_sources
        .iter()
        .filter(|(dep, srcs)| srcs.contains(&canonical_source) && !new_deps.contains(*dep))
        .map(|(dep, _)| dep.clone())
        .collect();
    for dep in stale {
        if let Some(srcs) = dep_to_sources.get_mut(&dep) {
            srcs.remove(&canonical_source);
            if srcs.is_empty() {
                dep_to_sources.remove(&dep);
            }
        }
    }

    // Insert / update for current deps and watch each one.
    for dep in &new_deps {
        dep_to_sources
            .entry(dep.clone())
            .or_insert_with(std::collections::HashSet::new)
            .insert(canonical_source.clone());

        if !watched_deps.contains(dep) && dep.exists() {
            // Watch the file's parent directory non-recursively so we get
            // notified about edits without explicit recursive coverage. (notify
            // on Windows is happier watching directories than individual files
            // for cross-editor compatibility — many editors atomic-rename.)
            let watch_target = dep.parent().unwrap_or(dep);
            match watcher.watch(watch_target, RecursiveMode::NonRecursive) {
                Ok(_) => {
                    watched_deps.insert(dep.clone());
                }
                Err(_) => {
                    // Already watched (parent matches an existing recursive
                    // watch on a source dir), or transient permission issue —
                    // record as watched anyway so we don't retry on every
                    // recompile.
                    watched_deps.insert(dep.clone());
                }
            }
        }
    }
}

/// Scan a `.source.md` file for `md://` URIs and resolve each one to its
/// filesystem path via mdpath. Failed resolutions are silently skipped — the
/// compiler will surface the error on the next compile pass with proper
/// diagnostics; for the watcher we just want the paths we CAN resolve.
fn scan_md_uri_deps(source_path: &Path, root: &Path) -> std::collections::HashSet<PathBuf> {
    use std::collections::HashSet;
    let mut deps: HashSet<PathBuf> = HashSet::new();
    let content = match std::fs::read_to_string(source_path) {
        Ok(c) => c,
        Err(_) => return deps,
    };

    // Find every md:// literal in the source. Each URI runs from `md://` up to
    // (but not including) the first whitespace, quote, backtick, or `>` —
    // robust enough for all directive arg styles (`source=md://...`,
    // bare-line bodies, `[[davinci]] uri = "md://..."`, etc.).
    let mut idx = 0;
    while let Some(pos) = content[idx..].find("md://") {
        let start = idx + pos;
        let rest = &content[start..];
        let end_off = rest
            .find(|c: char| c.is_whitespace() || c == '"' || c == '`' || c == '>' || c == '<')
            .unwrap_or(rest.len());
        let uri = &rest[..end_off];
        idx = start + end_off.max(1);

        let parsed = match mdpath::parse(uri) {
            Ok(p) => p,
            Err(_) => continue,
        };
        if let Ok(element) = mdpath::resolve(&parsed, root) {
            let canonical = std::fs::canonicalize(&element.file).unwrap_or(element.file);
            // Don't add the source file itself — that's covered by
            // `.source.md` event handling and would cause feedback loops.
            if canonical != source_path {
                deps.insert(canonical);
            }
        }
    }
    deps
}

fn compile_watch_pass(
    watch_paths: &[PathBuf],
    output_dir: &Option<PathBuf>,
    root: &Path,
    config: &mdloom_lib::MdloomConfig,
) -> Result<Vec<PathBuf>> {
    let mut sources: Vec<PathBuf> = Vec::new();
    for watch_path in watch_paths {
        for entry in walkdir::WalkDir::new(watch_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let p = entry.path().to_path_buf();
            if p.to_str()
                .map(|s| s.ends_with(".source.md"))
                .unwrap_or(false)
            {
                compile_one_watch(&p, output_dir, root, config);
                sources.push(p);
            }
        }
    }
    eprintln!("{} initial compile: {} files", "→".cyan(), sources.len());
    Ok(sources)
}

fn compile_one_watch(
    source_path: &PathBuf,
    output_dir: &Option<PathBuf>,
    root: &Path,
    config: &mdloom_lib::MdloomConfig,
) {
    let output_path = if let Some(dir) = output_dir {
        if let Some(derived) = derive_output_path(source_path) {
            let filename = derived.file_name().expect("has filename");
            dir.join(filename)
        } else {
            return;
        }
    } else if let Some(p) = derive_output_path(source_path) {
        p
    } else {
        return;
    };

    let ts = chrono_or_time();
    match compile_file(source_path, &output_path, root, config) {
        Ok(result) => {
            let errors: Vec<_> = result
                .violations
                .iter()
                .filter(|v| v.severity == ViolationSeverity::Error)
                .collect();
            if errors.is_empty() {
                eprintln!(
                    "{} {} {} → {}  {}",
                    ts.dimmed(),
                    "✓".green(),
                    source_path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .cyan(),
                    output_path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy(),
                    format!("({} directives)", result.directives_resolved).dimmed(),
                );
            } else {
                // File was NOT written — make this very visible
                eprintln!(
                    "{} {} {} — {} error{} (output NOT updated)",
                    ts.dimmed(),
                    "✗".red().bold(),
                    source_path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .red()
                        .bold(),
                    errors.len(),
                    if errors.len() == 1 { "" } else { "s" },
                );
                for e in &errors {
                    eprintln!(
                        "  {}:{} {} [{}]: {}",
                        source_path.display().to_string().dimmed(),
                        e.source_line,
                        "error".red(),
                        e.code,
                        e.message,
                    );
                    if !e.uri.is_empty() {
                        eprintln!("    uri: {}", e.uri.dimmed());
                    }
                }
                eprintln!(
                    "  {} fix the errors above, then save to recompile",
                    "→".yellow()
                );
            }
        }
        Err(e) => {
            eprintln!(
                "{} {} {} — compile failed: {}",
                ts.dimmed(),
                "✗".red().bold(),
                source_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .red()
                    .bold(),
                e,
            );
            eprintln!("  {} output NOT updated", "→".yellow());
        }
    }
}

fn chrono_or_time() -> String {
    // Simple HH:MM:SS timestamp without pulling in chrono
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let h = (secs / 3600) % 24;
    let m = (secs / 60) % 60;
    let s = secs % 60;
    format!("{:02}:{:02}:{:02}", h, m, s)
}
