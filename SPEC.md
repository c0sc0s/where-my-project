# Proj - 多副本仓库管理器 规范

## 项目概述
CLI 工具用于管理同一 Git 仓库的多个本地 clone，支持自动发现、别名管理和快速跳转。

## 核心功能

### 1. Watch - 监控仓库设置
- `proj watch <repo-name>` - 添加监控仓库名
- `proj watch --list` - 列出所有监控仓库
- `proj watch --remove <repo-name>` - 移除监控

### 2. Scan - 扫描发现副本
- `proj scan [--paths <path1,path2>]` - 扫描指定路径下所有监控仓库的 clone
- 自动检测 Git 分支、是否有未提交更改
- `proj scan --auto-alias` - 按分支名自动生成 alias

### 3. Alias - 别名管理
- `proj alias <index|path> <alias-name>` - 为副本设置别名
- `proj alias --list` - 显示所有 alias 映射

### 4. Status - 状态查看
- `proj status` - 显示所有副本的 git 状态（分支、clean/modified/ahead）
- `proj status <alias>` - 单个副本详细状态

### 5. Cd - 快速跳转
- `proj cd <alias|index> --raw` - 输出路径（供 shell 使用）
- 配合 PowerShell: `function pcd { Set-Location $(proj cd $args[0] --raw) }`

### 6. Init - 生成 shell 集成
- `proj init` - 输出 PowerShell 集成代码

## 架构要求

```
src/
├── bin/
│   └── proj.rs           # CLI 入口
├── lib.rs                # 库入口
├── core/                 # 核心层（CLI/TUI 共享）
│   ├── mod.rs
│   ├── models.rs         # ProjectInstance, Config, GitStatus
│   ├── storage.rs        # JSON 文件读写
│   ├── git.rs            # Git 操作封装
│   ├── scanner.rs        # 文件系统扫描
│   └── manager.rs        # 业务逻辑（增删改查）
└── cli/                  # CLI 层
    ├── mod.rs
    ├── args.rs           # clap 参数定义
    └── commands/         # 各命令实现
        ├── watch.rs
        ├── scan.rs
        ├── alias.rs
        ├── status.rs
        ├── cd.rs
        └── init.rs
```

## 数据模型

```rust
// Config - 存储在 ~/.proj.json
pub struct Config {
    pub version: u32,
    pub watched_repos: Vec<String>,
    pub scan_paths: Vec<String>,
    pub instances: Vec<ProjectInstance>,
}

pub struct ProjectInstance {
    pub repo_name: String,
    pub path: String,
    pub alias: Option<String>,
    pub last_branch: Option<String>,
    pub last_check: Option<DateTime<Utc>>,
}

pub struct GitStatus {
    pub branch: String,
    pub is_clean: bool,
    pub modified_count: usize,
    pub untracked_count: usize,
    pub ahead_count: usize,
}
```

## 技术依赖

```toml
[dependencies]
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = { version = "0.4", features = ["serde"] }
dirs = "5"
walkdir = "2"
anyhow = "1"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

## 文件位置

- 配置文件: `~/.proj.json`
- 代码位置: 当前目录下的 `proj/` 文件夹

## 验收标准

1. `cargo build` 成功编译
2. `proj watch live_studio_mono` 能保存监控配置
3. `proj scan --paths <某个目录>` 能找到该目录下的 git 仓库
4. `proj alias 1 my-project` 能设置别名
5. `proj status` 能显示 git 分支和状态
6. `proj cd my-project --raw` 能输出正确路径
7. 所有数据持久化到 `~/.proj.json`
