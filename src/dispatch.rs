use anyhow::Result;
use std::path::PathBuf;

use crate::cli::{Cli, Command, DispatchInput};
use crate::cmd_context::GlobalOptions;
use crate::{
    cmd_backfill, cmd_check, cmd_compile, cmd_config, cmd_crop, cmd_depends, cmd_draft, cmd_fix,
    cmd_init, cmd_layout, cmd_pin, cmd_pin_list, cmd_resolve, cmd_spec_generate, cmd_stats,
    cmd_status, cmd_tree,
};

pub(crate) fn run(cli: Cli) -> Result<()> {
    DispatchContext::from_cli(cli).run()
}

struct DispatchContext {
    command: Option<Command>,
    top_level_paths: Vec<PathBuf>,
    globals: GlobalOptions,
}

impl DispatchContext {
    fn from_cli(cli: Cli) -> Self {
        Self::from_input(cli.into_dispatch())
    }

    fn from_input(mut input: DispatchInput) -> Self {
        let command = input.take_command();
        let top_level_paths = input.take_top_level_paths();
        let globals = input.take_globals();
        Self {
            command,
            top_level_paths,
            globals,
        }
    }

    fn run(self) -> Result<()> {
        let Self {
            command,
            top_level_paths,
            globals,
        } = self;

        match command {
            Some(Command::Backfill(args)) => cmd_backfill::run(args),
            Some(Command::Fix(args)) => cmd_fix::run_with_globals(args, &globals),
            Some(Command::Draft(args)) => cmd_draft::run_with_globals(args, &globals),
            Some(Command::Resolve(args)) => cmd_resolve::run(args),
            Some(Command::Depends(args)) => cmd_depends::run(args),
            Some(Command::Pin(args)) => cmd_pin::run(args),
            Some(Command::PinList) => cmd_pin_list::run_with_globals(&globals),
            Some(Command::Init) => cmd_init::run(),
            Some(Command::Status(args)) => cmd_status::run_with_globals(args, &globals),
            Some(Command::Config(args)) => cmd_config::run_with_globals(args, &globals),
            Some(Command::Crop(args)) => cmd_crop::run_with_globals(args, &globals),
            Some(Command::Stats(args)) => cmd_stats::run_with_globals(args, &globals),
            Some(Command::Tree(args)) => cmd_tree::run(args),
            Some(Command::SpecGenerate(args)) => {
                cmd_spec_generate::run_with_globals(args, &globals)
            }
            Some(Command::Compile(args)) => cmd_compile::run_with_globals(args, &globals),
            Some(Command::Layout(args)) => cmd_layout::run(args),
            Some(Command::Check(args)) => cmd_check::run_command(args, &top_level_paths, &globals),
            None => cmd_check::run_default(&top_level_paths, &globals),
        }
    }
}
