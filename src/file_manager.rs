use crate::path_resolver::{resolve_cloud_file_path, RootType};
use crate::steam_api::CloudFile;
use crate::ui::panels::{SortColumn, SortOrder};
use crate::vdf_parser::{VdfFileEntry, VdfParser};
use anyhow::{anyhow, Result};
use chrono::{Local, TimeZone};
use std::path::PathBuf;
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

        let parser = VdfParser::new()?;
        let vdf_entries = parser.parse_remotecache(app_id)?;

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

        tracing::info!("尝试通过 CDP 补充文件信息");

        if let Ok(mut client) = crate::cdp_client::CdpClient::connect() {
            if let Ok(cdp_files) = client.fetch_game_files(app_id) {
                tracing::info!(count = cdp_files.len(), "CDP 返回文件");

                let file_map: std::collections::HashMap<String, usize> = files
                    .iter()
                    .enumerate()
                    .map(|(i, f)| (f.name.clone(), i))
                    .collect();

                for cdp_file in cdp_files {
                    if let Some(&idx) = file_map.get(&cdp_file.name) {
                        let f = &mut files[idx];
                        f.size = cdp_file.size;
                        f.timestamp = cdp_file.timestamp;
                        f.is_persisted = true;
                        if cdp_file.root_description.starts_with("CDP:") {
                            f.root_description = cdp_file.root_description;
                        }
                    } else {
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

// 文件排序器
pub struct FileSorter {
    sort_column: Option<SortColumn>,
    sort_order: SortOrder,
}

impl Default for FileSorter {
    fn default() -> Self {
        Self {
            sort_column: None,
            sort_order: SortOrder::None,
        }
    }
}

impl FileSorter {
    pub fn new() -> Self {
        Self::default()
    }

    // 获取当前排序列
    pub fn sort_column(&self) -> Option<SortColumn> {
        self.sort_column
    }

    // 获取当前排序顺序
    pub fn sort_order(&self) -> SortOrder {
        self.sort_order
    }

    // 返回 true 表示需要刷新文件列表（无序状态）
    pub fn toggle_sort(&mut self, column: SortColumn) -> bool {
        if self.sort_column == Some(column) {
            // 同一列：升序 -> 降序 -> 无序
            self.sort_order = match self.sort_order {
                SortOrder::Ascending => SortOrder::Descending,
                SortOrder::Descending => SortOrder::None,
                SortOrder::None => SortOrder::Ascending,
            };
        } else {
            // 新列：直接升序
            self.sort_column = Some(column);
            self.sort_order = SortOrder::Ascending;
        }

        // 如果是无序状态，清除排序列
        if self.sort_order == SortOrder::None {
            self.sort_column = None;
            return true; // 需要重新加载
        }

        false // 不需要重新加载
    }

    // 对文件列表进行排序
    pub fn sort_files(&self, files: &mut [CloudFile]) {
        if self.sort_order == SortOrder::None || self.sort_column.is_none() {
            return;
        }

        let column = self.sort_column.unwrap();
        let order = self.sort_order;

        files.sort_by(|a, b| {
            let result = match column {
                SortColumn::Name => a.name.cmp(&b.name),
                SortColumn::Size => a.size.cmp(&b.size),
                SortColumn::Time => a.timestamp.cmp(&b.timestamp),
            };
            match order {
                SortOrder::Ascending => result,
                SortOrder::Descending => result.reverse(),
                SortOrder::None => std::cmp::Ordering::Equal,
            }
        });
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

// 文件操作结构体
pub struct FileOperations {
    steam_manager: Arc<Mutex<crate::steam_api::SteamCloudManager>>,
}

impl FileOperations {
    pub fn new(steam_manager: Arc<Mutex<crate::steam_api::SteamCloudManager>>) -> Self {
        Self { steam_manager }
    }

    // 下载文件到指定路径
    pub fn download_file(&self, file: &CloudFile, path: &PathBuf) -> Result<()> {
        let url_prefix = "CDP:";
        if file.root_description.starts_with(url_prefix) {
            // 从 CDP 下载
            let content = &file.root_description[url_prefix.len()..];
            let url = content.split('|').next().unwrap_or("");

            tracing::info!("正在从 CDP 下载: {}", url);

            let resp = ureq::get(url)
                .call()
                .map_err(|e| anyhow!("HTTP 下载失败: {}", e))?;

            let mut reader = resp.into_body().into_reader();
            let mut data = Vec::new();
            std::io::Read::read_to_end(&mut reader, &mut data)
                .map_err(|e| anyhow!("读取响应流失败: {}", e))?;

            self.save_data_to_path(&data, path)?;
        } else {
            // 从 Steam API 下载
            let manager = self
                .steam_manager
                .lock()
                .map_err(|e| anyhow!("Steam 管理器锁错误: {}", e))?;

            let data = manager
                .read_file(&file.name)
                .map_err(|e| anyhow!("API 下载失败 (可能文件未缓存): {}", e))?;

            self.save_data_to_path(&data, path)?;
        }

        Ok(())
    }

    // 上传文件
    pub fn upload_file(&self, filename: &str, data: &[u8]) -> Result<bool> {
        let manager = self
            .steam_manager
            .lock()
            .map_err(|e| anyhow!("Steam 管理器锁错误: {}", e))?;

        manager
            .write_file(filename, data)
            .map_err(|e| anyhow!("上传文件失败: {}", e))
    }

    // 删除文件
    pub fn delete_file(&self, filename: &str) -> Result<bool> {
        let manager = self
            .steam_manager
            .lock()
            .map_err(|e| anyhow!("Steam 管理器锁错误: {}", e))?;

        manager
            .delete_file(filename)
            .map_err(|e| anyhow!("删除文件失败: {}", e))
    }

    // 取消云同步
    pub fn forget_file(&self, filename: &str) -> anyhow::Result<bool> {
        let manager = self
            .steam_manager
            .lock()
            .map_err(|e| anyhow!("Steam 管理器锁错误: {}", e))?;
        manager.forget_file(filename)
    }

    // 保存数据到路径
    fn save_data_to_path(&self, data: &[u8], path: &PathBuf) -> Result<()> {
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent).map_err(|e| anyhow!("创建目录失败: {}", e))?;
            }
        }

        std::fs::write(path, data).map_err(|e| anyhow!("保存文件失败: {}", e))?;

        tracing::info!("文件已保存: {}", path.display());
        Ok(())
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
                Ok(false) => failed_files.push(filename.to_string()),
                Err(e) => failed_files.push(format!("{} (错误: {})", filename, e)),
            }
        }

        (success_count, failed_files)
    }
}

// 下载选中的文件
pub fn download_file_with_dialog(
    file: &CloudFile,
    steam_manager: std::sync::Arc<std::sync::Mutex<crate::steam_api::SteamCloudManager>>,
) -> Result<Option<std::path::PathBuf>> {
    use rfd::FileDialog;

    if let Some(path) = FileDialog::new().set_file_name(&file.name).save_file() {
        let file_ops = FileOperations::new(steam_manager);
        file_ops.download_file(file, &path)?;
        Ok(Some(path))
    } else {
        Ok(None)
    }
}

// 上传文件
pub fn upload_file_with_dialog(
    steam_manager: std::sync::Arc<std::sync::Mutex<crate::steam_api::SteamCloudManager>>,
) -> Result<Option<String>> {
    use rfd::FileDialog;

    if let Some(path) = FileDialog::new().add_filter("所有文件", &["*"]).pick_file() {
        let data = std::fs::read(&path).map_err(|e| anyhow!("读取文件失败: {}", e))?;

        let filename = path
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.replace('\\', "/"))
            .unwrap_or("unknown_file".to_string());

        let file_ops = FileOperations::new(steam_manager);
        file_ops.upload_file(&filename, &data)?;
        Ok(Some(filename))
    } else {
        Ok(None)
    }
}

