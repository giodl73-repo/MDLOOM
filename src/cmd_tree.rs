use anyhow::{Context, Result};
use clap::Subcommand;
use colored::Colorize;
use proof_lib::tree::dirtree::{generate as dirtree_generate, DirtreeOptions, SortOrder};
use proof_lib::tree::schema::{
    generate_dependency, generate_org, generate_outline, generate_taxonomy, FieldMap,
};
use std::path::PathBuf;

#[derive(clap::Args)]
pub(crate) struct Args {
    #[command(subcommand)]
    action: TreeAction,
}

#[derive(Subcommand)]
enum TreeAction {
    /// Generate a dirtree from the filesystem
    Generate {
        /// Tree kind: dirtree | org | taxonomy | dependency | outline (default: dirtree)
        #[arg(long, default_value = "dirtree")]
        kind: String,
        /// Source URI for schema-driven kinds (md://path#section:table:0 or md://path.json)
        /// Not needed for dirtree (uses --root instead)
        source: Option<String>,
        // ── dirtree options ──────────────────────────
        /// Root directory to walk (dirtree only)
        #[arg(long, default_value = ".")]
        root: PathBuf,
        /// Max depth to recurse (dirtree only)
        #[arg(long)]
        max_depth: Option<usize>,
        /// Glob patterns to exclude, comma-separated (dirtree only)
        #[arg(long)]
        exclude: Option<String>,
        /// Sort order: name | ext | size | mtime
        #[arg(long, default_value = "name")]
        sort: String,
        /// Directories before files (dirtree only, default: true)
        #[arg(long, default_value = "true")]
        dirs_first: bool,
        // ── schema-driven field mapping ───────────────
        /// Column/field name for the node label (auto-detected if omitted)
        #[arg(long)]
        name: Option<String>,
        /// Column/field name for the parent reference (auto-detected if omitted)
        #[arg(long)]
        parent: Option<String>,
        /// Column/field name for display text (auto-detected if omitted)
        #[arg(long)]
        label: Option<String>,
        /// Source data format: table | json (default: table)
        #[arg(long, default_value = "table")]
        format: String,
        /// Value that marks a root node (default: —, -, null, empty)
        #[arg(long)]
        root_marker: Option<String>,
        // ── shared ────────────────────────────────────
        /// Indent width per level (default: 4)
        #[arg(long, default_value = "4")]
        indent_width: usize,
        /// Don't wrap output in a dirtree/tree fence
        #[arg(long)]
        no_fence: bool,
        /// Root directory for md:// resolution (default: cwd)
        #[arg(long)]
        resolve_root: Option<PathBuf>,
        /// Write output to file (default: stdout)
        #[arg(short = 'o', long)]
        output: Option<PathBuf>,
    },
}

pub(crate) fn run(args: Args) -> Result<()> {
    match args.action {
        TreeAction::Generate {
            kind,
            source,
            root,
            max_depth,
            exclude,
            sort,
            dirs_first,
            name,
            parent,
            label,
            format,
            root_marker,
            indent_width,
            no_fence,
            resolve_root,
            output,
        } => {
            let result = match kind.as_str() {
                "dirtree" => {
                    let sort_order = match sort.as_str() {
                        "ext" => SortOrder::Ext,
                        "size" => SortOrder::Size,
                        "mtime" => SortOrder::Mtime,
                        _ => SortOrder::Name,
                    };
                    let exclude_patterns = exclude
                        .map(|s| s.split(',').map(|p| p.trim().to_string()).collect())
                        .unwrap_or_default();
                    let opts = DirtreeOptions {
                        root,
                        max_depth,
                        exclude: exclude_patterns,
                        dirs_first,
                        sort: sort_order,
                        wrap_fence: !no_fence,
                        indent_width,
                    };
                    dirtree_generate(&opts)?
                }
                other => {
                    let src_uri = source.ok_or_else(|| {
                        anyhow::anyhow!(
                            "proof tree generate --kind {} requires a source URI argument\n\
                             Example: proof tree generate --kind org md://docs/team.md#:table:0",
                            other
                        )
                    })?;

                    let resolve_from =
                        resolve_root.unwrap_or_else(|| std::env::current_dir().unwrap());
                    let content = resolve_source(&src_uri, &resolve_from)?;

                    let mut field_map = FieldMap {
                        name,
                        parent,
                        label,
                        root_marker,
                        ..Default::default()
                    };

                    let body = match other {
                        "org" => generate_org(&content, &format, &mut field_map, indent_width)?,
                        "taxonomy" => {
                            generate_taxonomy(&content, &format, &mut field_map, indent_width)?
                        }
                        "dependency" => {
                            generate_dependency(&content, &format, &mut field_map, indent_width)?
                        }
                        "outline" => generate_outline(&content, indent_width)?,
                        unknown => anyhow::bail!(
                            "unknown tree kind {:?} — use dirtree, org, taxonomy, dependency, or outline",
                            unknown
                        ),
                    };

                    if no_fence {
                        body
                    } else {
                        format!("```{}\n{}\n```", other, body)
                    }
                }
            };

            match output {
                Some(path) => {
                    std::fs::write(&path, &result)?;
                    eprintln!(
                        "{} {} tree written to {}",
                        "✓".green(),
                        kind,
                        path.display()
                    );
                }
                None => println!("{}", result),
            }
        }
    }
    Ok(())
}

/// Resolve a source — md:// URI via mdpath, or plain file path.
fn resolve_source(src: &str, root: &std::path::Path) -> Result<String> {
    if src.starts_with("md://") {
        let parsed = mdpath::parse(src)
            .map_err(|e| anyhow::anyhow!("invalid md:// URI {:?}: {}", src, e))?;
        let element = mdpath::resolve(&parsed, root)
            .map_err(|e| anyhow::anyhow!("cannot resolve {:?}: {}", src, e))?;
        Ok(element.content)
    } else {
        // Plain file path
        let path = root.join(src);
        std::fs::read_to_string(&path)
            .with_context(|| format!("reading source file: {}", path.display()))
    }
}
