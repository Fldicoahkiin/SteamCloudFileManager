use crate::path_resolver::{get_root_description, resolve_cloud_file_path};
use crate::steam_api::CloudFile;
use crate::vdf_parser::{VdfFileEntry, VdfParser};
use anyhow::{Result, anyhow};
use chrono::{Local, TimeZone};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub struct FileService {
    steam_manager: Option<Arc<Mutex<crate::steam_worker::SteamWorkerManager>>>,
    vdf_parser: Option<VdfParser>,
}

impl FileService {
    pub fn new() -> Self {
        Self {
            steam_manager: None,
            vdf_parser: VdfParser::new().ok(),
        }
    }

    pub fn with_steam_manager(
        steam_manager: Arc<Mutex<crate::steam_worker::SteamWorkerManager>>,
    ) -> Self {
        Self {
            steam_manager: Some(steam_manager),
            vdf_parser: VdfParser::new().ok(),
        }
    }

    // 获取云文件列表
    // VDF -> Steam API 回退 -> CDP 补充
    pub fn get_cloud_files(&self, app_id: u32) -> Result<Vec<CloudFile>> {
        if app_id == 0 {
            return Err(anyhow!("未设置 App ID"));
        }

        // VDF
        let vdf_files = self.get_files_from_vdf(app_id)?;
        if !vdf_files.is_empty() {
            tracing::info!("获取到 {} 个云文件", vdf_files.len());
            return Ok(vdf_files);
        } else {
            tracing::debug!("VDF 返回空列表，尝试其他方式");
        }

        // Steam API
        if let Some(manager) = &self.steam_manager {
            match self.get_files_from_steam_api(manager) {
                Ok(files) => {
                    tracing::info!(source = "Steam API", "获取云文件成功");
                    return Ok(files);
                }
                Err(e) => tracing::warn!(error = %e, "Steam API 获取失败"),
            }
        }

        Err(anyhow!("无法获取文件列表：所有策略均失败"))
    }

    // 从 VDF 获取文件列表
    fn get_files_from_vdf(&self, app_id: u32) -> Result<Vec<CloudFile>> {
        tracing::debug!(app_id = app_id, "尝试从 VDF 解析文件列表");

        let parser = self
            .vdf_parser
            .as_ref()
            .ok_or_else(|| anyhow!("VdfParser 未初始化"))?;
        let vdf_entries = parser.parse_remotecache(app_id)?;

        // 获取并缓存 rootoverrides 配置
        if let Ok(ufs_config) = parser.get_ufs_config(app_id)
            && !ufs_config.rootoverrides.is_empty()
        {
            crate::path_resolver::set_root_overrides_cache(app_id, ufs_config.rootoverrides);
        }

        let steam_path = parser.get_steam_path().clone();
        let user_id = parser.get_user_id().to_string();

        let files: Vec<CloudFile> = vdf_entries
            .into_iter()
            .map(|entry| build_cloud_file_from_vdf(entry, &steam_path, &user_id, app_id))
            .collect();

        tracing::debug!(count = files.len(), "VDF 解析完成");
        Ok(files)
    }

