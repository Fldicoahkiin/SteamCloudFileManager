# Steam Cloud Root 路径映射表

> **重要说明**  
> Steam 官方文档只公开了根路径的字符串名称。数字 Root ID (0-12) 在任何官方文档中都未公开，仅在 `remotecache.vdf` 文件中使用。

---

## Steam 自动云的工作机制

### 开发者配置（Steamworks 后台）

开发者配置自动云时使用**字符串名称**：

```
根路径：WinAppDataLocal
子目录：MyCompany/MyGame/Saves/
模式（pattern）：*.sav
递归：是
```

Steam 会自动扫描目录并同步匹配的文件。

**appinfo.vdf 中存储的配置**：
```vdf
"ufs"
{
    "savefiles"
    {
        "0"
        {
            "root"       "WinAppDataLocal"    ← 字符串名称
            "path"       "MyCompany/MyGame/Saves/"
            "pattern"    "*.sav"
            "platforms"  "windows"
        }
    }
}
```

### 本地存储（remotecache.vdf）

Steam 客户端在本地使用**数字 ID**：

```vdf
"{AppID}"
{
    "MyCompany/MyGame/Saves/quicksave.sav"
    {
        "root"   "4"    ← 数字 Root ID
        "size"   "1024"
        "sha"    "..."
    }
}
```

### 为什么需要这个映射表

- 开发者看到的是字符串名称（如 `WinAppDataLocal`）
- remotecache.vdf 中存储的是数字（如 `4`）
- 官方未公开它们的对应关系，仅在 `remotecache.vdf` 文件中使用
- 本文档通过实际游戏验证，建立这个映射

---

## 根路径映射表

来源：https://partner.steamgames.com/doc/features/cloud

**为什么需要这个映射表？**  
本工具的"跳转到本地文件"功能需要将 remotecache.vdf 中的数字 Root ID 转换为实际的文件系统路径。以下映射关系基于代码实现和实际测试，**但官方未公开，需要验证**。

