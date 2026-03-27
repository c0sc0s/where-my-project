use anyhow::Result;
use colored::Colorize;
use comfy_table::{
    modifiers::UTF8_ROUND_CORNERS, Cell, Color as TableColor, ContentArrangement, Table,
};

use crate::{cli::args::ScanArgs, core::manager::ProjectManager, core::models::ProjectInstance};

pub fn run(args: ScanArgs) -> Result<()> {
    if args.tui {
        return crate::tui::scan::run(args);
    }

    let mut manager = ProjectManager::load()?;
    let instances = manager.scan(args.paths, args.auto_alias)?;

    if instances.is_empty() {
        println!("{}", "No repositories found.".yellow());
        println!(
            "{}",
            "Tip: Run 'proj watch <repo-name>' to add a repository to watch list.".dimmed()
        );
        return Ok(());
    }

    print_scan_table(&instances);

    // 显示统计
    let with_alias = instances.iter().filter(|i| i.alias.is_some()).count();
    let new_found = instances.len();

    println!();
    println!(
        "{} Found {} instance(s) | {} with alias",
        "✓".green().bold(),
        new_found.to_string().white().bold(),
        with_alias.to_string().yellow()
    );

    if args.auto_alias {
        println!("{}", "Auto-generated aliases from branch names.".dimmed());
    }

    Ok(())
}

fn print_scan_table(instances: &[ProjectInstance]) {
    let mut table = Table::new();

    table.apply_modifier(UTF8_ROUND_CORNERS);
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.set_header(vec![
        Cell::new("#").fg(TableColor::Cyan),
        Cell::new("Alias").fg(TableColor::Cyan),
        Cell::new("Repository").fg(TableColor::Cyan),
        Cell::new("Branch").fg(TableColor::Cyan),
        Cell::new("Path").fg(TableColor::Cyan),
    ]);

    for (idx, instance) in instances.iter().enumerate() {
        let alias = instance.alias.as_deref().unwrap_or("-");
        let repo = &instance.repo_name;
        let branch = instance.last_branch.as_deref().unwrap_or("-");
        let path = shorten_path(&instance.path, 45);

        let alias_color = if instance.alias.is_some() {
            TableColor::Green
        } else {
            TableColor::DarkGrey
        };

        table.add_row(vec![
            Cell::new(idx + 1).fg(TableColor::DarkGrey),
            Cell::new(alias).fg(alias_color),
            Cell::new(repo),
            Cell::new(branch).fg(TableColor::Magenta),
            Cell::new(path).fg(TableColor::DarkGrey),
        ]);
    }

    println!("{}", table);
}

fn shorten_path(path: &str, max_len: usize) -> String {
    if path.len() <= max_len {
        path.to_string()
    } else {
        format!("...{}", &path[path.len().saturating_sub(max_len - 3)..])
    }
}