    // 从 Steam API 获取文件列表
    fn get_files_from_steam_api(
        &self,
        manager: &Arc<Mutex<crate::steam_worker::SteamWorkerManager>>,
    ) -> Result<Vec<CloudFile>> {
        use chrono::{Local, TimeZone};
        let mut mgr = manager.lock().map_err(|e| anyhow!("锁错误: {}", e))?;
        let worker_files = mgr.get_files()?;

        // 使用缓存的 VdfParser 获取 Steam 路径信息
        let path_info = self
            .vdf_parser
            .as_ref()
            .map(|p| (p.get_steam_path().clone(), p.get_user_id().to_string()));
        let app_id = mgr.get_app_id();

        Ok(worker_files
            .into_iter()
            .map(|f| {
                // 验证文件是否真实存在于本地（而不是信任 API 缓存）
                let actual_exists = if let Some((ref steam_path, ref user_id)) = path_info {
                    resolve_cloud_file_path(f.root, &f.name, steam_path, user_id, app_id)
                        .map(|p| p.exists())
                        .unwrap_or(false)
                } else {
                    f.exists
                };

                CloudFile {
                    name: f.name,
                    size: f.size,
                    timestamp: Local
                        .timestamp_opt(f.timestamp, 0)
                        .single()
                        .unwrap_or_else(Local::now),
                    is_persisted: f.is_persisted,
                    exists: actual_exists,
                    root: f.root,
                    root_description: f.root_description,
                }
            })
            .collect())
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

        tracing::info!("尝试通过 CDP 补充文件信息");

        if let Ok(mut client) =
            crate::cdp_client::CdpClient::connect_for(crate::cdp_client::CdpTarget::FileList)
            && let Ok(cdp_files) = client.fetch_game_files(app_id)
        {
            tracing::info!(count = cdp_files.len(), "CDP 返回文件");

            let file_map: std::collections::HashMap<String, usize> = files
                .iter()
                .enumerate()
                .map(|(i, f)| (f.name.clone(), i))
                .collect();

            for cdp_file in cdp_files {
                if let Some(&idx) = file_map.get(&cdp_file.name) {
                    let f = &mut files[idx];
                    let vdf_root_desc = f.root_description.clone();

                    f.size = cdp_file.size;
                    // 只有当 CDP 时间戳不是接近当前时间时才更新（避免解析失败时的 Local::now()）
                    let now = Local::now();
                    let time_diff = (now - cdp_file.timestamp).num_seconds().abs();
                    if time_diff > 60 {
                        f.timestamp = cdp_file.timestamp;
                    }
                    f.is_persisted = true;

                    if cdp_file.root_description.starts_with("CDP:") {
                        // 提取 URL 和 CDP 文件夹名
                        let content = &cdp_file.root_description[4..];
                        let parts: Vec<&str> = content.split('|').collect();
                        let url = parts.first().unwrap_or(&"");
                        let cdp_folder = parts.get(1).unwrap_or(&"");

                        // 保留 URL 以便 Hash 检测下载云端文件
                        // 如果 CDP folder 为空或不匹配，使用 VDF 的 folder 信息拼接
                        if !url.is_empty() {
                            if !cdp_folder.is_empty() {
                                f.root_description = cdp_file.root_description.clone();
                            } else {
                                // CDP folder 为空，使用 VDF 的 root_description 拼接 URL
                                f.root_description = format!("CDP:{}|{}", url, vdf_root_desc);
                            }
                        }

                        tracing::debug!(
                            "合并 CDP 文件: {} | Root={} | 本地存在={} | VDF: {} | CDP: {} | URL: {}",
                            f.name,
                            f.root,
                            f.exists,
                            vdf_root_desc,
                            cdp_folder,
                            url
                        );
                    } else {
                        tracing::debug!(
                            "合并 CDP 文件: {} | Root={} | 本地存在={} | VDF: {} | 保留原 root_description",
                            f.name,
                            f.root,
                            f.exists,
                            f.root_description
                        );
                    }
                } else {
                    // CDP 返回了本地列表中没有的文件（云端独有）
                    // 验证文件是否真实存在于本地
                    let mut cdp_file = cdp_file;

                    // 使用缓存的 VdfParser 获取 Steam 路径信息
                    if let Some(p) = self.vdf_parser.as_ref() {
                        let steam_path = p.get_steam_path();
                        let user_id = p.get_user_id().to_string();
                        cdp_file.exists = resolve_cloud_file_path(
                            cdp_file.root,
                            &cdp_file.name,
                            steam_path,
                            &user_id,
                            app_id,
                        )
                        .map(|p| p.exists())
                        .unwrap_or(false);
                    } else {
                        // 无法获取路径信息，保守地设为 false（CDP 只能确认云端存在）
                        cdp_file.exists = false;
                    }

                    tracing::debug!(
                        "新增 CDP 文件: {} | Root={} | 本地存在={} | {}",
                        cdp_file.name,
                        cdp_file.root,
                        cdp_file.exists,
                        cdp_file.root_description
                    );
                    files.push(cdp_file);
                }
            }
        }

        // 输出每个文件的详细信息
        log_file_details(&files, app_id, self.vdf_parser.as_ref());

        Ok(files)
    }

    pub fn get_files_from_cdp_only(&self, app_id: u32) -> Result<Vec<CloudFile>> {
        if app_id == 0 {
            return Err(anyhow!("未设置 App ID"));
        }

        if !crate::cdp_client::CdpClient::is_cdp_running() {
            return Err(anyhow!(
                "CDP 服务未运行，请确保 Steam 客户端已打开并访问云存储页面"
            ));
        }

        tracing::info!("通过 CDP 获取系统级 App ID {} 的文件列表", app_id);

        let mut client =
            crate::cdp_client::CdpClient::connect_for(crate::cdp_client::CdpTarget::FileList)
                .map_err(|e| anyhow!("CDP 连接失败: {}", e))?;

        let cdp_files = client
            .fetch_game_files(app_id)
            .map_err(|e| anyhow!("CDP 获取文件失败: {}", e))?;

        tracing::info!(count = cdp_files.len(), "CDP 返回文件");

        // 使用缓存的 VdfParser 验证本地文件存在性
        let files: Vec<CloudFile> = cdp_files
            .into_iter()
            .map(|mut cdp_file| {
                if let Some(p) = self.vdf_parser.as_ref() {
                    let steam_path = p.get_steam_path();
                    let user_id = p.get_user_id().to_string();
                    cdp_file.exists = resolve_cloud_file_path(
                        cdp_file.root,
                        &cdp_file.name,
                        steam_path,
                        &user_id,
                        app_id,
                    )
                    .map(|p| p.exists())
                    .unwrap_or(false);
                }
                cdp_file
            })
            .collect();

        log_file_details(&files, app_id, self.vdf_parser.as_ref());

        Ok(files)
    }
}

