use std::{
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{anyhow, bail, Context, Result};

use crate::core::models::GitStatus;

pub fn is_git_repository(path: &Path) -> bool {
    path.join(".git").exists()
}

pub fn repo_name(path: &Path) -> Result<String> {
    let top_level = git_output(path, &["rev-parse", "--show-toplevel"])?;
    let top_level = PathBuf::from(top_level.trim());
    let name = top_level
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| {
            anyhow!(
                "failed to derive repository name from {}",
                top_level.display()
            )
        })?;
    Ok(name.to_string())
}

pub fn branch_name(path: &Path) -> Result<String> {
    git_output(path, &["rev-parse", "--abbrev-ref", "HEAD"]).map(|value| value.trim().to_string())
}

pub fn read_status(path: &Path) -> Result<GitStatus> {
    let output = git_output(path, &["status", "--porcelain=1", "--branch"])?;
    let mut lines = output.lines();
    let header = lines.next().unwrap_or("## HEAD");
    let branch = parse_branch(header);
    let ahead_count = parse_ahead_count(header);

    let mut modified_count = 0;
    let mut untracked_count = 0;

    for line in lines {
        if line.starts_with("??") {
            untracked_count += 1;
        } else if !line.trim().is_empty() {
            modified_count += 1;
        }
    }

    Ok(GitStatus {
        branch,
        is_clean: modified_count == 0 && untracked_count == 0,
        modified_count,
        untracked_count,
        ahead_count,
    })
}

fn git_output(path: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(path)
        .output()
        .with_context(|| format!("failed to execute git in {}", path.display()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        bail!(
            "git {} failed for {}: {}",
            args.join(" "),
            path.display(),
            stderr
        );
    }

    String::from_utf8(output.stdout).context("git output was not valid UTF-8")
}

fn parse_branch(header: &str) -> String {
    let raw = header.trim_start_matches("## ").trim();
    raw.split("...").next().unwrap_or(raw).trim().to_string()
}

fn parse_ahead_count(header: &str) -> usize {
    let marker = "ahead ";
    header
        .find(marker)
        .and_then(|index| {
            let digits = &header[index + marker.len()..];
            let digits: String = digits
                .chars()
                .take_while(|ch| ch.is_ascii_digit())
                .collect();
            digits.parse::<usize>().ok()
        })
        .unwrap_or(0)
}
