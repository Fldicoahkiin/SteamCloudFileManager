use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudGameInfo {
    pub app_id: u32,
    pub file_count: usize,
    pub total_size: u64,
    pub last_played: Option<i64>,
    pub playtime: Option<u32>,
    pub game_name: Option<String>,
    pub is_installed: bool,
    pub install_dir: Option<String>,
    pub categories: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameConfig {
    pub app_id: u32,
    pub last_played: Option<i64>,
    pub playtime: Option<u32>,
    pub launch_options: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameCategory {
    pub app_id: u32,
    pub tags: Vec<String>,
    pub is_favorite: bool,
    pub is_hidden: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppManifest {
    pub app_id: u32,
    pub name: String,
    pub install_dir: String,
    pub size_on_disk: Option<u64>,
}

// 扫描所有 Steam 库目录
pub fn discover_library_steamapps(steam_path: &Path) -> Vec<PathBuf> {
    let mut libs = Vec::new();
    let main = steam_path.join("steamapps");
    libs.push(main.clone());

    let libraryfolders_path = main.join("libraryfolders.vdf");
    if let Ok(content) = fs::read_to_string(&libraryfolders_path) {
        for line in content.lines() {
            if line.contains("\"path\"") {
                if let Some(path_str) = line.split('"').nth(3) {
                    let lib_path = PathBuf::from(path_str).join("steamapps");
                    if lib_path.exists() && lib_path != main {
                        libs.push(lib_path);
                    }
                }
            }
        }
    }

    libs
}

// 解析单个 app manifest 文件
pub fn parse_app_manifest(path: &Path) -> Result<AppManifest> {
    let content = fs::read_to_string(path)?;
    let mut app_id = None;
    let mut name = None;
    let mut install_dir = None;
    let mut size_on_disk = None;

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("\"appid\"") {
            if let Some(id_str) = line.split('"').nth(3) {
                app_id = id_str.parse().ok();
            }
        } else if line.starts_with("\"name\"") {
            if let Some(n) = line.split('"').nth(3) {
                name = Some(n.to_string());
            }
        } else if line.starts_with("\"installdir\"") {
            if let Some(dir) = line.split('"').nth(3) {
                install_dir = Some(dir.to_string());
            }
        } else if line.starts_with("\"SizeOnDisk\"") {
            if let Some(size_str) = line.split('"').nth(3) {
                size_on_disk = size_str.parse().ok();
            }
        }
    }

    if let (Some(app_id), Some(name), Some(install_dir)) = (app_id, name, install_dir) {
        Ok(AppManifest {
            app_id,
            name,
            install_dir,
            size_on_disk,
        })
    } else {
        Err(anyhow!("无法解析 manifest 文件"))
    }
}

// 扫描所有已安装游戏的 manifest
pub fn scan_app_manifests(steam_path: &Path) -> Result<HashMap<u32, AppManifest>> {
    let mut manifests = HashMap::new();
    for steamapps_path in discover_library_steamapps(steam_path) {
        if let Ok(entries) = fs::read_dir(&steamapps_path) {
            for entry in entries.flatten() {
                let filename = entry.file_name().to_string_lossy().to_string();
                if filename.starts_with("appmanifest_") && filename.ends_with(".acf") {
                    if let Ok(manifest) = parse_app_manifest(&entry.path()) {
                        manifests.entry(manifest.app_id).or_insert(manifest);
                    }
                }
            }
        }
    }

    Ok(manifests)
}

// 获取游戏配置信息
pub fn get_game_config(steam_path: &Path, user_id: &str, app_id: u32) -> Result<GameConfig> {
    let localconfig_path = steam_path
        .join("userdata")
        .join(user_id)
        .join("config")
        .join("localconfig.vdf");

    if !localconfig_path.exists() {
        return Err(anyhow!("localconfig.vdf 不存在"));
    }

    let content = fs::read_to_string(&localconfig_path)?;
    let app_id_str = app_id.to_string();
    let mut in_app_section = false;
    let mut last_played = None;
    let mut playtime = None;
    let mut launch_options = None;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with(&format!("\"{}\"", app_id_str)) {
            in_app_section = true;
            continue;
        }

        if in_app_section {
            if trimmed == "}" {
                break;
            }

            if trimmed.starts_with("\"LastPlayed\"") {
                if let Some(val) = trimmed.split('"').nth(3) {
                    last_played = val.parse().ok();
                }
            } else if trimmed.starts_with("\"Playtime\"") {
                if let Some(val) = trimmed.split('"').nth(3) {
                    playtime = val.parse().ok();
                }
            } else if trimmed.starts_with("\"LaunchOptions\"") {
                if let Some(val) = trimmed.split('"').nth(3) {
                    launch_options = Some(val.to_string());
                }
            }
        }
    }

    Ok(GameConfig {
        app_id,
        last_played,
        playtime,
        launch_options,
    })
}

// 解析 sharedconfig.vdf 获取游戏分类
pub fn parse_shared_config(steam_path: &Path, user_id: &str) -> Result<HashMap<u32, GameCategory>> {
    let mut categories = HashMap::new();
    let sharedconfig_path = steam_path
        .join("userdata")
        .join(user_id)
        .join("7")
        .join("remote")
        .join("sharedconfig.vdf");

    if !sharedconfig_path.exists() {
        return Ok(categories);
    }

    let content = fs::read_to_string(&sharedconfig_path)?;
    let mut current_app_id: Option<u32> = None;
    let mut current_tags = Vec::new();
    let mut is_favorite = false;
    let mut is_hidden = false;
    let mut in_tags_section = false;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with('"') && trimmed.ends_with('"') {
            if let Ok(app_id) = trimmed.trim_matches('"').parse::<u32>() {
                if let Some(prev_app_id) = current_app_id {
                    categories.insert(
                        prev_app_id,
                        GameCategory {
                            app_id: prev_app_id,
                            tags: current_tags.clone(),
                            is_favorite,
                            is_hidden,
                        },
                    );
                }

                current_app_id = Some(app_id);
                current_tags.clear();
                is_favorite = false;
                is_hidden = false;
                in_tags_section = false;
            }
        }

        if trimmed.starts_with("\"tags\"") {
            in_tags_section = true;
        } else if in_tags_section && trimmed.starts_with('"') {
            if let Some(tag) = trimmed.split('"').nth(3) {
                current_tags.push(tag.to_string());
            }
        } else if trimmed.starts_with("\"favorite\"") {
            is_favorite = trimmed.contains("\"1\"");
        } else if trimmed.starts_with("\"hidden\"") {
            is_hidden = trimmed.contains("\"1\"");
        }

        if trimmed == "}" && in_tags_section {
            in_tags_section = false;
        }
    }

    if let Some(app_id) = current_app_id {
        categories.insert(
            app_id,
            GameCategory {
                app_id,
                tags: current_tags,
                is_favorite,
                is_hidden,
            },
        );
    }

    Ok(categories)
}

