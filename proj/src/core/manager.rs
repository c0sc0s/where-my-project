use std::{env, path::PathBuf};

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

struct ScanRequest {
    scan_roots: Vec<PathBuf>,
    filters: Vec<String>,
    remembered_roots: Vec<PathBuf>,
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

    pub fn scan(&mut self, paths: Vec<String>) -> Result<Vec<ProjectInstance>> {
        self.scan_with_progress(paths, |_, _| {})
    }

    pub fn scan_with_progress<F>(
        &mut self,
        paths: Vec<String>,
        on_progress: F,
    ) -> Result<Vec<ProjectInstance>>
    where
        F: FnMut(&std::path::Path, usize),
    {
        let request = self.resolve_scan_request(&paths);
        self.merge_scan_paths(&request.remembered_roots);

        let mut instances = scanner::scan_repositories_with_progress(
            &request.scan_roots,
            &request.filters,
            on_progress,
        )?;
        let checked_at = Utc::now();

        for instance in &mut instances {
            instance.last_check = Some(checked_at);
        }

        self.config.instances = instances.clone();
        self.save()?;
        Ok(instances)
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
        Write-Host "Usage: projcd <repo|index|path>" -ForegroundColor Yellow
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

    fn resolve_scan_request(&self, inputs: &[String]) -> ScanRequest {
        let mut explicit_roots = Vec::new();
        let mut filters = Vec::new();

        for input in inputs {
            if looks_like_scan_path(input) {
                explicit_roots.push(PathBuf::from(input));
            } else {
                filters.push(input.clone());
            }
        }

        let scan_roots = if !explicit_roots.is_empty() {
            normalize_scan_roots(explicit_roots.clone())
        } else if !filters.is_empty() {
            self.search_roots()
        } else {
            self.default_scan_roots()
        };

        ScanRequest {
            scan_roots,
            filters,
            remembered_roots: normalize_scan_roots(explicit_roots),
        }
    }

    fn default_scan_roots(&self) -> Vec<PathBuf> {
        let config_roots = self.configured_scan_roots();
        if !config_roots.is_empty() {
            return config_roots;
        }

        normalize_scan_roots(vec![
            env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
        ])
    }

    fn search_roots(&self) -> Vec<PathBuf> {
        let mut roots = self.configured_scan_roots();

        roots.extend(discover_workspace_roots());

        roots.push(env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
        normalize_scan_roots(roots)
    }

    fn configured_scan_roots(&self) -> Vec<PathBuf> {
        normalize_scan_roots(
            self.config
                .scan_paths
                .iter()
                .map(PathBuf::from)
                .filter(|path| path.exists())
                .collect(),
        )
    }

    fn merge_scan_paths(&mut self, scan_paths: &[PathBuf]) {
        if scan_paths.is_empty() {
            return;
        }

        self.config.scan_paths.extend(
            scan_paths
                .iter()
                .map(|path| path.to_string_lossy().to_string()),
        );
        self.config.scan_paths.sort();
        self.config.scan_paths.dedup();
    }

    fn resolve_instance_index(&self, target: &str) -> Result<usize> {
        if let Ok(index) = target.parse::<usize>() {
            if index == 0 || index > self.config.instances.len() {
                bail!("instance index {} is out of range", index);
            }
            return Ok(index - 1);
        }

        if let Some(index) =
            self.config.instances.iter().position(|instance| {
                instance.path == target || format!("{}/", instance.path) == target
            })
        {
            return Ok(index);
        }

        let matches: Vec<(usize, &ProjectInstance)> = self
            .config
            .instances
            .iter()
            .enumerate()
            .filter(|(_, instance)| instance.repo_name.eq_ignore_ascii_case(target))
            .collect();

        match matches.as_slice() {
            [(index, _)] => Ok(*index),
            [] => Err(anyhow!("instance '{}' not found", target)),
            _ => {
                let options = matches
                    .iter()
                    .map(|(index, instance)| format!("{}: {}", index + 1, instance.path))
                    .collect::<Vec<_>>()
                    .join(", ");
                bail!(
                    "repository '{}' matches multiple projects; use an index or full path ({})",
                    target,
                    options
                );
            }
        }
    }
}

fn discover_workspace_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();

    if let Ok(current_dir) = env::current_dir() {
        for ancestor in current_dir.ancestors() {
            if ancestor.file_name().and_then(|name| name.to_str()) == Some("workspace") {
                roots.push(ancestor.to_path_buf());
                break;
            }
        }
    }

    if let Some(home) = dirs::home_dir() {
        let home_workspace = home.join("workspace");
        if home_workspace.exists() {
            roots.push(home_workspace);
        }
    }

    normalize_scan_roots(roots)
}

fn normalize_scan_roots(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut canonical_roots = paths
        .into_iter()
        .filter(|path| path.exists())
        .map(|path| path.canonicalize().unwrap_or(path))
        .collect::<Vec<_>>();

    canonical_roots.sort();
    canonical_roots.dedup();
    canonical_roots.sort_by_key(|path| path.components().count());

    let mut reduced = Vec::new();
    for path in canonical_roots {
        if reduced.iter().any(|root: &PathBuf| path.starts_with(root)) {
            continue;
        }
        reduced.push(path);
    }

    reduced
}

fn looks_like_scan_path(input: &str) -> bool {
    let candidate = PathBuf::from(input);
    candidate.is_absolute() || input.starts_with('.') || input.contains('\\') || input.contains('/')
}
