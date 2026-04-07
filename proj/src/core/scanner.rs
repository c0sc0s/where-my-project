use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use anyhow::Result;
use walkdir::{DirEntry, WalkDir};

use crate::core::{git, models::ProjectInstance};

pub fn scan_repositories(paths: &[PathBuf]) -> Result<Vec<ProjectInstance>> {
    scan_repositories_with_progress(paths, &[], |_, _| {})
}

pub fn scan_repositories_with_progress<F>(
    paths: &[PathBuf],
    filters: &[String],
    mut on_progress: F,
) -> Result<Vec<ProjectInstance>>
where
    F: FnMut(&Path, usize),
{
    let filters = filters
        .iter()
        .map(|value| value.to_ascii_lowercase())
        .collect::<HashSet<_>>();
    let mut found_paths = HashSet::new();
    let mut instances = Vec::new();

    for path in paths {
        let root_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());

        if let Some(instance) = build_project_instance(&root_path, &filters, &mut found_paths) {
            on_progress(&root_path, instances.len());
            instances.push(instance);
            on_progress(&root_path, instances.len());
            continue;
        }

        let mut walker = WalkDir::new(path)
            .max_depth(20)
            .follow_links(false)
            .into_iter()
            .filter_entry(|entry| should_enter(entry));

        while let Some(entry) = walker.next() {
            let entry = match entry {
                Ok(entry) => entry,
                Err(_) => continue,
            };

            if !entry.file_type().is_dir() {
                continue;
            }

            on_progress(entry.path(), instances.len());

            let dir_path = entry.path();
            let canonical = dir_path
                .canonicalize()
                .unwrap_or_else(|_| dir_path.to_path_buf());

            let Some(instance) = build_project_instance(&canonical, &filters, &mut found_paths)
            else {
                continue;
            };

            instances.push(instance);
            on_progress(&canonical, instances.len());
            walker.skip_current_dir();
        }
    }

    instances.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(instances)
}

fn should_enter(entry: &DirEntry) -> bool {
    let name = entry.file_name().to_str().unwrap_or("");
    !matches!(
        name,
        ".git" | "node_modules" | "target" | ".next" | "dist" | "dist_sec" | "build" | "release"
    )
}

fn is_project_dir(path: &Path) -> bool {
    if path.join(".git").exists() {
        return true;
    }

    if path.join("package.json").exists() {
        return true;
    }

    if path.join("Cargo.toml").exists() {
        return true;
    }

    if path.join("go.mod").exists() {
        return true;
    }

    false
}

fn build_project_instance(
    canonical: &Path,
    filters: &HashSet<String>,
    found_paths: &mut HashSet<PathBuf>,
) -> Option<ProjectInstance> {
    if !is_project_dir(canonical) || !found_paths.insert(canonical.to_path_buf()) {
        return None;
    }

    let dir_name = canonical
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    // For git worktrees, .git is a file pointing back to the main repo.
    // The folder name differs from the actual repo name, so we must resolve it.
    let is_worktree = canonical.join(".git").is_file();
    let repo_name = if is_worktree {
        git::repo_name(canonical).unwrap_or_else(|_| dir_name.clone())
    } else {
        dir_name
    };

    if !filters.is_empty() && !filters.contains(&repo_name.to_ascii_lowercase()) {
        return None;
    }

    let branch = git::branch_name(canonical).ok();
    let clean_path = normalize_path(canonical);

    Some(ProjectInstance {
        repo_name,
        path: clean_path,
        alias: None,
        last_branch: branch,
        last_check: None,
    })
}

fn normalize_path(path: &Path) -> String {
    let path_str = path.to_string_lossy().to_string();
    // Remove Windows UNC prefix
    if path_str.starts_with(r"\\?\") {
        path_str[4..].to_string()
    } else {
        path_str
    }
}