// 扫描所有有云存档的游戏
pub fn scan_cloud_games(steam_path: &Path, user_id: &str) -> Result<Vec<CloudGameInfo>> {
    let mut games = Vec::new();
    let userdata_path = steam_path.join("userdata").join(user_id);

    tracing::info!("Steam 路径: {:?}", steam_path);
    tracing::info!("用户数据路径: {:?}", userdata_path);

    if !userdata_path.exists() {
        tracing::error!("用户数据路径不存在: {:?}", userdata_path);
        return Ok(games);
    }

    let all_manifests = scan_app_manifests(steam_path).unwrap_or_default();
    let all_categories = parse_shared_config(steam_path, user_id).unwrap_or_default();

    // 解析 appinfo.vdf
    let all_appinfo = if let Ok(parser) = crate::vdf_parser::VdfParser::new() {
        parser.parse_appinfo_vdf().unwrap_or_default()
    } else {
        HashMap::new()
    };

    if let Ok(entries) = fs::read_dir(&userdata_path) {
        let entries: Vec<_> = entries.flatten().collect();
        tracing::info!("找到 {} 个用户数据目录条目", entries.len());

        for entry in entries {
            let entry_name = entry.file_name().to_string_lossy().to_string();
            if let Ok(app_id) = entry_name.parse::<u32>() {
                let vdf_path = entry.path().join("remotecache.vdf");
                if vdf_path.exists() {
                    tracing::debug!(app_id = app_id, "发现云存档游戏");

                    // 使用 VdfParser 解析文件
                    let files = if let Ok(parser) = crate::vdf_parser::VdfParser::new() {
                        parser.parse_remotecache(app_id).unwrap_or_default()
                    } else {
                        Vec::new()
                    };

                    let total_size: u64 = files.iter().map(|f| f.size).sum();
                    let config = get_game_config(steam_path, user_id, app_id).ok();
                    let manifest = all_manifests.get(&app_id);
                    let category = all_categories.get(&app_id);
                    let appinfo = all_appinfo.get(&app_id);

                    let game_name = manifest
                        .as_ref()
                        .map(|m| m.name.clone())
                        .or_else(|| appinfo.and_then(|a| a.name.clone()));

                    games.push(CloudGameInfo {
                        app_id,
                        file_count: files.len(),
                        total_size,
                        last_played: config.as_ref().and_then(|c| c.last_played),
                        playtime: config.as_ref().and_then(|c| c.playtime),
                        game_name,
                        is_installed: manifest.is_some(),
                        install_dir: manifest.as_ref().map(|m| m.install_dir.clone()),
                        categories: category
                            .as_ref()
                            .map(|c| c.tags.clone())
                            .unwrap_or_default(),
                    });
                }
            }
        }
    }

    games.sort_by(|a, b| b.last_played.unwrap_or(0).cmp(&a.last_played.unwrap_or(0)));
    tracing::info!("VDF 扫描完成，发现 {} 个有云存档的游戏", games.len());
    Ok(games)
}

