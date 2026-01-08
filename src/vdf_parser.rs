use anyhow::{anyhow, Result};
use byteorder::{LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{Cursor, Read};
use std::path::PathBuf;

#[derive(Clone)]
pub struct VdfParser {
    steam_path: PathBuf,
    user_id: String,
}

#[derive(Debug, Clone)]
pub struct VdfFileEntry {
    pub filename: String,
    pub root: u32,
    pub size: u64,
    pub timestamp: i64,
    pub sha: String,
    pub sync_state: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppInfo {
    pub app_id: u32,
    pub name: Option<String>,
    pub developer: Option<String>,
    pub publisher: Option<String>,
}

// Steam Cloud 配置 (来自 appinfo.vdf 的 ufs 节)
#[derive(Debug, Clone, Default)]
pub struct UfsConfig {
    pub quota: u64,
    pub maxnumfiles: u32,
    pub raw_text: String,
    pub savefiles: Vec<crate::path_resolver::SaveFileConfig>, // 解析后的 savefiles 配置
}

#[derive(Debug, Clone)]
pub struct UserInfo {
    pub user_id: String,
    pub persona_name: Option<String>,
    pub is_current: bool,
}

impl VdfParser {
    pub fn new() -> Result<Self> {
        let steam_path = Self::find_steam_path()?;
        let (user_id, _) = crate::user_manager::find_user_id(&steam_path)?;
        Ok(Self {
            steam_path,
            user_id,
        })
    }

    pub fn find_steam_path() -> Result<PathBuf> {
        #[cfg(target_os = "windows")]
        {
            let mut candidates: Vec<PathBuf> = Vec::new();

            // 环境变量
            if let Ok(p) = std::env::var("STEAM_PATH") {
                candidates.push(PathBuf::from(p));
            }

            // 从 Windows 注册表读取
            if let Some(path) = Self::read_steam_path_from_registry() {
                candidates.push(path);
            }

            // 默认位置
            if let Ok(p) = std::env::var("PROGRAMFILES(X86)") {
                candidates.push(PathBuf::from(p).join("Steam"));
            }
            if let Ok(p) = std::env::var("PROGRAMFILES") {
                candidates.push(PathBuf::from(p).join("Steam"));
            }
            if let Ok(p) = std::env::var("LOCALAPPDATA") {
                candidates.push(PathBuf::from(p).join("Steam"));
            }
            if let Ok(p) = std::env::var("APPDATA") {
                candidates.push(PathBuf::from(p).join("Steam"));
            }

            for c in candidates {
                if c.join("userdata").exists() || c.join("steam.exe").exists() {
                    tracing::debug!("找到 Steam 安装路径: {:?}", c);
                    return Ok(c);
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            let home = std::env::var("HOME")?;
            let path = PathBuf::from(&home)
                .join("Library")
                .join("Application Support")
                .join("Steam");

            if path.exists() {
                return Ok(path);
            }
        }

        #[cfg(target_os = "linux")]
        {
            let home = std::env::var("HOME")?;
            let paths = vec![
                PathBuf::from(&home).join(".steam").join("steam"),
                PathBuf::from(&home)
                    .join(".local")
                    .join("share")
                    .join("Steam"),
            ];

            for path in paths {
                if path.exists() {
                    return Ok(path);
                }
            }
        }

        Err(anyhow!(
            "未找到 Steam 安装目录\n\n请确保：\n1. 已安装 Steam 客户端\n2. Steam 安装在标准位置\n3. 至少运行过一次 Steam\n\n如果 Steam 安装在非标准位置，请设置环境变量 STEAM_PATH"
        ))
    }

    /// 从 Windows 注册表读取 Steam 安装路径
    #[cfg(target_os = "windows")]
    fn read_steam_path_from_registry() -> Option<PathBuf> {
        use std::ptr::null_mut;
        use winapi::um::winnt::{KEY_READ, REG_SZ};
        use winapi::um::winreg::{
            RegCloseKey, RegOpenKeyExA, RegQueryValueExA, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE,
        };

        // 尝试的注册表路径
        let registry_paths = [
            // 64 位系统上的 32 位 Steam
            (HKEY_LOCAL_MACHINE, "SOFTWARE\\WOW6432Node\\Valve\\Steam\0"),
            // 32 位系统或 64 位 Steam
            (HKEY_LOCAL_MACHINE, "SOFTWARE\\Valve\\Steam\0"),
            // 当前用户
            (HKEY_CURRENT_USER, "SOFTWARE\\Valve\\Steam\0"),
        ];

        for (root_key, sub_key) in registry_paths {
            unsafe {
                let mut hkey: winapi::shared::minwindef::HKEY = null_mut();

                // 打开注册表键
                let result = RegOpenKeyExA(
                    root_key,
                    sub_key.as_ptr() as *const i8,
                    0,
                    KEY_READ,
                    &mut hkey,
                );

                if result != 0 {
                    continue; // 打开失败，尝试下一个路径
                }

                // 读取 InstallPath 值
                let value_name = "InstallPath\0";
                let mut buffer = [0u8; 512];
                let mut buffer_size = buffer.len() as u32;
                let mut value_type: u32 = 0;

                let result = RegQueryValueExA(
                    hkey,
                    value_name.as_ptr() as *const i8,
                    null_mut(),
                    &mut value_type,
                    buffer.as_mut_ptr(),
                    &mut buffer_size,
                );

                RegCloseKey(hkey);

                if result == 0 && value_type == REG_SZ && buffer_size > 1 {
                    // 移除末尾的 null 字符
                    let path_len = buffer_size as usize - 1;
                    if let Ok(path_str) = String::from_utf8(buffer[..path_len].to_vec()) {
                        let path = PathBuf::from(path_str.trim());
                        if path.exists() {
                            tracing::info!("从注册表读取到 Steam 路径: {:?}", path);
                            return Some(path);
                        }
                    }
                }
            }
        }

        tracing::debug!("未能从注册表读取 Steam 路径");
        None
    }

    // 解析remotecache.vdf文件
    pub fn parse_remotecache(&self, app_id: u32) -> Result<Vec<VdfFileEntry>> {
        let vdf_path = self
            .steam_path
            .join("userdata")
            .join(&self.user_id)
            .join(app_id.to_string())
            .join("remotecache.vdf");

        if !vdf_path.exists() {
            return Err(anyhow!("remotecache.vdf不存在: {:?}", vdf_path));
        }

        tracing::debug!("解析 VDF 文件: {:?}", vdf_path);

        let content = fs::read_to_string(&vdf_path)?;
        let mut files = Vec::new();

        let mut pending_key: Option<String> = None;
        let mut in_entry = false;
        let mut current: Option<VdfFileEntry> = None;

        for raw in content.lines() {
            let line = raw.trim();

            if !in_entry {
                if line.starts_with('"') && line.ends_with('"') {
                    let key = line.trim_matches('"');
                    if key.chars().all(|c| c.is_ascii_digit()) {
                        pending_key = None;
                    } else {
                        pending_key = Some(key.to_string());
                    }
                } else if line == "{" {
                    if let Some(name) = pending_key.take() {
                        in_entry = true;
                        current = Some(VdfFileEntry {
                            filename: name,
                            root: 0,
                            size: 0,
                            timestamp: 0,
                            sha: String::new(),
                            sync_state: 0,
                        });
                    }
                }
                continue;
            }

            if line == "}" {
                if let Some(e) = current.take() {
                    files.push(e);
                }
                in_entry = false;
                continue;
            }

            if let Some(e) = current.as_mut() {
                if let Some((key, val)) = Self::extract_key_value(line) {
                    match key {
                        "root" => {
                            e.root = val.parse().unwrap_or(0);
                        }
                        "size" => {
                            e.size = val.parse::<u64>().unwrap_or(0);
                        }
                        "localtime" => {
                            e.timestamp = val.parse::<i64>().unwrap_or(0);
                        }
                        "remotetime" | "time" => {
                            if e.timestamp == 0 {
                                e.timestamp = val.parse::<i64>().unwrap_or(0);
                            }
                        }
                        "sha" => {
                            e.sha = val.to_string();
                        }
                        "syncstate" => {
                            e.sync_state = val.parse().unwrap_or(0);
                        }
                        _ => {}
                    }
                }
            }
        }

        tracing::debug!("VDF 解析完成: {} 个文件条目", files.len());
        Ok(files)
    }

    fn extract_key_value(line: &str) -> Option<(&str, &str)> {
        let mut it = line.split('"');
        it.next()?;
        let key = it.next()?;
        it.next()?;
        let val = it.next()?;
        Some((key, val))
    }

    pub fn with_user_id(steam_path: PathBuf, user_id: String) -> Self {
        Self {
            steam_path,
            user_id,
        }
    }
    pub fn get_steam_path(&self) -> &PathBuf {
        &self.steam_path
    }

    pub fn get_user_id(&self) -> &str {
        &self.user_id
    }

    // 解析 appinfo.vdf 文件获取游戏信息
    pub fn parse_appinfo_vdf(&self) -> Result<HashMap<u32, AppInfo>> {
        let appinfo_path = self.steam_path.join("appcache").join("appinfo.vdf");

        if !appinfo_path.exists() {
            tracing::debug!("appinfo.vdf 不存在，跳过解析");
            return Ok(HashMap::new());
        }

        let data = match fs::read(&appinfo_path) {
            Ok(d) => d,
            Err(e) => {
                tracing::warn!("无法读取 appinfo.vdf: {}", e);
                return Ok(HashMap::new());
            }
        };

        let mut cursor = Cursor::new(data);
        let mut apps = HashMap::new();

        let magic = match cursor.read_u32::<LittleEndian>() {
            Ok(m) => m,
            Err(_) => {
                tracing::warn!("appinfo.vdf 格式无效");
                return Ok(HashMap::new());
            }
        };

        if magic != 0x07564427 && magic != 0x07564428 && magic != 0x07564429 {
            tracing::warn!("appinfo.vdf 格式不支持: 0x{:X}", magic);
            return Ok(HashMap::new());
        }

        let _ = cursor.read_u32::<LittleEndian>();

        let mut count = 0;
        while let Ok(app_id) = cursor.read_u32::<LittleEndian>() {
            if app_id == 0 || count > 10000 {
                break;
            }

            // 跳过 size, infostate, last_updated, access_token
            for _ in 0..3 {
                let _ = cursor.read_u32::<LittleEndian>();
            }
            let _ = cursor.read_u64::<LittleEndian>();

            // 读取 SHA 哈希 (20 字节)
            let mut sha = vec![0u8; 20];
            if cursor.read_exact(&mut sha).is_err() {
                break;
            }

            // 跳过 change_number (4 字节)
            let _ = cursor.read_u32::<LittleEndian>();

            // 尝试在 VDF 结构中找到游戏名称
            if let Ok(name) = Self::parse_appinfo_name(&mut cursor, app_id) {
                if !name.is_empty() && name.len() < 200 {
                    apps.insert(
                        app_id,
                        AppInfo {
                            app_id,
                            name: Some(name),
                            developer: None,
                            publisher: None,
                        },
                    );
                }
            }

            // 跳到下一个条目（读取剩余数据）
            let mut buf = vec![0u8; 4096];
            let mut skipped = 0;
            while skipped < 500000 {
                if cursor.read(&mut buf).is_err() {
                    break;
                }
                skipped += buf.len();
                // 寻找下一个 app_id 标记或结束
                if buf.starts_with(&[0, 0, 0, 0]) {
                    break;
                }
            }

            count += 1;
        }

        tracing::info!("从 appinfo.vdf 解析到 {} 个游戏", apps.len());
        Ok(apps)
    }

    fn parse_appinfo_name(cursor: &mut Cursor<Vec<u8>>, app_id: u32) -> Result<String> {
        // VDF 二进制格式：尝试找到 "name" 字段
        let mut buf = vec![0u8; 1024];
        if cursor.read(&mut buf).is_err() {
            return Err(anyhow!("无法读取"));
        }

        // 寻找 "common" 部分和 "name" 字段
        let buf_str = String::from_utf8_lossy(&buf);

        // 尝试找到 name 模式
        if let Some(name_pos) = buf_str.find("name\0") {
            let start = name_pos + 5; // 跳过 "name\0"
            if start < buf.len() {
                // 找到 "name" 后的字符串
                let remaining = &buf[start..];
                if let Some(null_pos) = remaining.iter().position(|&b| b == 0) {
                    if let Ok(name) = String::from_utf8(remaining[..null_pos].to_vec()) {
                        if !name.is_empty() && name.is_ascii() {
                            tracing::debug!("App {} 名称: {}", app_id, name);
                            return Ok(name);
                        }
                    }
                }
            }
        }

        Err(anyhow!("未找到游戏名称"))
    }

    // 获取指定 app_id 的 ufs 云存储配置
    pub fn get_ufs_config(&self, app_id: u32) -> Result<UfsConfig> {
        let appinfo_path = self.steam_path.join("appcache").join("appinfo.vdf");
        if !appinfo_path.exists() {
            return Err(anyhow!("appinfo.vdf 不存在"));
        }

        // 检查文件修改时间，确保不是过期缓存
        if let Ok(metadata) = fs::metadata(&appinfo_path) {
            if let Ok(modified) = metadata.modified() {
                let age = std::time::SystemTime::now()
                    .duration_since(modified)
                    .unwrap_or_default();
                tracing::debug!("appinfo.vdf 最后修改: {:?} 前", age);
            }
        }

        let data = fs::read(&appinfo_path)?;
        tracing::debug!("读取 appinfo.vdf: {} bytes", data.len());

        Self::parse_app_ufs_config(&data, app_id)
    }

    // 解析 appinfo.vdf 获取指定 app 的 ufs 配置
    fn parse_app_ufs_config(data: &[u8], target_app_id: u32) -> Result<UfsConfig> {
        let mut cursor = Cursor::new(data);

        let magic = cursor.read_u32::<LittleEndian>()?;
        let version = match magic {
            0x07564427 => 27,
            0x07564428 => 28,
            0x07564429 => 29,
            _ => return Err(anyhow!("不支持的 appinfo.vdf 版本: 0x{:X}", magic)),
        };

        tracing::debug!("appinfo.vdf 版本: {}", version);
        cursor.read_u32::<LittleEndian>()?; // universe

        // 版本 29+ 有字符串表
        let string_table_offset = if version >= 29 {
            cursor.read_u64::<LittleEndian>()?
        } else {
            0
        };

        // 解析字符串表 (版本 29+)
        let string_table = if version >= 29 && string_table_offset > 0 {
            Self::parse_string_table(data, string_table_offset as usize)?
        } else {
            Vec::new()
        };

        // 查找 ufs 在字符串表中的索引
        let ufs_index = string_table.iter().position(|s| s == "ufs");
        let quota_index = string_table.iter().position(|s| s == "quota");
        let maxnumfiles_index = string_table.iter().position(|s| s == "maxnumfiles");

        tracing::debug!(
            "字符串索引: ufs={:?}, quota={:?}, maxnumfiles={:?}",
            ufs_index,
            quota_index,
            maxnumfiles_index
        );

        loop {
            let entry_start = cursor.position();
            let app_id = cursor.read_u32::<LittleEndian>()?;
            if app_id == 0 {
                break;
            }

            let size = cursor.read_u32::<LittleEndian>()?;

            // size 是 size 字段之后所有数据的大小
            // 头部字段: infostate(4) + last_updated(4) + access_token(8) + sha(20) + change_number(4) + binary_sha(20 for v28+)
            let header_size: usize = 4 + 4 + 8 + 20 + 4 + if version >= 28 { 20 } else { 0 };
            let vdf_size = (size as usize).saturating_sub(header_size);

            if app_id == target_app_id {
                // 跳过头部字段
                cursor.read_u32::<LittleEndian>()?; // infostate
                cursor.read_u32::<LittleEndian>()?; // last_updated
                cursor.read_u64::<LittleEndian>()?; // access_token

                let mut sha = vec![0u8; 20];
                cursor.read_exact(&mut sha)?;

                cursor.read_u32::<LittleEndian>()?; // change_number

                if version >= 28 {
                    let mut binary_sha = vec![0u8; 20];
                    cursor.read_exact(&mut binary_sha)?;
                }

                let mut vdf_data = vec![0u8; vdf_size];
                cursor.read_exact(&mut vdf_data)?;

                tracing::debug!("找到 app {} 的 VDF 数据: {} bytes", app_id, vdf_data.len());

                // 使用简化的解析方法
                return Self::extract_ufs_from_binary_vdf(&vdf_data, &string_table, version);
            } else {
                // 跳过整个条目: 已读取 app_id(4) + size(4)，剩余 size 字节
                cursor.set_position(entry_start + 8 + size as u64);
            }
        }

        Err(anyhow!("未找到 app_id {} 的配置", target_app_id))
    }

    // 解析字符串表 (版本 29+)
    fn parse_string_table(data: &[u8], offset: usize) -> Result<Vec<String>> {
        if offset >= data.len() {
            return Ok(Vec::new());
        }

        let mut strings = Vec::new();
        let mut pos = offset;

        while pos < data.len() {
            let start = pos;
            while pos < data.len() && data[pos] != 0 {
                pos += 1;
            }

            // 跳过空字符串 (与 Python 逻辑一致)
            if pos == start {
                pos += 1;
                continue;
            }

            if let Ok(s) = String::from_utf8(data[start..pos].to_vec()) {
                strings.push(s);
            }

            pos += 1; // 跳过 null

            // 防止无限循环
            if strings.len() > 50000 {
                break;
            }
        }

        tracing::debug!("解析字符串表: {} 个字符串", strings.len());
        Ok(strings)
    }

    // 从二进制 VDF 数据提取完整的 ufs 配置
    fn extract_ufs_from_binary_vdf(
        data: &[u8],
        string_table: &[String],
        version: u32,
    ) -> Result<UfsConfig> {
        let mut config = UfsConfig::default();

        if version >= 29 && !string_table.is_empty() {
            // 找到 "ufs" 在字符串表中的索引
            let ufs_idx = match string_table.iter().position(|s| s == "ufs") {
                Some(idx) => idx,
                None => {
                    config.raw_text = "未找到 ufs 配置 (字符串表中无 ufs)".to_string();
                    return Ok(config);
                }
            };

            // 搜索 ufs 节的起始位置: 0x00 (section type) + ufs_idx (4 bytes LE)
            let ufs_pattern = [
                0x00u8,
                (ufs_idx & 0xFF) as u8,
                ((ufs_idx >> 8) & 0xFF) as u8,
                ((ufs_idx >> 16) & 0xFF) as u8,
                ((ufs_idx >> 24) & 0xFF) as u8,
            ];

            tracing::debug!("搜索 ufs 节: idx={}, pattern={:02x?}", ufs_idx, ufs_pattern);

            let ufs_start = match Self::find_pattern(data, &ufs_pattern) {
                Some(pos) => {
                    tracing::debug!("找到 ufs 节起始位置: {}", pos);
                    pos + 5
                }
                None => {
                    config.raw_text = format!("未找到 ufs 配置 (模式 {:02x?} 未匹配)", ufs_pattern);
                    return Ok(config);
                }
            };

            // 解析完整的 ufs 节
            let mut cursor = Cursor::new(&data[ufs_start..]);
            let mut lines = Vec::new();
            lines.push("\"ufs\"".to_string());
            lines.push("{".to_string());

            Self::parse_vdf_section(&mut cursor, string_table, &mut lines, 1, &mut config);

            lines.push("}".to_string());
            config.raw_text = lines.join("\n");

            tracing::debug!("解析 ufs 完成，共 {} 行", lines.len());
        } else {
            config.raw_text = "不支持的 appinfo.vdf 版本".to_string();
        }

        Ok(config)
    }

    // 递归解析 VDF 节
    fn parse_vdf_section(
        cursor: &mut Cursor<&[u8]>,
        string_table: &[String],
        lines: &mut Vec<String>,
        indent: usize,
        config: &mut UfsConfig,
    ) {
        Self::parse_vdf_section_inner(cursor, string_table, lines, indent, config, "");
    }

    // 递归解析 VDF 节 (内部实现)
    fn parse_vdf_section_inner(
        cursor: &mut Cursor<&[u8]>,
        string_table: &[String],
        lines: &mut Vec<String>,
        indent: usize,
        config: &mut UfsConfig,
        parent_key: &str,
    ) {
        let indent_str = "    ".repeat(indent);

        while let Ok(type_byte) = cursor.read_u8() {
            if type_byte == 0x08 {
                break;
            }

            let key_idx = match cursor.read_u32::<LittleEndian>() {
                Ok(idx) => idx as usize,
                Err(_) => break,
            };

            let key = string_table
                .get(key_idx)
                .cloned()
                .unwrap_or_else(|| format!("#{}", key_idx));

            match type_byte {
                0x00 => {
                    lines.push(format!("{}\"{}\"", indent_str, key));
                    lines.push(format!("{}{{", indent_str));

                    // 检查是否进入 savefiles 的子条目 (如 "0", "1", ...)
                    if parent_key == "savefiles" && key.chars().all(|c| c.is_ascii_digit()) {
                        let savefile =
                            Self::parse_savefile_entry(cursor, string_table, lines, indent + 1);
                        config.savefiles.push(savefile);
                    } else {
                        Self::parse_vdf_section_inner(
                            cursor,
                            string_table,
                            lines,
                            indent + 1,
                            config,
                            &key,
                        );
                    }
                    lines.push(format!("{}}}", indent_str));
                }
                0x01 => {
                    let value = Self::read_null_string(cursor);
                    lines.push(format!("{}\"{}\" \"{}\"", indent_str, key, value));
                }
                0x02 => {
                    let value = cursor.read_i32::<LittleEndian>().unwrap_or(0);
                    lines.push(format!("{}\"{}\" \"{}\"", indent_str, key, value));
                    if key == "quota" {
                        config.quota = value as u64;
                    } else if key == "maxnumfiles" {
                        config.maxnumfiles = value as u32;
                    }
                }
                0x07 => {
                    let value = cursor.read_u64::<LittleEndian>().unwrap_or(0);
                    lines.push(format!("{}\"{}\" \"{}\"", indent_str, key, value));
                }
                _ => {
                    tracing::debug!("未知 VDF 类型: 0x{:02x}, key={}", type_byte, key);
                }
            }
        }
    }

    // 解析单个 savefile 条目
    fn parse_savefile_entry(
        cursor: &mut Cursor<&[u8]>,
        string_table: &[String],
        lines: &mut Vec<String>,
        indent: usize,
    ) -> crate::path_resolver::SaveFileConfig {
        let mut savefile = crate::path_resolver::SaveFileConfig {
            recursive: true,
            ..Default::default()
        };
        let indent_str = "    ".repeat(indent);

        while let Ok(type_byte) = cursor.read_u8() {
            if type_byte == 0x08 {
                break;
            }

            let key_idx = match cursor.read_u32::<LittleEndian>() {
                Ok(idx) => idx as usize,
                Err(_) => break,
            };

            let key = string_table
                .get(key_idx)
                .cloned()
                .unwrap_or_else(|| format!("#{}", key_idx));

            match type_byte {
                0x00 => {
                    lines.push(format!("{}\"{}\"", indent_str, key));
                    lines.push(format!("{}{{", indent_str));
                    // 解析 platforms 子节
                    if key == "platforms" {
                        Self::parse_platforms(
                            cursor,
                            string_table,
                            lines,
                            indent + 1,
                            &mut savefile.platforms,
                        );
                    } else {
                        Self::skip_section(cursor);
                    }
                    lines.push(format!("{}}}", indent_str));
                }
                0x01 => {
                    let value = Self::read_null_string(cursor);
                    lines.push(format!("{}\"{}\" \"{}\"", indent_str, key, value));
                    match key.as_str() {
                        "root" => savefile.root = value,
                        "path" => savefile.path = value,
                        "pattern" => savefile.pattern = value,
                        _ => {}
                    }
                }
                0x02 => {
                    let value = cursor.read_i32::<LittleEndian>().unwrap_or(0);
                    lines.push(format!("{}\"{}\" \"{}\"", indent_str, key, value));
                }
                _ => {
                    tracing::debug!("savefile 未知类型: 0x{:02x}", type_byte);
                }
            }
        }

        savefile.root_type = crate::path_resolver::root_name_to_type(&savefile.root);
        savefile
    }

    // 解析 platforms 子节
    fn parse_platforms(
        cursor: &mut Cursor<&[u8]>,
        string_table: &[String],
        lines: &mut Vec<String>,
        indent: usize,
        platforms: &mut Vec<String>,
    ) {
        let indent_str = "    ".repeat(indent);

        while let Ok(type_byte) = cursor.read_u8() {
            if type_byte == 0x08 {
                break;
            }

            let key_idx = match cursor.read_u32::<LittleEndian>() {
                Ok(idx) => idx as usize,
                Err(_) => break,
            };

            let key = string_table
                .get(key_idx)
                .cloned()
                .unwrap_or_else(|| format!("#{}", key_idx));

            if type_byte == 0x01 {
                let value = Self::read_null_string(cursor);
                lines.push(format!("{}\"{}\" \"{}\"", indent_str, key, value));
                platforms.push(value);
            }
        }
    }

    // 跳过一个 section
    fn skip_section(cursor: &mut Cursor<&[u8]>) {
        while let Ok(type_byte) = cursor.read_u8() {
            if type_byte == 0x08 {
                break;
            }
            // 跳过 key index
            let _ = cursor.read_u32::<LittleEndian>();
            match type_byte {
                0x00 => Self::skip_section(cursor),
                0x01 => {
                    Self::read_null_string(cursor);
                }
                0x02 => {
                    let _ = cursor.read_i32::<LittleEndian>();
                }
                0x07 => {
                    let _ = cursor.read_u64::<LittleEndian>();
                }
                _ => {}
            }
        }
    }

    // 读取 null 结尾的字符串
    fn read_null_string(cursor: &mut Cursor<&[u8]>) -> String {
        let mut bytes = Vec::new();
        loop {
            match cursor.read_u8() {
                Ok(0) => break,
                Ok(b) => bytes.push(b),
                Err(_) => break,
            }
        }
        String::from_utf8(bytes).unwrap_or_default()
    }

    // 查找字节模式
    fn find_pattern(data: &[u8], pattern: &[u8]) -> Option<usize> {
        data.windows(pattern.len()).position(|w| w == pattern)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_ufs_config() {
        // 测试需要 Steam 安装，跳过 CI
        let parser = match VdfParser::new() {
            Ok(p) => p,
            Err(e) => {
                println!("Steam not installed, skipping test: {}", e);
                return;
            }
        };

        // 测试多个游戏
        let test_apps = [
            (730, "CS2"),
            (570, "Dota 2"),
            (440, "TF2"),
            (292030, "The Witcher 3"),
            (337340, "Finding Paradise"),
        ];

        for (app_id, name) in test_apps {
            match parser.get_ufs_config(app_id) {
                Ok(config) => {
                    println!("\n=== {} ({}) ===", name, app_id);
                    println!("quota={}, maxnumfiles={}", config.quota, config.maxnumfiles);
                    println!("savefiles count: {}", config.savefiles.len());
                    for (i, sf) in config.savefiles.iter().enumerate() {
                        println!(
                            "  [{}] root={}, path={}, pattern={}, platforms={:?}",
                            i, sf.root, sf.path, sf.pattern, sf.platforms
                        );
                    }
                    println!("{}", config.raw_text);
                }
                Err(_) => {
                    println!("\n=== {} ({}) === Not installed", name, app_id);
                }
            }
        }
    }
}
