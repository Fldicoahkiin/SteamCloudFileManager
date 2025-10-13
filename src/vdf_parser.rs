use anyhow::{anyhow, Result};
use std::fs;
use std::path::{Path, PathBuf};

pub struct VdfParser {
    steam_path: PathBuf,
    user_id: String,
}

#[derive(Debug, Clone)]
pub struct VdfFileEntry {
    pub filename: String,
    pub root: u32,
    pub size: i32,
    pub timestamp: i64,
    pub sha: String,
    pub sync_state: i32,
    pub actual_path: Option<PathBuf>,
}

impl VdfParser {
    pub fn new() -> Result<Self> {
        let steam_path = Self::find_steam_path()?;
        let user_id = Self::find_user_id(&steam_path)?;

        Ok(Self {
            steam_path,
            user_id,
        })
    }

    fn find_steam_path() -> Result<PathBuf> {
        #[cfg(target_os = "windows")]
        {
            let paths = vec![
                PathBuf::from(r"C:\Program Files (x86)\Steam"),
                PathBuf::from(r"C:\Program Files\Steam"),
            ];

            for path in paths {
                if path.exists() {
                    return Ok(path);
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

    fn find_user_id(steam_path: &Path) -> Result<String> {
        let userdata_path = steam_path.join("userdata");

        if let Ok(entries) = fs::read_dir(&userdata_path) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.chars().all(|c| c.is_ascii_digit()) {
                    return Ok(name);
                }
            }
        }

        Err(anyhow!("未找到用户ID"))
    }

    fn get_game_install_dir(&self, app_id: u32) -> Result<PathBuf> {
        let manifest_path = self
            .steam_path
            .join("steamapps")
            .join(format!("appmanifest_{}.acf", app_id));

        if manifest_path.exists() {
            let content = fs::read_to_string(&manifest_path)?;

            for line in content.lines() {
                if line.contains("\"installdir\"") {
                    if let Some(dir) = line.split('"').nth(3) {
                        return Ok(self.steam_path.join("steamapps").join("common").join(dir));
                    }
                }
            }
        }

        Ok(self
            .steam_path
            .join("steamapps")
            .join("common")
            .join(format!("Game_{}", app_id)))
    }

    pub fn resolve_path(&self, root: u32, filename: &str, app_id: u32) -> Result<PathBuf> {
        let path = match root {
            0 => {
                // Steam默认remote文件夹
                self.steam_path
                    .join("userdata")
                    .join(&self.user_id)
                    .join(app_id.to_string())
                    .join("remote")
                    .join(filename)
            }
            1 => {
                // 游戏安装目录
                self.get_game_install_dir(app_id)?.join(filename)
            }
            2 => {
                // Documents文件夹
                #[cfg(windows)]
                {
                    let docs = std::env::var("USERPROFILE")?;
                    PathBuf::from(docs).join("Documents").join(filename)
                }
                #[cfg(target_os = "macos")]
                {
                    let home = std::env::var("HOME")?;
                    PathBuf::from(home).join("Documents").join(filename)
                }
                #[cfg(target_os = "linux")]
                {
                    let home = std::env::var("HOME")?;
                    PathBuf::from(home).join("Documents").join(filename)
                }
            }
            3 => {
                // AppData/Application Support
                #[cfg(windows)]
                {
                    let appdata = std::env::var("APPDATA")?;
                    PathBuf::from(appdata).join(filename)
                }
                #[cfg(target_os = "macos")]
                {
                    let home = std::env::var("HOME")?;
                    PathBuf::from(home)
                        .join("Library")
                        .join("Application Support")
                        .join(filename)
                }
                #[cfg(target_os = "linux")]
                {
                    let home = std::env::var("HOME")?;
                    PathBuf::from(home).join(".config").join(filename)
                }
            }
            4 => {
                // LocalAppData/Caches
                #[cfg(windows)]
                {
                    let localappdata = std::env::var("LOCALAPPDATA")?;
                    PathBuf::from(localappdata).join(filename)
                }
                #[cfg(target_os = "macos")]
                {
                    let home = std::env::var("HOME")?;
                    PathBuf::from(home)
                        .join("Library")
                        .join("Caches")
                        .join(filename)
                }
                #[cfg(target_os = "linux")]
                {
                    let home = std::env::var("HOME")?;
                    PathBuf::from(home)
                        .join(".local")
                        .join("share")
                        .join(filename)
                }
            }
            9 => {
                // Windows Saved Games文件夹 (Vista+)
                #[cfg(windows)]
                {
                    let userprofile = std::env::var("USERPROFILE")?;
                    PathBuf::from(userprofile)
                        .join("Saved Games")
                        .join(filename)
                }
                #[cfg(not(windows))]
                {
                    // macOS/Linux使用Documents/Saved Games
                    let home = std::env::var("HOME")?;
                    #[cfg(target_os = "macos")]
                    let base = PathBuf::from(home).join("Documents").join("Saved Games");
                    #[cfg(target_os = "linux")]
                    let base = PathBuf::from(home).join("Documents").join("Saved Games");
                    base.join(filename)
                }
            }
            12 => {
                // Windows LocalLow
                #[cfg(windows)]
                {
                    let userprofile = std::env::var("USERPROFILE")?;
                    PathBuf::from(userprofile)
                        .join("AppData")
                        .join("LocalLow")
                        .join(filename)
                }
                #[cfg(not(windows))]
                {
                    return Err(anyhow!("Root 12 (LocalLow)仅在Windows上支持"));
                }
            }
            _ => {
                return Err(anyhow!("未知的root值: {}", root));
            }
        };

        Ok(path)
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

        log::info!("解析VDF: {:?}", vdf_path);

        let content = fs::read_to_string(&vdf_path)?;
        let mut files = Vec::new();

        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i].trim();

            if line.starts_with('"')
                && line.ends_with('"')
                && (line.contains("/") || line.contains("\\"))
            {
                let filename = line.trim_matches('"');
                let mut entry = VdfFileEntry {
                    filename: filename.to_string(),
                    root: 0,
                    size: 0,
                    timestamp: 0,
                    sha: String::new(),
                    sync_state: 0,
                    actual_path: None,
                };

                i += 1;
                if i < lines.len() && lines[i].trim() == "{" {
                    i += 1;

                    while i < lines.len() && lines[i].trim() != "}" {
                        let attr_line = lines[i].trim();

                        // 解析root
                        if attr_line.contains("\"root\"") {
                            if let Some(val) = Self::extract_value(attr_line) {
                                entry.root = val.parse().unwrap_or(0);
                            }
                        }
                        // 解析size
                        else if attr_line.contains("\"size\"") {
                            if let Some(val) = Self::extract_value(attr_line) {
                                entry.size = val.parse().unwrap_or(0);
                            }
                        }
                        // 解析time
                        else if attr_line.contains("\"time\"")
                            && !attr_line.contains("localtime")
                            && !attr_line.contains("remotetime")
                        {
                            if let Some(val) = Self::extract_value(attr_line) {
                                entry.timestamp = val.parse().unwrap_or(0);
                            }
                        } else if attr_line.contains("\"sha\"") {
                            if let Some(val) = Self::extract_value(attr_line) {
                                entry.sha = val.to_string();
                            }
                        }
                        // 解析syncstate
                        else if attr_line.contains("\"syncstate\"") {
                            if let Some(val) = Self::extract_value(attr_line) {
                                entry.sync_state = val.parse().unwrap_or(0);
                            }
                        }

                        i += 1;
                    }
                    if let Ok(path) = self.resolve_path(entry.root, &entry.filename, app_id) {
                        entry.actual_path = Some(path);
                    }

                    files.push(entry);
                }
            }

            i += 1;
        }

        log::info!("解析完成: {} 个文件", files.len());
        Ok(files)
    }

    fn extract_value(line: &str) -> Option<&str> {
        let parts: Vec<&str> = line.split('"').collect();
        if parts.len() >= 4 {
            Some(parts[3])
        } else {
            None
        }
    }
}
