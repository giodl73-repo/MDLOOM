use anyhow::Result;
use colored::Colorize;

pub(crate) fn run() -> Result<()> {
    let path = std::path::Path::new("proof.toml");
    if path.exists() {
        eprintln!("{} proof.toml already exists", "warning:".yellow());
        return Ok(());
    }
    let content = include_str!("../schemas/default.toml");
    std::fs::write(path, content)?;
    println!("{} proof.toml created", "OK".green().bold());
    Ok(())
}
