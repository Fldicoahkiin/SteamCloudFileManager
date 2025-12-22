# Steam Cloud Root 路径映射表

> **📋 数据来源**  
> 此映射表通过 **appinfo.vdf (ufs)** 和 **运行日志** 验证。

---

## 数据来源说明

### 1. appinfo.vdf (ufs 配置)

Steam 在本地缓存了每个游戏的云存储配置，位于：
```
{Steam安装目录}/appcache/appinfo.vdf
```

**查看方法**：
1. 在 SteamCloudFileManager 中连接到游戏
2. 点击状态栏的 **"显示 appinfo.vdf"** 按钮

**ufs 配置示例** (The Witcher 3)：
```vdf
"ufs"
{
    "quota" "1000000000"
    "maxnumfiles" "1000"
    "savefiles"
    {
        "0"
        {
            "root" "WinMyDocuments"      // ← 字符串形式
            "path" "/The Witcher 3/gamesaves/"
            "pattern" "*"
            "platforms"
            {
                "1" "Windows"
            }
        }
    }
}
```

> ⚠️ **注意**：appinfo.vdf 中的 `root` 是**字符串名称**（如 "WinMyDocuments"），需要与日志中的数字对照。

### 2. 运行日志 (root 数字)

运行程序时，日志会显示 root 的**数字编号**：

```log
[文件名] | VDF root=2 | CDP folder=WinMyDocuments | [大小] | [时间] | [本地路径]
```

**日志示例**：
```
The Witcher 3/gamesaves/save.sav | VDF root=2 | CDP folder=WinMyDocuments | 3.02 MB
freebirdgames/findingparadise/Save4.rxdata | VDF root=7 | CDP folder=MacAppSupport | 316 KB
```

### 3. 数据来源对照表

| 来源 | root 格式 | 示例 |
|------|-----------|------|
| **appinfo.vdf (ufs)** | 字符串名称 | `"root" "WinMyDocuments"` |
| **remotecache.vdf** | 数字 | `"root" "2"` |
| **运行日志** | 数字 + 名称 | `VDF root=2 \| CDP folder=WinMyDocuments` |

### 4. 字符串名称与数字对照

| 数字 | 字符串名称 | 说明 |
|:----:|------------|------|
| 0 | `SteamCloudDocuments` | Steam Cloud 默认 |
| 1 | `GameInstall` | 游戏安装目录 |
| 2 | `WinMyDocuments` | 文档 |
| 3 | `WinAppDataRoaming` | AppData/Roaming |
| 4 | `WinAppDataLocal` | AppData/Local |
| 5 | `WinPictures` | 图片 |
| 6 | `WinMusic` | 音乐 |
| 7 | `MacAppSupport` / `WinVideos` | macOS: Application Support |
| 8 | `LinuxXdgDataHome` | Linux: ~/.local/share |
| 9 | `WinSavedGames` | Saved Games |
| 10 | `WinDownloads` | 下载 |
| 11 | `WinPublic` | Public |
| 12 | `WinAppDataLocalLow` | AppData/LocalLow |

---

## Root 映射总表