// 输出每个文件的详细信息日志
fn log_file_details(files: &[CloudFile], app_id: u32, parser: Option<&VdfParser>) {
    if files.is_empty() {
        return;
    }

    // 使用传入的 VdfParser 解析本地路径
    let (steam_path, user_id) = parser
        .map(|p| (p.get_steam_path().clone(), p.get_user_id().to_string()))
        .unwrap_or_else(|| (std::path::PathBuf::new(), String::new()));

    tracing::debug!(
        "========== 文件详情列表 ({} 个文件) | 平台: {} ==========",
        files.len(),
        get_platform_name()
    );

    for (i, f) in files.iter().enumerate() {
        // 解析 CDP 原始文件夹名
        let cdp_folder = if f.root_description.starts_with("CDP:") {
            let content = &f.root_description[4..];
            let parts: Vec<&str> = content.split('|').collect();
            parts.get(1).unwrap_or(&"").to_string()
        } else {
            "N/A".to_string()
        };

        // 解析本地绝对路径
        let local_path = if !steam_path.as_os_str().is_empty() {
            resolve_cloud_file_path(f.root, &f.name, &steam_path, &user_id, app_id)
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| "N/A".to_string())
        } else {
            "N/A".to_string()
        };

        let time_str = f.timestamp.format("%Y-%m-%d %H:%M:%S").to_string();
        let size_str = format_size(f.size);
        let exists_str = if f.exists { "✓" } else { "✗" };
        let synced_str = if f.is_persisted {
            "已同步"
        } else {
            "未同步"
        };

        // 显示原始数据：VDF root 数字 + CDP 文件夹名称 + 本地路径
        tracing::debug!(
            "[{:>3}] {} | VDF root={} | CDP folder={} | {} | {} | {} | {} | {}",
            i + 1,
            f.name,
            f.root,
            cdp_folder,
            size_str,
            time_str,
            exists_str,
            synced_str,
            local_path
        );
    }

    tracing::debug!("========== 文件列表结束 ==========");
}

// 获取当前平台名称
pub fn get_platform_name() -> &'static str {
    #[cfg(target_os = "macos")]
    return "macOS";
    #[cfg(target_os = "windows")]
    return "Windows";
    #[cfg(target_os = "linux")]
    return "Linux";
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    return "Unknown";
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
    }
}

// 文件操作结构体
pub struct FileOperations {
    steam_manager: Arc<Mutex<crate::steam_worker::SteamWorkerManager>>,
}

impl FileOperations {
    pub fn new(steam_manager: Arc<Mutex<crate::steam_worker::SteamWorkerManager>>) -> Self {
        Self { steam_manager }
    }

    // 确保应用的云同步已启用
    fn ensure_cloud_enabled(&self) {
        if let Ok(mut manager) = self.steam_manager.lock() {
            // 检查云同步是否已启用
            match manager.is_cloud_enabled_for_app() {
                Ok(true) => {
                    tracing::debug!("应用云同步已启用");
                }
                Ok(false) => {
                    tracing::info!("应用云同步未启用，尝试启用...");
                    if let Err(e) = manager.set_cloud_enabled_for_app(true) {
                        tracing::warn!("启用应用云同步失败: {}", e);
                    } else {
                        tracing::info!("应用云同步已启用");
                    }
                }
                Err(e) => {
                    tracing::warn!("检查云同步状态失败: {}", e);
                }
            }
        }
    }

    // 删除文件
    pub fn delete_file(&self, filename: &str) -> Result<bool> {
        let mut manager = self
            .steam_manager
            .lock()
            .map_err(|e| anyhow!("Steam 管理器锁错误: {}", e))?;

        manager
            .delete_file(filename)
            .map_err(|e| anyhow!("删除文件失败: {}", e))
    }

    // 移出云端
    pub fn forget_file(&self, filename: &str) -> anyhow::Result<bool> {
        let mut manager = self
            .steam_manager
            .lock()
            .map_err(|e| anyhow!("Steam 管理器锁错误: {}", e))?;
        manager.forget_file(filename)
    }

    // 写入文件到云端
    pub fn write_file(&self, filename: &str, data: &[u8]) -> anyhow::Result<bool> {
        let mut manager = self
            .steam_manager
            .lock()
            .map_err(|e| anyhow!("Steam 管理器锁错误: {}", e))?;
        manager.write_file(filename, data)
    }

    pub fn batch_operation<F>(&self, filenames: &[String], operation: F) -> (usize, Vec<String>)
    where
        F: Fn(&str) -> anyhow::Result<bool>,
    {
        let mut success_count = 0;
        let mut failed_files = Vec::new();

        for filename in filenames {
            match operation(filename) {
                Ok(true) => success_count += 1,
                Ok(false) => {
                    // Steam API 返回 false：自动云同步文件 (root != 0) 不支持 API 操作；
                    // root=0 的文件返回 false 则表示文件不存在
                    tracing::debug!(
                        "Steam API 返回 false (自动云同步文件或文件不存在): {}",
                        filename
                    );
                    failed_files.push(filename.to_string());
                }
                Err(e) => failed_files.push(format!("{} (错误: {})", filename, e)),
            }
        }

        (success_count, failed_files)
    }

    // 准备下载任务（用于异步下载）
    pub fn prepare_download_tasks(
        files: &[CloudFile],
        selected_indices: &[usize],
        base_dir: &std::path::Path,
        local_save_paths: &[(String, PathBuf)],
    ) -> Vec<crate::downloader::DownloadTask> {
        selected_indices
            .iter()
            .filter_map(|&index| {
                files
                    .get(index)
                    .map(|file| crate::downloader::DownloadTask {
                        file: file.clone(),
                        target_path: base_dir.join(&file.name),
                        local_save_paths: local_save_paths.to_vec(),
                    })
            })
            .collect()
    }

    // 选择下载目录
    pub fn pick_download_folder() -> Option<PathBuf> {
        rfd::FileDialog::new().pick_folder()
    }

