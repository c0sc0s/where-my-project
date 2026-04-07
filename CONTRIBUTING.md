# Contributing

<p align="center">
  <a href="#简体中文">简体中文</a> · <a href="#english">English</a>
</p>

## 简体中文

感谢你愿意改进 Where My Project。

这个项目希望保持几个特点：简单、直接、易改、出了问题能尽快暴露出来。提交贡献时，请尽量遵循这些原则。

### 提交建议

- 大一点的功能改动，建议先开 Issue 讨论方向
- 一个 Pull Request 只解决一个主题，尽量保持小而清晰
- 如果改动影响用户使用方式，请同步更新 `README.md`
- 如果改动改变了行为或边界条件，请补充或更新测试

### 本地开发

```powershell
cd proj
cargo build
cargo test
```

发布前常用构建命令：

```powershell
cd proj
cargo build --release
```

### 代码原则

- 优先写小函数、单一职责、易组合的代码
- 发现设计别扭时，先重构再加功能
- 不要吞异常，也不要制造“假成功”
- 尽量减少重复，但不要过早抽象
- 改动尽量聚焦，不混入无关格式化或顺手重写

### Pull Request 清单

- 变更目标是否清楚且范围单一
- 是否说明了为什么要改，而不只是改了什么
- 是否覆盖了关键测试或至少写明了手动验证方式
- 是否同步更新了文档、示例或安装说明
- 是否避免了与当前任务无关的改动

## English

Thanks for contributing to Where My Project.

This project aims to stay simple, explicit, easy to change, and quick to fail when assumptions are wrong. Please keep those goals in mind when sending changes.

### Contribution Guidelines

- Open an issue first for larger features or design changes
- Keep each pull request focused on one topic
- Update `README.md` when user-facing behavior changes
- Add or update tests when behavior or edge cases change

### Local Development

```powershell
cd proj
cargo build
cargo test
```

Common release build command:

```powershell
cd proj
cargo build --release
```

### Coding Principles

- Prefer small, composable functions with single responsibilities
- Refactor awkward design before adding more feature surface
- Do not swallow exceptions or fake success paths
- Remove real duplication early, but avoid premature abstraction
- Keep changes focused and avoid unrelated formatting churn

### Pull Request Checklist

- Is the change focused and easy to review?
- Does the PR explain why the change is needed?
- Did you add tests or describe manual verification?
- Did you update docs, examples, or install notes when needed?
- Did you avoid unrelated edits outside the task?
