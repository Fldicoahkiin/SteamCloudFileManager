# 贡献指南 / Contributing Guide

[English](#english) | [简体中文](#简体中文)

---

## 简体中文

感谢你对 SteamCloudFileManager 的关注！欢迎提交 Issue 和 Pull Request。

### 行为准则

- 保持友善和尊重
- 提供有建设性的反馈
- 专注于问题本身

### 如何贡献

#### 报告 Bug

1. 先搜索现有 Issue 确认没有重复
2. 使用 [Bug 报告模板](.github/ISSUE_TEMPLATE/bug_report.yml) 创建 Issue
3. 提供尽可能详细的信息：操作系统、软件版本、复现步骤、日志等

#### 请求新功能

1. 先搜索现有 Issue 和 Discussions 确认没有类似提议
2. 使用 [功能请求模板](.github/ISSUE_TEMPLATE/feature_request.yml) 创建 Issue
3. 清楚描述你想解决的问题和期望的解决方案

#### 贡献 Root 映射

我们维护一份 [Root 路径映射表](ROOT_PATH_MAPPING.md)，欢迎补充你测试过的游戏：

1. 使用 [Root 映射模板](.github/ISSUE_TEMPLATE/root_mapping.yml) 创建 Issue
2. 提供游戏名称、AppID、测试平台和实际路径信息

#### 提交代码

1. Fork 本仓库
2. 创建你的分支：`git checkout -b feature/your-feature`
3. 开发前请阅读代码结构（见 README 中的项目结构）
4. 确保代码通过检查：
   ```bash
   cargo fmt
   cargo clippy -- -D warnings
   cargo test
   ```
5. 提交变更：`git commit -m "feat: add your feature"`
6. 推送并创建 Pull Request

#### 贡献翻译

请查看 [i18n 贡献指南](I18N_GUIDE.md)。

### 开发环境设置

1. 安装 Rust (1.90+)：https://rustup.rs
2. 克隆仓库：`git clone https://github.com/Fldicoahkiin/SteamCloudFileManager.git`
3. 安装依赖并编译：`cargo build`
4. 运行：`cargo run`

### 提交信息规范

使用 [Conventional Commits](https://www.conventionalcommits.org/) 格式：

- `feat:` 新功能
- `fix:` Bug 修复
- `docs:` 文档更新
- `refactor:` 重构
- `chore:` 杂项（依赖更新等）

---

## English

Thanks for your interest in SteamCloudFileManager! Issues and Pull Requests are welcome.

### Code of Conduct

- Be friendly and respectful
- Provide constructive feedback
- Focus on the issue at hand

### How to Contribute

#### Reporting Bugs

1. Search existing Issues to avoid duplicates
2. Use the [Bug Report Template](.github/ISSUE_TEMPLATE/bug_report.yml)
3. Provide detailed information: OS, version, steps to reproduce, logs

#### Requesting Features

1. Search existing Issues and Discussions first
2. Use the [Feature Request Template](.github/ISSUE_TEMPLATE/feature_request.yml)
3. Clearly describe the problem and your proposed solution

#### Contributing Root Mappings

We maintain a [Root Path Mapping Table](ROOT_PATH_MAPPING.md). Contributions welcome:

1. Use the [Root Mapping Template](.github/ISSUE_TEMPLATE/root_mapping.yml)
2. Provide game name, AppID, tested platform, and actual paths

#### Submitting Code

1. Fork this repository
2. Create your branch: `git checkout -b feature/your-feature`
3. Read the project structure in README before coding
4. Ensure your code passes checks:
   ```bash
   cargo fmt
   cargo clippy -- -D warnings
   cargo test
   ```
5. Commit changes: `git commit -m "feat: add your feature"`
6. Push and create a Pull Request

#### Contributing Translations

Please check the [i18n Contribution Guide](I18N_GUIDE.md).

### Development Setup

1. Install Rust (1.90+): https://rustup.rs
2. Clone the repo: `git clone https://github.com/Fldicoahkiin/SteamCloudFileManager.git`
3. Build: `cargo build`
4. Run: `cargo run`

### Commit Message Convention

Use [Conventional Commits](https://www.conventionalcommits.org/):

- `feat:` New feature
- `fix:` Bug fix
- `docs:` Documentation
- `refactor:` Refactoring
- `chore:` Chores (dependency updates, etc.)
