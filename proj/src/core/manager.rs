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

        // 加载手动设置的 alias（通过 proj alias 命令设置的）
        let manual_alias_by_path: HashMap<String, String> = self
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
        let mut instances_to_alias = Vec::new();

        for mut instance in scanned {
            instance.last_check = Some(Utc::now());

            if auto_alias {
                // auto_alias 模式：所有实例都重新生成，确保全局最优
                instances_to_alias.push(instance);
            } else {
                // 非 auto_alias 模式：保留已有 alias
                instance.alias = manual_alias_by_path.get(&instance.path).cloned();
                if let Some(alias) = &instance.alias {
                    existing_aliases.insert(alias.clone());
                }
                merged.push(instance);
            }
        }

        // 批量生成 alias
        if !instances_to_alias.is_empty() {
            let generated = self.generate_smart_aliases(&instances_to_alias, &existing_aliases);
            merged.extend(generated);
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

Register-ArgumentCompleter -CommandName projcd -ParameterName name -ScriptBlock {
    param($commandName, $parameterName, $wordToComplete, $commandAst, $fakeBoundParameters)

    $aliases = proj alias --list 2>$null | ForEach-Object {
        if ($_ -match '^\[(\d+)\]\s+(\S+)\s+->\s+(.+)$') {
            [PSCustomObject]@{
                Index = $matches[1]
                Alias = $matches[2]
                Path = $matches[3]
            }
        }
    }

    $aliases | Where-Object {
        $_.Alias -like "$wordToComplete*" -and $_.Alias -ne '-'
    } | ForEach-Object {
        [System.Management.Automation.CompletionResult]::new(
            $_.Alias,
            $_.Alias,
            'ParameterValue',
            $_.Path
        )
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

    fn generate_smart_aliases(
        &self,
        instances: &[ProjectInstance],
        existing: &HashSet<String>,
    ) -> Vec<ProjectInstance> {
        let mut used = existing.clone();

        // 第 1 步：按 repo_name 分组
        let mut groups: HashMap<String, Vec<usize>> = HashMap::new();
        let bases: Vec<String> = instances
            .iter()
            .map(|inst| sanitize_alias(&inst.repo_name))
            .collect();

        for (i, base) in bases.iter().enumerate() {
            groups.entry(base.clone()).or_default().push(i);
        }

        // 为每个 instance 生成 alias
        let mut aliases = vec![String::new(); instances.len()];

        for (base, indices) in &groups {
            if indices.len() == 1 {
                // repo 名唯一，直接用
                let alias = ensure_unique(base, &used);
                used.insert(alias.clone());
                aliases[indices[0]] = alias;
                continue;
            }

            // repo 名重复 → 按 branch 消歧
            let mut branch_groups: HashMap<String, Vec<usize>> = HashMap::new();
            for &i in indices {
                let branch_key = instances[i]
                    .last_branch
                    .as_ref()
                    .map(|b| sanitize_alias(b))
                    .unwrap_or_default();
                branch_groups.entry(branch_key).or_default().push(i);
            }

            for (branch, branch_indices) in &branch_groups {
                if branch_indices.len() == 1 {
                    // 同 repo 名内 branch 唯一
                    let candidate = if branch.is_empty() {
                        base.clone()
                    } else {
                        format!("{}-{}", base, branch)
                    };
                    let alias = ensure_unique(&candidate, &used);
                    used.insert(alias.clone());
                    aliases[branch_indices[0]] = alias;
                    continue;
                }

                // 同 repo 名 + 同 branch → 用目录特征消歧
                let paths: Vec<&str> = branch_indices
                    .iter()
                    .map(|&i| instances[i].path.as_str())
                    .collect();
                let dir_keys = extract_distinguishing_dirs(&paths);

                for (j, &i) in branch_indices.iter().enumerate() {
                    let dir_key = &dir_keys[j];
                    let candidate = if branch.is_empty() {
                        if dir_key.is_empty() {
                            base.clone()
                        } else {
                            format!("{}-{}", base, dir_key)
                        }
                    } else if dir_key.is_empty() {
                        format!("{}-{}", base, branch)
                    } else {
                        format!("{}-{}-{}", base, branch, dir_key)
                    };
                    let alias = ensure_unique(&candidate, &used);
                    used.insert(alias.clone());
                    aliases[i] = alias;
                }
            }
        }

        instances
            .iter()
            .zip(aliases)
            .map(|(inst, alias)| {
                let mut result = inst.clone();
                result.alias = Some(alias);
                result
            })
            .collect()
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

/// 给定一组路径，为每个路径提取一个能区分彼此的目录名。
/// 从路径末尾往前找，跳过 repo 名本身和通用目录名，
/// 取第一个在组内有区分度的目录段。
fn extract_distinguishing_dirs(paths: &[&str]) -> Vec<String> {
    const SKIP_NAMES: &[&str] = &[
        "packages", "apps", "src", "lib", "workspace", "bytedance", "node_modules",
    ];

    // 把每个路径拆成段（去掉最后一段 repo 名）
    let segments: Vec<Vec<String>> = paths
        .iter()
        .map(|path| {
            let parts: Vec<&str> = path.split(['\\', '/']).filter(|s| !s.is_empty()).collect();
            if parts.len() < 2 {
                return vec![];
            }
            // 去掉最后一段（repo 名本身）
            parts[..parts.len() - 1]
                .iter()
                .rev()
                .filter(|p| !SKIP_NAMES.contains(p))
                .map(|p| sanitize_alias(p))
                .filter(|s| !s.is_empty())
                .collect()
        })
        .collect();

    // 从最近的父目录开始，找到第一层能区分所有路径的目录
    let max_depth = segments.iter().map(|s| s.len()).max().unwrap_or(0);

    for depth in 0..max_depth {
        let keys: Vec<String> = segments
            .iter()
            .map(|segs| segs.get(depth).cloned().unwrap_or_default())
            .collect();

        // 检查这一层是否能区分所有路径
        let unique_keys: HashSet<&String> = keys.iter().collect();
        if unique_keys.len() == keys.len() {
            return keys;
        }
    }

    // 无法通过单层目录区分，组合最近两层
    segments
        .iter()
        .map(|segs| {
            let parts: Vec<&str> = segs.iter().take(2).map(String::as_str).collect();
            parts.join("-")
        })
        .collect()
}

/// 确保 alias 唯一：如果候选值已被占用，加数字后缀
fn ensure_unique(candidate: &str, used: &HashSet<String>) -> String {
    if !used.contains(candidate) {
        return candidate.to_string();
    }
    let mut idx = 2;
    loop {
        let suffixed = format!("{}-{}", candidate, idx);
        if !used.contains(&suffixed) {
            return suffixed;
        }
        idx += 1;
    }
}
