# SteamCloudFileManager

<p align="center">
  <img src="assets/steam_cloud-iOS-Default-1024x1024@1x.png" width="160" alt="steam_cloud" />
</p>

**English** | [简体中文](README.md)

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

> Cross-platform Steam Cloud file management utility built with Rust and egui.

## Features

A GUI tool for managing Steam Cloud saves, allowing direct cloud file operations without launching games.

Steam's built-in cloud save management is quite basic. This tool provides more complete file listings and batch operation support:

- View complete cloud save file list (tree view)
- Batch download/upload files (drag & drop support)
- Delete or unsync specific files
- File search and filtering (local/cloud)
- Quickly switch between games (auto-scan game library)
- View actual file locations on local disk
- Display cloud sync status and quota
- Multi-user support

## Platform Support

| Platform | Architecture | Status | Package Format |
|----------|--------------|--------|----------------|
| Windows | x64 | ✅ Supported | `.zip` |
| Windows | ARM64 | ❌ Not Supported | Steam SDK does not provide ARM64 binaries |
| macOS | Intel (x64) | ✅ Supported | `.dmg` |
| macOS | Apple Silicon (ARM64) | ✅ Supported | `.dmg` |
| Linux | x64 | ✅ Supported | `.deb`, `.rpm`, `.AppImage`, `.tar.gz` |

## Installation

### Windows

