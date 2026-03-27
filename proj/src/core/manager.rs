use std::{
    collections::{HashMap, HashSet},
    env,
    path::PathBuf,
};

use anyhow::{anyhow, bail, Result};
use chrono::Utc;

use crate::core::{
    git,
    models::{Config, InstanceStatus, ProjectInstance},
    scanner, storage,
};

pub struct ProjectManager {
    config: Config,
}

impl ProjectManager {
    pub fn load() -> Result<Self> {
        Ok(Self {
            config: storage::load_config()?,
        })
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn add_watched_repo(&mut self, repo_name: String) -> Result<()> {
        if !self
            .config
            .watched_repos
            .iter()
            .any(|item| item == &repo_name)
        {
            self.config.watched_repos.push(repo_name);
            self.config.watched_repos.sort();
            self.config.watched_repos.dedup();
            self.save()?;
        }
        Ok(())
    }

    pub fn remove_watched_repo(&mut self, repo_name: &str) -> Result<bool> {
        let before = self.config.watched_repos.len();
        self.config.watched_repos.retain(|item| item != repo_name);
        let removed = before != self.config.watched_repos.len();
        if removed {
            self.save()?;
        }
        Ok(removed)
    }

    pub fn watched_repos(&self) -> &[String] {
        &self.config.watched_repos
    }

    pub fn scan(
        &mut self,
        paths: Option<Vec<String>>,
        auto_alias: bool,
    ) -> Result<Vec<ProjectInstance>> {
        self.scan_with_progress(paths, auto_alias, |_, _| {})
    }

    pub fn scan_with_progress<F>(
        &mut self,
        paths: Option<Vec<String>>,
        auto_alias: bool,
        on_progress: F,
    ) -> Result<Vec<ProjectInstance>>
    where
        F: FnMut(&std::path::Path, usize),
    {
        let scan_paths = self.resolve_scan_paths(paths);
        let scan_path_strings: Vec<String> = scan_paths
            .iter()
            .map(|path| path.to_string_lossy().to_string())
            .collect();

        self.merge_scan_paths(&scan_path_strings);
        let scanned = scanner::scan_repositories_with_progress(
            &scan_paths,
            &self.config.watched_repos,
            on_progress,
        )?;

        let alias_by_path: HashMap<String, String> = self
            .config
            .instances
            .iter()
            .filter_map(|instance| {
                instance
                    .alias
                    .clone()
                    .map(|alias| (instance.path.clone(), alias))
            })
            .collect();

        let mut merged = Vec::new();
        let mut existing_aliases = HashSet::new();

        for mut instance in scanned {
            instance.last_check = Some(Utc::now());
            instance.alias = alias_by_path.get(&instance.path).cloned();

            if auto_alias && instance.alias.is_none() {
                instance.alias = Some(self.generate_auto_alias(&instance, &existing_aliases));
            }

            if let Some(alias) = &instance.alias {
                existing_aliases.insert(alias.clone());
            }

            merged.push(instance);
        }

        self.config.instances = merged.clone();
        self.save()?;
        Ok(merged)
    }

    pub fn set_alias(&mut self, target: &str, alias: String) -> Result<ProjectInstance> {
        self.ensure_alias_available(&alias, Some(target))?;
        let index = self.resolve_instance_index(target)?;
        self.config.instances[index].alias = Some(alias);
        let instance = self.config.instances[index].clone();
        self.save()?;
        Ok(instance)
    }

    pub fn alias_mappings(&self) -> Vec<(usize, &ProjectInstance)> {
        self.config
            .instances
            .iter()
            .enumerate()
            .filter(|(_, instance)| instance.alias.is_some())
            .collect()
    }

    pub fn statuses(&self) -> Result<Vec<InstanceStatus>> {
        self.config
            .instances
            .iter()
            .cloned()
            .map(|instance| {
                let git_status = git::read_status(PathBuf::from(&instance.path).as_path())?;
                Ok(InstanceStatus {
                    instance,
                    git_status,
                })
            })
            .collect()
    }

    pub fn status_for(&self, target: &str) -> Result<InstanceStatus> {
        let index = self.resolve_instance_index(target)?;
        let instance = self.config.instances[index].clone();
        let git_status = git::read_status(PathBuf::from(&instance.path).as_path())?;
        Ok(InstanceStatus {
            instance,
            git_status,
        })
    }

    pub fn resolve_path(&self, target: &str) -> Result<String> {
        let index = self.resolve_instance_index(target)?;
        Ok(self.config.instances[index].path.clone())
    }

    pub fn init_script(&self) -> String {
        r#"function projcd {
    param([string]$name)
    if (-not $name) {
        Write-Host "Usage: projcd <alias|index>" -ForegroundColor Yellow
        return
    }

    $path = proj cd $name --raw 2>$null
    if ($path) {
        $path = $path.Trim()
        if (Test-Path $path) {
            Set-Location $path
            return
        }
    }

    Write-Host "Project '$name' not found" -ForegroundColor Red
}

function projlist {
    $selectionFile = [System.IO.Path]::GetTempFileName()
    try {
        proj list --selection-file $selectionFile
        if (Test-Path $selectionFile) {
            $path = Get-Content $selectionFile -Raw
            if ($path) {
                $path = $path.Trim()
                if (Test-Path $path) {
                    Set-Location $path
                }
            }
        }
    } finally {
        Remove-Item $selectionFile -ErrorAction SilentlyContinue
    }
}

Set-Alias -Name pcd -Value projcd
Set-Alias -Name pl -Value projlist"#
            .to_string()
    }