    // 批量移出云端
    pub fn forget_files(&self, filenames: &[String]) -> (usize, Vec<String>) {
        // 确保云同步已启用（未安装的游戏可能被禁用）
        self.ensure_cloud_enabled();

        let (success_count, failed_files) =
            self.batch_operation(filenames, |filename| self.forget_file(filename));

        if success_count > 0 {
            tracing::info!("移出云端完成，触发云同步...");
            if let Ok(mut manager) = self.steam_manager.lock()
                && let Err(e) = manager.sync_cloud_files()
            {
                tracing::warn!("触发云同步失败: {}", e);
            }
        }

        (success_count, failed_files)
    }

    // 批量删除文件
    pub fn delete_files(&self, filenames: &[String]) -> (usize, Vec<String>) {
        // 确保云同步已启用（未安装的游戏可能被禁用）
        self.ensure_cloud_enabled();

        let (success_count, failed_files) =
            self.batch_operation(filenames, |filename| self.delete_file(filename));

        if success_count > 0 {
            tracing::info!("删除完成，触发云同步...");
            if let Ok(mut manager) = self.steam_manager.lock()
                && let Err(e) = manager.sync_cloud_files()
            {
                tracing::warn!("触发云同步失败: {}", e);
            }
        }

        (success_count, failed_files)
    }

    // 移出云端指定索引的文件
    pub fn forget_by_indices(
        &self,
        files: &[CloudFile],
        selected_files: &[usize],
    ) -> FileOperationResult {
        if selected_files.is_empty() {
            return FileOperationResult::Error("请选择要移出云端的文件".to_string());
        }

        // 分类文件
        let mut api_cloud_files = Vec::new(); // root=0, 云端已同步
        let mut ufs_cloud_files = Vec::new(); // root!=0, 云端已同步
        let mut skipped_local_only = 0;

        for &index in selected_files {
            if let Some(file) = files.get(index) {
                if !file.is_persisted {
                    // 本地独有文件，跳过（云端本来就没有）
                    tracing::debug!("跳过本地独有文件 (云端不存在): {}", file.name);
                    skipped_local_only += 1;
                } else if file.root != 0 {
                    // 自动云同步文件，尝试 API 操作
                    ufs_cloud_files.push(file.name.clone());
                } else {
                    api_cloud_files.push(file.name.clone());
                }
            }
        }

        let has_any_cloud = !api_cloud_files.is_empty() || !ufs_cloud_files.is_empty();
        if !has_any_cloud {
            if skipped_local_only > 0 {
                return FileOperationResult::Error(format!(
                    "所选 {} 个文件仅存在于本地，云端无记录，无需移出",
                    skipped_local_only
                ));
            }
            return FileOperationResult::Error("请选择要移出云端的文件".to_string());
        }

        let mut total_forgotten = 0;
        let mut all_failed = Vec::new();
        let mut ufs_api_failed = 0;

        // root=0 文件：使用 API 移出云端
        if !api_cloud_files.is_empty() {
            tracing::info!("开始移出云端 {} 个 API 域文件", api_cloud_files.len());
            let (forgotten, failed) = self.forget_files(&api_cloud_files);
            total_forgotten += forgotten;
            all_failed.extend(failed);
        }

        // root!=0 文件：尝试 API 移出
        if !ufs_cloud_files.is_empty() {
            tracing::info!(
                "尝试通过 API 移出 {} 个自动云同步文件",
                ufs_cloud_files.len()
            );
            let (forgotten, failed) = self.forget_files(&ufs_cloud_files);
            total_forgotten += forgotten;
            if forgotten > 0 {
                tracing::info!("自动云同步文件 API 移出成功: {} 个", forgotten);
            }
            ufs_api_failed = failed.len();
            if ufs_api_failed > 0 {
                tracing::debug!(
                    "自动云同步文件 API 移出失败: {} 个 (预期行为，请尝试「删除」操作)",
                    ufs_api_failed
                );
            }
        }

        // 构建结果消息
        let mut messages = Vec::new();

        if total_forgotten > 0 {
            messages.push(format!("已移出云端 {} 个文件", total_forgotten));
        }
        if ufs_api_failed > 0 {
            messages.push(format!(
                "{} 个自动云同步文件无法通过 API 移出，请尝试使用「删除」功能",
                ufs_api_failed
            ));
        }
        if !all_failed.is_empty() {
            messages.push(format!(
                "{} 个文件移出失败: {}",
                all_failed.len(),
                all_failed.join(", ")
            ));
        }
        if skipped_local_only > 0 {
            messages.push(format!("跳过 {} 个本地独有文件", skipped_local_only));
        }

        if messages.is_empty() {
            return FileOperationResult::Error("没有文件被移出云端".to_string());
        }

        let msg = messages.join("，");

        if total_forgotten > 0 {
            FileOperationResult::SuccessWithRefresh(msg)
        } else {
            FileOperationResult::Error(msg)
        }
    }

