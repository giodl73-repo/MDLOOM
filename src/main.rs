#![allow(
    clippy::clone_on_copy,
    clippy::cloned_ref_to_slice_refs,
    clippy::collapsible_if,
    clippy::cmp_owned,
    clippy::double_ended_iterator_last,
    clippy::manual_strip,
    clippy::needless_range_loop,
    clippy::ptr_arg,
    clippy::too_many_arguments,
    clippy::type_complexity,
    clippy::unnecessary_map_or,
    clippy::unwrap_or_default
)]

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
mod cmd_index;
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
