use std::{
    io::{self, Write},
    path::{Path, PathBuf},
};

use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use walkdir::{DirEntry, WalkDir};

use crate::{
    cli::args::ScanArgs,
    core::{manager::ProjectManager, models::ProjectInstance},
};

pub fn run(args: ScanArgs) -> Result<()> {
    let mut manager = ProjectManager::load()?;
    let scan_paths = resolve_scan_paths(&manager, args.paths.clone());
    let total = count_scan_steps(&scan_paths);

    let progress = ProgressBar::new(total.max(1));
    progress.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} repos:{msg}",
        )?
        .progress_chars("##-"),
    );

    let scan_result = manager.scan_with_progress(args.paths, args.auto_alias, |path, found| {
        progress.set_position((progress.position() + 1).min(progress.length().unwrap_or(1)));
        progress.set_message(format!("{} | {}", found, shorten_path(path)));
    });

    progress.finish_and_clear();
    let instances = scan_result?;

    if instances.is_empty() {
        println!("No repositories found.");
        return Ok(());
    }

    println!("Scanned {} repos.", instances.len());
    prompt_for_aliases(&mut manager, &instances)?;
    Ok(())
}

fn prompt_for_aliases(manager: &mut ProjectManager, instances: &[ProjectInstance]) -> Result<()> {
    let mut stdout = io::stdout();
    let stdin = io::stdin();

    for (index, instance) in instances.iter().enumerate() {
        if instance.alias.is_some() {
            continue;
        }

        writeln!(
            stdout,
            "[{}] {} ({})",
            index + 1,
            instance.repo_name,
            instance.path
        )?;
        write!(stdout, "Alias (leave empty to skip): ")?;
        stdout.flush()?;

        let mut input = String::new();
        stdin.read_line(&mut input)?;
        let alias = input.trim();
        if alias.is_empty() {
            continue;
        }

        manager.set_alias(&instance.path, alias.to_string())?;
    }

    Ok(())
}

fn resolve_scan_paths(manager: &ProjectManager, paths: Option<Vec<String>>) -> Vec<PathBuf> {
    if let Some(paths) = paths {
        return paths.into_iter().map(PathBuf::from).collect();
    }

    if !manager.config().scan_paths.is_empty() {
        return manager
            .config()
            .scan_paths
            .iter()
            .map(PathBuf::from)
            .collect();
    }

    vec![std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))]
}

fn count_scan_steps(paths: &[PathBuf]) -> u64 {
    let mut total = 0_u64;
    for path in paths {
        total += WalkDir::new(path)
            .follow_links(false)
            .into_iter()
            .filter_entry(should_enter)
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().is_dir())
            .count() as u64;
    }
    total
}

fn should_enter(entry: &DirEntry) -> bool {
    entry.file_name().to_str() != Some(".git")
}

fn shorten_path(path: &Path) -> String {
    let display = path.display().to_string();
    if display.chars().count() <= 60 {
        return display;
    }

    let mut chars: Vec<char> = display.chars().collect();
    let keep = 57.min(chars.len());
    let tail = chars.split_off(chars.len() - keep);
    format!("...{}", tail.into_iter().collect::<String>())
}
