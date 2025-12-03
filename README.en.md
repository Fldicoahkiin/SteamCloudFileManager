# SteamCloudFileManager

<p align="center">
  <img src="assets/steam_cloud-iOS-Default-1024x1024@1x.png" width="160" alt="steam_cloud" />
</p>

**English** | [简体中文](README.md)

[![License: GPL-3.0](https://img.shields.io/badge/License-GPL%203.0-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![Rust](https://img.shields.io/badge/rust-1.90+-orange)](https://www.rust-lang.org)
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

| Platform | Architecture | Status | Notes |
|----------|--------------|--------|-------|
| Windows | x64 | ✅ Supported | |
| Windows | ARM64 | ❌ Not Supported | Steam SDK does not provide ARM64 binaries |
| macOS | Intel (x64) | ✅ Supported | |
| macOS | Apple Silicon (ARM64) | ✅ Supported | |
| Linux | x64 | ✅ Supported | |


## Installation

Download precompiled binaries from [Releases](https://github.com/Fldicoahkiin/SteamCloudFileManager/releases)

Or build from source:

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

- [ ] Batch download
- [ ] Batch upload
- [ ] Drag-and-drop upload
- [ ] Hierarchical folder management
- [ ] Multi-language support

## Contributing

Welcome to submit Issues and Pull Requests

## Contributors

<a href="https://github.com/Fldicoahkiin/SteamCloudFileManager/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=Fldicoahkiin/SteamCloudFileManager" />
</a>

## License

This project is licensed under GPL-3.0 - see [LICENSE](LICENSE) file for details.

## Acknowledgments

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
