use anyhow::Result;

use crate::core::manager::ProjectManager;

pub fn run() -> Result<()> {
    let manager = ProjectManager::load()?;
    println!("{}", manager.init_script());
    Ok(())
}
