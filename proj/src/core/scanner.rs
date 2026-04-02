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
            .max_depth(20)
            .follow_links(false)
            .into_iter()
            .filter_entry(|entry| should_enter(entry))
            .filter_map(|entry| entry.ok())
        {
            if !entry.file_type().is_dir() {
                continue;
            }

            on_progress(entry.path(), instances.len());

            if !is_project_dir(&entry) {
                continue;
            }

            let dir_path = entry.path();
            let canonical = dir_path.canonicalize().unwrap_or_else(|_| dir_path.to_path_buf());

            if !found_paths.insert(canonical.clone()) {
                continue;
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
                git::repo_name(&canonical).unwrap_or_else(|_| dir_name.clone())
            } else {
                dir_name
            };

            if !watched.is_empty() && !watched.contains(repo_name.as_str()) {
                continue;
            }

            let branch = git::branch_name(&canonical).ok();
            let clean_path = normalize_path(&canonical);

            instances.push(ProjectInstance {
                repo_name,
                path: clean_path,
                alias: None,
                last_branch: branch,
                last_check: None,
            });
            on_progress(&canonical, instances.len());
        }
    }

    instances.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(instances)
}

fn should_enter(entry: &DirEntry) -> bool {
    let name = entry.file_name().to_str().unwrap_or("");
    name != ".git" && name != "node_modules" && name != "target" && name != ".next"
}

fn is_project_dir(entry: &DirEntry) -> bool {
    let path = entry.path();

    // 有 .git 的是项目
    if path.join(".git").exists() {
        return true;
    }

    // 有 package.json 的是项目
    if path.join("package.json").exists() {
        return true;
    }

    // 有 Cargo.toml 的是项目
    if path.join("Cargo.toml").exists() {
        return true;
    }

    // 有 go.mod 的是项目
    if path.join("go.mod").exists() {
        return true;
    }

    false
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
