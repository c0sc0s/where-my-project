use anyhow::Result;
use colored::Colorize;

use crate::{cli::args::WorkArgs, core::manager::ProjectManager};

pub fn run(args: WorkArgs) -> Result<()> {
    let manager = ProjectManager::load()?;
    let path = manager.resolve_path(&args.target)?;

    // 输出路径供 shell 使用
    println!("{}", path);

    // 显示友好信息
    let instance = manager
        .statuses()?
        .into_iter()
        .find(|s| s.instance.path == path)
        .map(|s| s.instance);

    if let Some(inst) = instance {
        eprintln!(
            "{} {}  {}",
            "▶ Working in:".cyan().bold(),
            inst.repo_name.yellow(),
            inst.path.dimmed()
        );
    }

    Ok(())
}