    // 删除指定索引的文件
    pub fn delete_by_indices(
        &self,
        app_id: u32,
        files: &[CloudFile],
        selected_files: &[usize],
        local_save_paths: &[(String, PathBuf)],
    ) -> FileOperationResult {
        if selected_files.is_empty() {
            return FileOperationResult::Error("请选择要删除的文件".to_string());
        }

        // 按 root 类型分类文件
        let mut api_cloud_files = Vec::new(); // root=0, 云端已同步，用 API 删除
        let mut ufs_files = Vec::new(); // root!=0, 自动云同步文件
        let mut local_only_files = Vec::new(); // 仅本地存在、未同步的文件

        for &index in selected_files {
            if let Some(file) = files.get(index) {
                if file.root == 0 {
                    // API 域文件：使用 FileDelete + FileForget
                    if file.is_persisted {
                        api_cloud_files.push(file.name.clone());
                    }
                    if file.exists && !file.is_persisted {
                        local_only_files.push(file.clone());
                    }
                } else {
                    // 自动云同步文件 (root != 0)：需要通过删除本地文件来触发同步
                    ufs_files.push(file.clone());
                }
            }
        }

        let mut total_deleted = 0;
        let mut all_failed = Vec::new();
        let mut ufs_no_local: Vec<String> = Vec::new(); // 自动云同步文件但本地不存在

        // API 域文件 (root=0)：使用 Steam API 删除
        if !api_cloud_files.is_empty() {
            tracing::info!("通过 Steam API 删除 {} 个云端文件", api_cloud_files.len());
            let (deleted_local, failed_local) = self.delete_files(&api_cloud_files);
            let (deleted_cloud, failed_cloud) = self.forget_files(&api_cloud_files);
            total_deleted += deleted_local.max(deleted_cloud);
            // 只收集两个操作都失败的文件
            let failed_local_set: std::collections::HashSet<_> = failed_local.iter().collect();
            let both_failed: Vec<String> = failed_cloud
                .into_iter()
                .filter(|f| failed_local_set.contains(f))
                .collect();
            all_failed.extend(both_failed);
        }

        // 自动云同步文件 (root != 0)：通过删除本地文件触发云同步
        let steam_info = crate::vdf_parser::VdfParser::new().ok().map(|p| {
            (
                p.get_steam_path().to_path_buf(),
                p.get_user_id().to_string(),
            )
        });

        if !ufs_files.is_empty() {
            tracing::info!(
                "处理 {} 个自动云同步文件，将通过删除本地文件触发云同步",
                ufs_files.len()
            );

            for file in &ufs_files {
                // 优先用 local_save_paths（已应用 rootoverrides），再用 resolve_cloud_file_path 兜底
                // 两者不一致时优先用存在的路径
                let local_path = crate::conflict::find_local_path_for_file(file, local_save_paths)
                    .map(|base| base.join(&file.name));

                let resolved_path = if let Some((steam_path, user_id)) = &steam_info {
                    crate::path_resolver::resolve_cloud_file_path(
                        file.root, &file.name, steam_path, user_id, app_id,
                    )
                    .ok()
                } else {
                    None
                };

                let path_to_use = match (&local_path, &resolved_path) {
                    (Some(lp), Some(rp)) if lp != rp => {
                        // rootoverrides 导致路径不一致，优先用存在的
                        if lp.exists() {
                            tracing::debug!(
                                "自动云同步路径不一致，使用 local_save_paths 路径 (存在): {}",
                                lp.display()
                            );
                            Some(lp.clone())
                        } else if rp.exists() {
                            tracing::debug!(
                                "自动云同步路径不一致，使用 resolve 路径 (存在): {}",
                                rp.display()
                            );
                            Some(rp.clone())
                        } else {
                            tracing::warn!(
                                "自动云同步路径均不存在: local_save_paths={}, resolve={}",
                                lp.display(),
                                rp.display()
                            );
                            Some(lp.clone())
                        }
                    }
                    (Some(lp), _) => Some(lp.clone()),
                    (_, Some(rp)) => Some(rp.clone()),
                    _ => None,
                };

                if let Some(path) = path_to_use {
                    if path.exists() {
                        match std::fs::remove_file(&path) {
                            Ok(_) => {
                                tracing::info!(
                                    "已删除自动云同步文件的本地副本 (root={}): {}",
                                    file.root,
                                    path.display()
                                );
                                total_deleted += 1;
                            }
                            Err(e) => {
                                tracing::error!(
                                    "删除自动云同步文件的本地副本失败 (root={}): {} - {}",
                                    file.root,
                                    path.display(),
                                    e
                                );
                                all_failed.push(file.name.clone());
                            }
                        }
                    } else {
                        // 本地文件不存在，交由 API 处理
                        tracing::warn!(
                            "自动云同步文件的本地副本不存在 (root={}, 路径={})，尝试 API 删除: {}",
                            file.root,
                            path.display(),
                            file.name
                        );
                        ufs_no_local.push(file.name.clone());
                    }
                } else {
                    // 无法解析本地路径，交由 API 处理
                    tracing::warn!(
                        "无法解析自动云同步文件的本地路径 (root={})，尝试 API 删除: {}",
                        file.root,
                        file.name
                    );
                    ufs_no_local.push(file.name.clone());
                }
            }
        }

        // 无本地副本的 UFS 文件：尝试 API 删除（FileDelete + FileForget）
        let mut ufs_api_success = 0;
        let mut ufs_failed = Vec::new();
        if !ufs_no_local.is_empty() {
            tracing::info!(
                "尝试通过 API 删除 {} 个无本地副本的自动云同步文件",
                ufs_no_local.len()
            );

            let (deleted_count, delete_failed) = self.delete_files(&ufs_no_local);
            let (forgotten_count, _) = self.forget_files(&ufs_no_local);

            ufs_api_success = deleted_count.max(forgotten_count);
            total_deleted += ufs_api_success;

            if ufs_api_success > 0 {
                tracing::info!(
                    "API 删除自动云同步文件成功: {} 个 (delete={}, forget={})",
                    ufs_api_success,
                    deleted_count,
                    forgotten_count
                );
            }

            // API 也失败的文件
            let api_success_names: std::collections::HashSet<_> = ufs_no_local
                .iter()
                .filter(|name| !delete_failed.contains(name))
                .cloned()
                .collect();
            for name in &ufs_no_local {
                if !api_success_names.contains(name) {
                    ufs_failed.push(name.clone());
                }
            }

            if !ufs_failed.is_empty() {
                tracing::warn!(
                    "API 也无法删除 {} 个自动云同步文件（游戏未安装，请安装游戏后重试）",
                    ufs_failed.len()
                );
            }
        }

        // 仅本地存在的 root=0 文件：直接删除本地文件
        for file in &local_only_files {
            let full_path = if let Some((steam_path, user_id)) = &steam_info {
                crate::path_resolver::resolve_cloud_file_path(
                    file.root, &file.name, steam_path, user_id, app_id,
                )
                .ok()
            } else {
                None
            };

            let path_to_use = full_path.or_else(|| {
                crate::conflict::find_local_path_for_file(file, local_save_paths)
                    .map(|base| base.join(&file.name))
            });

            if let Some(path) = path_to_use {
                if path.exists() {
                    match std::fs::remove_file(&path) {
                        Ok(_) => {
                            tracing::info!("已删除本地文件: {}", path.display());
                            total_deleted += 1;
                        }
                        Err(e) => {
                            tracing::error!("删除本地文件失败: {} - {}", path.display(), e);
                            all_failed.push(file.name.clone());
                        }
                    }
                } else {
                    total_deleted += 1;
                }
            } else {
                all_failed.push(file.name.clone());
            }
        }

        // 自动云同步文件删除后触发云同步
        let ufs_local_deleted = ufs_files.len()
            - ufs_no_local.len()
            - all_failed
                .iter()
                .filter(|f| ufs_files.iter().any(|u| &u.name == *f))
                .count();
        if ufs_local_deleted > 0 || ufs_api_success > 0 {
            tracing::info!(
                "触发云同步 (本地删除={}, API删除={})...",
                ufs_local_deleted,
                ufs_api_success
            );
            if let Ok(mut manager) = self.steam_manager.lock()
                && let Err(e) = manager.sync_cloud_files()
            {
                tracing::warn!("触发云同步失败: {}", e);
            }
        }

        // 构建结果消息
        let mut messages = Vec::new();

        if total_deleted > 0 {
            messages.push(format!("已删除 {} 个文件", total_deleted));
        }
        // 通过删除本地文件触发云端同步删除，提醒等待
        if ufs_local_deleted > 0 {
            messages.push(
                "自动云同步文件的云端副本将在 Steam 同步后自动删除，请稍后刷新确认".to_string(),
            );
        }
        if !ufs_failed.is_empty() {
            messages.push(format!(
                "{} 个自动云同步文件无法删除（游戏未安装且 API 不支持，请安装游戏后重试）",
                ufs_failed.len()
            ));
        }
        if !all_failed.is_empty() {
            messages.push(format!("{} 个文件删除失败", all_failed.len()));
        }

        if messages.is_empty() {
            return FileOperationResult::Error("没有文件被删除".to_string());
        }

        let msg = messages.join("，");

        if total_deleted > 0 {
            FileOperationResult::SuccessWithRefresh(msg)
        } else {
            FileOperationResult::Error(msg)
        }
    }