| Root | CDP 文件夹名 | Windows | macOS | Linux | 状态 |
|:----:|-------------|---------|-------|-------|:----:|
| [0](#root0) | `Steam Cloud` | `{Steam}/userdata/{UID}/{AppID}/remote/` | `{Steam}/userdata/{UID}/{AppID}/remote/` | `{Steam}/userdata/{UID}/{AppID}/remote/` | ⚠️ |
| [1](#root1) | `GameInstall` | `{Steam}/steamapps/common/{Game}/` | `{Steam}/steamapps/common/{Game}/` | `{Steam}/steamapps/common/{Game}/` | ⚠️ |
| [2](#root2) | `Documents` | `%USERPROFILE%\Documents\` | `~/Documents/` | `~/Documents/` | ⚠️ |
| [3](#root3) | `AppData Roaming` | `%APPDATA%\` | `~/Library/Application Support/` | `~/.config/` | ⚠️ |
| [4](#root4) | `AppData Local` | `%LOCALAPPDATA%\` | `~/Library/Caches/` | `~/.local/share/` | ⚠️ |
| [5](#root5) | `Pictures` | `%USERPROFILE%\Pictures\` | `~/Pictures/` | `~/Pictures/` | ⚠️ |
| [6](#root6) | `Music` | `%USERPROFILE%\Music\` | `~/Music/` | `~/Music/` | ⚠️ |
| [7](#root7) | `Videos`/`MacAppSupport` | `%USERPROFILE%\Videos\` | `~/Library/Application Support/` ⚠️ | `~/Videos/` | ⚠️ |
| [8](#root8) | `Desktop` | `%USERPROFILE%\Desktop\` | `~/Desktop/` | `~/Desktop/` | ⚠️ |
| [9](#root9) | `Saved Games` | `%USERPROFILE%\Saved Games\` | `~/Documents/Saved Games/` | `~/Documents/Saved Games/` | ⚠️ |
| [10](#root10) | `Downloads` | `%USERPROFILE%\Downloads\` | `~/Downloads/` | `~/Downloads/` | ⚠️ |
| [11](#root11) | `Public` | `%PUBLIC%\` | `/Users/Shared/` | `/tmp/` | ⚠️ |
| [12](#root12) | `AppData LocalLow` | `%LOCALAPPDATA%Low\` | `~/Library/Caches/` | `~/.local/share/` | ⚠️ |

> ⚠️ Root=7 macOS 特殊：映射到 `Application Support` 而非 `Movies`

**状态**: ✅ 已验证 | ⚠️ 未测试 | ❌ 错误

---

## 详细路径映射

### Root=0 - Steam Cloud {#root0}

| 平台 | 预期路径 | 验证状态 |
|------|----------|----------|
| Windows | `{Steam}/userdata/{UID}/{AppID}/remote/` | ⚠️ 未测试 |
| macOS | `{Steam}/userdata/{UID}/{AppID}/remote/` | ⚠️ 未测试 |
| Linux | `{Steam}/userdata/{UID}/{AppID}/remote/` | ⚠️ 未测试 |

#### Windows 验证 {#root0-windows}
```日志
<!-- 游戏: ??? | App ID: ??? -->
待验证，请提交日志
```

#### macOS 验证 {#root0-macos}
```日志
<!-- 游戏: ??? | App ID: ??? -->
待验证，请提交日志
```

#### Linux 验证 {#root0-linux}
```日志
<!-- 游戏: ??? | App ID: ??? -->
待验证，请提交日志
```

---

### Root=1 - GameInstall {#root1}

| 平台 | 预期路径 | 验证状态 |
|------|----------|----------|
| Windows | `{Steam}/steamapps/common/{GameDir}/` | ⚠️ 未测试 |
| macOS | `{Steam}/steamapps/common/{GameDir}/` 或 `~/Library/Application Support/{Game}/` | ⚠️ 未测试 |
| Linux | `{Steam}/steamapps/common/{GameDir}/` | ⚠️ 未测试 |

#### Windows 验证 {#root1-windows}
```日志
<!-- 游戏: ??? | App ID: ??? -->
待验证，请提交日志
```

#### macOS 验证 {#root1-macos}
```日志
<!-- 游戏: ??? | App ID: ??? -->
待验证，请提交日志
```

#### Linux 验证 {#root1-linux}
```日志
<!-- 游戏: ??? | App ID: ??? -->
待验证，请提交日志
```

---

### Root=2 - Documents {#root2}

| 平台 | 预期路径 | 验证状态 |
|------|----------|----------|
| Windows | `%USERPROFILE%\Documents\` | ⚠️ 未测试 |
| macOS | `~/Documents/` | ⚠️ 未测试 |
| Linux | `~/Documents/` | ⚠️ 未测试 |

#### Windows 验证 {#root2-windows}
```日志
<!-- 游戏: ??? | App ID: ??? -->
待验证，请提交日志
```

#### macOS 验证 {#root2-macos}
```日志
<!-- 游戏: ??? | App ID: ??? -->
待验证，请提交日志
```

#### Linux 验证 {#root2-linux}
```日志
<!-- 游戏: ??? | App ID: ??? -->
待验证，请提交日志
```

---

### Root=3 - AppData Roaming {#root3}

| 平台 | 预期路径 | 验证状态 |
|------|----------|----------|
| Windows | `%APPDATA%\` | ⚠️ 未测试 |
| macOS | `~/Library/Application Support/` | ⚠️ 未测试 |
| Linux | `~/.config/` | ⚠️ 未测试 |

#### Windows 验证 {#root3-windows}
```日志
<!-- 游戏: ??? | App ID: ??? -->
待验证，请提交日志
```

#### macOS 验证 {#root3-macos}
```日志
<!-- 游戏: ??? | App ID: ??? -->
待验证，请提交日志
```

#### Linux 验证 {#root3-linux}
```日志
<!-- 游戏: ??? | App ID: ??? -->
待验证，请提交日志
```

---

### Root=4 - AppData Local {#root4}

| 平台 | 预期路径 | 验证状态 |
|------|----------|----------|
| Windows | `%LOCALAPPDATA%\` | ⚠️ 未测试 |
| macOS | `~/Library/Caches/` | ⚠️ 未测试 |
| Linux | `~/.local/share/` | ⚠️ 未测试 |

#### Windows 验证 {#root4-windows}
```日志
<!-- 游戏: ??? | App ID: ??? -->
待验证，请提交日志
```

#### macOS 验证 {#root4-macos}
```日志
<!-- 游戏: ??? | App ID: ??? -->
待验证，请提交日志
```

#### Linux 验证 {#root4-linux}
```日志
<!-- 游戏: ??? | App ID: ??? -->
待验证，请提交日志
```

---

### Root=5 - Pictures {#root5}

| 平台 | 预期路径 | 验证状态 |
|------|----------|----------|
| Windows | `%USERPROFILE%\Pictures\` | ⚠️ 未测试 |
| macOS | `~/Pictures/` | ⚠️ 未测试 |
| Linux | `~/Pictures/` | ⚠️ 未测试 |

#### Windows 验证 {#root5-windows}
```日志
<!-- 游戏: ??? | App ID: ??? -->
待验证，请提交日志
```

#### macOS 验证 {#root5-macos}
```日志
<!-- 游戏: ??? | App ID: ??? -->
待验证，请提交日志
```

#### Linux 验证 {#root5-linux}
```日志
<!-- 游戏: ??? | App ID: ??? -->
待验证，请提交日志
```

---

### Root=6 - Music {#root6}

| 平台 | 预期路径 | 验证状态 |
|------|----------|----------|
| Windows | `%USERPROFILE%\Music\` | ⚠️ 未测试 |
| macOS | `~/Music/` | ⚠️ 未测试 |
| Linux | `~/Music/` | ⚠️ 未测试 |

#### Windows 验证 {#root6-windows}
```日志
<!-- 游戏: ??? | App ID: ??? -->
待验证，请提交日志
```

#### macOS 验证 {#root6-macos}
```日志
<!-- 游戏: ??? | App ID: ??? -->
待验证，请提交日志
```

#### Linux 验证 {#root6-linux}
```日志
<!-- 游戏: ??? | App ID: ??? -->
待验证，请提交日志
```

---

### Root=7 - Videos / MacAppSupport {#root7}

| 平台 | 预期路径 | CDP 名称 | 验证状态 |
|------|----------|----------|----------|
| Windows | `%USERPROFILE%\Videos\` | `Videos` | ⚠️ 未测试 |
| macOS | `~/Library/Application Support/` ⚠️特殊 | `MacAppSupport` | ⚠️ 未测试 |
| Linux | `~/Videos/` | `Videos` | ⚠️ 未测试 |

> ⚠️ **macOS 特殊映射**: Root=7 在 macOS 上映射到 Application Support 而非 Movies

#### Windows 验证 {#root7-windows}
```日志
<!-- 游戏: ??? | App ID: ??? -->
待验证，请提交日志
```

#### macOS 验证 {#root7-macos}
```日志
<!-- 游戏: ??? | App ID: ??? -->
待验证，请提交日志
```

#### Linux 验证 {#root7-linux}
```日志
<!-- 游戏: ??? | App ID: ??? -->
待验证，请提交日志
```

---

### Root=8 - Desktop {#root8}

| 平台 | 预期路径 | 验证状态 |
|------|----------|----------|
| Windows | `%USERPROFILE%\Desktop\` | ⚠️ 未测试 |
| macOS | `~/Desktop/` | ⚠️ 未测试 |
| Linux | `~/Desktop/` | ⚠️ 未测试 |

#### Windows 验证 {#root8-windows}
```日志
<!-- 游戏: ??? | App ID: ??? -->
待验证，请提交日志
```

#### macOS 验证 {#root8-macos}
```日志
<!-- 游戏: ??? | App ID: ??? -->
待验证，请提交日志
```

#### Linux 验证 {#root8-linux}
```日志
<!-- 游戏: ??? | App ID: ??? -->
待验证，请提交日志
```

---

### Root=9 - Saved Games {#root9}

| 平台 | 预期路径 | 验证状态 |
|------|----------|----------|
| Windows | `%USERPROFILE%\Saved Games\` | ⚠️ 未测试 |
| macOS | `~/Documents/Saved Games/` | ⚠️ 未测试 |
| Linux | `~/Documents/Saved Games/` | ⚠️ 未测试 |

#### Windows 验证 {#root9-windows}
```日志
<!-- 游戏: ??? | App ID: ??? -->
待验证，请提交日志
```

#### macOS 验证 {#root9-macos}
```日志
<!-- 游戏: ??? | App ID: ??? -->
待验证，请提交日志
```

#### Linux 验证 {#root9-linux}
```日志
<!-- 游戏: ??? | App ID: ??? -->
待验证，请提交日志
```

---

### Root=10 - Downloads {#root10}

| 平台 | 预期路径 | 验证状态 |
|------|----------|----------|
| Windows | `%USERPROFILE%\Downloads\` | ⚠️ 未测试 |
| macOS | `~/Downloads/` | ⚠️ 未测试 |
| Linux | `~/Downloads/` | ⚠️ 未测试 |

#### Windows 验证 {#root10-windows}
```日志
<!-- 游戏: ??? | App ID: ??? -->
待验证，请提交日志
```

#### macOS 验证 {#root10-macos}
```日志
<!-- 游戏: ??? | App ID: ??? -->
待验证，请提交日志
```

#### Linux 验证 {#root10-linux}
```日志
<!-- 游戏: ??? | App ID: ??? -->
待验证，请提交日志
```

---

### Root=11 - Public {#root11}

| 平台 | 预期路径 | 验证状态 |
|------|----------|----------|
| Windows | `%PUBLIC%\` | ⚠️ 未测试 |
| macOS | `/Users/Shared/` | ⚠️ 未测试 |
| Linux | `/tmp/` | ⚠️ 未测试 |

#### Windows 验证 {#root11-windows}
```日志
<!-- 游戏: ??? | App ID: ??? -->
待验证，请提交日志
```

#### macOS 验证 {#root11-macos}
```日志
<!-- 游戏: ??? | App ID: ??? -->
待验证，请提交日志
```

#### Linux 验证 {#root11-linux}
```日志
<!-- 游戏: ??? | App ID: ??? -->
待验证，请提交日志
```

---

### Root=12 - AppData LocalLow {#root12}

| 平台 | 预期路径 | 验证状态 |
|------|----------|----------|
| Windows | `%LOCALAPPDATA%Low\` | ⚠️ 未测试 |
| macOS | `~/Library/Caches/` | ⚠️ 未测试 |
| Linux | `~/.local/share/` | ⚠️ 未测试 |

#### Windows 验证 {#root12-windows}
```日志
<!-- 游戏: ??? | App ID: ??? -->
待验证，请提交日志
```

#### macOS 验证 {#root12-macos}
```日志
<!-- 游戏: ??? | App ID: ??? -->
待验证，请提交日志
```

#### Linux 验证 {#root12-linux}
```日志
<!-- 游戏: ??? | App ID: ??? -->
待验证，请提交日志
```

---

## 如何贡献验证数据

### 1. 获取验证数据

**方法 A - ufs 配置**：
1. 运行 SteamCloudFileManager，连接到游戏
2. 点击 **"显示 appinfo.vdf"** 按钮
3. 复制 ufs 配置中的 `savefiles` 部分

**方法 B - 运行日志**：
1. 运行程序 `RUST_LOG=info cargo run`
2. 连接到游戏后查看日志输出
3. 复制包含 `VDF root=X | CDP folder=XXX` 的行

### 2. 提交格式

在对应的 Root 下添加验证案例（包含 ufs 和日志）：

```
游戏: The Witcher 3 | App ID: 292030 | 平台: Windows

ufs 配置:
"0"
{
    "root" "WinMyDocuments"
    "path" "/The Witcher 3/gamesaves/"
    "pattern" "*"
}

运行日志:
The Witcher 3/gamesaves/save.sav | VDF root=2 | CDP folder=WinMyDocuments | 3.02 MB

实际路径: C:\Users\XXX\Documents\The Witcher 3\gamesaves\
```

### 3. 更新验证状态

验证成功后，将对应 Root 的 `⚠️` 改为 `✅`

---

## 参考资料

- [Steamworks Steam Cloud](https://partner.steamgames.com/doc/features/cloud) - 官方文档
- [ISteamRemoteStorage API](https://partner.steamgames.com/doc/api/ISteamRemoteStorage) - API 参考

---

**最后更新**: 2025-12-22  
**维护者**: [@Fldicoahkiin](https://github.com/Fldicoahkiin)