1. Download `SteamCloudFileManager-windows-x86_64.zip` from [Releases](https://github.com/Fldicoahkiin/SteamCloudFileManager/releases)
2. Extract to any location
3. Double-click `SteamCloudFileManager.exe` to run

> **Note**:
>
> - Windows: Logs are saved in the `logs/` folder in the application directory.
> - macOS: Logs are saved in `~/Library/Logs/SteamCloudFileManager/` directory.
> - Linux: Logs are saved in `~/.local/share/SteamCloudFileManager/logs/` directory.

### macOS

1. Download from [Releases](https://github.com/Fldicoahkiin/SteamCloudFileManager/releases):
   - Intel: `SteamCloudFileManager-macos-x86_64.dmg`
   - Apple Silicon: `SteamCloudFileManager-macos-aarch64.dmg`
2. Open the DMG file
3. Drag the app to Applications folder

### Linux

#### Debian/Ubuntu

Download the `.deb` package from [Releases](https://github.com/Fldicoahkiin/SteamCloudFileManager/releases), then install:

```bash
# Install
sudo dpkg -i steamcloudfilemanager_*.deb
sudo apt-get install -f

# Run
steamcloudfilemanager
```

#### Fedora/RHEL/openSUSE

Download the `.rpm` package from [Releases](https://github.com/Fldicoahkiin/SteamCloudFileManager/releases), then install:

```bash
# Install
sudo dnf install ./steamcloudfilemanager-*.rpm
# or
sudo rpm -i steamcloudfilemanager-*.rpm

# Run
steamcloudfilemanager
```

#### AppImage (Universal)

Download the `.AppImage` file from [Releases](https://github.com/Fldicoahkiin/SteamCloudFileManager/releases), then run:

```bash
# Add execute permission
chmod +x SteamCloudFileManager-linux-x86_64.AppImage

# Run
./SteamCloudFileManager-linux-x86_64.AppImage
```

#### Arch Linux (AUR)

Download the AUR package from [Releases](https://github.com/Fldicoahkiin/SteamCloudFileManager/releases), then build and install:

```bash
# Extract AUR package
tar -xzf SteamCloudFileManager-linux-x86_64-aur.tar.gz
cd SteamCloudFileManager-linux-x86_64-aur

# Build and install with makepkg
makepkg -si

# Run
steamcloudfilemanager
```

#### .tar.gz (Universal)

Download the `.tar.gz` package from [Releases](https://github.com/Fldicoahkiin/SteamCloudFileManager/releases), then extract and run:

```bash
# Extract
tar -xzf SteamCloudFileManager-linux-x86_64.tar.gz
cd SteamCloudFileManager-linux-x86_64

# Run
./steamcloudfilemanager
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

**Why debug mode is required?**

- CDP (Chrome DevTools Protocol) is the debugging interface for Steam's built-in browser
- We use this interface to retrieve cloud file lists and download links
- CDP port is only enabled when debug mode is activated

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

### Important Notes

> ⚠️ **Important**

**Deletion Risks**:

- Deleting cloud save files is **irreversible**
- Deleted files will be synced and removed from all devices
- Make sure to backup important files before deletion

**Backup Recommendations**:

- Download backups before any deletion or modification
- Regularly backup important game saves
- Use "Batch Download" feature to quickly backup entire game saves

**Cloud Sync Notes**:

- After upload/delete, wait for Steam to complete sync (usually after disconnect)
- Do not close Steam or shutdown during sync

## Technical Architecture

### Cloud Sync Mechanism

Steam cloud sync uses a three-tier architecture:

```
Steam Cloud Server
        ↕ (Async background sync)
Steam Client Local Cache
        ↕ (Steam API)
Our Software
```

**Important Notes**:

- Upload/delete operations write to **local cache** immediately
- Actual sync to cloud is performed **asynchronously** in the background
- **After disconnect**, Steam automatically triggers sync - this is Steam's safety mechanism
- For details, see [CLOUD_SYNC_EXPLAINED.md](docs/CLOUD_SYNC_EXPLAINED.md)

### VDF Parsing

- Directly reads `remotecache.vdf` to get complete file list
- Shows actual storage location of files on local disk
- Supports all Root path types (0-12)

**What is Root Path?**

Root path is the file storage location type in Steam's cloud save system. Different games may save their files in different directories:

- **Root 0** - Steam Cloud default directory (`userdata/{user_id}/{app_id}/remote/`)
- **Root 1** - Game installation directory (`steamapps/common/{GameDir}/`)
- **Root 2** - Documents folder (Windows: `Documents/`, macOS: `~/Documents/`)
- **Root 3** - AppData Roaming (Windows: `%APPDATA%`, macOS: `~/Library/Application Support/`)
- **Root 7** - macOS Application Support / Windows Videos
- **Root 12** - Windows LocalLow / macOS Caches
- **Others** - Pictures, Music, Videos, Desktop, and other system folders

Our software automatically identifies and displays the actual storage location of each file.

> **Note**: The Root path mapping table is still being updated. Different games may use different Root values, and cross-platform behavior may vary. (Not fully tested yet🥺👉👈

- **[Root Path Mapping Table](ROOT_PATH_MAPPING.md)** - Complete path mapping rules and game examples

### CDP Protocol

- Communicates with client via Steam CEF debug interface
- Retrieves cloud file list and download links in real-time
- Automatically merges cloud status into local view

### Steamworks API

- Uses `ISteamRemoteStorage` API
- Handles file upload and delete operations

## TODO

### Feature Development

- [ ] File conflict detection and handling
- [ ] Multi-language support (i18n)
- [ ] Cloud save backup and restore
- [ ] File comparison (local vs cloud)
- [ ] Automatic backup schedule
- [ ] Version update detection
- [ ] Batch file renaming
- [ ] File version history management
- [ ] Symlink sync support (experimental)

### Package Manager Support

- [ ] AUR (Arch User Repository) - `pacman -S steamcloudfilemanager`
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

- [cargo-bundle](https://github.com/burtonageo/cargo-bundle) - macOS .dmg
- [cargo-deb](https://github.com/kornelski/cargo-deb) - Debian/Ubuntu .deb
- [cargo-generate-rpm](https://github.com/cat-in-136/cargo-generate-rpm) - Fedora/RHEL .rpm
- [cargo-appimage](https://github.com/StratusFearMe21/cargo-appimage) - Universal AppImage
- [cargo-aur](https://github.com/fosskers/cargo-aur) - Arch Linux PKGBUILD

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
