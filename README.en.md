# SteamCloudFileManager

<p align="center">
  <img src="assets/steam_cloud-iOS-Default-1024x1024@1x.png" width="160" alt="steam_cloud" />
</p>

**English** | [简体中文](README.md)

[![License: GPL-3.0](https://img.shields.io/badge/License-GPL%203.0-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![GitHub release](https://img.shields.io/github/v/release/Fldicoahkiin/SteamCloudFileManager?include_prereleases)](https://github.com/Fldicoahkiin/SteamCloudFileManager/releases)
[![GitHub downloads](https://img.shields.io/github/downloads/Fldicoahkiin/SteamCloudFileManager/total)](https://github.com/Fldicoahkiin/SteamCloudFileManager/releases)
[![CI](https://github.com/Fldicoahkiin/SteamCloudFileManager/actions/workflows/build.yml/badge.svg)](https://github.com/Fldicoahkiin/SteamCloudFileManager/actions)
[![Rust](https://img.shields.io/badge/rust-1.90+-orange)](https://www.rust-lang.org)
[![dependency status](https://deps.rs/repo/github/Fldicoahkiin/SteamCloudFileManager/status.svg)](https://deps.rs/repo/github/Fldicoahkiin/SteamCloudFileManager)
[![Platform](https://img.shields.io/badge/platform-Windows%20|%20macOS%20|%20Linux-lightgrey)](https://github.com/Fldicoahkiin/SteamCloudFileManager)

> Cross-platform Steam Cloud file management utility built with Rust and egui.

## Features

A GUI tool for managing Steam Cloud saves, allowing direct cloud file operations without launching games.

Steam's built-in cloud save management is quite basic. This tool provides more complete file listings and batch operation support:

- View complete cloud save file list (including folder structure)
- Batch download/upload files
- Delete or unsync specific files
- Quickly switch between different games
- View actual file locations on local disk
- Display cloud sync status

## Platform Support

| Platform | Architecture | Status | Package Format |
|----------|--------------|--------|----------------|
| Windows | x64 | ✅ Supported | `.zip` |
| Windows | ARM64 | ❌ Not Supported | Steam SDK does not provide ARM64 binaries |
| macOS | Intel (x64) | ✅ Supported | `.dmg` |
| macOS | Apple Silicon (ARM64) | ✅ Supported | `.dmg` |
| Linux | x64 | ✅ Supported | `.tar.gz`, `.deb`, `.rpm`, `.AppImage` |

## Installation

### Windows

1. Download `SteamCloudFileManager-windows-x86_64.zip` from [Releases](https://github.com/Fldicoahkiin/SteamCloudFileManager/releases)
2. Extract to any location
3. Double-click `SteamCloudFileManager.exe` to run

**Note:** Windows version is portable, logs are saved in the `logs/` folder in the application directory.

### macOS

1. Download from [Releases](https://github.com/Fldicoahkiin/SteamCloudFileManager/releases):
   - Intel: `SteamCloudFileManager-macos-x86_64.dmg`
   - Apple Silicon: `SteamCloudFileManager-macos-aarch64.dmg`
2. Open the DMG file
3. Drag the app to Applications folder

### Linux

#### .tar.gz (Universal)

```bash
# Download and extract
wget https://github.com/Fldicoahkiin/SteamCloudFileManager/releases/download/v0.1.7-beta/SteamCloudFileManager-linux-x86_64.tar.gz
tar -xzf SteamCloudFileManager-linux-x86_64.tar.gz
cd SteamCloudFileManager-linux-x86_64

# Run
./steamcloudfilemanager
```

#### AppImage (Universal)

```bash
# Download AppImage
wget https://github.com/Fldicoahkiin/SteamCloudFileManager/releases/download/v0.1.7-beta/SteamCloudFileManager-linux-x86_64.AppImage

# Add execute permission
chmod +x SteamCloudFileManager-linux-x86_64.AppImage

# Run
./SteamCloudFileManager-linux-x86_64.AppImage
```

#### Debian/Ubuntu

```bash
# Download .deb package
wget https://github.com/Fldicoahkiin/SteamCloudFileManager/releases/download/v0.1.7-beta/steamcloudfilemanager_0.1.7-beta_amd64.deb

# Install
sudo dpkg -i steamcloudfilemanager_0.1.7-beta_amd64.deb
sudo apt-get install -f

# Run
steamcloudfilemanager
```

#### Fedora/RHEL/openSUSE

```bash
# Download .rpm package
wget https://github.com/Fldicoahkiin/SteamCloudFileManager/releases/download/v0.1.7-beta/steamcloudfilemanager-0.1.7-1.x86_64.rpm

# Install
sudo dnf install ./steamcloudfilemanager-0.1.7-1.x86_64.rpm
# or
sudo rpm -i steamcloudfilemanager-0.1.7-1.x86_64.rpm

# Run
steamcloudfilemanager
```

### Build from Source

```bash
git clone https://github.com/Fldicoahkiin/SteamCloudFileManager.git
cd SteamCloudFileManager
cargo build --release
```

**Build Dependencies:**

- **Cargo**
- **Rust 1.90.0+** (egui 0.33 requires Rust 1.88+, 1.90+ recommended)
  - Uses Rust 2021 edition
  - Install: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

- **C++ Toolchain:**
  - **Windows**:
    - Visual Studio 2019 or newer (recommend installing "Desktop development with C++" workload)
    - Or [Build Tools for Visual Studio](https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022)
  - **macOS**:
    - Xcode Command Line Tools: `xcode-select --install`
  - **Linux**:
    - gcc/g++ or clang
    - Ubuntu/Debian: `sudo apt install build-essential`
    - Fedora: `sudo dnf install gcc gcc-c++`
    - Arch: `sudo pacman -S base-devel`

**Runtime Dependencies:**

- Steam client (must run in debug mode)

## Usage

### Steam Debug Mode

This tool uses CDP protocol to communicate with Steam, which **requires** Steam to run in debug mode.

**Windows:**

1. Right-click Steam shortcut, select "Properties"
2. Add to the end of "Target" field: `-cef-enable-debugging`
3. Click "OK" and launch Steam

**macOS:**

1. Quit Steam
2. Run in terminal:

   ```bash
   open -a Steam --args -cef-enable-debugging
   ```

**Linux:**

1. Close Steam
2. Run in terminal:

   ```bash
   steam -cef-enable-debugging &
   ```

   Or modify Steam shortcut, add `-cef-enable-debugging` to the end of Exec line

**Note:** This tool provides a "Restart Steam in Debug Mode" button that can automatically complete the above operations with guidance.

### Basic Operations

1. Ensure Steam is running in debug mode
2. Launch this tool
3. Select a game:
   - Click "Game Library" button to select from scanned game list
   - Or directly enter the game's App ID in the input box
4. Click "Connect" button
5. After connection, you can download/upload/delete files

App IDs can be found in Steam Store URLs or on [SteamDB](https://steamdb.info/).

## Technical Architecture

### VDF Parsing

- Directly reads `remotecache.vdf` for complete file list
- Shows actual file storage locations on local disk
- Supports all Root path types (0-12)

### CDP Protocol

- Communicates with client via Steam CEF debug interface
- Retrieves cloud file list and download links in real-time
- Automatically merges cloud status into local view

### Steamworks API

- Uses `ISteamRemoteStorage` API
- Handles file upload and delete operations

## TODO

### Feature Development

- [ ] Folder tree view (90% complete)
- [ ] Batch download/upload functionality
- [ ] File conflict detection and handling
- [ ] Multi-language support
- [ ] Cloud save backup and restore

### Package Manager Support

- [ ] AUR (Arch User Repository)
- [ ] Homebrew (macOS) - `brew install steamcloudfilemanager`
- [ ] APT Repository (Debian/Ubuntu) - `apt install steamcloudfilemanager`
- [ ] DNF/YUM Repository (Fedora/RHEL) - `dnf install steamcloudfilemanager`
- [ ] Flatpak - `flatpak install steamcloudfilemanager`
- [ ] Snap - `snap install steamcloudfilemanager`

## Contributing

Welcome to submit Issues and Pull Requests

## Contributors

<a href="https://github.com/Fldicoahkiin/SteamCloudFileManager/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=Fldicoahkiin/SteamCloudFileManager" />
</a>

## License

This project is licensed under GPL-3.0 - see [LICENSE](LICENSE) file for details.

## Acknowledgments

### Core Dependencies

- [steamworks-rs](https://github.com/Thinkofname/steamworks-rs)
- [egui](https://github.com/emilk/egui)
- [eframe](https://github.com/emilk/egui/tree/master/crates/eframe)
- [keyvalues-parser](https://github.com/CosmicHorrorDev/vdf-rs)
- [tungstenite](https://github.com/snapview/tungstenite-rs)

### Utility Libraries

- [rfd](https://github.com/PolyMeilex/rfd)
- [sysinfo](https://github.com/GuillaumeGomez/sysinfo)
- [ureq](https://github.com/algesten/ureq)
- [anyhow](https://github.com/dtolnay/anyhow)
- [tracing](https://github.com/tokio-rs/tracing)

### Packaging Tools

- [cargo-bundle](https://github.com/burtonageo/cargo-bundle)
- [cargo-deb](https://github.com/kornelski/cargo-deb)
- [cargo-generate-rpm](https://github.com/cat-in-136/cargo-generate-rpm)
- [linuxdeploy](https://github.com/linuxdeploy/linuxdeploy)

### Reference Projects

- [SteamCloudFileManagerLite](https://github.com/GMMan/SteamCloudFileManagerLite)
- [Facepunch.Steamworks](https://github.com/Facepunch/Facepunch.Steamworks)

### Documentation

- [Steamworks SDK](https://partner.steamgames.com/doc/sdk/api)
- [Steamworks Steam Cloud Documentation](https://partner.steamgames.com/doc/features/cloud)
- [VDF Parser (Python)](https://github.com/ValvePython/vdf)
- [Stack Exchange: Steam Cloud Data](https://gaming.stackexchange.com/questions/146644)
- [Quick Guide to Steam Cloud Saves](https://www.gamedeveloper.com/game-platforms/quick-guide-to-steam-cloud-saves)

## Star History

[![Star History Chart](https://api.star-history.com/svg?repos=Fldicoahkiin/SteamCloudFileManager&type=Date)](https://star-history.com/#Fldicoahkiin/SteamCloudFileManager&Date)