// 从路径上传文件
pub fn upload_file_from_path(
    path: &std::path::Path,
    steam_manager: std::sync::Arc<std::sync::Mutex<crate::steam_api::SteamCloudManager>>,
) -> Result<String> {
    if !path.is_file() {
        return Err(anyhow!("只能上传文件"));
    }

    let filename = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| anyhow!("无法获取文件名"))?
        .to_string();

    let data = std::fs::read(path).map_err(|e| anyhow!("读取文件失败: {}", e))?;

    let file_ops = FileOperations::new(steam_manager);
    file_ops.upload_file(&filename, &data)?;
    Ok(filename)
}

// 批量取消云同步
pub fn batch_forget_files(
    filenames: &[String],
    steam_manager: std::sync::Arc<std::sync::Mutex<crate::steam_api::SteamCloudManager>>,
) -> (usize, Vec<String>) {
    let file_ops = FileOperations::new(steam_manager);
    file_ops.batch_operation(filenames, |filename| file_ops.forget_file(filename))
}

// 批量删除文件
pub fn batch_delete_files(
    filenames: &[String],
    steam_manager: std::sync::Arc<std::sync::Mutex<crate::steam_api::SteamCloudManager>>,
) -> (usize, Vec<String>) {
    let file_ops = FileOperations::new(steam_manager);
    file_ops.batch_operation(filenames, |filename| file_ops.delete_file(filename))
}

