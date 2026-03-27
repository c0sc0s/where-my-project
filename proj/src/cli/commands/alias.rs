use anyhow::{bail, Result};

use crate::{cli::args::AliasArgs, core::manager::ProjectManager};

pub fn run(args: AliasArgs) -> Result<()> {
    let mut manager = ProjectManager::load()?;

    if args.list {
        for (index, instance) in manager.alias_mappings() {
            println!(
                "[{}] {} -> {}",
                index + 1,
                instance.alias.as_deref().unwrap_or("-"),
                instance.path
            );
        }
        return Ok(());
    }

    let Some(target) = args.target else {
        bail!("usage: proj alias <index|path> <alias-name> | --list");
    };
    let Some(alias_name) = args.alias_name else {
        bail!("usage: proj alias <index|path> <alias-name> | --list");
    };

    let instance = manager.set_alias(&target, alias_name.clone())?;
    println!("alias '{}' -> {}", alias_name, instance.path);
    Ok(())
}
