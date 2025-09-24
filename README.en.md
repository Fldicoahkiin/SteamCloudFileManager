# SteamCloudFileManager

**English** | [简体中文](README.md)

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-%E2%89%A51.70-orange)](https://www.rust-lang.org)
[![Platform](https://img.shields.io/badge/platform-Windows%20|%20macOS%20|%20Linux-lightgrey)](https://github.com/Fldicoahkiin/SteamCloudFileManager)

> Cross-platform Steam Cloud file management utility built with Rust and egui.

This tool provides a graphical interface for managing Steam Cloud save files across different games. It connects directly to Steam's Remote Storage API, allowing users to view, download, upload, and manage their cloud saves without launching individual games.

## Table of Contents

- [Background](#background)
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

## Background

Steam Cloud automatically synchronizes game saves across devices, but lacks a unified interface for direct file management. This project addresses that gap by providing:

- Direct access to Steam Remote Storage without launching games
- Batch operations for multiple save files
- Cross-platform compatibility with native system integration
- Real-time quota monitoring and file metadata display

The implementation uses the Steamworks SDK through [steamworks-rs](https://github.com/Thinkofname/steamworks-rs) bindings, ensuring compatibility with official Steam APIs.

## Install

### Dependencies

**Runtime Requirements:**
- Steam client (must be running and logged in)
- Operating System:
  - Windows 10 or later
  - macOS 10.15 (Catalina) or later
  - Linux with glibc 2.31+ (Ubuntu 20.04, Debian 11, Fedora 34, or equivalent)

**Build Requirements:**
- Rust 1.70 or later
- Cargo package manager
- C++ build tools (platform-specific):
  - Windows: Visual Studio 2019+ or Build Tools for Visual Studio
  - macOS: Xcode Command Line Tools
  - Linux: gcc/g++ or clang

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

| Function | Status | Description |
|----------|--------|-------------|
| `GetFileCount()` | ✅ | Retrieve total file count |
| `GetFileNameAndSize()` | ✅ | Get file metadata |
| `FileExists()` | ✅ | Check file existence |
| `FilePersisted()` | ✅ | Verify persistence status |
| `GetFileTimestamp()` | ✅ | Retrieve modification time |
| `FileRead()` | ✅ | Download file content |
| `FileWrite()` | ✅ | Upload file content |
| `FileDelete()` | ✅ | Remove file from cloud |
| `FileForget()` | ✅ | Stop tracking file |
| `IsCloudEnabledForAccount()` | ✅ | Check account cloud status |
| `IsCloudEnabledForApp()` | ✅ | Check app cloud status |
| `SetCloudEnabledForApp()` | ✅ | Toggle app cloud sync |

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

## Contributing

Welcome to submit Issues and Pull Requests

## Contributors

<a href="https://github.com/Fldicoahkiin/SteamCloudFileManager/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=Fldicoahkiin/SteamCloudFileManager" />
</a>

## License

MIT License - see [LICENSE](LICENSE) file for details.

Copyright (c) 2025 SteamCloudFileManager Contributors

## Thanks

- [SteamCloudFileManagerLite](https://github.com/GMMan/SteamCloudFileManagerLite)
- [Steamworks SDK](https://partner.steamgames.com/doc/sdk/api)

## Star History

[![Star History Chart](https://api.star-history.com/svg?repos=Fldicoahkiin/SteamCloudFileManager&type=Date)](https://star-history.com/#Fldicoahkiin/SteamCloudFileManager&Date)
