use anyhow::Result;

use crate::{cli::args::CdArgs, core::manager::ProjectManager};

pub fn run(args: CdArgs) -> Result<()> {
    let manager = ProjectManager::load()?;
    let path = manager.resolve_path(&args.target)?;

    if args.raw {
        println!("{path}");
    } else {
        println!("Set-Location {path}");
    }

    Ok(())
}
