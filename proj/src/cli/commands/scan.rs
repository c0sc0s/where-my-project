use anyhow::Result;
use colored::Colorize;
use comfy_table::{
    modifiers::UTF8_ROUND_CORNERS, Cell, Color as TableColor, ContentArrangement, Table,
};
use indicatif::{ProgressBar, ProgressStyle};

use crate::{cli::args::ScanArgs, core::manager::ProjectManager, core::models::ProjectInstance};

pub fn run(args: ScanArgs) -> Result<()> {
    if args.tui {
        return crate::tui::scan::run(args);
    }

    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    spinner.set_message("Scanning for repositories...");
    spinner.enable_steady_tick(std::time::Duration::from_millis(80));

    let mut manager = ProjectManager::load()?;
    let instances = manager.scan(args.paths)?;

    spinner.finish_and_clear();

    if instances.is_empty() {
        println!("{}", "No repositories found.".yellow());
        println!(
            "{}",
            "Tip: Run 'proj scan <path>' on a directory that contains projects.".dimmed()
        );
        return Ok(());
    }

    print_scan_table(&instances);

    println!();
    println!(
        "{} Found {} project(s)",
        "✓".green().bold(),
        instances.len().to_string().white().bold()
    );

    Ok(())
}

fn print_scan_table(instances: &[ProjectInstance]) {
    let mut table = Table::new();

    table.apply_modifier(UTF8_ROUND_CORNERS);
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.set_header(vec![
        Cell::new("#").fg(TableColor::Cyan),
        Cell::new("Repository").fg(TableColor::Cyan),
        Cell::new("Branch").fg(TableColor::Cyan),
        Cell::new("Path").fg(TableColor::Cyan),
    ]);

    for (idx, instance) in instances.iter().enumerate() {
        let repo = &instance.repo_name;
        let branch = instance.last_branch.as_deref().unwrap_or("-");
        let path = shorten_path(&instance.path, 45);

        table.add_row(vec![
            Cell::new(idx + 1).fg(TableColor::DarkGrey),
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
