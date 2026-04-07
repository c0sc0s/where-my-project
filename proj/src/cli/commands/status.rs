use anyhow::Result;
use colored::Colorize;
use comfy_table::{
    modifiers::UTF8_ROUND_CORNERS, Cell, Color as TableColor, ContentArrangement, Table,
};

use crate::{cli::args::StatusArgs, core::manager::ProjectManager, core::models::InstanceStatus};

pub fn run(args: StatusArgs) -> Result<()> {
    let manager = ProjectManager::load()?;

    if let Some(target) = args.target {
        let status = manager.status_for(&target)?;
        print_single_status(&status);
        return Ok(());
    }

    let statuses = manager.statuses()?;

    if statuses.is_empty() {
        println!("{}", "No projects found. Run 'proj scan' first.".yellow());
        return Ok(());
    }

    print_status_table(&statuses);

    // 统计信息
    let total = statuses.len();
    let clean = statuses.iter().filter(|s| s.git_status.is_clean).count();
    let dirty = total - clean;

    println!();
    println!(
        "{} Total: {} | {} Clean: {} | {} Dirty: {}",
        "📊".to_string(),
        total.to_string().white().bold(),
        "✓".green(),
        clean.to_string().green(),
        "✗".red(),
        dirty.to_string().red()
    );

    Ok(())
}

fn print_status_table(statuses: &[InstanceStatus]) {
    let mut table = Table::new();

    table.apply_modifier(UTF8_ROUND_CORNERS);
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.set_header(vec![
        Cell::new("#").fg(TableColor::Cyan),
        Cell::new("Repository").fg(TableColor::Cyan),
        Cell::new("Branch").fg(TableColor::Cyan),
        Cell::new("Status").fg(TableColor::Cyan),
        Cell::new("Path").fg(TableColor::Cyan),
    ]);

    for (idx, status) in statuses.iter().enumerate() {
        let repo = &status.instance.repo_name;
        let branch = &status.git_status.branch;

        // 状态显示
        let status_str = if status.git_status.is_clean {
            "✓ clean".green().to_string()
        } else {
            let mut parts = vec![];
            if status.git_status.modified_count > 0 {
                parts.push(format!("{} modified", status.git_status.modified_count));
            }
            if status.git_status.untracked_count > 0 {
                parts.push(format!("{} untracked", status.git_status.untracked_count));
            }
            if status.git_status.ahead_count > 0 {
                parts.push(format!("↑{} ahead", status.git_status.ahead_count));
            }
            format!("✗ {}", parts.join(", ")).red().to_string()
        };

        // 缩短路径显示
        let path = shorten_path(&status.instance.path, 40);

        table.add_row(vec![
            Cell::new(idx + 1).fg(TableColor::DarkGrey),
            Cell::new(repo),
            Cell::new(branch).fg(TableColor::Magenta),
            Cell::new(status_str),
            Cell::new(path).fg(TableColor::DarkGrey),
        ]);
    }

    println!("{}", table);
}

fn print_single_status(status: &InstanceStatus) {
    println!(
        "{} {}",
        "Repository:".cyan().bold(),
        status.instance.repo_name
    );
    println!("{} {}", "Path:".cyan().bold(), status.instance.path);
    println!(
        "{} {}",
        "Branch:".cyan().bold(),
        status.git_status.branch.magenta()
    );

    let clean_status = if status.git_status.is_clean {
        "✓ Clean".green()
    } else {
        "✗ Dirty".red()
    };
    println!("{} {}", "Status:".cyan().bold(), clean_status);

    if !status.git_status.is_clean {
        println!(
            "  {} {}",
            "Modified:".yellow(),
            status.git_status.modified_count
        );
        println!(
            "  {} {}",
            "Untracked:".yellow(),
            status.git_status.untracked_count
        );
    }

    if status.git_status.ahead_count > 0 {
        println!(
            "  {} {} commits ahead",
            "↑".yellow(),
            status.git_status.ahead_count
        );
    }
}

fn shorten_path(path: &str, max_len: usize) -> String {
    if path.len() <= max_len {
        path.to_string()
    } else {
        format!("...{}", &path[path.len().saturating_sub(max_len - 3)..])
    }
}