    // 同步本地文件到云端（针对本地独有文件）
    pub fn sync_to_cloud_by_indices(
        &self,
        app_id: u32,
        files: &[CloudFile],
        selected_files: &[usize],
        local_save_paths: &[(String, PathBuf)],
    ) -> FileOperationResult {
        if selected_files.is_empty() {
            return FileOperationResult::Error("请选择要同步的文件".to_string());
        }

        let mut synced_count = 0;
        let mut skipped_count = 0;
        let mut failed_files = Vec::new();

        let steam_info = crate::vdf_parser::VdfParser::new().ok().map(|p| {
            (
                p.get_steam_path().to_path_buf(),
                p.get_user_id().to_string(),
            )
        });

        for &index in selected_files {
            if let Some(file) = files.get(index) {
                if file.is_persisted {
                    skipped_count += 1;
                    continue;
                }

                let full_path = if let Some((steam_path, user_id)) = &steam_info {
                    crate::path_resolver::resolve_cloud_file_path(
                        file.root, &file.name, steam_path, user_id, app_id,
                    )
                    .ok()
                } else {
                    None
                };

                let path_to_use = full_path.or_else(|| {
                    crate::conflict::find_local_path_for_file(file, local_save_paths)
                        .map(|base| base.join(&file.name))
                });

                if let Some(path) = path_to_use {
                    if path.exists() {
                        match std::fs::read(&path) {
                            Ok(data) => match self.write_file(&file.name, &data) {
                                Ok(_) => {
                                    tracing::info!("同步文件到云端: {}", file.name);
                                    synced_count += 1;
                                }
                                Err(e) => {
                                    tracing::error!("同步失败: {} - {}", file.name, e);
                                    failed_files.push(file.name.clone());
                                }
                            },
                            Err(e) => {
                                tracing::error!("读取文件失败: {} - {}", path.display(), e);
                                failed_files.push(file.name.clone());
                            }
                        }
                    } else {
                        failed_files.push(file.name.clone());
                    }
                } else {
                    failed_files.push(file.name.clone());
                }
            }
        }

        if !failed_files.is_empty() {
            return FileOperationResult::Error(format!(
                "部分文件同步失败: {}",
                failed_files.join(", ")
            ));
        }

        if synced_count > 0 {
            if let Ok(mut manager) = self.steam_manager.lock()
                && let Err(e) = manager.sync_cloud_files()
            {
                tracing::warn!("触发云同步失败: {}", e);
            }
            FileOperationResult::SuccessWithRefresh(format!("已同步 {} 个文件到云端", synced_count))
        } else if skipped_count > 0 {
            FileOperationResult::Error(format!("所有 {} 个文件已在云端，无需同步", skipped_count))
        } else {
            FileOperationResult::Error("没有文件被同步".to_string())
        }
    }
}

