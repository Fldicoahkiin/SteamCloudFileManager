# Steam Cloud Root 路径映射表

> **重要说明**  
> Steam 官方文档只公开了根路径的字符串名称。数字 Root ID (0-12) 在任何官方文档中都未公开，仅在 `remotecache.vdf` 文件中使用。

---

## 为什么需要这个映射表

| 配置位置 | 使用的标识 | 示例 |
|---------|-----------|------|
| Steamworks 后台 / appinfo.vdf | 字符串名称 | `WinAppDataLocal` |
| 本地 remotecache.vdf | 数字 ID | `4` |

本工具的"跳转到本地文件"功能需要将 remotecache.vdf 中的数字 Root ID 转换为实际的文件系统路径。官方未公开它们的对应关系，本文档通过实际游戏验证建立这个映射。

---

## 根路径映射表

**官方文档来源**：https://partner.steamgames.com/doc/features/cloud

### 官方公开的 Root 名称

| Root 名称 | 平台 | 路径 |
|----------|------|------|
| `App Install Directory` | All | `[Steam]\steamapps\common\[Game]\` |
| `SteamCloudDocuments` | All | `~/.SteamCloud/[username]/[Game]/` |
| `WinMyDocuments` | Windows | `%USERPROFILE%\My Documents\` |
| `WinAppDataLocal` | Windows | `%USERPROFILE%\AppData\Local\` |
| `WinAppDataLocalLow` | Windows | `%USERPROFILE%\AppData\LocalLow\` |
| `WinAppDataRoaming` | Windows | `%USERPROFILE%\AppData\Roaming\` |
| `WinSavedGames` | Windows | `%USERPROFILE%\Saved Games\` |
| `MacHome` | macOS | `~/` |
| `MacAppSupport` | macOS | `~/Library/Application Support/` |
| `MacDocuments` | macOS | `~/Documents/` |
| `LinuxHome` | Linux | `~/` |
| `LinuxXdgDataHome` | Linux | `$XDG_DATA_HOME/` |

### 数字 Root ID 映射（待验证）

> **注意**：数字 Root ID 与字符串名称的对应关系**官方未公开**，以下基于实际验证和代码推测。

| Root ID | 已验证的名称 | 来源 |
|:-------:|-------------|------|
| **0** | `SteamCloudDocuments` | [默认路径](#root-0---steamclouddocuments) |
| **1** | `GameInstall` | [✅ macOS](#root-1---gameinstall-app-install-directory) |
| **2** | `WinMyDocuments` | [待验证](#root-2---winmydocuments) |
| **3** | `WinAppDataRoaming` | [待验证](#root-3---winappdataroaming) |
| **4** | `WinAppDataLocal` (Win) / `MacHome` (Mac) | [✅ macOS](#root-4---winappdatalocal--machome) |
| **5** | （未知） | [待验证](#root-5---winpictures) |
| **6** | （未知） | [待验证](#root-6---winmusic) |
| **7** | `MacAppSupport` | [✅ macOS](#root-7---macappsupport) |
| **8** | `LinuxXdgDataHome` | [待验证](#root-8---linuxxdgdatahome) |
| **9** | `WinSavedGames` | [待验证](#root-9---winsavedgames) |
| **10** | （未知） | [待验证](#root-10---windownloads) |
| **11** | （未知） | [待验证](#root-11---winpublic) |
| **12** | `WinAppDataLocalLow` | [待验证](#root-12---winappdatalocallow) |

### Windows 环境变量

| 变量 | 典型值 |
|------|--------|
| `%USERPROFILE%` | `C:\Users\{Username}` |
| `%USERPROFILE%\My Documents` | `C:\Users\{Username}\Documents`|
| `%APPDATA%` | `C:\Users\{Username}\AppData\Roaming` |
| `%LOCALAPPDATA%` | `C:\Users\{Username}\AppData\Local` |

### Linux 环境变量

| 变量 | 默认值 |
|------|--------|
| `$XDG_CONFIG_HOME` | `~/.config` |
| `$XDG_DATA_HOME` | `~/.local/share` |

---

## 已验证的游戏案例

### Root 0 - SteamCloudDocuments

默认路径：`{Steam}/userdata/{UID}/{AppID}/remote/`

---

### Root 1 - GameInstall (App Install Directory)

| 游戏 | AppID | 平台 | 完整路径 |
|------|-------|------|----------|
| Celeste | 504230 | macOS | `~/Library/Application Support/Celeste/` |

通过 rootoverrides 将 gameinstall 重定向到 MacAppSupport

<details>
<summary><b>Celeste (504230) 完整配置</b></summary>

**appinfo.vdf ufs 配置：**
```vdf
"ufs"
{
    "quota" "1000000000"
    "maxnumfiles" "1000"
    "savefiles"
    {
        "0"
        {
            "root" "gameinstall"
            "path" "Saves"
            "pattern" "*.celeste"
        }
    }
    "rootoverrides"
    {
        "1"
        {
            "root" "gameinstall"
            "os" "Linux"
            "oscompare" "="
            "useinstead" "LinuxXdgDataHome"
            "addpath" "Celeste"
        }
        "2"
        {
            "root" "gameinstall"
            "os" "MacOS"
            "oscompare" "="
            "useinstead" "MacAppSupport"
            "addpath" "Celeste"
        }
    }
}
```

**remotecache.vdf：**
```vdf
"504230"
{
    "ChangeNumber"      "11"
    "OSType"            "-102"
    "Saves/0.celeste"
    {
        "root"              "1"
        "size"              "27648"
        "localtime"         "1731570960"
        "sha"               "..."
        "syncstate"         "1"
        "platformstosync2"  "-1"
    }
    "Saves/settings.celeste"
    {
        "root"              "1"
        "size"              "5080"
        "syncstate"         "1"
    }
}
```

**路径解析：**
- 原始：`root=gameinstall`, `path=Saves`
- macOS rootoverrides：`useinstead=MacAppSupport`, `addpath=Celeste`
- 最终：`~/Library/Application Support/Celeste/Saves/`

</details>

---

### Root 2 - WinMyDocuments

*待验证*

---

### Root 3 - WinAppDataRoaming

*待验证*

---

### Root 4 - WinAppDataLocal / MacHome

| 游戏 | AppID | 平台 | 完整路径 |
|------|-------|------|----------|
| Finding Paradise | 337340 | macOS | `~/Finding Paradise - Freebird Games/` |

> **关键发现**：Root 4 在 macOS 上映射到 `~/`（用户主目录），而不是 Windows 上的 `AppData\Local`

---

### Root 5 - WinPictures

*待验证*

---

### Root 6 - WinMusic

*待验证*

---

### Root 7 - MacAppSupport

| 游戏 | AppID | 完整路径 |
|------|-------|----------|
| Finding Paradise | 337340 | `~/Library/Application Support/freebirdgames/findingparadise/` |

<details>
<summary><b>Finding Paradise (337340) 完整配置</b></summary>

**appinfo.vdf ufs 配置：**
```vdf
"savefiles" {
    "0" { "root" "LinuxXdgDataHome" "path" "freebirdgames/findingparadise" "platforms" { "1" "Linux" } }
    "1" { "root" "MacAppSupport" "path" "freebirdgames/findingparadise" "platforms" { "1" "MacOS" } }
    "2" { "root" "WinAppDataRoaming" "path" "Finding Paradise - Freebird Games" "platforms" { "1" "Windows" } }
    "3" { "root" "WinAppDataLocalLow" "path" "Serenity Forge/Finding Paradise" "platforms" { "1" "Windows" } }
}
```

**remotecache.vdf 示例 (macOS)：**
```vdf
"freebirdgames/findingparadise/Save4.rxdata" { "root" "7" "size" "323614" "syncstate" "1" }
```

</details>

---

### Root 8 - LinuxXdgDataHome

*待验证*

---

### Root 9 - WinSavedGames

*待验证*

---

### Root 10 - WinDownloads

*待验证*

---

### Root 11 - WinPublic

*待验证*

---

### Root 12 - WinAppDataLocalLow

*待验证*

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

完整的 `ufs` 节结构：

```vdf
"ufs"
{
    "quota"         "配额字节数"
    "maxnumfiles"   "最大文件数"
    "hidecloudui"   "0或1"          // 可选

    "savefiles"
    {
        "0"
        {
            "root"       "根目录名称"    // 如 WinAppDataLocal, MacAppSupport
            "path"       "子目录路径"    // 如 MyGame/Saves/
            "pattern"    "文件匹配模式"  // 如 *.sav, *
            "platforms"                  // 可选
            {
                "1"      "平台名称"      // Windows, MacOS, Linux
            }
        }
    }


    "rootoverrides"                     // 可选，根先决替代
    //
    // 官方文档说明：
    // 如果应用程序是跨平台的，且每个操作系统需要不同的目录，可使用根先决替代。
    // 若使用此功能，原始根路径的"操作系统"下拉菜单中应指明 [所有操作系统]。
    //
    // 由 5 个部分组成：
    // 1. 原始根 (root)        - 对应上面设置的根之一
    // 2. 操作系统 (os)        - 使用此先决替代的操作系统
    // 3. 新根 (useinstead)    - 原始根映射到指定操作系统的新位置
    // 4. 添加/替代路径 (addpath) - 插入到新根和原始子目录之间的子目录路径
    // 5. 替代路径             - 启用后，addpath 完全替代原始子目录
    {
        "0"                              // 索引：独立于 savefiles，从 0 开始（可能不连续）
        {
            "root"          "原始根目录"    // 如 gameinstall, WinSavedGames
            "os"            "目标平台"      // MacOS, Linux, Windows
            "oscompare"     "="             // 比较符，已知值："="（可能还有其他如"!="，待验证）
            "useinstead"    "新根目录"      // 新的根目录名称，如 MacAppSupport, LinuxXdgDataHome
            "addpath"       "附加路径"      // 可选，附加到路径末尾，如 "Celeste"
            "pathtransforms" { }            // 可选，路径转换（用途待验证）
        }
    }
}
```

### Steamworks 后台界面与 VDF 字段对应

![Steamworks Auto-Cloud 配置界面](docs/steamworks_autocloud_ui.png)

| Steamworks 后台 | VDF 字段 | 说明 |
|----------------|----------|------|
| Original Root | `root` | 要覆盖的原始根目录 |
| OS | `os` | 目标操作系统 (Mac OS X, Linux + SteamOS 等) |
| New Root | `useinstead` | 新的根目录位置 |
| Add/Replace Path | `addpath` | 附加/替代路径 |
| Replace Path [✓] | (待确认) | 勾选后 addpath 完全替代原始子目录 |

> **待验证事项**：
> - `oscompare` 其他可能值待验证
> - `pathtransforms` 用途不明
> - "Replace Path" 勾选框对应的 VDF 字段待确认
> - 索引可能不连续

---

**最后更新**：2026-01-21  
**维护者**：[@Fldicoahkiin](https://github.com/Fldicoahkiin)

