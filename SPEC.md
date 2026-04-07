# Proj - 多副本仓库管理器 规范

## 项目概述
CLI 工具用于管理本地项目目录，支持自动发现、状态查看和快速跳转。

## 核心功能

### 1. Scan - 扫描发现项目
- `proj scan <path1> <path2>` - 直接扫描指定路径下的项目
- 自动检测 Git 分支、是否有未提交更改
- 记住扫描过的路径，后续 `proj scan` 无参数时可复用

### 2. Status - 状态查看
- `proj status` - 显示所有副本的 git 状态（分支、clean/modified/ahead）
- `proj status <repo|index|path>` - 单个副本详细状态

### 3. Cd - 快速跳转
- `proj cd <repo|index|path> --raw` - 输出路径（供 shell 使用）
- 配合 PowerShell: `function pcd { Set-Location $(proj cd $args[0] --raw) }`

### 4. Init - 生成 shell 集成
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
        ├── scan.rs
        ├── status.rs
        ├── cd.rs
        └── init.rs
```

## 数据模型

```rust
// Config - 存储在 ~/.proj.json
pub struct Config {
    pub version: u32,
    pub scan_paths: Vec<String>,
    pub instances: Vec<ProjectInstance>,
}

pub struct ProjectInstance {
    pub repo_name: String,
    pub path: String,
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
2. `proj scan <某个目录>` 能找到该目录下的项目
3. `proj status` 能显示 git 分支和状态
4. `proj cd <repo|index|path> --raw` 能输出正确路径
5. 所有数据持久化到 `~/.proj.json`
