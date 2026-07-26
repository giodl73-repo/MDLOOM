use anyhow::Result;
use colored::Colorize;

pub(crate) fn run() -> Result<()> {
    let path = std::path::Path::new("mdloom.toml");
    if path.exists() {
        eprintln!("{} mdloom.toml already exists", "warning:".yellow());
        return Ok(());
    }
    let content = include_str!("../schemas/default.toml");
    std::fs::write(path, content)?;
    println!("{} mdloom.toml created", "OK".green().bold());
    Ok(())
}
