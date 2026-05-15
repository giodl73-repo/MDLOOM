use anyhow::{Context, Result};
use colored::Colorize;
use proof_lib::layout::{self, extract_content_lines, Align, Direction, LayoutConfig};
use std::path::PathBuf;
use std::process;

#[derive(clap::Args)]
pub(crate) struct Args {
    /// Source figures: md:// URIs or file paths
    sources: Vec<String>,
    /// Spaces between frames (default: 3)
    #[arg(long, default_value = "3")]
    gap: usize,
    /// Vertical alignment: top | center | bottom (default: top)
    #[arg(long, default_value = "top")]
    align: String,
    /// Labels above each frame (one per source, space-separated)
    #[arg(long, num_args = 0..)]
    labels: Vec<String>,
    /// Number of frames per row before wrapping (default: all)
    #[arg(long)]
    cols: Option<usize>,
    /// Max output width in columns (default: 120)
    #[arg(long, default_value = "120")]
    width: usize,
    /// Composition direction: horizontal | vertical (or h | v)
    #[arg(long, default_value = "horizontal")]
    direction: String,
    /// Add a border box around each frame
    #[arg(long)]
    border: bool,
    /// Write output to file (default: stdout)
    #[arg(short = 'o', long)]
    output: Option<PathBuf>,
    /// Root directory for md:// URI resolution (default: current directory)
    #[arg(long)]
    root: Option<PathBuf>,
}

pub(crate) fn run(args: Args) -> Result<()> {
    let Args {
        sources,
        gap,
        align: align_str,
        labels,
        cols,
        width,
        direction: direction_str,
        border,
        output,
        root,
    } = args;
    if sources.is_empty() {
        eprintln!(
            "{} no sources provided — pass md:// URIs or file paths",
            "error:".red()
        );
        process::exit(2);
    }

    let align = Align::parse(&align_str)?;
    let direction = Direction::parse(&direction_str)?;
    let root = root.unwrap_or_else(|| std::env::current_dir().unwrap());

    let config = LayoutConfig {
        gap,
        align,
        labels,
        cols,
        width,
        direction,
        border,
    };

    // Resolve each source to content lines.
    let mut figures: Vec<Vec<String>> = Vec::new();
    for source in &sources {
        let content = if source.starts_with("md://") {
            // Resolve via mdpath.
            let parsed = mdpath::parse(source)
                .map_err(|e| anyhow::anyhow!("invalid md:// URI {:?}: {}", source, e))?;
            let element = mdpath::resolve(&parsed, &root)
                .map_err(|e| anyhow::anyhow!("cannot resolve {:?}: {}", source, e))?;
            element.content
        } else {
            // File path — read and use whole file content.
            let path = root.join(source);
            std::fs::read_to_string(&path)
                .with_context(|| format!("reading figure file: {}", path.display()))?
        };
        figures.push(extract_content_lines(&content));
    }

    let result = layout::layout(figures, &config);

    match output {
        Some(path) => {
            std::fs::write(&path, &result)?;
            eprintln!("{} layout written to {}", "✓".green(), path.display());
        }
        None => println!("{}", result),
    }

    Ok(())
}
