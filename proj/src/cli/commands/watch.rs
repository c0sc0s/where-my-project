use anyhow::{bail, Result};

use crate::{cli::args::WatchArgs, core::manager::ProjectManager};

pub fn run(args: WatchArgs) -> Result<()> {
    let mut manager = ProjectManager::load()?;

    if args.list {
        for repo in manager.watched_repos() {
            println!("{repo}");
        }
        return Ok(());
    }

    if let Some(repo_name) = args.remove {
        let removed = manager.remove_watched_repo(&repo_name)?;
        if removed {
            println!("removed watch: {repo_name}");
        } else {
            println!("watch not found: {repo_name}");
        }
        return Ok(());
    }

    let Some(repo_name) = args.repo_name else {
        bail!("usage: proj watch <repo-name> | --list | --remove <repo-name>");
    };

    manager.add_watched_repo(repo_name.clone())?;
    println!("watching: {repo_name}");
    Ok(())
}
