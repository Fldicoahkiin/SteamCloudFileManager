# Steam Cloud Root 路径映射表

> **重要说明**\
> Steam 官方文档只公开了根路径的字符串名称。数字 Root ID (0-12) 在任何官方文档中都未公开，仅在 `remotecache.vdf` 文件中使用。

---

## 为什么需要这个映射表

### 标识符差异对比

**1. Steamworks 配置 (appinfo.vdf)**
使用**字符串名称**定义根路径：

```vdf
"savefiles"
{
    "0"
    {
        "root"      "WinAppDataLocal"  // <--- 字符串标识
        "path"      "MyGame/Saves"
    }
}
```

**2. 本地同步记录 (remotecache.vdf)**
使用**数字 ID** 记录文件位置：

```vdf
"MyGame/Saves/save.dat"
{
    "root"      "3"                    // <--- 数字标识
    "size"      "1024"
}
```

本工具的"跳转到本地文件"功能需要将 remotecache.vdf 中的数字 Root ID 转换为实际的文件系统路径。官方未公开它们的对应关系，本文档通过实际游戏验证建立这个映射。

---

## 根路径映射表

**官方文档来源**：https://partner.steamgames.com/doc/features/cloud

### 官方公开的 Root 名称

| Root 名称               | 平台    | 路径                                        |
| ----------------------- | ------- | ------------------------------------------- |
| `App Install Directory` | All     | `[Steam]\steamapps\common\[Game]\`          |
| `SteamCloudDocuments`   | All     | [见下方说明](#steamclouddocuments-root-11-路径说明) |
| `WinMyDocuments`        | Windows | `%USERPROFILE%\My Documents\`               |
| `WinAppDataLocal`       | Windows | `%USERPROFILE%\AppData\Local\`              |
| `WinAppDataLocalLow`    | Windows | `%USERPROFILE%\AppData\LocalLow\`           |
| `WinAppDataRoaming`     | Windows | `%USERPROFILE%\AppData\Roaming\`            |
| `WinSavedGames`         | Windows | `%USERPROFILE%\Saved Games\`                |
| `WindowsHome`           | Windows | `%USERPROFILE%\`                            |
| `MacHome`               | macOS   | `~/`                                        |
| `MacAppSupport`         | macOS   | `~/Library/Application Support/`            |
| `MacDocuments`          | macOS   | `~/Documents/`                              |
| `LinuxHome`             | Linux   | `~/`                                        |
| `LinuxXdgDataHome`      | Linux   | `$XDG_DATA_HOME/`                           |
| `LinuxXdgConfigHome`    | Linux   | `$XDG_CONFIG_HOME/`                         |
| `AndroidExternalData`   | Android | `Android/data/<package>/files/`             |

### 数字 Root ID 映射

> **数据来源**：数字 Root ID 与字符串名称的完整对应关系来自 Steam SDK 内部枚举 [`ERemoteStorageFileRoot`](https://github.com/emily33901/SteamStructs/blob/master/ERemoteStorageFileRoot.h)（[OpenSteamworks 镜像](https://github.com/fire64/opensteamworks/blob/master/ERemoteStorageFileRoot.h)），路径映射则通过实际游戏测试和 `cloud_log.txt` 分析确认。

| Root ID | SDK 枚举名                                        | Root 名称                 | 路径示例                                                            |
| :-----: | ------------------------------------------------- | ------------------------- | ------------------------------------------------------------------- |
|  **0**  | `k_ERemoteStorageFileRootDefault`                 | `Default`                 | `{Steam}/userdata/{UID}/{AppID}/remote/`                            |
|  **1**  | `k_ERemoteStorageFileRootGameInstall`             | `GameInstall`             | `{SteamInstall}/steamapps/common/{Game}/`                           |
|  **2**  | `k_ERemoteStorageFileRootWinMyDocuments`          | `WinMyDocuments`          | Win: `%USERPROFILE%\Documents\`                                     |
|  **3**  | `k_ERemoteStorageFileRootWinAppDataLocal`         | `WinAppDataLocal`         | Win: `%LOCALAPPDATA%\`                                              |
|  **4**  | `k_ERemoteStorageFileRootWinAppDataRoaming`       | `WinAppDataRoaming`       | Win: `%APPDATA%\`                                                   |
|  **5**  | `k_ERemoteStorageFileRootSteamUserBaseStorage`    | `SteamUserBaseStorage`    | _(用途待验证)_                                                      |
|  **6**  | `k_ERemoteStorageFileRootMacHome`                 | `MacHome`                 | Mac: `~/`                                                           |
|  **7**  | `k_ERemoteStorageFileRootMacAppSupport`           | `MacAppSupport`           | Mac: `~/Library/Application Support/`                               |
|  **8**  | `k_ERemoteStorageFileRootMacDocuments`            | `MacDocuments`            | Mac: `~/Documents/`                                                 |
|  **9**  | `k_ERemoteStorageFileRootWinSavedGames`           | `WinSavedGames`           | Win: `%USERPROFILE%\Saved Games\`                                   |
| **10**  | `k_ERemoteStorageFileRootWinProgramData`          | `WinProgramData`          | Win: `%PROGRAMDATA%\`                                               |
| **11**  | `k_ERemoteStorageFileRootSteamCloudDocuments`     | `SteamCloudDocuments`     | [见下方说明](#steamclouddocuments-root-11-路径说明) |
| **12**  | `k_ERemoteStorageFileRootWinAppDataLocalLow`      | `WinAppDataLocalLow`      | Win: `%LOCALAPPDATA%Low\`                                           |
| **13**  | `k_ERemoteStorageFileRootMacCaches`               | `MacCaches`               | Mac: `~/Library/Caches/`                                            |
| **14**  | `k_ERemoteStorageFileRootLinuxHome`               | `LinuxHome`               | Linux: `~/`                                                         |
| **15**  | `k_ERemoteStorageFileRootLinuxXdgDataHome`        | `LinuxXdgDataHome`        | Linux: `$XDG_DATA_HOME/`                                            |
| **16**  | `k_ERemoteStorageFileRootLinuxXdgConfigHome`      | `LinuxXdgConfigHome`      | Linux: `$XDG_CONFIG_HOME/`                                          |
| **17**  | `k_ERemoteStorageFileRootAndroidSteamPackageRoot` | `AndroidSteamPackageRoot` | Android: _(待验证)_                                                 |
| **18**  | _(枚举中未定义，可能为后续新增)_                  | `WindowsHome`             | Win: `%USERPROFILE%\`                                               |

### 跨平台映射说明

**Root ID 在不同系统上的表现**：

- **Root ID 代表逻辑位置**：例如 Root 2 始终代表 "Windows 我的文档"。
- **平台特异性**：某些 Root ID 仅在特定平台有效（如 `Win...` 在 Windows，`Mac...` 在 macOS）。
- **跨平台同步**：如果游戏在 Windows 上使用了 Root 2 (`WinMyDocuments`) 存储存档，而在 macOS 上希望同步到 `~/Documents` (Root 8)，开发者需要在 Steamworks 的 **Root Overrides** 中配置映射规则（例如：将 Windows 下的 Root 2 映射为 macOS 下的 Root 8）。若未配置 Override，Steam 在不支持该 Root 的平台上可能会忽略这些文件或尝试使用默认回退路径。

### Windows 环境变量

| 变量                         | 典型值                                |
| ---------------------------- | ------------------------------------- |
| `%USERPROFILE%`              | `C:\Users\{Username}`                 |
| `%USERPROFILE%\My Documents` | `C:\Users\{Username}\Documents`       |
| `%APPDATA%`                  | `C:\Users\{Username}\AppData\Roaming` |
| `%LOCALAPPDATA%`             | `C:\Users\{Username}\AppData\Local`   |

### Linux 环境变量

| 变量               | 默认值           |
| ------------------ | ---------------- |
| `$XDG_CONFIG_HOME` | `~/.config`      |
| `$XDG_DATA_HOME`   | `~/.local/share` |

### SteamCloudDocuments (Root 11) 路径说明

`SteamCloudDocuments` 是 Auto-Cloud 专用的 Root 类型，用于将存档存放在用户文档目录。实际路径取决于平台：

| 平台    | `SteamCloudDocuments` (Root 11) 实际路径                      |
| ------- | ------------------------------------------------------------- |
| macOS   | `~/Documents/Steam Cloud/[Steam用户名]/[游戏名]/`             |
| Linux   | `~/.SteamCloud/[Steam用户名]/[游戏名]/`                       |
| Windows | `%USERPROFILE%\Documents\Steam Cloud\[Steam用户名]\[游戏名]\` |

> **已验证**：macOS 和 Windows 的 `cloud_log.txt` 均证实 Root 11 使用上述平台相关路径。

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

完整的 `ufs` 节结构：

```vdf
"ufs"
{
    "quota"                 "配额字节数"
    "maxnumfiles"           "最大文件数"
    "hidecloudui"           "0或1"      // 可选，Steamworks 后台可见
    "sync_while_suspended"  "0或1"      // 可选，Steamworks 后台可见
    "ignoreexternalfiles"   "0或1"      // 可选，Steamworks 后台可见

    "savefiles"
    {
        "0"
        {
            "root"       "根目录名称"    // 如 WinAppDataLocal, MacAppSupport
            "path"       "子目录路径"    // 如 MyGame/Saves/
            "pattern"    "文件匹配模式"  // 如 *.sav, *
            "recursive"  "1"             // 可选，"1" 表示递归扫描子目录
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
    // 5. 替代路径             - 启用后，addpath 完全替换为 pathtransforms
    {
        "0"                              // 索引：独立于 savefiles，从 0 开始（可能不连续）
        {
            "root"          "原始根目录"    // 如 gameinstall, WinSavedGames
            "os"            "目标平台"      // Windows, MacOS, Linux (大小写敏感)
            "oscompare"     "="             // 比较符，目前观察到的值均为 "="
            "useinstead"    "新根目录"      // 新的根目录名称，如 MacAppSupport, LinuxXdgDataHome
            "addpath"       "附加路径"      // 可选，附加到路径末尾 (与 pathtransforms 互斥)
            "pathtransforms"                // 可选，路径转换 (与 addpath 互斥)
            {
                "0"
                {
                    "find"      ""          // 匹配模式，空字符串表示匹配所有
                    "replace"   "SaveData"  // 替换内容
                }
            }
        }
    }
}
```

### Steamworks 后台界面与 VDF 字段对应

![Steamworks Auto-Cloud 配置界面](docs/steamworks_autocloud_ui.png)

| Steamworks 后台  | VDF 字段         | 说明                                                   |
| ---------------- | ---------------- | ------------------------------------------------------ |
| Original Root    | `root`           | 要覆盖的原始根目录                                     |
| OS               | `os`             | 目标操作系统 (`Windows`, `MacOS`, `Linux`，大小写敏感) |
| (隐藏)           | `oscompare`      | 比较符，UI 不可编辑，目前观察到的值均为 `"="`          |
| New Root         | `useinstead`     | 新的根目录名称 (字符串类型)                            |
| Add/Replace Path | `addpath`        | 附加路径，未勾选 "Replace Path" 时使用                 |
| Replace Path [✓] | `pathtransforms` | 勾选后使用 pathtransforms 结构替代 addpath             |

### pathtransforms 结构

当 Steamworks UI 中勾选 "Replace Path" 时，后端使用 `pathtransforms` 结构而非 `addpath` 字段，二者互斥：

- **未勾选**：只有 `addpath` 字段，无 `pathtransforms`
- **已勾选**：只有 `pathtransforms` 结构，无 `addpath`

`pathtransforms` 是一个包含索引子节 ("0", "1", ...) 的结构，每个子节包含：

- `find`: 要查找替换的原始路径（通常是 savefile 的 path 值）
- `replace`: 替换后的新路径

**重要说明**：根据对 Steam `appinfo.vdf` 的实际分析，`pathtransforms` 的 `find` 字段应包含原始 savefile 的 `path` 值（要被替换的路径），`replace` 字段是新的路径。当 `find` 为空字符串时，`replace` 值会被**插入到路径开头**而非完全替换。

### oscompare 字段分析

**已验证**：

- 目前观察到的所有 rootoverride 条目中，`oscompare` 值均为 `"="`
- Steamworks UI 中该字段不可编辑

**推测**：

- 该字段以比较操作符的形式存在，暗示可能支持其他值（如 `"!="` 表示"不等于"）
- 可能是 Valve 预留的扩展字段，用于未来支持更复杂的平台匹配逻辑
- 可能存在通过直接编辑配置使用其他操作符的内部用法
- 使用场景推测：`"!="` 可用于表示"除了某平台以外的所有平台"

### UFS 顶层配置字段分析

以下字段在 Steamworks 后台 UI 中可见，但在已分析的 appinfo.vdf 中均未观察到：

| 字段                   | 已验证                                  | 推测用途                                                                      |
| ---------------------- | --------------------------------------- | ----------------------------------------------------------------------------- |
| `hidecloudui`          | Steamworks 后台可见，appinfo 中未观察到 | 控制 Steam 客户端是否显示云存储相关 UI（同步状态、冲突解决对话框等）          |
| `sync_while_suspended` | Steamworks 后台可见，appinfo 中未观察到 | 控制游戏挂起时是否继续同步云存档（可能用于 Steam Deck 休眠场景）              |
| `ignoreexternalfiles`  | Steamworks 后台可见，appinfo 中未观察到 | 控制是否忽略外部文件（非 Steam 创建的文件），可能用于防止第三方工具干扰云存档 |

**推测**：

- 这些字段可能是服务端配置，不需要下发到客户端的 appinfo.vdf
- 或者默认值均为 0/false，VDF 序列化时省略默认值
- 需要进一步验证：在 Steamworks 后台启用这些选项后，观察 appinfo.vdf 是否会出现对应字段

# 查看指定游戏的 UFS 配置

cargo run -- --ufs <app_id>
