# Steam Cloud Root 路径映射表

> **⚠️ 重要说明**  
> 此映射表基于实际游戏测试和社区验证，**仍在持续更新中**。  
> 不同游戏可能使用不同的 Root 值，且跨平台行为可能不一致。  
> 欢迎通过 Issue 或 PR 提供更多游戏案例来完善此表。

## Root 类型映射表

| Root | Windows | macOS | Linux | CDP 文件夹名 | 使用场景 | 验证状态 |
|------|---------|-------|-------|------------|---------|----------|
| **0** | `{SteamPath}/userdata/{UserID}/{AppID}/remote/` | `{SteamPath}/userdata/{UserID}/{AppID}/remote/` | `{SteamPath}/userdata/{UserID}/{AppID}/remote/` | `Steam Cloud` / `SteamCloud` / (empty) | Steam 默认云存档 | ✅ 已验证 (macOS) |
| **1** | `{SteamPath}/steamapps/common/{GameDir}/` | `{SteamPath}/steamapps/common/{GameDir}/` | `{SteamPath}/steamapps/common/{GameDir}/` | `GameInstall` / `Game Install` | 游戏安装目录 | ✅ 已验证 (macOS) |
| **2** | `%USERPROFILE%\Documents\` | `~/Documents/` | `~/Documents/` | `Documents` / `My Documents` | 文档文件夹 | ⚠️ 需验证 |
| **3** | `%APPDATA%\` | `~/Library/Application Support/` | `~/.config/` | `AppData Roaming` / `Roaming` | AppData Roaming | ⚠️ 需验证 |
| **4** | `%LOCALAPPDATA%\` | `~/Library/Caches/` | `~/.local/share/` | `AppData Local` / `Local` | Local AppData | ⚠️ 需验证 |
| **5** | `%USERPROFILE%\Pictures\` | `~/Pictures/` | `~/Pictures/` | `Pictures` | 图片文件夹 | ⚠️ 需验证 |
| **6** | `%USERPROFILE%\Music\` | `~/Music/` | `~/Music/` | `Music` | 音乐文件夹 | ⚠️ 需验证 |
| **7** | `%USERPROFILE%\Videos\` | `~/Library/Application Support/` | `~/Videos/` | `Videos` / `Movies` / **`MacAppSupport`** (macOS) | 视频文件夹 / macOS 应用支持 | ✅ 已验证 (macOS) |
| **8** | `%USERPROFILE%\Desktop\` | `~/Desktop/` | `~/Desktop/` | `Desktop` | 桌面文件夹 | ⚠️ 需验证 |
| **9** | `%USERPROFILE%\Saved Games\` | `~/Documents/Saved Games/` | `~/Documents/Saved Games/` | `Saved Games` | Windows Saved Games | ⚠️ 需验证 |
| **10** | `%USERPROFILE%\Downloads\` | `~/Downloads/` | `~/Downloads/` | `Downloads` | 下载文件夹 | ⚠️ 需验证 |
| **11** | `%PUBLIC%\` | `/Users/Shared/` | `/tmp/` | `Public` / `Shared` | 公共共享目录 | ⚠️ 需验证 |
| **12** | `%USERPROFILE%\AppData\LocalLow\` | `~/Library/Caches/` | `~/.local/share/` | `AppData LocalLow` / `LocalLow` | Windows LocalLow | ✅ 已验证 (macOS) |

## 实际游戏案例

> **注意**: 每个平台需要单独测试验证。未测试的平台标记为 ⚠️。

### Root=0 - Steam 默认云存档

| 游戏 | App ID | VDF 路径 | Windows | macOS | Linux |
|------|--------|----------|---------|-------|-------|
| CS2 | 730 | `cfg/config.cfg` | ⚠️ 未测试 | ✅ `~/Library/Application Support/Steam/userdata/{UserID}/730/remote/cfg/config.cfg` | ⚠️ 未测试 |

---

### Root=1 - 游戏安装目录

| 游戏 | App ID | VDF 路径 | Windows | macOS | Linux |
|------|--------|----------|---------|-------|-------|
| Celeste | 504230 | `Saves/0.celeste` | ⚠️ 未测试 | ✅ `~/Library/Application Support/Celeste/Saves/0.celeste` | ⚠️ 未测试 |

**macOS 注意**: 优先查找 `~/Library/Application Support/{GameName}/`，如不存在则回退到 `steamapps/common/{GameDir}/`

---

### Root=7 - 视频文件夹 / macOS Application Support

| 游戏 | App ID | VDF 路径 | Windows | macOS | Linux |
|------|--------|----------|---------|-------|-------|
| Finding Paradise | 337340 | `freebirdgames/findingparadise/Save4.rxdata` | ⚠️ 未测试 | ✅ `~/Library/Application Support/freebirdgames/findingparadise/Save4.rxdata` | ⚠️ 未测试 |

**平台差异**: 
- Windows: `%USERPROFILE%/Videos/`
- macOS: `~/Library/Application Support/` (特殊映射)
- Linux: `~/Videos/`

---

### Root=12 - Windows LocalLow / macOS Caches

| 游戏 | App ID | VDF 路径 | Windows | macOS | Linux |
|------|--------|----------|---------|-------|-------|
| Disco Elysium | 632470 | `ZAUM Studio/Disco Elysium/SaveGames/slot_0.ntwtf.zip` | ⚠️ 未测试 | ✅ `~/Library/Caches/ZAUM Studio/Disco Elysium/SaveGames/slot_0.ntwtf.zip` | ⚠️ 未测试 |

**平台差异**:
- Windows: `%USERPROFILE%/AppData/LocalLow/`
- macOS: `~/Library/Caches/`
- Linux: `~/.local/share/`

---

## 平台特殊说明

### macOS 路径映射注意事项

1. **Root=7 特殊映射** ✅ 已验证
   - Windows/Linux: `Videos` 文件夹
   - macOS: `~/Library/Application Support/` (特殊映射)
   - CDP 显示: `MacAppSupport`
   - 验证游戏: Finding Paradise (337340)

2. **Root=1 优先级** ✅ 已验证
   - macOS 优先检查: `~/Library/Application Support/{GameName}/`
   - 回退路径: `{SteamPath}/steamapps/common/{GameDir}/`
   - 验证游戏: Celeste (504230) 使用 Application Support

3. **Root=3 vs Root=7** ⚠️ 需进一步验证
   - Root=3: `~/Library/Application Support/` (AppData Roaming 语义)
   - Root=7: `~/Library/Application Support/` (实际映射)
   - 两者在 macOS 上指向同一位置，但语义不同
   - 需要更多游戏案例确认差异

### Windows 路径映射 ⚠️ 需验证

- Root=2: `%USERPROFILE%\Documents\`
- Root=3: `%APPDATA%\` (通常为 `%USERPROFILE%\AppData\Roaming\`)
- Root=4: `%LOCALAPPDATA%\` (通常为 `%USERPROFILE%\AppData\Local\`)
- Root=9: `%USERPROFILE%\Saved Games\`
- Root=12: `%USERPROFILE%\AppData\LocalLow\`

### Linux 路径映射 ⚠️ 需验证

- Root=2: `~/Documents/`
- Root=3: `~/.config/`
- Root=4: `~/.local/share/`
- Root=7: `~/Videos/`
- Root=11: `/tmp/` (临时目录)

---

## 如何贡献测试数据

如果你发现某个游戏的 Root 路径映射与此表不符，或想添加新的游戏案例，请：

### 1. 收集必要信息

- **游戏信息**: 游戏名称、App ID
- **系统信息**: 操作系统（Windows/macOS/Linux）和版本
- **VDF 信息**: 
  - `root` 值（在 `remotecache.vdf` 中）
  - 文件路径（VDF 中的相对路径）
- **CDP 信息** (可选):
  - Steam 网页上显示的文件夹名称
- **实际路径**: 文件在磁盘上的完整绝对路径

### 2. 验证方法

1. 运行游戏，生成存档文件
2. 查看 `{SteamPath}/userdata/{UserID}/{AppID}/remotecache.vdf`
3. 找到文件的 `root` 值
4. 在磁盘上确认文件实际位置
5. 对比映射表中的路径

### 3. 提交方式

- **Issue**: 标题格式 `[Root Mapping] {游戏名称} (App ID) - {平台}`
- **Pull Request**: 直接更新此文档，在对应的表格中添加或更新数据

### 4. 验证标准

- ✅ **已验证**: 至少有一个真实游戏案例在该平台上确认
- ⚠️ **需验证**: 基于官方文档或推导，但缺少实际游戏案例
- ❌ **错误**: 已发现与实际不符，需要修正

---

## 参考资料

### 官方文档

- [Steamworks Steam Cloud Documentation](https://partner.steamgames.com/doc/features/cloud) - 官方 Root 路径配置说明
- [ISteamRemoteStorage API Reference](https://partner.steamgames.com/doc/api/ISteamRemoteStorage) - Steam Cloud API 参考

### 社区验证资源

- [Quick Guide to Steam Cloud Saves](https://www.gamedeveloper.com/game-platforms/quick-guide-to-steam-cloud-saves) - 确认 Root 路径宏定义
- [Stack Exchange: Steam Cloud Save Location](https://gaming.stackexchange.com/questions/146644) - 社区验证的 Root 值映射
- [Elena Temple Dev Blog](https://www.grimtalin.com/2018/04/elena-temple-steam-cloud-saves.html) - Unity 游戏 Root 路径配置实例

### Steamworks Root 宏定义

根据 [Steamworks 官方文档](https://partner.steamgames.com/doc/features/cloud)，Steam Cloud 支持以下预定义 Root 宏：

| 宏名称 | Root 值 | 说明 |
|---------|---------|------|
| `WinMyDocuments` | 2 | Windows 文档文件夹 |
| `WinAppDataRoaming` | 3 | Windows AppData Roaming |
| `WinAppDataLocal` | 4 | Windows AppData Local |
| `WinSavedGames` | 9 | Windows Saved Games |
| `WinAppDataLocalLow` | 12 | Windows AppData LocalLow |
| `MacOS` | - | macOS 系统路径（根据 Root 值动态映射） |
| `LinuxHome` | - | Linux 用户目录（根据 Root 值动态映射） |

**注意**: 宏名称用于游戏配置文件，实际运行时会转换为对应的 Root 数值。

---

**最后更新**: 2025-12-07  
**维护者**: [@Fldicoahkiin](https://github.com/Fldicoahkiin)