    fn save(&self) -> Result<()> {
        storage::save_config(&self.config)
    }

    fn resolve_scan_paths(&self, paths: Option<Vec<String>>) -> Vec<PathBuf> {
        let mut resolved = if let Some(paths) = paths {
            paths.into_iter().map(PathBuf::from).collect::<Vec<_>>()
        } else if !self.config.scan_paths.is_empty() {
            self.config
                .scan_paths
                .iter()
                .map(PathBuf::from)
                .collect::<Vec<_>>()
        } else {
            vec![env::current_dir().unwrap_or_else(|_| PathBuf::from("."))]
        };

        resolved.sort();
        resolved.dedup();
        resolved
    }

    fn merge_scan_paths(&mut self, scan_paths: &[String]) {
        self.config.scan_paths.extend(scan_paths.iter().cloned());
        self.config.scan_paths.sort();
        self.config.scan_paths.dedup();
    }

    fn generate_auto_alias(
        &self,
        instance: &ProjectInstance,
        existing: &HashSet<String>,
    ) -> String {
        let branch = instance
            .last_branch
            .as_deref()
            .map(sanitize_alias)
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| sanitize_alias(&instance.repo_name));

        if !existing.contains(&branch) {
            return branch;
        }

        let repo_prefix = sanitize_alias(&instance.repo_name);
        let mut index = 2;
        loop {
            let candidate = format!("{repo_prefix}-{index}");
            if !existing.contains(&candidate) {
                return candidate;
            }
            index += 1;
        }
    }

    fn ensure_alias_available(&self, alias: &str, current_target: Option<&str>) -> Result<()> {
        for (index, instance) in self.config.instances.iter().enumerate() {
            if instance.alias.as_deref() != Some(alias) {
                continue;
            }

            let instance_key = (index + 1).to_string();
            let same_target = current_target
                .map(|target| target == instance_key || target == instance.path || target == alias)
                .unwrap_or(false);

            if !same_target {
                bail!("alias '{}' is already used by {}", alias, instance.path);
            }
        }
        Ok(())
    }

    fn resolve_instance_index(&self, target: &str) -> Result<usize> {
        if let Ok(index) = target.parse::<usize>() {
            if index == 0 || index > self.config.instances.len() {
                bail!("instance index {} is out of range", index);
            }
            return Ok(index - 1);
        }

        self.config
            .instances
            .iter()
            .position(|instance| {
                instance.path == target
                    || instance.alias.as_deref() == Some(target)
                    || format!("{}/", instance.path) == target
            })
            .ok_or_else(|| anyhow!("instance '{}' not found", target))
    }
}

fn sanitize_alias(value: &str) -> String {
    let sanitized = value
        .chars()
        .map(|ch| match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' => ch.to_ascii_lowercase(),
            '-' | '_' => ch,
            _ => '-',
        })
        .collect::<String>();

    sanitized
        .trim_matches('-')
        .split('-')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}