// 文件操作结果
pub enum FileOperationResult {
    Error(String),              // 错误消息
    SuccessWithRefresh(String), // 成功消息 + 需要刷新
}

use std::path::Path;
use std::time::{Duration, SystemTime};
use walkdir::WalkDir;

// 常量定义
pub const MAX_FILE_SIZE: u64 = 100 * 1024 * 1024; // 100 MB

// 上传任务
#[derive(Debug, Clone)]
pub struct UploadTask {
    pub local_path: PathBuf,
    pub cloud_path: String,
    pub size: u64,
    pub status: TaskStatus,
    pub retry_count: usize,
    pub error: Option<String>,
}

// 任务状态
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TaskStatus {
    Pending,         // 等待中
    Retrying(usize), // 重试中 (第N次)
    Success,         // 成功
    Failed,          // 失败
}

// 上传队列
pub struct UploadQueue {
    pub tasks: Vec<UploadTask>,
    pub virtual_root: Option<String>,
}

impl UploadQueue {
    pub fn new() -> Self {
        Self {
            tasks: Vec::new(),
            virtual_root: None,
        }
    }

    // 添加单个文件
    pub fn add_file(&mut self, local_path: PathBuf) -> Result<()> {
        let size = std::fs::metadata(&local_path)?.len();

        // 验证文件大小
        if size > MAX_FILE_SIZE {
            return Err(anyhow!(
                "文件 {} 超过 100MB 限制 (实际: {})",
                local_path.display(),
                format_size(size)
            ));
        }

        let filename = local_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow!("无法获取文件名"))?;

        let cloud_path = if let Some(ref root) = self.virtual_root {
            format!("{}/{}", root, filename)
        } else {
            filename.to_string()
        };

        self.tasks.push(UploadTask {
            local_path,
            cloud_path,
            size,
            status: TaskStatus::Pending,
            retry_count: 0,
            error: None,
        });

        Ok(())
    }

    // 添加文件夹（递归）
    pub fn add_folder(&mut self, folder_path: &Path) -> Result<()> {
        let folder_name = folder_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow!("无法获取文件夹名"))?;

        for entry in WalkDir::new(folder_path) {
            let entry = entry?;
            if entry.file_type().is_file() {
                let file_path = entry.path();
                let size = entry.metadata()?.len();

                // 跳过超大文件，但记录
                if size > MAX_FILE_SIZE {
                    tracing::warn!(
                        "跳过超大文件: {} ({})",
                        file_path.display(),
                        format_size(size)
                    );
                    continue;
                }

                // 计算相对路径
                let relative_path = file_path
                    .strip_prefix(folder_path)
                    .map_err(|e| anyhow!("无法计算相对路径: {}", e))?;

                let relative_path_str = relative_path
                    .to_str()
                    .ok_or_else(|| anyhow!("路径包含非 UTF-8 字符: {:?}", relative_path))?
                    .replace("\\", "/");

                // 构建云端路径：[virtual_root]/[folder_name]/[relative_path]
                // 注意：如果 relative_path 为空（即直接是文件夹本身，虽然 WalkDir 这里是文件所以不会空，但以防万一），
                // 或者 relative_path 只是文件名，这里逻辑是一样的。
                // 例如：folder=qwe, file=qwe/233 -> relative=233 -> cloud=qwe/233

                let folder_relative_path = if relative_path_str.is_empty() {
                    folder_name.to_string()
                } else {
                    format!("{}/{}", folder_name, relative_path_str)
                };

                let cloud_path = if let Some(ref root) = self.virtual_root {
                    format!("{}/{}", root, folder_relative_path)
                } else {
                    folder_relative_path
                };

                self.tasks.push(UploadTask {
                    local_path: file_path.to_path_buf(),
                    cloud_path,
                    size,
                    status: TaskStatus::Pending,
                    retry_count: 0,
                    error: None,
                });
            }
        }

        Ok(())
    }

    pub fn total_size(&self) -> u64 {
        self.tasks.iter().map(|t| t.size).sum()
    }

    pub fn total_files(&self) -> usize {
        self.tasks.len()
    }
}

// 打开文件夹
pub fn open_folder(path: &std::path::Path) {
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("explorer").arg(path).spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(path).spawn();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open").arg(path).spawn();
    }
}

// 上传重试配置
pub struct UploadRetryConfig {
    pub max_retries: usize,
    pub retry_delay: Duration,
    pub backoff_multiplier: f32,
}

impl Default for UploadRetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            retry_delay: Duration::from_secs(1),
            backoff_multiplier: 1.5,
        }
    }
}

