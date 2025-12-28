# SteamCloudFileManager

<p align="center">
  <img src="assets/steam_cloud-iOS-Default-1024x1024@1x.png" width="160" alt="steam_cloud" />
</p>

**English** | [简体中文](README.md)

[![Rust](https://img.shields.io/badge/rust-1.90+-orange?logo=rust)](https://www.rust-lang.org)
[![dependency status](https://deps.rs/repo/github/Fldicoahkiin/SteamCloudFileManager/status.svg)](https://deps.rs/repo/github/Fldicoahkiin/SteamCloudFileManager)
[![GitHub stars](https://img.shields.io/github/stars/Fldicoahkiin/SteamCloudFileManager?style=social)](https://github.com/Fldicoahkiin/SteamCloudFileManager/stargazers)
[![GitHub forks](https://img.shields.io/github/forks/Fldicoahkiin/SteamCloudFileManager?style=social)](https://github.com/Fldicoahkiin/SteamCloudFileManager/network/members)

[![License: GPL-3.0](https://img.shields.io/badge/License-GPL%203.0-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![GitHub release](https://img.shields.io/github/v/release/Fldicoahkiin/SteamCloudFileManager?include_prereleases&color=brightgreen)](https://github.com/Fldicoahkiin/SteamCloudFileManager/releases)
[![GitHub downloads](https://img.shields.io/github/downloads/Fldicoahkiin/SteamCloudFileManager/total?color=success)](https://github.com/Fldicoahkiin/SteamCloudFileManager/releases)
[![Build](https://github.com/Fldicoahkiin/SteamCloudFileManager/actions/workflows/build.yml/badge.svg)](https://github.com/Fldicoahkiin/SteamCloudFileManager/actions/workflows/build.yml)
[![Release](https://github.com/Fldicoahkiin/SteamCloudFileManager/actions/workflows/release.yml/badge.svg)](https://github.com/Fldicoahkiin/SteamCloudFileManager/actions/workflows/release.yml)

![Windows](https://img.shields.io/badge/Windows-0078D6?logo=microsoft&logoColor=white)
![macOS](https://img.shields.io/badge/macOS-000000?logo=apple&logoColor=white)
![Ubuntu](https://img.shields.io/badge/Ubuntu-E95420?logo=ubuntu&logoColor=white)
![Debian](https://img.shields.io/badge/Debian-A81D33?logo=debian&logoColor=white)
![Fedora](https://img.shields.io/badge/Fedora-51A2DA?logo=fedora&logoColor=white)
![Arch Linux](https://img.shields.io/badge/Arch_Linux-1793D1?logo=archlinux&logoColor=white)
![AppImage](https://img.shields.io/badge/AppImage-2c9784?logo=linux&logoColor=white)

> Cross-platform Steam Cloud file management utility built with Rust and egui.

## Features

A cloud save management utility built with Rust and the Steamworks SDK. By directly interfacing with low-level Steam APIs, it provides complete visibility and control over your cloud files. It supports uploading, downloading, and deleting individual files, and introduces a symlink sync solution to resolve cross-platform configuration sync issues.

- **VDF File Tree Visualization**: Fully parses `remotecache.vdf` to reconstruct the cloud directory structure.
- **Batch Transfer**: Supports multi-file selection and drag-and-drop for uploads/downloads.
- **Deep Control**: Directly delete cloud files and force sync status updates.
- **Root Path Mapping**: Automatically resolves Root IDs (0-12) to absolute local disk paths.
- **Search & Filter**: Supports regex search for filenames, paths, and sync status.
- **Game Library Scanning**: Automatically discovers local games by parsing `libraryfolders.vdf`.
- **Symlink Sync**: Supports mounting locally unsupported files to Steam Cloud via symlinks (Experimental).
- **Multi-Platform Support**: Windows / macOS / Linux.

## Platform Compatibility

Supports **Windows (x64)**, **macOS (Intel & Apple Silicon)**, and **Linux (x64)**.
Build artifacts include standard installers and portable versions (Generic Binary / AppImage).

> *Note: Due to upstream Steamworks SDK limitations, we cannot currently build ARM64 versions for Windows and Linux.*

## Installation

### Windows

1. Download [![Portable-x64](https://img.shields.io/badge/Portable-x64-0078D6.svg?logo=windows)](https://github.com/Fldicoahkiin/SteamCloudFileManager/releases)
2. Extract to any location
3. Double-click `SteamCloudFileManager.exe` to run

> **Note**:
>
> - Windows: Logs are saved in the `logs/` folder in the application directory.
> - macOS: Logs are saved in `~/Library/Logs/SteamCloudFileManager/` directory.
> - Linux: Logs are saved in `~/.local/share/SteamCloudFileManager/logs/` directory.

### macOS

1. Download the version for your architecture:
   - Intel: [![DMG-Intel](https://img.shields.io/badge/DMG-Intel-0071C5.svg?logo=apple)](https://github.com/Fldicoahkiin/SteamCloudFileManager/releases)
   - Apple Silicon: [![DMG-Apple Silicon](https://img.shields.io/badge/DMG-Apple%20Silicon-CDCDCD.svg?logo=apple)](https://github.com/Fldicoahkiin/SteamCloudFileManager/releases)
2. Open the DMG file
3. Drag the app to the Applications folder
4. If you encounter a "Damaged" or "Cannot be opened" error, run the following command in Terminal to fix the signature:
   
   ```bash
   xattr -c "/Applications/Steam Cloud File Manager.app"
   ```

### Linux

#### Debian/Ubuntu

Download [![Deb-x64](https://img.shields.io/badge/Deb-x64-D70A53.svg?logo=debian&logoColor=D70A53)](https://github.com/Fldicoahkiin/SteamCloudFileManager/releases) , then install:

```bash
# Install
sudo dpkg -i steam-cloud-file-manager_*.deb
sudo apt-get install -f

# Run
steam-cloud-file-manager
```

#### Fedora/RHEL/openSUSE

Download [![Rpm-x64](https://img.shields.io/badge/Rpm-x64-CC0000.svg?logo=redhat&logoColor=CC0000)](https://github.com/Fldicoahkiin/SteamCloudFileManager/releases) , then install:

```bash
# Install
sudo dnf install ./steam-cloud-file-manager-*.rpm
# or
sudo rpm -i steam-cloud-file-manager-*.rpm

# Run
steam-cloud-file-manager
```

#### AppImage (Universal)

Download [![AppImage-x64](https://img.shields.io/badge/AppImage-x64-F1C40F.svg?logo=linux)](https://github.com/Fldicoahkiin/SteamCloudFileManager/releases) , then run:

```bash
# Add execute permission
chmod +x SteamCloudFileManager-linux-x86_64.AppImage

# Run
./SteamCloudFileManager-linux-x86_64.AppImage
```

#### Arch Linux (AUR)

Download [![AUR-x64](https://img.shields.io/badge/AUR-x64-1793d1.svg?logo=arch-linux)](https://github.com/Fldicoahkiin/SteamCloudFileManager/releases) , then build and install:

```bash
# Extract AUR package
tar -xzf SteamCloudFileManager-linux-x86_64-aur.tar.gz
cd SteamCloudFileManager-linux-x86_64-aur

# Build and install with makepkg
makepkg -si

# Run
steam-cloud-file-manager
```

#### .tar.gz (Universal)

Download [![tar.gz-x64](https://img.shields.io/badge/tar.gz-x64-F0F0F0.svg?logo=linux&logoColor=F0F0F0)](https://github.com/Fldicoahkiin/SteamCloudFileManager/releases) , then extract and run:

```bash
# Extract
tar -xzf SteamCloudFileManager-linux-x86_64.tar.gz
cd SteamCloudFileManager-linux-x86_64

# Run
./SteamCloudFileManager
```

### Build from Source

```bash
git clone https://github.com/Fldicoahkiin/SteamCloudFileManager.git
cd SteamCloudFileManager
cargo build --release
```

**Build Dependencies:**

- **Cargo**
- **Rust 1.90.0+**
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

This tool relies on the Steam Client's CDP (Chrome DevTools Protocol) interface to fetch cloud file lists. **This interface is disabled by default and must be enabled by adding the `-cef-enable-debugging` argument when launching Steam.**

The software features a built-in **"Restart Steam in Debug Mode"** button to automate this. If it fails, please launch Steam manually with the argument.

- **Why is this necessary?**: We need the CDP port to inspect Steam's internal browser data, which is currently the only way to list cloud files without launching the game.

### Basic Workflow

1. Ensure Steam is running in debug mode.
2. Select the target game in the software (Logic scan or App ID).
3. Click **"Connect"** to initialize the API.
4. Once loaded, manipulate files via the tree view on the left.

App IDs can be found in Steam Store URLs or on [SteamDB](https://steamdb.info/).

> ⚠️ **Warning**
>
> - **Irreversible Action**: Deletions are committed to the local cache immediately.
> - **Data Safety**: We recommend backing up original files before batch operations.
> - **Sync Mechanism**: Changes are written to the local cache and uploaded asynchronously by Steam. Do not kill the Steam process before sync completes.

## Technical Architecture

### Cloud Sync Mechanism

```
Steam Cloud Server
        ↕ (Async background sync)
Steam Client Local Cache
        ↕ (Steam API)
Steam Cloud File Manager
```

### VDF Parsing & Root Mapping

The tool parses `remotecache.vdf` in real-time to list files. It also parses **`appinfo.vdf`** (Global App Config) to extract game cloud rules (`ufs` section), automatically handling Steam's Root ID mapping system to translate virtual paths like `Root 0` (Cloud), `Root 1` (InstallDir), and `Root 2` (Documents) into absolute local disk paths.
See source code or [ROOT_PATH_MAPPING.md](ROOT_PATH_MAPPING.md) for details.

### CDP Protocol

- Communicates with client via Steam CEF debug interface
- Retrieves cloud file list and download links in real-time
- Automatically merges cloud status into local view

### Steamworks API

- Uses `ISteamRemoteStorage` API
- Handles file upload and delete operations

## TODO

### Feature Development

- [x] Multi-language support (i18n) - Chinese/English
- [x] Version update detection
- [x] Tree view
- [x] Batch upload/download
- [x] File conflict detection and handling
- [ ] Cloud save backup and restore
- [ ] Symlink sync support (experimental)
- [ ] Automatic backup schedule

### Package Manager Support

- [ ] AUR (Arch User Repository) - `pacman -S steamcloudfilemanager`
- [ ] Homebrew (macOS) - `brew install steamcloudfilemanager`
- [ ] APT Repository (Debian/Ubuntu) - `apt install steamcloudfilemanager`
- [ ] DNF/YUM Repository (Fedora/RHEL) - `dnf install steamcloudfilemanager`
- [ ] Flatpak - `flatpak install steamcloudfilemanager`
- [ ] Snap - `snap install steamcloudfilemanager`

## Contributing

Welcome to submit Issues and Pull Requests!

### Contributing Translations

If you want to add a new language, please check the [i18n Contribution Guide](I18N_GUIDE.md).

Currently supported languages:
- 简体中文
- English

## Contributors

<a href="https://github.com/Fldicoahkiin/SteamCloudFileManager/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=Fldicoahkiin/SteamCloudFileManager" />
</a>

## Project Structure

```
src/
├── main.rs                 # Entry: initialize logger, start eframe
├── app.rs                  # Main app: state holder, UI render loop
├── app_state.rs            # State structure definitions
├── app_handlers.rs         # Business logic handlers
├── async_handlers.rs       # Async task management (channel holders)
│
├── steam_api.rs            # Steam API wrapper (CloudFile struct)
├── steam_worker.rs         # External process communication (JSON RPC)
├── steam_process.rs        # Steam process management (start/stop)
│
├── file_manager.rs         # File operations (upload/download/delete)
├── file_tree.rs            # File tree structure
├── downloader.rs           # Batch downloader
├── backup.rs               # Backup functionality
├── conflict.rs             # Conflict detection
│
├── vdf_parser.rs           # VDF file parsing (appinfo.vdf, loginusers.vdf)
├── path_resolver.rs        # Path resolution (savefiles config → actual paths)
├── cdp_client.rs           # CDP web parsing (fetch remote file list)
├── game_scanner.rs         # Game scanning (merge VDF + CDP)
├── user_manager.rs         # User management
│
├── update.rs               # Auto update
├── logger.rs               # Logging system
├── i18n.rs                 # Internationalization
├── version.rs              # Version info
│
└── ui/
    ├── mod.rs              # UI module exports
    ├── app_panels.rs       # Panel rendering (top/bottom/center, action buttons, status bar)
    ├── controls.rs         # Control rendering
    ├── file_list.rs        # File list (table/tree view)
    ├── windows.rs          # Windows (game selector, user selector)
    ├── settings.rs         # Settings window
    ├── theme.rs            # Theme system (dark/light mode)
    ├── upload_dialog.rs    # Upload dialog
    ├── backup_dialog.rs    # Backup dialog
    ├── conflict_dialog.rs  # Conflict dialog
    ├── guide_dialog.rs     # Guide dialog
    ├── appinfo_dialog.rs   # AppInfo dialog
    └── font_loader.rs      # Font loader
```

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

<a href="https://star-history.com/#Fldicoahkiin/SteamCloudFileManager&Date">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/svg?repos=Fldicoahkiin/SteamCloudFileManager&type=Date&theme=dark" />
    <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/svg?repos=Fldicoahkiin/SteamCloudFileManager&type=Date" />
    <img alt="Star History Chart" src="https://api.star-history.com/svg?repos=Fldicoahkiin/SteamCloudFileManager&type=Date" />
  </picture>
</a>