| Root ID | Steamworks 根名称 | Windows 路径 | macOS 路径 | Linux 路径 |
|:-------:|------------------|--------------|-----------|-----------|
| **0** | `SteamCloudDocuments` | `{Steam}\userdata\{UID}\{AppID}\remote\` | `{Steam}/userdata/{UID}/{AppID}/remote/` | `{Steam}/userdata/{UID}/{AppID}/remote/` |
| **1** | `App Install Directory` | `{Steam}\steamapps\common\{GameFolder}\` | `{Steam}/steamapps/common/{GameFolder}/` | `{Steam}/steamapps/common/{GameFolder}/` |
| **2?** | `WinMyDocuments` / `MacDocuments` / `LinuxHome` | `%USERPROFILE%\Documents\` | `~/Documents/` | `~/Documents/` |
| **3?** | `WinAppDataRoaming` / `MacAppSupport` / `LinuxXdgConfigHome` | `%APPDATA%\` | `~/Library/Application Support/` | `~/.config/` |
| **4?** | `WinAppDataLocal` / `MacHome` / `LinuxXdgDataHome` | `%LOCALAPPDATA%\` | `~/Library/Caches/` | `~/.local/share/` |
| **5?** | `WinPictures` | `%USERPROFILE%\Pictures\` | `~/Pictures/` | `~/Pictures/` |
| **6?** | `WinMusic` | `%USERPROFILE%\Music\` | `~/Music/` | `~/Music/` |
| **7?** | `MacAppSupport` / `WinVideos` | `%USERPROFILE%\Videos\` | `~/Library/Application Support/` | `~/Videos/` |
| **8?** | （未知）| `%USERPROFILE%\Desktop\` | `~/Desktop/` | `~/Desktop/` |
| **9?** | `WinSavedGames` | `%USERPROFILE%\Saved Games\` | `~/Documents/Saved Games/` ※  | `~/Documents/Saved Games/` ※ |
| **10?** | （未知）| `%USERPROFILE%\Downloads\` | `~/Downloads/` | `~/Downloads/` |
| **11?** | （未知）| `%PUBLIC%\` | `/Users/Shared/` | `/tmp/` ※ |
| **12?** | `WinAppDataLocalLow` | `%USERPROFILE%\AppData\LocalLow\` | `~/Library/Caches/` ※ | `~/.local/share/` ※ |

**符号说明**：
- **数字?**：基于代码实现，但**未经实际游戏验证**
- **※**：推测的跨平台映射，需要验证

### Windows 环境变量

| 变量 | 典型值 |
|------|--------|
| `%USERPROFILE%` | `C:\Users\{Username}` |
| `%APPDATA%` | `C:\Users\{Username}\AppData\Roaming` |
| `%LOCALAPPDATA%` | `C:\Users\{Username}\AppData\Local` |

---

## 已验证的游戏案例

### SteamCloudDocuments

| 游戏 | AppID | 平台 | Root ID | 实际路径 |
|------|-------|------|:-------:|---------|
| | | | | |

### App Install Directory

| 游戏 | AppID | 平台 | Root ID | 实际路径 |
|------|-------|------|:-------:|---------|
| | | | | |

### WinMyDocuments

| 游戏 | AppID | 平台 | Root ID | 实际路径 |
|------|-------|------|:-------:|---------|
| | | | | |

### MacDocuments

| 游戏 | AppID | 平台 | Root ID | 实际路径 |
|------|-------|------|:-------:|---------|
| | | | | |

### LinuxHome

| 游戏 | AppID | 平台 | Root ID | 实际路径 |
|------|-------|------|:-------:|---------|
| | | | | |

### WinAppDataLocal

| 游戏 | AppID | 平台 | Root ID | 实际路径 |
|------|-------|------|:-------:|---------|
| | | | | |

### MacHome

| 游戏 | AppID | 平台 | Root ID | 实际路径 |
|------|-------|------|:-------:|---------|
| | | | | |

### LinuxXdgDataHome

| 游戏 | AppID | 平台 | Root ID | 实际路径 |
|------|-------|------|:-------:|---------|
| | | | | |

### WinAppDataLocalLow

| 游戏 | AppID | 平台 | Root ID | 实际路径 |
|------|-------|------|:-------:|---------|
| | | | | |

### WinAppDataRoaming

| 游戏 | AppID | 平台 | Root ID | 实际路径 |
|------|-------|------|:-------:|---------|
| | | | | |

### MacAppSupport

| 游戏 | AppID | 平台 | Root ID | 实际路径 |
|------|-------|------|:-------:|---------|
| | | | | |

### LinuxXdgConfigHome

| 游戏 | AppID | 平台 | Root ID | 实际路径 |
|------|-------|------|:-------:|---------|
| | | | | |

### WinSavedGames

| 游戏 | AppID | 平台 | Root ID | 实际路径 |
|------|-------|------|:-------:|---------|
| | | | | |

---

## 如何验证

### 使用本工具

1. 连接游戏
2. 查看日志：`[文件名] | VDF root=X | appinfo.vdf root=YYY`
3. 确认：数字 X 对应字符串 YYY

### 手动验证

1. **查看 remotecache.vdf**：
   ```bash
   cat ~/Library/Application\ Support/Steam/userdata/*/12345/remotecache.vdf
   ```
   找到 `"root" "X"`

2. **查看 appinfo.vdf ufs 配置**：
   使用本工具连接游戏，自动解析显示
   找到 `"root" "YYY"`

3. **确认映射**：X ↔ YYY

---

## 技术参考

### remotecache.vdf 格式

```vdf
"{AppID}"
{
    "{文件相对路径}"
    {
        "root"          "{0-12}"
        "size"          "{字节}"
        "localtime"     "{时间戳}"
        "sha"           "{SHA-1}"
        "syncstate"     "{0-2}"
    }
}
```

### appinfo.vdf ufs 配置格式

```vdf
"ufs"
{
    "quota"         "104857600"
    "maxnumfiles"   "500"
    "savefiles"
    {
        "0"
        {
            "root"       "字符串名称"
            "path"       "路径"
            "pattern"    "*.sav"
            "platforms"  "操作系统"
        }
    }
}
```

---

**最后更新**：2026-01-12  
**维护者**：[@Fldicoahkiin](https://github.com/Fldicoahkiin)