// 文件操作结果
pub enum FileOperationResult {
    Success(String),            // 成功消息
    Error(String),              // 错误消息
    SuccessWithRefresh(String), // 成功消息 + 需要刷新
}

// 下载选中的文件
pub fn download_selected_file_coordinated(
    files: &[CloudFile],
    selected_files: &[usize],
    steam_manager: std::sync::Arc<std::sync::Mutex<crate::steam_api::SteamCloudManager>>,
) -> FileOperationResult {
    // 验证选择
    if selected_files.is_empty() {
        return FileOperationResult::Error("请选择要下载的文件".to_string());
    }

    if selected_files.len() != 1 {
        return FileOperationResult::Error("请只选择一个文件".to_string());
    }

    let file_index = selected_files[0];
    if file_index >= files.len() {
        return FileOperationResult::Error("文件索引无效".to_string());
    }

    let file = &files[file_index];

    // 执行下载
    match download_file_with_dialog(file, steam_manager) {
        Ok(Some(path)) => FileOperationResult::Success(format!("文件已下载: {}", path.display())),
        Ok(None) => FileOperationResult::Success("取消下载".to_string()),
        Err(e) => FileOperationResult::Error(e.to_string()),
    }
}

// 上传文件
pub fn upload_file_coordinated(
    is_connected: bool,
    steam_manager: std::sync::Arc<std::sync::Mutex<crate::steam_api::SteamCloudManager>>,
) -> FileOperationResult {
    if !is_connected {
        return FileOperationResult::Error("未连接到Steam".to_string());
    }

    match upload_file_with_dialog(steam_manager) {
        Ok(Some(filename)) => {
            FileOperationResult::SuccessWithRefresh(format!("文件已上传: {}", filename))
        }
        Ok(None) => FileOperationResult::Success("取消上传".to_string()),
        Err(e) => FileOperationResult::Error(e.to_string()),
    }
}

// 从路径上传文件
pub fn upload_file_from_path_coordinated(
    path: &std::path::Path,
    steam_manager: std::sync::Arc<std::sync::Mutex<crate::steam_api::SteamCloudManager>>,
) -> FileOperationResult {
    match upload_file_from_path(path, steam_manager) {
        Ok(filename) => FileOperationResult::SuccessWithRefresh(format!("上传成功: {}", filename)),
        Err(e) => FileOperationResult::Error(e.to_string()),
    }
}

// 取消云同步选中的文件
pub fn forget_selected_files_coordinated(
    files: &[CloudFile],
    selected_files: &[usize],
    steam_manager: std::sync::Arc<std::sync::Mutex<crate::steam_api::SteamCloudManager>>,
) -> FileOperationResult {
    if selected_files.is_empty() {
        return FileOperationResult::Error("请选择要取消云同步的文件".to_string());
    }

    let filenames: Vec<String> = selected_files
        .iter()
        .filter_map(|&index| files.get(index).map(|f| f.name.clone()))
        .collect();

    let (forgotten_count, failed_files) = batch_forget_files(&filenames, steam_manager);

    if !failed_files.is_empty() {
        return FileOperationResult::Error(format!(
            "部分文件取消云同步失败: {}",
            failed_files.join(", ")
        ));
    }

    if forgotten_count > 0 {
        FileOperationResult::SuccessWithRefresh(format!("已取消云同步 {} 个文件", forgotten_count))
    } else {
        FileOperationResult::Error("没有文件被取消云同步".to_string())
    }
}

// 删除选中的文件
pub fn delete_selected_files_coordinated(
    files: &[CloudFile],
    selected_files: &[usize],
    steam_manager: std::sync::Arc<std::sync::Mutex<crate::steam_api::SteamCloudManager>>,
) -> FileOperationResult {
    if selected_files.is_empty() {
        return FileOperationResult::Error("请选择要删除的文件".to_string());
    }

    let filenames: Vec<String> = selected_files
        .iter()
        .filter_map(|&index| files.get(index).map(|f| f.name.clone()))
        .collect();

    let (deleted_count, failed_files) = batch_delete_files(&filenames, steam_manager);

    if !failed_files.is_empty() {
        return FileOperationResult::Error(format!(
            "部分文件删除失败: {}",
            failed_files.join(", ")
        ));
    }

    if deleted_count > 0 {
        FileOperationResult::SuccessWithRefresh(format!("已删除 {} 个文件", deleted_count))
    } else {
        FileOperationResult::Error("没有文件被删除".to_string())
    }
}
