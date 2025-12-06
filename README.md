# SteamCloudFileManager

<p align="center">
  <img src="assets/steam_cloud-iOS-Default-1024x1024@1x.png" width="160" alt="steam_cloud" />
</p>

[English](README.en.md) | **简体中文**

[![License: GPL-3.0](https://img.shields.io/badge/License-GPL%203.0-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![Rust](https://img.shields.io/badge/rust-1.90+-orange)](https://www.rust-lang.org)
[![dependency status](https://deps.rs/repo/github/Fldicoahkiin/SteamCloudFileManager/status.svg)](https://deps.rs/repo/github/Fldicoahkiin/SteamCloudFileManager)
[![Platform](https://img.shields.io/badge/platform-Windows%20|%20macOS%20|%20Linux-lightgrey)](https://github.com/Fldicoahkiin/SteamCloudFileManager)

> 基于 Rust 和 egui 构建的跨平台 Steam 云存档管理工具

## 功能

一个图形界面的 Steam 云存档管理工具，无需启动游戏就能直接操作云端文件。

Steam 客户端自带的云存档管理功能比较简陋，这个工具提供了更完整的文件列表和批量操作支持：

- 查看完整的云存档文件列表（包括文件夹结构）
- 批量下载/上传文件
- 删除或取消同步指定文件
- 快速切换不同游戏
- 查看文件在本地磁盘的实际位置
- 显示云端同步状态

## 平台支持

| 平台 | 架构 | 支持状态 | 说明 |
|------|------|----------|------|
| Windows | x64 | ✅ 支持 | |
| Windows | ARM64 | ❌ 不支持 | Steam SDK 不提供 ARM64 版本 |
| macOS | Intel (x64) | ✅ 支持 | |
| macOS | Apple Silicon (ARM64) | ✅ 支持 | |
| Linux | x64 | ✅ 支持 | |


## 安装

从 [Releases](https://github.com/Fldicoahkiin/SteamCloudFileManager/releases) 下载预编译版本

或者自己构建：

```bash
git clone https://github.com/Fldicoahkiin/SteamCloudFileManager.git
cd SteamCloudFileManager
cargo build --release
```

**构建依赖：**

- **Cargo**
- **Rust 1.90.0+** (因为 egui 0.33 需要 Rust 1.88+，推荐 1.90+)
  - 使用 Rust 2021 edition
  - 安装：`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

- **C++ 编译工具链：**
  - **Windows**:
    - Visual Studio 2019 或更新版本（推荐安装 "Desktop development with C++" 工作负载）
    - 或 [Build Tools for Visual Studio](https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022)
  - **macOS**:
    - Xcode Command Line Tools: `xcode-select --install`
  - **Linux**:
    - gcc/g++ 或 clang
    - Ubuntu/Debian: `sudo apt install build-essential`
    - Fedora: `sudo dnf install gcc gcc-c++`
    - Arch: `sudo pacman -S base-devel`

**运行依赖：**

- Steam 客户端（必须以调试模式运行）

## 使用

### Steam 调试模式

本工具使用 CDP 协议与 Steam 通信，**必须**以调试模式启动 Steam。

**Windows:**

1. 右键点击 Steam 快捷方式，选择“属性”
2. 在“目标”栏末尾添加：`-cef-enable-debugging`
3. 点击“确定”并启动 Steam

**macOS:**

1. 退出 Steam
2. 在终端执行：

   ```bash
   open -a Steam --args -cef-enable-debugging
   ```

**Linux:**

1. 关闭 Steam
2. 在终端执行：

   ```bash
   steam -cef-enable-debugging &
   ```

   或者修改 Steam 快捷方式，在 Exec 行末尾添加 `-cef-enable-debugging`

**注意：** 本软件提供了“以调试模式重启 Steam”按钮，可以自动根据引导完成上述操作。

### 基本操作

1. 确保 Steam 已以调试模式运行
2. 启动本工具
3. 选择游戏：
   - 点击“游戏库”按钮，从扫描到的游戏列表中选择
   - 或者直接在 App ID 输入框中输入游戏的 App ID
4. 点击“连接”按钮
5. 连接成功后即可下载/上传/删除文件

App ID 可以通过 Steam 商店 URL 或 [SteamDB](https://steamdb.info/) 上找到。

## 技术架构

### VDF 解析

- 直接读取 `remotecache.vdf` 获取完整文件列表
- 显示文件在本地磁盘的实际存储位置
- 支持所有 Root 路径类型（0-12）

### CDP 协议

- 通过 Steam CEF 调试接口与客户端通信
- 实时获取云端文件列表和下载链接
- 自动合并云端状态到本地视图

### Steamworks API

- 使用 `ISteamRemoteStorage` API
- 处理文件上传和删除操作

## TODO

### 正在开发 

#### 文件夹树状结构
- [ ] 创建 `FileTreeNode` 数据结构（文件夹/文件节点）
- [ ] 实现路径解析：最后一个 `/` 后面是文件名，前面是路径
- [ ] 树构建算法：递归构建文件夹层级
- [ ] UI 显示：树形线条 + 缩进
- [ ] 文件夹：📁 图标 + 小箭头 (▼/▶)
- [ ] 文件：不显示图标，只显示文件名
- [ ] 点击小箭头：展开/折叠文件夹
- [ ] 点击文件夹名：选中该文件夹和其下所有文件
- [ ] 点击文件名：选中单个文件
- [ ] 文件夹优先排序（同一级时文件夹在前）
- [ ] 将 "文件夹" 列改名为 "根文件夹"
- [ ] 默认全部展开

#### 批量下载功能
- [ ] 实现文件夹选择逻辑（点击文件夹名选中所有子文件）
- [ ] 文件夹下载功能（递归下载所有文件）
- [ ] 下载时创建文件夹结构
- [ ] 下载根文件夹命名：`游戏名-根目录类型/`
- [ ] 保持子文件夹层级：`游戏名-根目录类型/saves/manual/save1.sav`
- [ ] 显示下载进度（当前文件/总文件数）

#### 批量上传功能
- [ ] 选择本地文件夹功能
- [ ] 递归扫描文件夹下所有文件
- [ ] 上传时保持相对路径结构
- [ ] 显示上传进度

#### 搜索和筛选
- [ ] 文件名搜索功能
- [ ] 搜索时自动展开匹配路径
- [ ] 高亮匹配结果
- [ ] 按文件夹筛选

#### ⚙️ 排序和显示选项
- [ ] 可配置排序规则（名称/大小/时间）
- [ ] 记住文件夹展开状态
- [ ] 自定义列显示/隐藏

#### 🛠️ 其他优化
- [ ] 虚拟滚动优化大文件夹性能
- [ ] 文件夹右键菜单
- [ ] 文件夹统计信息（文件数、总大小）
- [ ] 键盘导航支持

### 已完成 (v0.1.0-beta)

- [x] Steam API 集成
- [x] CDP (Chrome DevTools Protocol) 集成
- [x] 基础文件列表显示
- [x] 单文件下载/上传
- [x] 文件删除和取消同步
- [x] 多选模式
- [x] 游戏库扫描和切换
- [x] 游戏库刷新按钮
- [x] Steam 重启引导对话框
- [x] 实时状态进度显示
- [x] 跨平台支持 (Windows/macOS/Linux)

## 贡献

欢迎提交 Issue 和 Pull Request！

## 贡献者

<a href="https://github.com/Fldicoahkiin/SteamCloudFileManager/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=Fldicoahkiin/SteamCloudFileManager" />
</a>

## 许可证

本项目采用 GPL-3.0 许可证 - 详见 [LICENSE](LICENSE) 文件

## 致谢

- [SteamCloudFileManagerLite](https://github.com/GMMan/SteamCloudFileManagerLite)
- [steamworks-rs](https://github.com/Thinkofname/steamworks-rs)
- [egui](https://github.com/emilk/egui)
- [eframe](https://github.com/emilk/egui/tree/master/crates/eframe)
- [keyvalues-parser](https://github.com/CosmicHorrorDev/vdf-rs)
- [tungstenite](https://github.com/snapview/tungstenite-rs)
- [rfd](https://github.com/PolyMeilex/rfd)
- [sysinfo](https://github.com/GuillaumeGomez/sysinfo)
- [ureq](https://github.com/algesten/ureq)
- [Steamworks SDK](https://partner.steamgames.com/doc/sdk/api)
- [Steamworks Steam Cloud Documentation](https://partner.steamgames.com/doc/features/cloud)
- [VDF Parser (Python)](https://github.com/ValvePython/vdf)
- [Facepunch.Steamworks](https://github.com/Facepunch/Facepunch.Steamworks)
- [Stack Exchange: Steam Cloud Data](https://gaming.stackexchange.com/questions/146644)
- [Quick Guide to Steam Cloud Saves](https://www.gamedeveloper.com/game-platforms/quick-guide-to-steam-cloud-saves)

## Star History

[![Star History Chart](https://api.star-history.com/svg?repos=Fldicoahkiin/SteamCloudFileManager&type=Date)](https://star-history.com/#Fldicoahkiin/SteamCloudFileManager&Date)
