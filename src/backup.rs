use crate::downloader::download_single_file;
use crate::path_resolver::get_root_type_name;
use crate::steam_api::CloudFile;
use anyhow::{Result, anyhow};
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

// 备份清单文件格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupManifest {
    pub version: u32,
    pub app_id: u32,
    pub game_name: String,
    pub backup_time: String, // ISO 8601 格式
    pub total_files: usize,
    pub total_size: u64,
    pub files: Vec<BackupFileEntry>,
    pub roots: Vec<RootInfo>,
}

// 备份文件条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupFileEntry {
    pub name: String,
    pub size: u64,
    pub sha1: Option<String>,
    pub root_index: u32,
    pub root_name: String,
    pub relative_path: String,
}

// Root 信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootInfo {
    pub index: u32,
    pub name: String,
    pub folder: String,
}

// 备份进度
#[derive(Debug, Clone)]
pub struct BackupProgress {
    pub total_files: usize,
    pub completed_files: usize,
    pub current_file: String,
    pub failed_files: Vec<(String, String)>,
}

impl BackupProgress {
    pub fn new(total_files: usize) -> Self {
        Self {
            total_files,
            completed_files: 0,
            current_file: String::new(),
            failed_files: Vec::new(),
        }
    }

    pub fn percent(&self) -> f32 {
        if self.total_files == 0 {
            return 100.0;
        }
        (self.completed_files as f32 / self.total_files as f32) * 100.0
    }
}

// 备份结果
#[derive(Debug, Clone)]
pub struct BackupResult {
    pub success: bool,
    pub backup_path: PathBuf,
    pub total_files: usize,
    pub success_count: usize,
    pub failed_files: Vec<(String, String)>,
}

// 获取备份根目录
pub fn get_backup_root_dir() -> Result<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        // Windows: exe 目录下的 backups/
        let exe_path = std::env::current_exe()?;
        let exe_dir = exe_path
            .parent()
            .ok_or_else(|| anyhow!("无法获取 exe 目录"))?;
        Ok(exe_dir.join("backups"))
    }

    #[cfg(target_os = "macos")]
    {
        // macOS: ~/Library/Application Support/SteamCloudFileManager/backups/
        let home = std::env::var("HOME")?;
        Ok(PathBuf::from(home)
            .join("Library")
            .join("Application Support")
            .join("SteamCloudFileManager")
            .join("backups"))
    }

    #[cfg(target_os = "linux")]
    {
        // Linux: ~/.local/share/SteamCloudFileManager/backups/
        let home = std::env::var("HOME")?;
        Ok(PathBuf::from(home)
            .join(".local")
            .join("share")
            .join("SteamCloudFileManager")
            .join("backups"))
    }
}

// 生成备份目录名称
pub fn generate_backup_dir_name(game_name: &str, app_id: u32) -> String {
    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    // 清理游戏名（移除特殊字符）
    let clean_name: String = game_name
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '-' || *c == '_')
        .take(30)
        .collect();
    let clean_name = clean_name.trim().replace(' ', "_");

    format!("{}_{}_{}", clean_name, app_id, timestamp)
}

// 获取 Root 文件夹名称
pub fn get_root_folder_name(root: u32) -> String {
    let type_name = get_root_type_name(root);
    format!("root{}_{}", root, type_name)
}

// 备份管理器
pub struct BackupManager {
    backup_root: PathBuf,
}

impl BackupManager {
    pub fn new() -> Result<Self> {
        let backup_root = get_backup_root_dir()?;
        Ok(Self { backup_root })
    }

    // 创建备份
    pub fn create_backup(
        &self,
        app_id: u32,
        game_name: &str,
        files: &[CloudFile],
        cancel_flag: Arc<AtomicBool>,
        progress_callback: impl Fn(&BackupProgress),
    ) -> Result<BackupResult> {
        if files.is_empty() {
            return Err(anyhow!("没有文件需要备份"));
        }

        // 创建备份目录
        let backup_dir_name = generate_backup_dir_name(game_name, app_id);
        let backup_path = self.backup_root.join(&backup_dir_name);
        std::fs::create_dir_all(&backup_path)?;

        tracing::info!("开始备份到: {}", backup_path.display());

        let mut progress = BackupProgress::new(files.len());
        let mut manifest_files = Vec::new();
        let mut roots_map: HashMap<u32, RootInfo> = HashMap::new();

        for file in files {
            // 检查取消
            if cancel_flag.load(Ordering::Relaxed) {
                tracing::info!("备份已取消");
                break;
            }

            progress.current_file = file.name.clone();
            progress_callback(&progress);

            // 获取 root 文件夹
            let root_folder = get_root_folder_name(file.root);

            // 记录 root 信息
            roots_map.entry(file.root).or_insert_with(|| RootInfo {
                index: file.root,
                name: get_root_type_name(file.root).to_string(),
                folder: root_folder.clone(),
            });

            // 创建目标路径
            let target_dir = backup_path.join(&root_folder);
            let target_path = target_dir.join(&file.name);

            // 创建父目录
            if let Some(parent) = target_path.parent()
                && let Err(e) = std::fs::create_dir_all(parent)
            {
                let err_msg = format!("创建目录失败: {}", e);
                tracing::warn!("{}: {}", file.name, err_msg);
                progress.failed_files.push((file.name.clone(), err_msg));
                progress.completed_files += 1;
                continue;
            }

            // 下载文件（使用共享下载模块）
            match download_single_file(file, &target_path) {
                Ok(_) => {
                    tracing::debug!("备份成功: {}", file.name);
                    manifest_files.push(BackupFileEntry {
                        name: file.name.clone(),
                        size: file.size,
                        sha1: None,
                        root_index: file.root,
                        root_name: get_root_type_name(file.root).to_string(),
                        relative_path: file.name.clone(),
                    });
                }
                Err(e) => {
                    let err_msg = e.to_string();
                    tracing::warn!("备份失败: {} - {}", file.name, err_msg);
                    progress.failed_files.push((file.name.clone(), err_msg));
                }
            }

            progress.completed_files += 1;
            progress_callback(&progress);
        }

        // 生成 manifest.json
        let manifest = BackupManifest {
            version: 1,
            app_id,
            game_name: game_name.to_string(),
            backup_time: Local::now().to_rfc3339(),
            total_files: manifest_files.len(),
            total_size: manifest_files.iter().map(|f| f.size).sum(),
            files: manifest_files,
            roots: roots_map.into_values().collect(),
        };

        let manifest_path = backup_path.join("manifest.json");
        let manifest_json = serde_json::to_string_pretty(&manifest)?;
        std::fs::write(&manifest_path, manifest_json)?;

        tracing::info!(
            "备份完成: {} 个文件成功, {} 个失败",
            manifest.total_files,
            progress.failed_files.len()
        );

        let cancelled = cancel_flag.load(Ordering::Relaxed);
        Ok(BackupResult {
            success: progress.failed_files.is_empty() && !cancelled,
            backup_path,
            total_files: files.len(),
            success_count: manifest.total_files,
            failed_files: progress.failed_files,
        })
    }

    // 打开备份目录
    pub fn open_backup_dir(&self) -> Result<()> {
        if !self.backup_root.exists() {
            std::fs::create_dir_all(&self.backup_root)?;
        }

        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("open")
                .arg(&self.backup_root)
                .spawn()?;
        }

        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("explorer")
                .arg(&self.backup_root)
                .spawn()?;
        }

        #[cfg(target_os = "linux")]
        {
            std::process::Command::new("xdg-open")
                .arg(&self.backup_root)
                .spawn()?;
        }

        Ok(())
    }
}
