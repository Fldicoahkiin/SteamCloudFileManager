use crate::conflict::{AsyncHashChecker, DiffFlags, HashStatus, SyncStatus};
use crate::game_scanner::CloudGameInfo;
use crate::i18n::{I18n, Language};
use crate::steam_api::CloudFile;
use crate::vdf_parser::UserInfo;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

// 连接状态
#[derive(Default)]
pub struct ConnectionState {
    pub app_id_input: String,
    pub is_connected: bool,
    pub is_connecting: bool,
    pub remote_ready: bool,
    pub since_connected: Option<Instant>,
}

impl ConnectionState {
    pub fn reset(&mut self) {
        self.is_connected = false;
        self.is_connecting = false;
        self.remote_ready = false;
        self.since_connected = None;
    }
}

// 文件对比详情（用于 UI 显示）
#[derive(Debug, Clone, Default)]
pub struct FileComparisonInfo {
    pub status: SyncStatus,
    pub diff_flags: DiffFlags,
    pub hash_status: HashStatus,
}

// 文件列表状态
#[derive(Default)]
pub struct FileListState {
    pub files: Vec<CloudFile>,
    pub selected_files: Vec<usize>,
    pub file_tree: Option<crate::file_tree::FileTree>,
    pub local_save_paths: Vec<(String, PathBuf)>,
    pub search_query: String,
    pub show_only_local: bool,
    pub show_only_cloud: bool,
    pub last_selected_index: Option<usize>,
    pub is_refreshing: bool,
    pub sync_status_map: HashMap<String, SyncStatus>, // 文件名 -> 同步状态
    pub comparison_map: HashMap<String, FileComparisonInfo>, // 文件名 -> 对比详情
    pub hash_checker: AsyncHashChecker,               // 异步 hash 检测器
    pub hash_checked_app_id: Option<u32>, // 已完成 Hash 检测的 app_id (缓存，避免重复检测)
}

impl FileListState {
    pub fn clear(&mut self) {
        self.files.clear();
        self.selected_files.clear();
        self.file_tree = None;
        self.local_save_paths.clear();
        self.sync_status_map.clear();
        self.comparison_map.clear();
        self.hash_checker.cancel();
        self.hash_checked_app_id = None;
    }

    // 更新同步状态
    pub fn update_sync_status(&mut self) -> Vec<crate::conflict::FileComparison> {
        let comparisons = crate::conflict::detect_all(&self.files, &self.local_save_paths);

        self.sync_status_map.clear();
        self.comparison_map.clear();

        for c in &comparisons {
            self.sync_status_map.insert(c.filename.clone(), c.status);
            self.comparison_map.insert(
                c.filename.clone(),
                FileComparisonInfo {
                    status: c.status,
                    diff_flags: c.diff_flags.clone(),
                    hash_status: c.hash_status,
                },
            );
        }
        comparisons
    }
}

// 游戏库状态
#[derive(Default)]
pub struct GameLibraryState {
    pub cloud_games: Vec<CloudGameInfo>,
    pub show_game_selector: bool,
    pub is_scanning_games: bool,
    pub all_users: Vec<UserInfo>,
    pub show_user_selector: bool,
    pub vdf_count: usize,
    pub cdp_count: usize,
}

// 弹窗状态
pub struct DialogState {
    pub show_error: bool,
    pub error_message: String,
    pub show_settings: bool,
    pub settings_state: crate::ui::SettingsWindowState,
    pub show_debug_warning: bool,
    pub guide_dialog: Option<crate::ui::GuideDialog>,
    pub upload_preview: Option<crate::ui::UploadPreviewDialog>,
    pub upload_progress: Option<crate::ui::UploadProgressDialog>,
    pub upload_complete: Option<crate::ui::UploadCompleteDialog>,
    pub conflict_dialog: crate::ui::ConflictDialog,
    pub show_backup: bool,
    pub backup_preview: Option<crate::ui::BackupPreviewDialog>,
    pub backup_progress: Option<crate::ui::BackupProgressDialog>,
    pub download_progress: Option<crate::ui::DownloadProgressDialog>,
    pub appinfo_dialog: Option<crate::ui::AppInfoDialog>,
    pub symlink_dialog: Option<crate::ui::SymlinkDialog>,
}

impl Default for DialogState {
    fn default() -> Self {
        Self {
            show_error: false,
            error_message: String::new(),
            show_settings: false,
            settings_state: crate::ui::SettingsWindowState::default(),
            show_debug_warning: !crate::cdp_client::CdpClient::is_cdp_running(),
            guide_dialog: None,
            upload_preview: None,
            upload_progress: None,
            upload_complete: None,
            conflict_dialog: crate::ui::ConflictDialog::new(),
            show_backup: false,
            backup_preview: None,
            backup_progress: None,
            download_progress: None,
            appinfo_dialog: None,
            symlink_dialog: None,
        }
    }
}

impl DialogState {
    pub fn show_error(&mut self, message: &str) {
        self.error_message = message.to_string();
        self.show_error = true;
    }
}

// 其他状态
pub struct MiscState {
    pub status_message: String,
    pub quota_info: Option<(u64, u64)>,
    pub i18n: I18n,
}

impl Default for MiscState {
    fn default() -> Self {
        let i18n = I18n::new(Language::Chinese);
        Self {
            status_message: i18n.status_enter_app_id().to_string(),
            quota_info: None,
            i18n,
        }
    }
}
