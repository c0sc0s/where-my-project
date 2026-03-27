# Proj TUI 增强规范

## 目标
使用 ratatui 实现专业 TUI 界面，替代/增强现有命令行输出。

## 新增命令

### 1. `proj list` - 可交互项目列表
- 显示所有项目：表格形式，带颜色
- 键盘操作：
  - ↑/↓ 或 j/k：上下移动选择
  - Enter：跳转到选中目录（输出路径到 stdout，供 shell 捕获）
  - / 或 i：进入搜索模式，实时过滤列表
  - q/Esc：退出
  - r：刷新 git 状态
  - d：删除选中项目（需确认）
- 显示信息：alias、branch、状态（clean/dirty）、路径
- 选中行高亮显示

### 2. `proj scan` - 带进度条的扫描
- 扫描前显示：准备扫描 X 个路径
- 实时进度条：显示当前扫描的目录、已找到的仓库数
- 扫描完成后：
  - 显示结果表格
  - 提示是否保存到列表
  - 支持交互式设置 alias

### 3. `proj tui` - 完整 TUI 仪表盘（可选）
- 左侧：项目列表
- 右侧：选中项目的 git 详情
- 底部：快捷键提示

## 技术方案

### 依赖添加
```toml
[dependencies]
ratatui = "0.24"
crossterm = "0.27"
indicatif = "0.17"  # 进度条
```

### 架构调整
```
src/
├── tui/              # 新增 TUI 模块
│   ├── mod.rs
│   ├── app.rs        # TUI 应用状态管理
│   ├── ui.rs         # 渲染逻辑
│   ├── events.rs     # 事件处理
│   └── widgets/      # 自定义组件
│       ├── project_list.rs
│       ├── scan_progress.rs
│       └── status_bar.rs
```

### 实现细节

#### list 命令 TUI
1. 初始化时加载所有项目并获取 git 状态（异步）
2. 使用 ` ratatui::widgets::Table` 显示项目列表
3. 使用 `crossterm` 捕获键盘事件
4. 选中项目后输出路径到 stdout，然后退出

#### scan 命令进度条
1. 使用 `indicatif::ProgressBar`
2. 扫描每个目录前更新进度信息
3. 发现仓库时更新计数

#### 颜色方案
- 选中行：蓝色背景
- Clean 状态：绿色
- Dirty 状态：红色
- Alias：黄色
- Branch：洋红
- 路径：灰色

## 验收标准

1. `proj list` 启动后显示可交互表格
2. 上下键可移动选择，Enter 输出路径并退出
3. 可按 / 进入搜索模式实时过滤
4. `proj scan --tui` 显示进度条和实时发现的仓库
5. 所有 TUI 界面按 q 可正常退出
6. Windows PowerShell 7 中显示正常（无乱码）
7. TUI 退出后终端状态恢复正常

## 兼容性
- 保留现有非 TUI 命令行为（添加 --tui 标志启用）
- 或者 `proj list` 默认 TUI，`proj status` 保持原样
