use anyhow::Result;
use colored::Colorize;
use std::path::PathBuf;

#[derive(clap::Args)]
pub(crate) struct Args {
    /// The md:// URI to pin
    uri: String,
    /// Stable identifier for this pin
    #[arg(long, required = true)]
    id: String,
    /// Human description
    #[arg(long, default_value = "")]
    description: String,
    /// Template name for base invariants
    #[arg(long)]
    template: Option<String>,
    /// Protection tier: warn | error | lock
    #[arg(long, default_value = "warn")]
    protection: String,
    /// Root directory (default: current directory)
    #[arg(long)]
    root: Option<PathBuf>,
    /// Config file to update (default: proof.toml in current directory)
    #[arg(long)]
    config_file: Option<PathBuf>,
}

pub(crate) fn run(args: Args) -> Result<()> {
    let Args {
        uri,
        id,
        description,
        template,
        protection,
        root,
        config_file,
    } = args;
    let root = root.unwrap_or_else(|| std::env::current_dir().unwrap());
    let config_path = config_file.unwrap_or_else(|| root.join("proof.toml"));

    let parsed = mdpath::parse(&uri).map_err(|e| anyhow::anyhow!("invalid md:// URI: {}", e))?;
    let element = mdpath::resolve(&parsed, &root)
        .map_err(|e| anyhow::anyhow!("cannot resolve URI: {}", e))?;

    let stable_uri = element.uri.clone();

    let desc = if description.is_empty() {
        element.label.as_deref().unwrap_or("").to_string()
    } else {
        description
    };

    let template_line = template
        .as_deref()
        .map(|t| format!("\ntemplate = {:?}", t))
        .unwrap_or_default();

    let toml_snippet = format!(
        "\n[[davinci]]\nid = {:?}\nuri = {:?}\ndescription = {:?}\nprotection = {:?}{}\n",
        id, stable_uri, desc, protection, template_line
    );

    let existing = std::fs::read_to_string(&config_path).unwrap_or_default();
    if existing.contains(&format!("id = {:?}", id)) {
        eprintln!(
            "{} DaVinci '{}' already exists in {} — update it manually",
            "warn:".yellow(),
            id,
            config_path.display()
        );
        return Ok(());
    }
    std::fs::write(&config_path, format!("{}{}", existing, toml_snippet))?;

    eprintln!(
        "{} Pinned {} as '{}'",
        "✓".green().bold(),
        stable_uri.cyan(),
        id
    );
    eprintln!("  Kind: {}", element.kind.as_deref().unwrap_or("figure"));
    eprintln!("  Lines: {}–{}", element.line_start, element.line_end);
    if let Some(label) = &element.label {
        eprintln!("  Label: {}", label);
    }
    eprintln!();
    eprintln!("Add invariants to {}", config_path.display());
    eprintln!("Then run: proof check --daVinci .");

    Ok(())
}
