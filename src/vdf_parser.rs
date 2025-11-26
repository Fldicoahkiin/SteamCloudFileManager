use anyhow::{anyhow, Result};
use byteorder::{LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{Cursor, Read};
use std::path::PathBuf;

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
            if let Ok(p) = std::env::var("STEAM_PATH") {
                candidates.push(PathBuf::from(p));
            }
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

        Err(anyhow!("未找到Steam安装目录"))
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

        log::debug!("解析 VDF 文件: {:?}", vdf_path);

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

        log::debug!("VDF 解析完成: {} 个文件条目", files.len());
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

    pub fn get_all_users_info(&self) -> Result<Vec<UserInfo>> {
        crate::user_manager::get_all_users_info(&self.steam_path, &self.user_id)
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
            log::debug!("appinfo.vdf 不存在，跳过解析");
            return Ok(HashMap::new());
        }

        let data = match fs::read(&appinfo_path) {
            Ok(d) => d,
            Err(e) => {
                log::warn!("无法读取 appinfo.vdf: {}", e);
                return Ok(HashMap::new());
            }
        };

        let mut cursor = Cursor::new(data);
        let mut apps = HashMap::new();

        let magic = match cursor.read_u32::<LittleEndian>() {
            Ok(m) => m,
            Err(_) => {
                log::warn!("appinfo.vdf 格式无效");
                return Ok(HashMap::new());
            }
        };

        if magic != 0x07564427 && magic != 0x07564428 && magic != 0x07564429 {
            log::warn!("appinfo.vdf 格式不支持: 0x{:X}", magic);
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

        log::info!("从 appinfo.vdf 解析到 {} 个游戏", apps.len());
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
                            log::debug!("App {} 名称: {}", app_id, name);
                            return Ok(name);
                        }
                    }
                }
            }
        }

        Err(anyhow!("未找到游戏名称"))
    }
}
