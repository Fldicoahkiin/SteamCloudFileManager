# SteamCloudFileManager
<p align="center">
  <img src="assets/steam_cloud-iOS-Default-1024x1024@1x.png" width="160" alt="steam_cloud" />
</p>

[English](README.en.md) | **简体中文**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-%E2%89%A51.70-orange)](https://www.rust-lang.org)
[![Platform](https://img.shields.io/badge/platform-Windows%20|%20macOS%20|%20Linux-lightgrey)](https://github.com/Fldicoahkiin/SteamCloudFileManager)

> 基于 Rust 和 egui 构建的跨平台 Steam 云存档管理工具

本工具提供了一个图形界面来管理不同游戏的 Steam 云存档文件。它直接连接到 Steam 的远程存储 API，允许用户无需启动游戏即可查看、下载、上传和管理云存档。

## 目录

- [背景](#背景)
- [安装](#安装)
  - [依赖项](#依赖项)
  - [从源码构建](#从源码构建)
- [使用方法](#使用方法)
  - [基本操作](#基本操作)
  - [文件管理](#文件管理)
  - [切换游戏](#切换游戏)
- [API](#api)
- [贡献](#贡献)
- [许可证](#许可证)
- [致谢](#致谢)

## 背景

Steam 云自动在不同设备间同步游戏存档，但缺少统一的文件管理界面。本项目通过以下特性解决了这个问题：

- 无需启动游戏即可直接访问 Steam 远程存储
- 支持多个存档文件的批量操作
- 跨平台兼容，与系统原生集成
- 实时配额监控和文件元数据显示

本项目通过 [steamworks-rs](https://github.com/Thinkofname/steamworks-rs) 绑定使用 Steamworks SDK，确保与官方 Steam API 的兼容性。

## 安装

### 依赖项

**运行时要求：**
- Steam 客户端（必须运行并已登录）
- 操作系统：
  - Windows 10 或更高版本
  - macOS 10.15 (Catalina) 或更高版本
  - Linux（glibc 2.31+，如 Ubuntu 20.04、Debian 11、Fedora 34 或同等版本）

**构建要求：**
- Rust 1.70+ (推荐使用 1.82.0 stable 或更新版本)
  - edition 2021 支持
  - 如需使用 edition 2024，需要 Rust nightly 版本
- Cargo 包管理器
- C++ 构建工具（因平台而异）：
  - Windows: Visual Studio 2019+ 或 Visual Studio 构建工具
  - macOS: Xcode 命令行工具
  - Linux: gcc/g++ 或 clang

### 从源码构建

克隆仓库：
```bash
git clone https://github.com/yourusername/SteamCloudFileManager.git
cd SteamCloudFileManager
```

构建项目：
```bash
cargo build --release
```

编译后的二进制文件位于：
- Windows: `target/release/SteamCloudFileManager.exe`
- macOS/Linux: `target/release/SteamCloudFileManager`

## 使用方法

### 基本操作

1. **启动 Steam 客户端**
   确保 Steam 正在运行并且你已登录账户。

2. **启动应用程序**
   ```bash
   ./target/release/SteamCloudFileManager
   ```

3. **连接到游戏**
   - 在输入框中输入游戏的 App ID
   - 点击"连接"
   - 文件列表将自动加载

   查找 App ID 的方法：
   - Steam 商店 URL：`https://store.steampowered.com/app/[APP_ID]/`
   - SteamDB：https://steamdb.info/

### 文件管理

**下载文件**
1. 从列表中选择一个或多个文件
2. 点击"下载选中文件"
3. 在文件对话框中选择保存位置

**上传文件**
1. 点击"上传文件"
2. 选择要上传的本地文件
3. 文件将立即同步到 Steam 云

**删除文件**
1. 选择目标文件
2. 点击"删除选中文件"
3. 确认删除操作

**取消云同步**
1. 选择要取消同步的文件
2. 点击"取消云同步"
3. 文件保留在本地但停止同步

### 切换游戏

要管理不同游戏的文件：
1. 点击"断开连接"
2. 输入新的 App ID
3. 点击"连接"

或者，直接输入新的 App ID 并连接即可立即切换。

## API

应用程序通过以下主要 API 与 Steam 进行交互：

### Steam 远程存储 API

| 函数 | 状态 | 描述 |
|------|------|------|
| `GetFileCount()` | ✅ | 获取文件总数 |
| `GetFileNameAndSize()` | ✅ | 获取文件元数据 |
| `FileExists()` | ✅ | 检查文件是否存在 |
| `FilePersisted()` | ✅ | 验证持久化状态 |
| `GetFileTimestamp()` | ✅ | 获取修改时间 |
| `FileRead()` | ✅ | 下载文件内容 |
| `FileWrite()` | ✅ | 上传文件内容 |
| `FileDelete()` | ✅ | 从云端删除文件 |
| `FileForget()` | ✅ | 停止跟踪文件 |
| `IsCloudEnabledForAccount()` | ✅ | 检查账户云状态 |
| `IsCloudEnabledForApp()` | ✅ | 检查应用云状态 |
| `SetCloudEnabledForApp()` | ✅ | 切换应用云同步 |

### 内部 API

```rust
pub struct SteamCloudManager {
    pub fn connect(app_id: u32) -> Result<()>
    pub fn disconnect()
    pub fn get_files() -> Result<Vec<CloudFile>>
    pub fn read_file(filename: &str) -> Result<Vec<u8>>
    pub fn write_file(filename: &str, data: &[u8]) -> Result<()>
    pub fn delete_file(filename: &str) -> Result<bool>
    pub fn forget_file(filename: &str) -> Result<bool>
}
```

## 贡献

欢迎提交Issue和Pull Request

## 贡献者

<a href="https://github.com/Fldicoahkiin/SteamCloudFileManager/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=Fldicoahkiin/SteamCloudFileManager" />
</a>

## 许可证

MIT 许可证 - 详见 [LICENSE](LICENSE) 文件

## 致谢

- [SteamCloudFileManagerLite](https://github.com/GMMan/SteamCloudFileManagerLite)
- [Steamworks SDK](https://partner.steamgames.com/doc/sdk/api)

## Star History

[![Star History Chart](https://api.star-history.com/svg?repos=Fldicoahkiin/SteamCloudFileManager&type=Date)](https://star-history.com/#Fldicoahkiin/SteamCloudFileManager&Date)
