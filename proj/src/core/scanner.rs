use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use anyhow::Result;
use walkdir::{DirEntry, WalkDir};

use crate::core::{git, models::ProjectInstance};

pub fn scan_repositories(
    paths: &[PathBuf],
    watched_repos: &[String],
) -> Result<Vec<ProjectInstance>> {
    scan_repositories_with_progress(paths, watched_repos, |_, _| {})
}

pub fn scan_repositories_with_progress<F>(
    paths: &[PathBuf],
    watched_repos: &[String],
    mut on_progress: F,
) -> Result<Vec<ProjectInstance>>
where
    F: FnMut(&Path, usize),
{
    let watched: HashSet<&str> = watched_repos.iter().map(String::as_str).collect();
    let mut found_paths = HashSet::new();
    let mut instances = Vec::new();

    for path in paths {
        for entry in WalkDir::new(path)
            .follow_links(false)
            .into_iter()
            .filter_entry(|entry| should_enter(entry))
            .filter_map(|entry| entry.ok())
        {
            if !entry.file_type().is_dir() {
                continue;
            }

            on_progress(entry.path(), instances.len());

            let Some(repo_root) = repository_root(&entry) else {
                continue;
            };

            let canonical = repo_root
                .canonicalize()
                .unwrap_or_else(|_| repo_root.clone());
            if !found_paths.insert(canonical.clone()) {
                continue;
            }

            let repo_name = git::repo_name(&repo_root)?;
            if !watched.is_empty() && !watched.contains(repo_name.as_str()) {
                continue;
            }

            let branch = git::branch_name(&repo_root).ok();
            let clean_path = normalize_path(&canonical);
            instances.push(ProjectInstance {
                repo_name,
                path: clean_path,
                alias: None,
                last_branch: branch,
                last_check: None,
            });
            on_progress(&repo_root, instances.len());
        }
    }

    instances.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(instances)
}

fn should_enter(entry: &DirEntry) -> bool {
    entry.file_name().to_str() != Some(".git")
}

fn repository_root(entry: &DirEntry) -> Option<PathBuf> {
    let path = entry.path();
    if git::is_git_repository(path) {
        return Some(path.to_path_buf());
    }

    None
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
