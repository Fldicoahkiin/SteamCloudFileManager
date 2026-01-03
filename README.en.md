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

#### Homebrew

```bash
brew tap Fldicoahkiin/tap
brew install steam-cloud-file-manager
```

#### Manual Installation

1. Download the version for your architecture:
   - Intel: [![DMG-Intel](https://img.shields.io/badge/DMG-Intel-0071C5.svg?logo=apple)](https://github.com/Fldicoahkiin/SteamCloudFileManager/releases)
   - Apple Silicon: [![DMG-Apple Silicon](https://img.shields.io/badge/DMG-Apple%20Silicon-CDCDCD.svg?logo=apple)](https://github.com/Fldicoahkiin/SteamCloudFileManager/releases)
2. Open the DMG file
3. Drag the app to the Applications folder
4. If you encounter a "Damaged" or "Cannot be opened" error, run the following command in Terminal to fix the signature:
   
   ```bash
   xattr -c "/Applications/Steam Cloud File Manager.app"
   ```

### Arch Linux (AUR)

```bash
yay -S steam-cloud-file-manager-bin
# or
paru -S steam-cloud-file-manager-bin
```

Manual build:

```bash
git clone https://aur.archlinux.org/steam-cloud-file-manager-bin.git
cd steam-cloud-file-manager-bin
makepkg -si
steam-cloud-file-manager
```

Or download [![AUR-x64](https://img.shields.io/badge/AUR-x64-1793d1.svg?logo=arch-linux)](https://github.com/Fldicoahkiin/SteamCloudFileManager/releases) pre-built package:

```bash
tar -xzf SteamCloudFileManager-*-linux-x86_64-aur.tar.gz
cd SteamCloudFileManager-*-linux-x86_64-aur
makepkg -si
steam-cloud-file-manager
```


### Debian/Ubuntu

Download [![Deb-x64](https://img.shields.io/badge/Deb-x64-D70A53.svg?logo=debian&logoColor=D70A53)](https://github.com/Fldicoahkiin/SteamCloudFileManager/releases)

```bash
sudo dpkg -i steam-cloud-file-manager_*.deb
sudo apt-get install -f
steam-cloud-file-manager
```

### Fedora/RHEL/openSUSE

Download [![Rpm-x64](https://img.shields.io/badge/Rpm-x64-CC0000.svg?logo=redhat&logoColor=CC0000)](https://github.com/Fldicoahkiin/SteamCloudFileManager/releases)

```bash
sudo dnf install ./steam-cloud-file-manager-*.rpm
steam-cloud-file-manager
```

### AppImage (Universal)

Download [![AppImage-x64](https://img.shields.io/badge/AppImage-x64-F1C40F.svg?logo=linux)](https://github.com/Fldicoahkiin/SteamCloudFileManager/releases)

```bash
chmod +x SteamCloudFileManager-*.AppImage
./SteamCloudFileManager-*.AppImage
```

### .tar.gz (Universal)

Download [![tar.gz-x64](https://img.shields.io/badge/tar.gz-x64-F0F0F0.svg?logo=linux&logoColor=F0F0F0)](https://github.com/Fldicoahkiin/SteamCloudFileManager/releases)

```bash
tar -xzf SteamCloudFileManager-*-linux-x86_64.tar.gz
./steam-cloud-file-manager
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

This tool uses the CDP protocol to communicate with Steam, so you **must** launch Steam with debugging enabled.

**Why is this required?**

- CDP (Chrome DevTools Protocol) is the debugging interface for Steam's built-in browser.
- We use this interface to fetch the cloud file list and download links.
- The CDP port is only active when debug mode is enabled.

**Windows:**

1. Right-click your Steam shortcut and select "Properties".
2. Add `-cef-enable-debugging` to the end of the "Target" field.
3. Click "OK" and launch Steam.

**macOS:**

1. Quit Steam.
2. Run in Terminal:

   ```bash
   open -a Steam --args -cef-enable-debugging
   ```

**Linux:**

1. Close Steam.
2. Run in Terminal:

   ```bash
   steam -cef-enable-debugging &
   ```

   Or edit your Steam shortcut and append `-cef-enable-debugging` to the Exec line.

**Note:** The software features a built-in "Restart Steam in Debug Mode" button that automates this process.

### Basic Workflow

1. Ensure Steam is running in debug mode.
2. Select the target game:
   - **Game Library**: Click the game library button to select a local game (auto-connects).
   - **Manual Input**: Enter App ID and click **"Connect"**.
3. Once loaded, manipulate files via the tree view on the left.

App IDs can be found in Steam Store URLs or on [SteamDB](https://steamdb.info/).

> ⚠️ **Warning**
>
> - **Irreversible Action**: Deletions are committed to the local cache immediately.
> - **Data Safety**: We recommend backing up original files before batch operations.
> - **Sync Mechanism**: Changes are written to the local cache and uploaded asynchronously by Steam. Do not kill the Steam process before sync completes.

## Technical Architecture

### Core Flow

```mermaid
graph TD
    User([User Action]) -->|Select Game/File| App[Steam Cloud File Manager]
    App --> VDF["VDF Parser: remotecache.vdf"]
    VDF --> PathResolver["Path Resolver: Root ID Mapping"]
    PathResolver --> FileList[File List View]
    FileList --> CDP["CDP Client: Get Download Links"]
    CDP --> SteamBrowser["Steam Built-in Browser 127.0.0.1:8080"]
    SteamBrowser --> CloudPage["Cloud Storage Page store.steampowered.com"]
    CloudPage -->|File List + Download Links| FileList
    FileList -->|Upload/Download/Delete| SteamAPI["Steam API: ISteamRemoteStorage"]
    SteamAPI --> LocalCache["Local Cache userdata/uid/appid/remote/"]
    LocalCache -.->|Background Async Sync| CloudServer[Steam Cloud Server]
    CloudServer -.->|Sync to Other Devices| LocalCache
    LocalCache -->|Refresh| VDF
```

### Data Flow

```mermaid
graph TD
    subgraph Toolbar
        Upload[Upload]
        Download[Download]
        SyncToCloud[Sync to Cloud]
        Delete[Delete]
        Forget[Forget]
        Compare[Compare Files]
        Refresh[Refresh]
        Backup[Backup]
    end
    
    subgraph SteamAPI[Steam API]
        WriteFile[write_file]
        ReadFile[read_file]
        DeleteFile[delete_file]
        ForgetFile[forget_file]
        SyncCloud[sync_cloud_files]
    end
    
    subgraph DataFetch[Data Fetching]
        CDPClient[CDP Client]
        VDFParser[VDF Parser]
    end
    
    Upload -->|Select Local File| WriteFile
    WriteFile --> SyncCloud
    
    Download --> CDPClient
    CDPClient -->|Get Download URL| HTTP[HTTP Download]
    HTTP --> LocalDisk[Local Disk]
    
    SyncToCloud -->|Upload Local-Only Files| WriteFile
    
    Delete --> DeleteFile
    DeleteFile --> SyncCloud
    
    Forget -->|Remove from Cloud, Keep Local| ForgetFile
    ForgetFile --> SyncCloud
    
    Compare --> CDPClient
    Compare --> VDFParser
    CDPClient -->|Calculate Cloud Hash| HashCompare[Hash Compare]
    VDFParser -->|Calculate Local Hash| HashCompare
    
    Backup --> ReadFile
    ReadFile --> LocalDisk
    
    Refresh --> VDFParser
    Refresh --> CDPClient
    VDFParser --> FileList[File List]
    CDPClient --> FileList
    
    SyncCloud -.->|Background Async| CloudServer[Steam Cloud]
```

### Data Source Priority

| Source | Data Content | Priority | Description |
|--------|--------------|----------|-------------|
| **VDF** | Local cached file list, sync status | Primary | Parses `remotecache.vdf` |
| **CDP** | Real-time cloud file list, download URLs | Supplement | Via Steam's built-in browser |
| **Steam API** | File read/write/delete, quota query | Operations | `ISteamRemoteStorage` interface |

### Sync Status (`is_persisted`)

```
Newly uploaded file
  is_persisted = false  ← Only in local cache
  ↓
  Steam background upload (takes seconds to minutes)
  ↓
  is_persisted = true   ← Synced to cloud
```

> ⚠️ **Important**: `sync_cloud_files()` returns immediately; actual upload happens asynchronously in background. Steam forces sync completion on disconnect.

### CDP Protocol

Fetches real-time cloud data via Steam's CEF (Chromium Embedded Framework) debug interface:

1. **Detect**: Access `http://127.0.0.1:8080/json` to get debug target list
2. **Connect**: Establish WebSocket connection to target page
3. **Navigate**: Go to `store.steampowered.com/account/remotestorage`
4. **Inject**: Execute JavaScript to extract file list and download URLs
5. **Merge**: Combine CDP data with VDF data, supplementing download URLs and real-time status

### VDF Parsing & Root Mapping

The tool parses `remotecache.vdf` in real-time for file lists, and parses **`appinfo.vdf`** to extract game cloud rules (`ufs` section), automatically handling Steam's Root ID mapping:

| Root ID | Meaning | Example Path (Windows) |
|---------|---------|----------------------|
| 0 | Cloud (Steam cloud directory) | `userdata/{uid}/{appid}/remote/` |
| 1 | InstallDir (Game install directory) | `steamapps/common/GameName/` |
| 2 | Documents (My Documents) | `C:/Users/xxx/Documents/` |
| 3 | SavedGames | `C:/Users/xxx/Saved Games/` |

- **[Root Path Mapping Table](ROOT_PATH_MAPPING.md)** - Complete path mapping rules

> **Note**: The Root path mapping table is continuously updated. Different games may use different Root values, and cross-platform behavior may vary.


## TODO

### Feature Development

- [x] Multi-language support (i18n) - Chinese/English
- [x] Version update detection
- [x] Tree view
- [x] Batch upload/download
- [x] File conflict detection and handling
- [x] Cloud save backup
- [x] Symlink sync support (experimental)
- [ ] Automatic backup schedule

### Package Manager Support

- [x] AUR (Arch User Repository) - `yay -S steam-cloud-file-manager-bin`
- [x] Homebrew (macOS) - `brew tap Fldicoahkiin/tap && brew install steam-cloud-file-manager`
- [ ] APT Repository (Debian/Ubuntu) - `apt install steam-cloud-file-manager`
- [ ] DNF/YUM Repository (Fedora/RHEL) - `dnf install steam-cloud-file-manager`
- [ ] Flatpak - `flatpak install steam-cloud-file-manager`
- [ ] Snap - `snap install steam-cloud-file-manager`

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
├── symlink_manager.rs      # Symlink management
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
├── icons.rs                # Icon system (Phosphor Icons)
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
    ├── symlink_dialog.rs   # Symlink dialog
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

- [rfd](https://github.com/PolyMeilex/rfd) - Native file dialogs
- [ureq](https://github.com/algesten/ureq) - HTTP client
- [anyhow](https://github.com/dtolnay/anyhow) - Error handling
- [tracing](https://github.com/tokio-rs/tracing) - Logging/tracing
- [serde](https://github.com/serde-rs/serde) - Serialization framework
- [image](https://github.com/image-rs/image) - Image processing
- [self_update](https://github.com/jaemk/self_update) - Auto update
- [regex](https://github.com/rust-lang/regex) - Regular expressions
- [chrono](https://github.com/chronotope/chrono) - Date/time
- [walkdir](https://github.com/BurntSushi/walkdir) - Directory traversal
- [open](https://github.com/Byron/open-rs) - Open files/URLs
- [dirs](https://github.com/dirs-dev/dirs-rs) - System directories
- [uuid](https://github.com/uuid-rs/uuid) - UUID generation
- [sha1](https://github.com/RustCrypto/hashes) - Hash computation
- [byteorder](https://github.com/BurntSushi/byteorder) - Byte order handling
- [url](https://github.com/servo/rust-url) - URL parsing

### UI Extensions

- [egui-phosphor](https://github.com/amPerl/egui-phosphor) - Phosphor icon library
- [egui_extras](https://github.com/emilk/egui/tree/master/crates/egui_extras) - egui extension components

### Packaging Tools

- [cargo-bundle](https://github.com/burtonageo/cargo-bundle) - macOS .dmg
- [cargo-deb](https://github.com/kornelski/cargo-deb) - Debian/Ubuntu .deb
- [cargo-generate-rpm](https://github.com/cat-in-136/cargo-generate-rpm) - Fedora/RHEL .rpm
- [cargo-appimage](https://github.com/StratusFearMe21/cargo-appimage) - Universal AppImage
- [cargo-aur](https://github.com/fosskers/cargo-aur) - Arch Linux PKGBUILD

### Reference Projects

- [SteamCloudFileManagerLite](https://github.com/GMMan/SteamCloudFileManagerLite)
- [Facepunch.Steamworks](https://github.com/Facepunch/Facepunch.Steamworks)
- [SteamTools (Watt Toolkit)](https://github.com/BeyondDimension/SteamTools)

### Documentation

- [Steamworks SDK](https://partner.steamgames.com/doc/sdk/api)
- [Steamworks Steam Cloud Documentation](https://partner.steamgames.com/doc/features/cloud)
- [VDF Parser (Python)](https://github.com/ValvePython/vdf)
- [Stack Exchange: Steam Cloud Data](https://gaming.stackexchange.com/questions/146644)
- [Quick Guide to Steam Cloud Saves](https://www.gamedeveloper.com/game-platforms/quick-guide-to-steam-cloud-saves)
- [Elena Temple Dev Blog: Steam Cloud Saves](https://www.grimtalin.com/2018/04/elena-temple-steam-cloud-saves.html)

## Star History

<a href="https://star-history.com/#Fldicoahkiin/SteamCloudFileManager&Date">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/svg?repos=Fldicoahkiin/SteamCloudFileManager&type=Date&theme=dark" />
    <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/svg?repos=Fldicoahkiin/SteamCloudFileManager&type=Date" />
    <img alt="Star History Chart" src="https://api.star-history.com/svg?repos=Fldicoahkiin/SteamCloudFileManager&type=Date" />
  </picture>
</a>