// 进度回调类型
pub type ProgressCallback = Box<dyn Fn(usize, usize, &str) + Send>;

// 上传执行器
pub struct UploadExecutor {
    steam_manager: Arc<Mutex<crate::steam_worker::SteamWorkerManager>>,
    retry_config: UploadRetryConfig,
    progress_callback: Option<ProgressCallback>,
}

impl UploadExecutor {
    pub fn new(steam_manager: Arc<Mutex<crate::steam_worker::SteamWorkerManager>>) -> Self {
        Self {
            steam_manager,
            retry_config: UploadRetryConfig::default(),
            progress_callback: None,
        }
    }

    pub fn with_progress_callback<F>(mut self, callback: F) -> Self
    where
        F: Fn(usize, usize, &str) + Send + 'static,
    {
        self.progress_callback = Some(Box::new(callback));
        self
    }

    // 执行上传
    pub fn execute(&self, queue: &mut UploadQueue) -> Result<UploadResult> {
        let start_time = SystemTime::now();
        let mut success_count = 0;
        let mut failed_files = Vec::new();
        let total_size: u64 = queue.tasks.iter().map(|t| t.size).sum();
        let total_files = queue.tasks.len();

        for (index, task) in queue.tasks.iter_mut().enumerate() {
            // 发送进度更新
            if let Some(ref callback) = self.progress_callback {
                callback(index + 1, total_files, &task.cloud_path);
            }

            match self.upload_task_with_retry(task) {
                Ok(_) => {
                    task.status = TaskStatus::Success;
                    success_count += 1;
                }
                Err(e) => {
                    task.status = TaskStatus::Failed;
                    task.error = Some(e.to_string());
                    failed_files.push((task.cloud_path.clone(), e.to_string()));
                }
            }
        }

        let elapsed = start_time.elapsed().unwrap_or(Duration::from_secs(0));

        // 上传完成后，触发云同步
        if success_count > 0 {
            tracing::info!("上传完成，触发云同步...");
            if let Err(e) = self.sync_cloud_files() {
                tracing::warn!("触发云同步失败: {}", e);
            }
        }

        Ok(UploadResult {
            success_count,
            failed_count: failed_files.len(),
            total_size,
            elapsed_secs: elapsed.as_secs(),
            failed_files,
        })
    }

    // 上传单个任务
    fn upload_task_with_retry(&self, task: &mut UploadTask) -> Result<()> {
        let mut attempt = 0;
        let mut delay = self.retry_config.retry_delay;

        loop {
            attempt += 1;
            task.retry_count = attempt;

            // 读取文件数据
            let data = std::fs::read(&task.local_path)?;

            // 尝试上传
            match self.upload_to_steam(&task.cloud_path, &data) {
                Ok(_) => {
                    tracing::info!("文件上传成功: {} (尝试 {})", task.cloud_path, attempt);
                    return Ok(());
                }
                Err(e) => {
                    if attempt >= self.retry_config.max_retries {
                        tracing::error!(
                            "文件上传失败: {} (已重试 {} 次): {}",
                            task.cloud_path,
                            self.retry_config.max_retries,
                            e
                        );
                        return Err(e);
                    }

                    tracing::warn!(
                        "文件上传失败: {} (尝试 {}/{}): {}，{}秒后重试...",
                        task.cloud_path,
                        attempt,
                        self.retry_config.max_retries,
                        e,
                        delay.as_secs()
                    );

                    task.status = TaskStatus::Retrying(attempt);
                    std::thread::sleep(delay);
                    delay = Duration::from_secs_f32(
                        delay.as_secs_f32() * self.retry_config.backoff_multiplier,
                    );
                }
            }
        }
    }

    // 上传到 Steam
    fn upload_to_steam(&self, cloud_path: &str, data: &[u8]) -> Result<()> {
        let mut manager = self
            .steam_manager
            .lock()
            .map_err(|e| anyhow!("Steam 管理器锁错误: {}", e))?;

        manager.write_file(cloud_path, data)?;
        Ok(())
    }

    // 触发云同步
    fn sync_cloud_files(&self) -> Result<()> {
        let mut manager = self
            .steam_manager
            .lock()
            .map_err(|e| anyhow!("Steam 管理器锁错误: {}", e))?;

        manager.sync_cloud_files()?;
        Ok(())
    }
}

// 上传结果
#[derive(Debug, Clone)]
pub struct UploadResult {
    pub success_count: usize,
    pub failed_count: usize,
    pub total_size: u64,
    pub elapsed_secs: u64,
    pub failed_files: Vec<(String, String)>,
}

// 格式化文件大小
pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

// 解析大小字符串为字节数（如 "1.5 MB" -> 1572864）
pub fn parse_size(s: &str) -> u64 {
    let s = s.replace(",", "").to_lowercase();
    let s = s.replace("\u{a0}", " ");
    let parts: Vec<&str> = s.split_whitespace().collect();

    if parts.is_empty() {
        return 0;
    }

    let num = parts[0].parse::<f64>().unwrap_or(0.0);
    if parts.len() > 1 {
        match parts[1] {
            "kb" | "k" => (num * 1024.0) as u64,
            "mb" | "m" => (num * 1024.0 * 1024.0) as u64,
            "gb" | "g" => (num * 1024.0 * 1024.0 * 1024.0) as u64,
            "b" | "bytes" => num as u64,
            _ => num as u64,
        }
    } else {
        num as u64
    }
}
