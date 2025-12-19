# Steam Cloud Root 路径映射表

> **⚠️ 重要说明**  
> 此映射表需要社区验证，欢迎通过 Issue 或 PR 提交日志来完善。

---

## 数据来源说明

| 来源 | 包含信息 | 示例 |
|------|----------|------|
| **VDF** | `root` 数字 (0-12) | `VDF root=7` |
| **CDP** | 文件夹名称字符串 | `CDP folder=MacAppSupport` |

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

### 1. 获取日志

运行 SteamCloudFileManager，选择游戏后复制日志：

```
========== 文件详情列表 (N 个文件) ==========
格式: [序号] 文件名 | VDF root=数字 | CDP folder=名称 | 大小 | 时间 | 本地存在 | 同步 | 本地路径
[  1] xxx.xxx | VDF root=X | CDP folder=XXX | X KB | XXXX-XX-XX XX:XX:XX | ✓ | 已同步 | /path/to/file
========== 文件列表结束 ==========
```

### 2. 提交格式

在对应的 Root 和平台下的 `\`\`\`日志` 代码块中填入：

```日志
<!-- 游戏: 游戏名称 | App ID: 123456 -->
[  1] save.dat | VDF root=7 | CDP folder=MacAppSupport | 2 KB | 2025-12-07 21:05:30 | ✓ | 已同步 | ~/Library/Application Support/xxx/save.dat
```

### 3. 更新验证状态

验证成功后，将对应平台的 `⚠️ 未测试` 改为 `✅ 已验证`

---

## 参考资料

- [Steamworks Steam Cloud](https://partner.steamgames.com/doc/features/cloud) - 官方文档
- [ISteamRemoteStorage API](https://partner.steamgames.com/doc/api/ISteamRemoteStorage) - API 参考

---

**最后更新**: 2025-12-19  
**维护者**: [@Fldicoahkiin](https://github.com/Fldicoahkiin)