// 扫描结果
pub struct ScanResult {
    pub games: Vec<CloudGameInfo>,
    pub vdf_count: usize,
    pub cdp_count: usize,
}

// 获取并合并游戏列表（包括 CDP 数据）
pub fn fetch_and_merge_games(steam_path: PathBuf, user_id: String) -> Result<ScanResult> {
    let mut games = scan_cloud_games(&steam_path, &user_id)?;
    let vdf_count = games.len();

    let mut cdp_count = 0;
    let mut cdp_order = std::collections::HashMap::new();
    if crate::cdp_client::CdpClient::is_cdp_running() {
        if let Ok(mut client) = crate::cdp_client::CdpClient::connect() {
            if let Ok(cdp_games) = client.fetch_game_list() {
                cdp_count = cdp_games.len();
                let map: std::collections::HashMap<u32, usize> = games
                    .iter()
                    .enumerate()
                    .map(|(i, g)| (g.app_id, i))
                    .collect();

                let mut added = 0;
                let mut updated = 0;

                for (idx, cdp_game) in cdp_games.into_iter().enumerate() {
                    cdp_order.insert(cdp_game.app_id, idx);

                    if let Some(&i) = map.get(&cdp_game.app_id) {
                        let g = &mut games[i];
                        g.file_count = cdp_game.file_count;
                        g.total_size = cdp_game.total_size;
                        if let Some(name) = cdp_game.game_name {
                            if g.game_name.is_none() || g.game_name.as_deref() == Some("") {
                                g.game_name = Some(name);
                                updated += 1;
                            }
                        }
                    } else {
                        games.push(cdp_game);
                        added += 1;
                    }
                }
                tracing::info!(
                    "CDP 合并完成: 新增 {} 个游戏, 更新 {} 个信息",
                    added,
                    updated
                );
            }
        }
    }

    // 排序：已安装 -> CDP顺序 -> 名称
    games.sort_by(|a, b| {
        if a.is_installed != b.is_installed {
            return b.is_installed.cmp(&a.is_installed);
        }

        match (cdp_order.get(&a.app_id), cdp_order.get(&b.app_id)) {
            (Some(ia), Some(ib)) => ia.cmp(ib),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => {
                let name_a = a.game_name.as_deref().unwrap_or("");
                let name_b = b.game_name.as_deref().unwrap_or("");
                if name_a.is_empty() && name_b.is_empty() {
                    a.app_id.cmp(&b.app_id)
                } else {
                    name_a.cmp(name_b)
                }
            }
        }
    });

    Ok(ScanResult {
        games,
        vdf_count,
        cdp_count,
    })
}
