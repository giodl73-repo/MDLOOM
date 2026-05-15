use anyhow::Result;
use colored::Colorize;
use std::path::PathBuf;

#[derive(clap::Args)]
pub(crate) struct Args {
    /// The md:// URI to resolve
    uri: String,
    /// Root directory (default: current directory, or where proof.toml lives)
    #[arg(short, long)]
    root: Option<PathBuf>,
    /// Output format: text (default) | json
    #[arg(short = 'f', long, default_value = "text")]
    format: String,
}

pub(crate) fn run(args: Args) -> Result<()> {
    let Args { uri, root, format } = args;
    let root = root.unwrap_or_else(|| std::env::current_dir().unwrap());

    let parsed = mdpath::parse(&uri).map_err(|e| anyhow::anyhow!("invalid md:// URI: {}", e))?;

    let element =
        mdpath::resolve(&parsed, &root).map_err(|e| anyhow::anyhow!("resolve failed: {}", e))?;

    match format.as_str() {
        "json" => {
            let json = serde_json::json!({
                "uri": element.uri,
                "file": element.file.display().to_string(),
                "line_start": element.line_start,
                "line_end": element.line_end,
                "element_type": format!("{:?}", element.element_type),
                "kind": element.kind,
                "label": element.label,
                "section_heading": element.section_heading,
                "content": element.content,
            });
            println!("{}", serde_json::to_string_pretty(&json)?);
        }
        _ => {
            // text format
            println!("{}", element.uri.cyan());
            if let Some(h) = &element.section_heading {
                println!("  section:  {}", h);
            }
            if let Some(label) = &element.label {
                println!("  label:    {}", label);
            }
            if let Some(kind) = &element.kind {
                println!("  kind:     {}", kind);
            }
            println!("  lines:    {}–{}", element.line_start, element.line_end);
            println!("  file:     {}", element.file.display());
            println!();
            println!("{}", element.content);
        }
    }

    Ok(())
}
