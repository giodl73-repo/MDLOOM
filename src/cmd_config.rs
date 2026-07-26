use crate::cmd_context::GlobalOptions;
use anyhow::Result;
use mdloom_lib::lint::load_config_for_path;
use mdloom_lib::MdloomConfig;
use std::path::PathBuf;

#[derive(clap::Args)]
pub(crate) struct Args {
    /// File or directory whose effective config should be printed
    #[arg(default_value = ".")]
    path: PathBuf,
}

pub(crate) fn run_with_globals(args: Args, globals: &GlobalOptions) -> Result<()> {
    run(args, globals.config())
}

fn run(args: Args, config_override: &Option<PathBuf>) -> Result<()> {
    let path = args.path;
    let cfg = if config_override.is_some() {
        load_config_for_path(&path, config_override)?
    } else {
        let probe_path = if path.is_dir() {
            path.join("__mdloom_config_probe__.md")
        } else {
            path.clone()
        };
        let root = std::env::current_dir()?;
        MdloomConfig::resolve_for(&probe_path, &root)
    };
    print!("{}", toml::to_string_pretty(&cfg)?);
    Ok(())
}
