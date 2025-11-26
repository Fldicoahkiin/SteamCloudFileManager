use crate::path_resolver::{resolve_cloud_file_path, RootType};
use crate::steam_api::CloudFile;
use crate::vdf_parser::{VdfFileEntry, VdfParser};
use anyhow::{anyhow, Result};
use chrono::{Local, TimeZone};
use std::sync::{Arc, Mutex};

pub struct FileService {
    steam_manager: Option<Arc<Mutex<crate::steam_api::SteamCloudManager>>>,
}

impl FileService {
    pub fn new() -> Self {
        Self {
            steam_manager: None,
        }
    }

    pub fn with_steam_manager(
        steam_manager: Arc<Mutex<crate::steam_api::SteamCloudManager>>,
    ) -> Self {
        Self {
            steam_manager: Some(steam_manager),
        }
    }

    // 获取云文件列表
    // VDF -> Steam API 回退 -> CDP 补充
    pub fn get_cloud_files(&self, app_id: u32) -> Result<Vec<CloudFile>> {
        if app_id == 0 {
            return Err(anyhow!("未设置 App ID"));
        }

        // VDF
        match self.get_files_from_vdf(app_id) {
            Ok(files) if !files.is_empty() => {
                log::info!("使用 VDF 获取到 {} 个文件", files.len());
                return Ok(files);
            }
            Ok(_) => log::debug!("VDF 返回空列表，尝试其他方式"),
            Err(e) => log::debug!("VDF 获取失败: {}，尝试 Steam API", e),
        }

        // Steam API
        if let Some(manager) = &self.steam_manager {
            match self.get_files_from_steam_api(manager) {
                Ok(files) => {
                    log::info!("使用 Steam API 获取到 {} 个文件", files.len());
                    return Ok(files);
                }
                Err(e) => log::warn!("Steam API 获取失败: {}", e),
            }
        }

        Err(anyhow!("无法获取文件列表：所有策略均失败"))
    }

    // 从 VDF 获取文件列表
    fn get_files_from_vdf(&self, app_id: u32) -> Result<Vec<CloudFile>> {
        log::debug!("尝试从 VDF 解析文件列表，App ID: {}", app_id);

        let parser = VdfParser::new()?;
        let vdf_entries = parser.parse_remotecache(app_id)?;

        let steam_path = parser.get_steam_path().clone();
        let user_id = parser.get_user_id().to_string();

        let files: Vec<CloudFile> = vdf_entries
            .into_iter()
            .map(|entry| build_cloud_file_from_vdf(entry, &steam_path, &user_id, app_id))
            .collect();

        log::debug!("VDF 解析完成: {} 个文件", files.len());
        Ok(files)
    }

    // 从 Steam API 获取文件列表
    fn get_files_from_steam_api(
        &self,
        manager: &Arc<Mutex<crate::steam_api::SteamCloudManager>>,
    ) -> Result<Vec<CloudFile>> {
        let mgr = manager.lock().map_err(|e| anyhow!("锁错误: {}", e))?;
        mgr.get_files_from_api()
    }

    // 合并 CDP 数据到现有文件列表
    pub fn merge_cdp_files(
        &self,
        mut files: Vec<CloudFile>,
        app_id: u32,
    ) -> Result<Vec<CloudFile>> {
        if !crate::cdp_client::CdpClient::is_cdp_running() {
            return Ok(files);
        }

        log::info!("尝试通过 CDP 补充文件信息...");

        if let Ok(mut client) = crate::cdp_client::CdpClient::connect() {
            if let Ok(cdp_files) = client.fetch_game_files(app_id) {
                log::info!("CDP 返回 {} 个文件", cdp_files.len());

                let file_map: std::collections::HashMap<String, usize> = files
                    .iter()
                    .enumerate()
                    .map(|(i, f)| (f.name.clone(), i))
                    .collect();

                for cdp_file in cdp_files {
                    if let Some(&idx) = file_map.get(&cdp_file.name) {
                        // 更新现有文件信息
                        let f = &mut files[idx];
                        f.size = cdp_file.size;
                        f.timestamp = cdp_file.timestamp;
                        f.is_persisted = true;
                        if cdp_file.root_description.starts_with("CDP:") {
                            f.root_description = cdp_file.root_description;
                        }
                    } else {
                        // 添加新文件
                        files.push(cdp_file);
                    }
                }
            }
        }

        Ok(files)
    }
}

impl Default for FileService {
    fn default() -> Self {
        Self::new()
    }
}

// 从 VDF 条目构建 CloudFile
fn build_cloud_file_from_vdf(
    entry: VdfFileEntry,
    steam_path: &std::path::Path,
    user_id: &str,
    app_id: u32,
) -> CloudFile {
    let timestamp = if entry.timestamp > 0 {
        Local
            .timestamp_opt(entry.timestamp, 0)
            .single()
            .unwrap_or_else(Local::now)
    } else {
        Local::now()
    };

    let root_desc = get_root_description(entry.root);

    // 使用 path_resolver 解析路径
    let actual_path =
        resolve_cloud_file_path(entry.root, &entry.filename, steam_path, user_id, app_id).ok();

    let exists = actual_path.as_ref().map(|p| p.exists()).unwrap_or(false);

    CloudFile {
        name: entry.filename,
        size: entry.size,
        timestamp,
        is_persisted: entry.sync_state == 2,
        root: entry.root,
        root_description: root_desc,
        exists,
        conflict: false,
    }
}

// 获取 Root 类型的描述
fn get_root_description(root: u32) -> String {
    RootType::from_u32(root)
        .map(|r| r.description().to_string())
        .unwrap_or_else(|| format!("未知Root ({})", root))
}
