use anyhow::Result;
use std::path::PathBuf;

pub(crate) fn paths_or_cwd(paths: Vec<PathBuf>) -> Result<Vec<PathBuf>> {
    if paths.is_empty() {
        Ok(vec![std::env::current_dir()?])
    } else {
        Ok(paths)
    }
}

pub(crate) fn check_paths_or_cwd(
    command_paths: Vec<PathBuf>,
    top_level_paths: &[PathBuf],
) -> Result<Vec<PathBuf>> {
    if !command_paths.is_empty() {
        Ok(command_paths)
    } else if !top_level_paths.is_empty() {
        Ok(top_level_paths.to_vec())
    } else {
        Ok(vec![std::env::current_dir()?])
    }
}
