use anyhow::Result;
use clap::Parser;

mod cli;
mod cmd_backfill;
mod cmd_check;
mod cmd_compile;
mod cmd_config;
mod cmd_context;
mod cmd_crop;
mod cmd_depends;
mod cmd_draft;
mod cmd_fix;
mod cmd_init;
mod cmd_layout;
mod cmd_paths;
mod cmd_pin;
mod cmd_pin_list;
mod cmd_resolve;
mod cmd_spec_generate;
mod cmd_stats;
mod cmd_status;
mod cmd_tree;
mod dispatch;

use cli::Cli;

fn main() -> Result<()> {
    dispatch::run(Cli::parse())
}
