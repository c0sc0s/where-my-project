use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub version: u32,
    pub watched_repos: Vec<String>,
    pub scan_paths: Vec<String>,
    pub instances: Vec<ProjectInstance>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: 1,
            watched_repos: Vec::new(),
            scan_paths: Vec::new(),
            instances: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInstance {
    pub repo_name: String,
    pub path: String,
    pub alias: Option<String>,
    pub last_branch: Option<String>,
    pub last_check: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct GitStatus {
    pub branch: String,
    pub is_clean: bool,
    pub modified_count: usize,
    pub untracked_count: usize,
    pub ahead_count: usize,
}

#[derive(Debug, Clone)]
pub struct InstanceStatus {
    pub instance: ProjectInstance,
    pub git_status: GitStatus,
}
