pub mod args;
pub mod commands;

use anyhow::Result;
use clap::Parser;

use crate::cli::{
    args::{Cli, Commands},
    commands::{alias, cd, init, list, scan, status, version, watch, work},
};

pub async fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Watch(args) => watch::run(args),
        Commands::List(args) => list::run(args),
        Commands::Scan(args) => scan::run(args),
        Commands::Alias(args) => alias::run(args),
        Commands::Status(args) => status::run(args),
        Commands::Cd(args) => cd::run(args),
        Commands::Work(args) => work::run(args),
        Commands::Init => init::run(),
        Commands::Version => version::run(),
    }
}
