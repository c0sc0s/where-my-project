pub mod args;
pub mod commands;

use anyhow::Result;
use clap::Parser;

use crate::cli::{
    args::{Cli, Commands},
    commands::{cd, init, list, scan, status, version, work},
};

pub async fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::List(args)) => list::run(args),
        Some(Commands::Scan(args)) => scan::run(args),
        Some(Commands::Status(args)) => status::run(args),
        Some(Commands::Cd(args)) => cd::run(args),
        Some(Commands::Work(args)) => work::run(args),
        Some(Commands::Init) => init::run(),
        Some(Commands::Version) => version::run(),
        None => list::run(args::ListArgs {
            selection_file: None,
        }),
    }
}
