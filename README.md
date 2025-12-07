# SteamCloudFileManager

<p align="center">
  <img src="assets/steam_cloud-iOS-Default-1024x1024@1x.png" width="160" alt="steam_cloud" />
</p>

[English](README.en.md) | **简体中文**

[![License: GPL-3.0](https://img.shields.io/badge/License-GPL%203.0-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![GitHub release](https://img.shields.io/github/v/release/Fldicoahkiin/SteamCloudFileManager?include_prereleases&color=brightgreen)](https://github.com/Fldicoahkiin/SteamCloudFileManager/releases)
[![GitHub downloads](https://img.shields.io/github/downloads/Fldicoahkiin/SteamCloudFileManager/total?color=success)](https://github.com/Fldicoahkiin/SteamCloudFileManager/releases)
[![Build](https://github.com/Fldicoahkiin/SteamCloudFileManager/actions/workflows/build.yml/badge.svg)](https://github.com/Fldicoahkiin/SteamCloudFileManager/actions/workflows/build.yml)
[![Release](https://github.com/Fldicoahkiin/SteamCloudFileManager/actions/workflows/release.yml/badge.svg)](https://github.com/Fldicoahkiin/SteamCloudFileManager/actions/workflows/release.yml)
[![Rust](https://img.shields.io/badge/rust-1.90+-orange?logo=rust)](https://www.rust-lang.org)
[![dependency status](https://deps.rs/repo/github/Fldicoahkiin/SteamCloudFileManager/status.svg)](https://deps.rs/repo/github/Fldicoahkiin/SteamCloudFileManager)
![Windows](https://img.shields.io/badge/Windows-0078D6?logo=windows&logoColor=white)
![macOS](https://img.shields.io/badge/macOS-000000?logo=apple&logoColor=white)
![Ubuntu](https://img.shields.io/badge/Ubuntu-E95420?logo=ubuntu&logoColor=white)
![Debian](https://img.shields.io/badge/Debian-A81D33?logo=debian&logoColor=white)
![Fedora](https://img.shields.io/badge/Fedora-51A2DA?logo=fedora&logoColor=white)
![Arch Linux](https://img.shields.io/badge/Arch_Linux-1793D1?logo=archlinux&logoColor=white)
[![GitHub stars](https://img.shields.io/github/stars/Fldicoahkiin/SteamCloudFileManager?style=social)](https://github.com/Fldicoahkiin/SteamCloudFileManager/stargazers)
[![GitHub forks](https://img.shields.io/github/forks/Fldicoahkiin/SteamCloudFileManager?style=social)](https://github.com/Fldicoahkiin/SteamCloudFileManager/network/members)

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

| 平台 | 架构 | 支持状态 | 打包格式 |
|------|------|----------|----------|
| Windows | x64 | ✅ 支持 | `.zip` |
| Windows | ARM64 | ❌ 不支持 | Steam SDK 不提供 ARM64 版本 |
| macOS | Intel (x64) | ✅ 支持 | `.dmg` |
| macOS | Apple Silicon (ARM64) | ✅ 支持 | `.dmg` |
| Linux | x64 | ✅ 支持 | `.tar.gz`, `.deb`, `.rpm`, `.AppImage` |

## 安装

### Windows

1. 从 [Releases](https://github.com/Fldicoahkiin/SteamCloudFileManager/releases) 下载 `SteamCloudFileManager-windows-x86_64.zip`
2. 解压到任意位置
3. 双击 `SteamCloudFileManager.exe` 运行

**注意：** Windows 版本为便携模式，日志保存在应用所在目录的 `logs/` 文件夹。

### macOS

1. 从 [Releases](https://github.com/Fldicoahkiin/SteamCloudFileManager/releases) 下载对应版本：
   - Intel 芯片：`SteamCloudFileManager-macos-x86_64.dmg`
   - Apple Silicon：`SteamCloudFileManager-macos-aarch64.dmg`
2. 打开 DMG 文件
3. 将应用拖入 Applications 文件夹

### Linux

#### .tar.gz

```bash
# 下载并解压
wget https://github.com/Fldicoahkiin/SteamCloudFileManager/releases/download/v0.1.7-beta/SteamCloudFileManager-linux-x86_64.tar.gz
tar -xzf SteamCloudFileManager-linux-x86_64.tar.gz
cd SteamCloudFileManager-linux-x86_64

# 运行
./steamcloudfilemanager
```

#### AppImage

```bash
# 下载 AppImage
wget https://github.com/Fldicoahkiin/SteamCloudFileManager/releases/download/v0.1.7-beta/SteamCloudFileManager-linux-x86_64.AppImage

# 添加执行权限
chmod +x SteamCloudFileManager-linux-x86_64.AppImage

# 运行
./SteamCloudFileManager-linux-x86_64.AppImage
```

#### Debian/Ubuntu

```bash
# 下载 .deb 包
wget https://github.com/Fldicoahkiin/SteamCloudFileManager/releases/download/v0.1.7-beta/steamcloudfilemanager_0.1.7-beta_amd64.deb

# 安装
sudo dpkg -i steamcloudfilemanager_0.1.7-beta_amd64.deb
sudo apt-get install -f

# 运行
steamcloudfilemanager
```

#### Fedora/RHEL/openSUSE

```bash
# 下载 .rpm 包
wget https://github.com/Fldicoahkiin/SteamCloudFileManager/releases/download/v0.1.7-beta/steamcloudfilemanager-0.1.7-1.x86_64.rpm

# 安装
sudo dnf install ./steamcloudfilemanager-0.1.7-1.x86_64.rpm
# 或
sudo rpm -i steamcloudfilemanager-0.1.7-1.x86_64.rpm

# 运行
steamcloudfilemanager
```

### 从源码构建

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

### 功能开发

- [ ] 文件夹树状视图（进度 90%）
- [ ] 批量下载/上传功能
- [ ] 文件冲突检测与处理
- [ ] 多语言支持
- [ ] 云存档备份与恢复

### 包管理器支持

- [ ] AUR (Arch User Repository)
- [ ] Homebrew (macOS) - `brew install steamcloudfilemanager`
- [ ] APT 仓库 (Debian/Ubuntu) - `apt install steamcloudfilemanager`
- [ ] DNF/YUM 仓库 (Fedora/RHEL) - `dnf install steamcloudfilemanager`
- [ ] Flatpak - `flatpak install steamcloudfilemanager`
- [ ] Snap - `snap install steamcloudfilemanager`

## 贡献

欢迎提交 Issue 和 Pull Request！

## 贡献者

<a href="https://github.com/Fldicoahkiin/SteamCloudFileManager/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=Fldicoahkiin/SteamCloudFileManager" />
</a>

## 许可证

本项目采用 GPL-3.0 许可证 - 详见 [LICENSE](LICENSE) 文件

## 致谢

### 核心依赖

- [steamworks-rs](https://github.com/Thinkofname/steamworks-rs)
- [egui](https://github.com/emilk/egui)
- [eframe](https://github.com/emilk/egui/tree/master/crates/eframe)
- [keyvalues-parser](https://github.com/CosmicHorrorDev/vdf-rs)
- [tungstenite](https://github.com/snapview/tungstenite-rs)

### 工具库

- [rfd](https://github.com/PolyMeilex/rfd)
- [sysinfo](https://github.com/GuillaumeGomez/sysinfo)
- [ureq](https://github.com/algesten/ureq)
- [anyhow](https://github.com/dtolnay/anyhow)
- [tracing](https://github.com/tokio-rs/tracing)

### 打包工具

- [cargo-bundle](https://github.com/burtonageo/cargo-bundle)
- [cargo-deb](https://github.com/kornelski/cargo-deb)
- [cargo-generate-rpm](https://github.com/cat-in-136/cargo-generate-rpm)
- [linuxdeploy](https://github.com/linuxdeploy/linuxdeploy)

### 参考项目

- [SteamCloudFileManagerLite](<https://github.com/GMMan/SteamCloudFileManagerLite>)
- [Facepunch.Steamworks](https://github.com/Facepunch/Facepunch.Steamworks)

### 文档资料

- [Steamworks SDK](https://partner.steamgames.com/doc/sdk/api)
- [Steamworks Steam Cloud Documentation](https://partner.steamgames.com/doc/features/cloud)
- [VDF Parser (Python)](https://github.com/ValvePython/vdf)
- [Stack Exchange: Steam Cloud Data](https://gaming.stackexchange.com/questions/146644)
- [Quick Guide to Steam Cloud Saves](https://www.gamedeveloper.com/game-platforms/quick-guide-to-steam-cloud-saves)

## Star History

[![Star History Chart](https://api.star-history.com/svg?repos=Fldicoahkiin/SteamCloudFileManager&type=Date)](https://star-history.com/#Fldicoahkiin/SteamCloudFileManager&Date)
