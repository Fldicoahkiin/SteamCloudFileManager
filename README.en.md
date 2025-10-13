# SteamCloudFileManager
<p align="center">
  <img src="assets/steam_cloud-iOS-Default-1024x1024@1x.png" width="160" alt="steam_cloud" />
</p>

**English** | [简体中文](README.md)

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.90+-orange)](https://www.rust-lang.org)
[![Platform](https://img.shields.io/badge/platform-Windows%20|%20macOS%20|%20Linux-lightgrey)](https://github.com/Fldicoahkiin/SteamCloudFileManager)

> Cross-platform Steam Cloud file management utility built with Rust and egui.

This tool provides a graphical interface for managing Steam Cloud save files across different games. It connects directly to Steam's Remote Storage API, allowing users to view, download, upload, and manage their cloud saves without launching individual games.

## Table of Contents

- [Install](#install)
  - [Dependencies](#dependencies)
  - [Building from Source](#building-from-source)
- [Usage](#usage)
  - [Basic Operations](#basic-operations)
  - [File Management](#file-management)
  - [Switching Games](#switching-games)
- [API](#api)
- [Contributing](#contributing)
- [License](#license)
- [Thanks](#thanks)

## Install

### Dependencies

**Runtime Requirements:**
- Steam client (must be running and logged in)
- Operating System:
  - Windows 10 or later
  - macOS 10.15 (Catalina) or later
  - Linux with glibc 2.31+ (Ubuntu 20.04, Debian 11, Fedora 34, or equivalent)

**Build Requirements:**
- **Rust 1.88+** (recommended 1.90.0 or later)
  - edition 2021
  - egui 0.33 requires Rust 1.88+
- Cargo package manager
- C++ build tools (platform-specific):
  - Windows: Visual Studio 2019+ or Build Tools for Visual Studio
  - macOS: Xcode Command Line Tools
  - Linux: gcc/g++ or clang

**Key Dependencies:**
```toml
eframe = "0.33"        # GUI framework
egui = "0.33"          # Immediate mode UI
egui_extras = "0.33"  # Table component
steamworks = "0.12"    # Steam API bindings
chrono = "0.4"         # Time handling
rfd = "0.15"           # File dialogs
```

### Building from Source

Clone the repository:
```bash
git clone https://github.com/yourusername/SteamCloudFileManager.git
cd SteamCloudFileManager
```

Build the project:
```bash
cargo build --release
```

The compiled binary will be located at:
- Windows: `target/release/SteamCloudFileManager.exe`
- macOS/Linux: `target/release/SteamCloudFileManager`

## Usage

### Basic Operations

1. **Launch Steam Client**
   Ensure Steam is running and you're logged into your account.

2. **Start the Application**
   ```bash
   ./target/release/SteamCloudFileManager
   ```

3. **Connect to a Game**
   - Enter the game's App ID in the input field
   - Click "连接" (Connect)
   - Files will load automatically

   To find App IDs:
   - Steam Store URL: `https://store.steampowered.com/app/[APP_ID]/`
   - SteamDB: https://steamdb.info/

### File Management

**Download Files**
1. Select one or more files from the list
2. Click "下载选中文件" (Download Selected)
3. Choose save location in the file dialog

**Upload Files**
1. Click "上传文件" (Upload File)
2. Select local file(s) to upload
3. Files are immediately synchronized to Steam Cloud

**Delete Files**
1. Select target files
2. Click "删除选中文件" (Delete Selected)
3. Confirm the deletion

**Cancel Cloud Sync**
1. Select files to unsync
2. Click "取消云同步" (Cancel Cloud Sync)
3. Files remain locally but stop syncing

### Switching Games

To manage files for a different game:
1. Click "断开连接" (Disconnect)
2. Enter new App ID
3. Click "连接" (Connect)

Alternatively, enter a new App ID directly and connect to switch immediately.

## API

The application interfaces with Steam through these primary APIs:

### Steam Remote Storage APIs

| Function | Description |
|----------|-------------|
| `GetFileCount()` | Retrieve total file count |
| `GetFileNameAndSize()` | Get file metadata |
| `FileExists()` | Check file existence |
| `FilePersisted()` | Verify persistence status |
| `GetFileTimestamp()` | Retrieve modification time |
| `FileRead()` | Download file content |
| `FileWrite()` | Upload file content |
| `FileDelete()` | Remove file from cloud |
| `FileForget()` | Stop tracking file |
| `IsCloudEnabledForAccount()` | Check account cloud status |
| `IsCloudEnabledForApp()` | Check app cloud status |
| `SetCloudEnabledForApp()` | Toggle app cloud sync |

### Internal APIs

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

### VDF File Parsing

This tool uses a **dual-approach strategy** to ensure maximum compatibility:

**Primary Method: VDF Parsing**
- Directly reads `remotecache.vdf` file for complete file list
- Supports all root path types (0-12), not just `remote/` folder
- Displays actual file storage locations on local disk
- Works with most modern games

**Fallback Method: Steam API**
- Automatically falls back when VDF file is missing or parsing fails
- Uses `ISteamRemoteStorage` API
- Only supports root=0 files
- Ensures basic functionality

For more technical details, see [STEAM_CLOUD_LIMITATIONS.md](STEAM_CLOUD_LIMITATIONS.md)

## Contributing

Welcome to submit Issues and Pull Requests

## Contributors

<a href="https://github.com/Fldicoahkiin/SteamCloudFileManager/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=Fldicoahkiin/SteamCloudFileManager" />
</a>

## License

MIT License - see [LICENSE](LICENSE) file for details.

## References

### Official Documentation
- [Steamworks Steam Cloud Documentation](https://partner.steamgames.com/doc/features/cloud) - Root Paths Configuration
- [ISteamRemoteStorage API](https://partner.steamgames.com/doc/api/ISteamRemoteStorage) - C++ API Reference
- [Steamworks SDK](https://partner.steamgames.com/doc/sdk) - Complete SDK Download

### Community Verification
- [Stack Exchange: What data is in Steam Cloud?](https://gaming.stackexchange.com/questions/146644) - Root Value Mapping Confirmation
- Reddit r/Steam - VDF File Format Discussion

### Open Source Implementations
- [Facepunch.Steamworks](https://github.com/Facepunch/Facepunch.Steamworks) - C# Steamworks Wrapper
- [VDF Parser (Python)](https://github.com/ValvePython/vdf) - VDF File Parser Library
- [Rust Steamworks](https://github.com/Thinkofname/steamworks-rs) - Rust Bindings Used by This Project

### Technical Articles
- [Quick Guide to Steam Cloud Saves](https://www.gamedeveloper.com/game-platforms/quick-guide-to-steam-cloud-saves) - Root Override Configuration

## Thanks

- [SteamCloudFileManagerLite](https://github.com/GMMan/SteamCloudFileManagerLite)
- [Steamworks SDK](https://partner.steamgames.com/doc/sdk/api)

## Star History

[![Star History Chart](https://api.star-history.com/svg?repos=Fldicoahkiin/SteamCloudFileManager&type=Date)](https://star-history.com/#Fldicoahkiin/SteamCloudFileManager&Date)
