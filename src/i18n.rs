use crate::icons;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Language {
    #[default]
    Chinese,
    English,
}

impl Language {
    // 返回所有支持的语言列表
    pub const fn all() -> &'static [Language] {
        &[Language::Chinese, Language::English]
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Language::Chinese => "简体中文",
            Language::English => "English",
        }
    }
}

pub struct I18n {
    lang: Language,
}

impl I18n {
    pub fn new(lang: Language) -> Self {
        Self { lang }
    }

    pub fn set_language(&mut self, lang: Language) {
        self.lang = lang;
    }

    pub fn language(&self) -> Language {
        self.lang
    }

    // ========== UI 通用文本 ==========

    pub fn app_title(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "Steam 云存档管理器",
            Language::English => "Steam Cloud File Manager",
        }
    }

    pub fn refresh(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "刷新",
            Language::English => "Refresh",
        }
    }

    pub fn cancel(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "取消",
            Language::English => "Cancel",
        }
    }

    pub fn close(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "关闭",
            Language::English => "Close",
        }
    }

    pub fn ok(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "确定",
            Language::English => "OK",
        }
    }

    pub fn logged_in(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "已登录",
            Language::English => "Logged In",
        }
    }

    pub fn not_logged_in(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "未登录",
            Language::English => "Not Logged In",
        }
    }

    pub fn connect(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "连接",
            Language::English => "Connect",
        }
    }

    pub fn disconnect(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "断开",
            Language::English => "Disconnect",
        }
    }

    pub fn disconnect_sync_hint(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "断开后 Steam 将自动同步",
            Language::English => "Steam will auto-sync after disconnect",
        }
    }

    pub fn refresh_open_url_hint(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "在 Steam 中打开云存储页面",
            Language::English => "Open cloud storage page in Steam",
        }
    }

    pub fn show_appinfo_vdf(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "显示 appinfo.vdf",
            Language::English => "Show appinfo.vdf",
        }
    }

    // ========== 账户和游戏选择 ==========

    pub fn account_cloud_status(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "账户云存储",
            Language::English => "Account Cloud",
        }
    }

    pub fn select_account(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "账户",
            Language::English => "Accounts",
        }
    }

    pub fn select_game(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "游戏库",
            Language::English => "Game Library",
        }
    }

    // ========== 文件操作 ==========

    pub fn select_all(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "全选",
            Language::English => "Select All",
        }
    }

    pub fn invert_selection(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "反选",
            Language::English => "Invert",
        }
    }

    pub fn clear_selection(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "清除选择",
            Language::English => "Clear",
        }
    }

    pub fn download(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "下载",
            Language::English => "Download",
        }
    }

    pub fn upload(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "上传",
            Language::English => "Upload",
        }
    }

    pub fn delete(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "删除",
            Language::English => "Delete",
        }
    }

    pub fn forget(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "移出云端",
            Language::English => "Forget",
        }
    }

    pub fn sync_to_cloud(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "同步云端",
            Language::English => "Sync",
        }
    }

    pub fn file_name(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "文件名",
            Language::English => "File Name",
        }
    }

    pub fn size(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "大小",
            Language::English => "Size",
        }
    }

    pub fn selected_count(&self, count: usize) -> String {
        match self.lang {
            Language::Chinese => format!("已选择 {} 个", count),
            Language::English => format!("{} selected", count),
        }
    }

    // ========== 窗口标题 ==========
    pub fn select_game_title(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "选择游戏",
            Language::English => "Select Game",
        }
    }

    pub fn settings_title(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "设置",
            Language::English => "Settings",
        }
    }

    pub fn settings_log(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "日志",
            Language::English => "Log",
        }
    }

    pub fn settings_about(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "关于",
            Language::English => "About",
        }
    }

    pub fn settings_appearance(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "外观",
            Language::English => "Appearance",
        }
    }

    pub fn settings_advanced(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "高级",
            Language::English => "Advanced",
        }
    }

    // ========== 高级设置 ==========
    pub fn steam_path_label(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "Steam 安装路径",
            Language::English => "Steam Install Path",
        }
    }

    pub fn steam_path_auto_detect(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "自动检测",
            Language::English => "Auto Detect",
        }
    }

    pub fn steam_path_browse(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "浏览...",
            Language::English => "Browse...",
        }
    }

    pub fn steam_path_valid(&self, user_count: usize) -> String {
        match self.lang {
            Language::Chinese => format!("✓ 路径有效 (检测到 {} 个用户)", user_count),
            Language::English => format!(
                "✓ Valid path ({} user{} found)",
                user_count,
                if user_count != 1 { "s" } else { "" }
            ),
        }
    }

    pub fn steam_path_not_exists(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "✗ 路径不存在",
            Language::English => "✗ Path not exists",
        }
    }

    pub fn steam_path_no_userdata(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "✗ 缺少 userdata 目录",
            Language::English => "✗ Missing userdata folder",
        }
    }

    pub fn steam_path_no_users(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "✗ 未找到用户",
            Language::English => "✗ No users found",
        }
    }

    pub fn steam_path_hint(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "💡 如果 Steam 安装在非标准位置，请手动选择目录",
            Language::English => {
                "💡 Select directory manually if Steam is in non-standard location"
            }
        }
    }

    pub fn steam_path_restart_hint(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "修改后需要重启应用生效",
            Language::English => "Restart required after changing",
        }
    }

    pub fn reset_all_settings(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "恢复默认设置",
            Language::English => "Reset All Settings",
        }
    }

    pub fn reset_confirm(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "确定要恢复所有设置为默认值吗？",
            Language::English => "Reset all settings to default?",
        }
    }

    pub fn config_dir_label(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "配置文件:",
            Language::English => "Config File:",
        }
    }

    pub fn open_config_dir(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "打开配置目录",
            Language::English => "Open Config Directory",
        }
    }

    pub fn theme_mode_label(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "主题模式:",
            Language::English => "Theme Mode:",
        }
    }

    pub fn error_title(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "错误",
            Language::English => "Error",
        }
    }

    // ========== About 窗口内容 ==========
    pub fn author(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "作者:",
            Language::English => "Author:",
        }
    }

    pub fn github_repository(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "Github仓库:",
            Language::English => "Github Repository:",
        }
    }

    // 状态消息
    pub fn connecting(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "连接中...",
            Language::English => "Connecting...",
        }
    }

    // ========== 游戏选择器相关 ==========
    pub fn games_with_cloud(&self, count: usize) -> String {
        match self.lang {
            Language::Chinese => format!("{} 个有云存档的游戏", count),
            Language::English => format!(
                "{} game{} with cloud saves",
                count,
                if count != 1 { "s" } else { "" }
            ),
        }
    }

    pub fn scanning_games(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "正在扫描游戏库...",
            Language::English => "Scanning game library...",
        }
    }

    pub fn no_cloud_games_found(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "未发现云存档的游戏",
            Language::English => "No games with cloud saves found",
        }
    }

    pub fn installed(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "已安装",
            Language::English => "Installed",
        }
    }

    pub fn not_installed(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "未安装",
            Language::English => "Not Installed",
        }
    }

    // ========== 用户选择器 ==========
    pub fn select_user(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "选择用户",
            Language::English => "Select User",
        }
    }

    pub fn user_id(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "用户 ID",
            Language::English => "User ID",
        }
    }

    pub fn current_user(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "当前用户",
            Language::English => "Current User",
        }
    }

    pub fn switch(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "切换",
            Language::English => "Switch",
        }
    }

    pub fn steam_users(&self, count: usize) -> String {
        match self.lang {
            Language::Chinese => format!("{} 个 Steam 用户", count),
            Language::English => {
                format!("{} Steam user{}", count, if count != 1 { "s" } else { "" })
            }
        }
    }

    // ========== About 窗口更多翻译 ==========
    pub fn checking_update(&self) -> String {
        match self.lang {
            Language::Chinese => format!("{} 检查中...", icons::SPINNER),
            Language::English => format!("{} Checking...", icons::SPINNER),
        }
    }

    pub fn check_update_btn(&self) -> String {
        match self.lang {
            Language::Chinese => format!("{} 检查更新", icons::REFRESH),
            Language::English => format!("{} Check Update", icons::REFRESH),
        }
    }

    pub fn already_latest(&self) -> String {
        match self.lang {
            Language::Chinese => format!("{} 当前已是最新版本", icons::CHECK),
            Language::English => format!("{} Already up to date", icons::CHECK),
        }
    }

    pub fn new_version_found(&self, version: &str) -> String {
        match self.lang {
            Language::Chinese => format!("🎉 发现新版本: {}", version),
            Language::English => format!("🎉 New version available: {}", version),
        }
    }

    pub fn new_version_hint(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "发现新版本，点击下载并安装：",
            Language::English => "New version found, click to download and install:",
        }
    }

    pub fn download_and_install(&self) -> String {
        match self.lang {
            Language::Chinese => format!("{} 下载并安装", icons::DOWNLOAD),
            Language::English => format!("{} Download & Install", icons::DOWNLOAD),
        }
    }

    pub fn view_details(&self) -> String {
        match self.lang {
            Language::Chinese => format!("{} 查看详情", icons::GLOBE),
            Language::English => format!("{} View Details", icons::GLOBE),
        }
    }

    pub fn downloading_update(&self) -> String {
        match self.lang {
            Language::Chinese => format!("{} 正在下载更新...", icons::DOWNLOAD),
            Language::English => format!("{} Downloading update...", icons::DOWNLOAD),
        }
    }

    pub fn installing_update(&self) -> String {
        match self.lang {
            Language::Chinese => format!("{} 正在安装更新...", icons::GEAR),
            Language::English => format!("{} Installing update...", icons::GEAR),
        }
    }

    pub fn update_success(&self) -> String {
        match self.lang {
            Language::Chinese => format!("{} 更新安装成功！", icons::CHECK),
            Language::English => format!("{} Update installed successfully!", icons::CHECK),
        }
    }

    pub fn restart_to_apply(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "请重启应用以使用新版本",
            Language::English => "Please restart the app to use the new version",
        }
    }

    pub fn restart_now(&self) -> String {
        match self.lang {
            Language::Chinese => format!("{} 立即重启", icons::REFRESH),
            Language::English => format!("{} Restart Now", icons::REFRESH),
        }
    }

    pub fn log_enabled_hint(&self) -> &'static str {
        match self.lang {
            Language::Chinese => " 日志存储已启用，重启后生效",
            Language::English => " Log storage enabled, restart to apply",
        }
    }

    pub fn log_disabled_hint(&self) -> &'static str {
        match self.lang {
            Language::Chinese => " 日志存储已禁用，重启后生效",
            Language::English => " Log storage disabled, restart to apply",
        }
    }

    pub fn enable_log_storage(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "启用日志存储",
            Language::English => "Enable Log Storage",
        }
    }

    pub fn open_log_dir(&self) -> &'static str {
        match self.lang {
            Language::Chinese => " 打开日志目录",
            Language::English => " Open Log Directory",
        }
    }

    pub fn log_dir_label(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "日志目录:",
            Language::English => "Log Directory:",
        }
    }

    // ========== Guide 对话框 ==========
    pub fn restarting_steam(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "正在重启 Steam",
            Language::English => "Restarting Steam",
        }
    }

    pub fn manual_operation_required(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "需要手动操作：",
            Language::English => "Manual operation required:",
        }
    }

    pub fn i_understand(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "我知道了",
            Language::English => "I Understand",
        }
    }

    #[cfg(target_os = "macos")]
    pub fn manual_restart_macos_title(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "手动重启 Steam (macOS)",
            Language::English => "Manual Restart Steam (macOS)",
        }
    }

    #[cfg(target_os = "windows")]
    pub fn manual_restart_windows_title(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "手动重启 Steam (Windows)",
            Language::English => "Manual Restart Steam (Windows)",
        }
    }

    #[cfg(target_os = "linux")]
    pub fn manual_restart_linux_title(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "手动重启 Steam (Linux)",
            Language::English => "Manual Restart Steam (Linux)",
        }
    }

    // ========== Upload 对话框 ==========
    pub fn prepare_upload(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "准备上传",
            Language::English => "Prepare Upload",
        }
    }

    pub fn will_upload_files(&self, count: usize) -> String {
        match self.lang {
            Language::Chinese => format!("将要上传 {} 个文件到 Steam Cloud", count),
            Language::English => format!(
                "Will upload {} file{} to Steam Cloud",
                count,
                if count != 1 { "s" } else { "" }
            ),
        }
    }

    pub fn total_size_label(&self, size: &str) -> String {
        match self.lang {
            Language::Chinese => format!("总大小: {}", size),
            Language::English => format!("Total size: {}", size),
        }
    }

    pub fn warning(&self) -> String {
        match self.lang {
            Language::Chinese => format!("{} 警告：", icons::WARNING),
            Language::English => format!("{} Warning:", icons::WARNING),
        }
    }

    pub fn overwrite_warning(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "• 同名文件将被覆盖",
            Language::English => "• Files with same name will be overwritten",
        }
    }

    pub fn add_files(&self) -> String {
        match self.lang {
            Language::Chinese => format!("{} 添加文件", icons::ADD_FILE),
            Language::English => format!("{} Add Files", icons::ADD_FILE),
        }
    }

    pub fn add_folder(&self) -> String {
        match self.lang {
            Language::Chinese => format!("{} 添加文件夹", icons::ADD_FOLDER),
            Language::English => format!("{} Add Folder", icons::ADD_FOLDER),
        }
    }

    pub fn confirm_upload(&self) -> String {
        match self.lang {
            Language::Chinese => format!("{} 确认上传", icons::CHECK),
            Language::English => format!("{} Confirm Upload", icons::CHECK),
        }
    }

    pub fn remove_file(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "移除",
            Language::English => "Remove",
        }
    }

    pub fn cloud_path(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "云端路径",
            Language::English => "Cloud Path",
        }
    }

    pub fn edit_path(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "编辑路径",
            Language::English => "Edit Path",
        }
    }

    pub fn local_file(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "本地文件",
            Language::English => "Local File",
        }
    }

    pub fn no_files_to_upload(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "没有文件待上传，请添加文件",
            Language::English => "No files to upload, please add files",
        }
    }

    pub fn clear_all(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "清空列表",
            Language::English => "Clear All",
        }
    }

    pub fn uploading_files(&self) -> String {
        match self.lang {
            Language::Chinese => format!("{} 正在上传文件", icons::UPLOAD),
            Language::English => format!("{} Uploading Files", icons::UPLOAD),
        }
    }

    pub fn uploading_file(&self, name: &str) -> String {
        match self.lang {
            Language::Chinese => format!("正在上传: {}", name),
            Language::English => format!("Uploading: {}", name),
        }
    }

    pub fn upload_progress(&self, current: usize, total: usize) -> String {
        match self.lang {
            Language::Chinese => format!("进度: {} / {} 文件", current, total),
            Language::English => format!("Progress: {} / {} files", current, total),
        }
    }

    pub fn speed(&self, speed: &str) -> String {
        match self.lang {
            Language::Chinese => format!("速度: {}/s", speed),
            Language::English => format!("Speed: {}/s", speed),
        }
    }

    pub fn upload_complete(&self) -> String {
        match self.lang {
            Language::Chinese => format!("{} 上传完成", icons::CHECK),
            Language::English => format!("{} Upload Complete", icons::CHECK),
        }
    }

    pub fn upload_success(&self, count: usize) -> String {
        match self.lang {
            Language::Chinese => format!("{} 成功上传 {} 个文件", icons::ROCKET, count),
            Language::English => format!(
                "{} Successfully uploaded {} file{}",
                icons::ROCKET,
                count,
                if count != 1 { "s" } else { "" }
            ),
        }
    }

    pub fn upload_partial(&self, success: usize, failed: usize) -> String {
        match self.lang {
            Language::Chinese => format!(
                "{} 上传完成：成功 {}，失败 {}",
                icons::WARNING,
                success,
                failed
            ),
            Language::English => format!(
                "{} Upload complete: {} succeeded, {} failed",
                icons::WARNING,
                success,
                failed
            ),
        }
    }

    pub fn elapsed_time(&self, secs: u64) -> String {
        match self.lang {
            Language::Chinese => format!("用时: {} 秒", secs),
            Language::English => {
                format!("Time: {} second{}", secs, if secs != 1 { "s" } else { "" })
            }
        }
    }

    pub fn avg_speed(&self, speed: &str) -> String {
        match self.lang {
            Language::Chinese => format!("平均速度: {}/s", speed),
            Language::English => format!("Avg speed: {}/s", speed),
        }
    }

    pub fn failed_files(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "失败文件列表：",
            Language::English => "Failed files:",
        }
    }

    pub fn reason(&self, err: &str) -> String {
        match self.lang {
            Language::Chinese => format!("  原因: {}", err),
            Language::English => format!("  Reason: {}", err),
        }
    }

    // ========== Steam 重启状态消息 ==========
    pub fn closing_steam(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "正在关闭 Steam...",
            Language::English => "Closing Steam...",
        }
    }

    pub fn starting_steam(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "正在启动 Steam...",
            Language::English => "Starting Steam...",
        }
    }

    pub fn steam_restart_success(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "Steam 已成功重启!",
            Language::English => "Steam restarted successfully!",
        }
    }

    pub fn user_switched(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "已切换用户",
            Language::English => "User switched",
        }
    }

    // ========== 错误消息 ==========
    pub fn error_enter_app_id(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "请输入App ID",
            Language::English => "Please enter App ID",
        }
    }

    pub fn error_invalid_app_id(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "无效的 App ID",
            Language::English => "Invalid App ID",
        }
    }

    // 状态消息
    pub fn status_enter_app_id(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "请输入App ID并连接到Steam",
            Language::English => "Enter App ID and connect to Steam",
        }
    }

    pub fn status_loading_files(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "正在加载文件列表...",
            Language::English => "Loading file list...",
        }
    }

    pub fn status_files_loaded(&self, count: usize) -> String {
        match self.lang {
            Language::Chinese => format!("已加载 {} 个文件", count),
            Language::English => {
                format!("Loaded {} file{}", count, if count != 1 { "s" } else { "" })
            }
        }
    }

    // 上传失败消息
    pub fn upload_failed(&self, err: &str) -> String {
        match self.lang {
            Language::Chinese => format!("上传失败: {}", err),
            Language::English => format!("Upload failed: {}", err),
        }
    }

    pub fn error_no_files_selected(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "请选择要操作的文件",
            Language::English => "Please select files to operate",
        }
    }

    pub fn error_not_connected(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "未连接到 Steam",
            Language::English => "Not connected to Steam",
        }
    }

    // ========== 提示文本 ==========
    pub fn hint_you_can(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "您可以：",
            Language::English => "You can:",
        }
    }

    pub fn hint_select_game(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "点击上方的 '游戏库' 按钮选择游戏",
            Language::English => "Click 'Game Library' button above to choose a game",
        }
    }

    pub fn hint_enter_app_id(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "或直接输入 App ID 并点击 '连接'",
            Language::English => "Or enter App ID directly and click 'Connect'",
        }
    }

    pub fn no_cloud_files(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "没有找到云文件",
            Language::English => "No cloud files found",
        }
    }

    pub fn no_cloud_files_hint(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "该游戏没有云存档文件",
            Language::English => "This game has no cloud save files",
        }
    }

    pub fn scan_games_failed(&self, err: &str) -> String {
        match self.lang {
            Language::Chinese => format!("扫描游戏失败: {}", err),
            Language::English => format!("Failed to scan games: {}", err),
        }
    }

    pub fn refresh_files_failed(&self, err: &str) -> String {
        match self.lang {
            Language::Chinese => format!("刷新文件列表失败: {}", err),
            Language::English => format!("Failed to refresh file list: {}", err),
        }
    }

    pub fn cdp_no_data_error(&self) -> &'static str {
        match self.lang {
            Language::Chinese => {
                "CDP 未获取到游戏数据！\n\n可能原因：\n\
                1. Steam 客户端未响应跳转请求\n\
                2. 页面加载未完成\n\
                3. 未登录 Steam 网页\n\n"
            }
            Language::English => {
                "CDP failed to get game data!\n\nPossible reasons:\n\
                1. Steam client not responding to redirect request\n\
                2. Page not fully loaded\n\
                3. Not logged into Steam web\n\n"
            }
        }
    }

    pub fn connecting_to_steam(&self, app_id: u32) -> String {
        match self.lang {
            Language::Chinese => format!("正在连接到 Steam (App ID: {})...", app_id),
            Language::English => format!("Connecting to Steam (App ID: {})...", app_id),
        }
    }

    pub fn loading_files_for_app(&self, app_id: u32) -> String {
        match self.lang {
            Language::Chinese => format!("正在加载文件列表 (App ID: {})...", app_id),
            Language::English => format!("Loading file list (App ID: {})...", app_id),
        }
    }

    pub fn connect_steam_failed(&self, err: &str) -> String {
        match self.lang {
            Language::Chinese => format!("连接Steam失败: {}", err),
            Language::English => format!("Failed to connect to Steam: {}", err),
        }
    }

    pub fn vdf_parser_not_initialized(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "VDF 解析器未初始化",
            Language::English => "VDF parser not initialized",
        }
    }

    pub fn scanning_game_library(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "正在扫描游戏库...",
            Language::English => "Scanning game library...",
        }
    }

    pub fn drop_files_to_upload(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "释放文件以上传",
            Language::English => "Drop files to upload",
        }
    }

    // ========== 调试模式警告 ==========
    pub fn debug_mode_not_enabled(&self) -> String {
        match self.lang {
            Language::Chinese => format!("{} Steam 调试模式未启用", icons::WARNING),
            Language::English => format!("{} Steam Debug Mode Not Enabled", icons::WARNING),
        }
    }

    pub fn steam_running(&self) -> String {
        match self.lang {
            Language::Chinese => format!("{} Steam 正在运行", icons::CHECK),
            Language::English => format!("{} Steam is running", icons::CHECK),
        }
    }

    pub fn steam_not_running(&self) -> String {
        match self.lang {
            Language::Chinese => format!("{} Steam 未运行", icons::CLOSE),
            Language::English => format!("{} Steam is not running", icons::CLOSE),
        }
    }

    pub fn debug_mode_hint(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "需要启用 Steam 的 CEF 调试模式才能使用网页登录功能",
            Language::English => "CEF debug mode is required for web login functionality",
        }
    }

    pub fn auto_restart_steam(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "自动重启 Steam",
            Language::English => "Auto Restart Steam",
        }
    }

    pub fn start_steam(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "启动 Steam",
            Language::English => "Start Steam",
        }
    }

    pub fn auto_restart_hint(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "自动关闭并重启 Steam，添加调试参数",
            Language::English => "Automatically restart Steam with debug parameters",
        }
    }

    pub fn start_steam_hint(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "以调试模式启动 Steam",
            Language::English => "Start Steam in debug mode",
        }
    }

    pub fn view_manual_steps(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "查看手动操作",
            Language::English => "View Manual Steps",
        }
    }

    pub fn manual_steps_hint(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "显示如何手动添加启动参数",
            Language::English => "Show how to manually add startup parameters",
        }
    }

    pub fn dismiss_temporarily(&self) -> String {
        match self.lang {
            Language::Chinese => format!("{} 暂时忽略", icons::CLOSE),
            Language::English => format!("{} Dismiss", icons::CLOSE),
        }
    }

    pub fn dismiss_hint(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "隐藏此提示（可在设置中重新显示）",
            Language::English => "Hide this hint (can be re-enabled in settings)",
        }
    }

    // ========== 状态栏 ==========
    pub fn status_label(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "状态:",
            Language::English => "Status:",
        }
    }

    pub fn cloud_on(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "云存储: 开启",
            Language::English => "Cloud: On",
        }
    }

    pub fn cloud_off(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "云存储: 关闭",
            Language::English => "Cloud: Off",
        }
    }

    pub fn quota_usage(&self, percent: f32, used: &str, total: &str) -> String {
        match self.lang {
            Language::Chinese => format!("配额: {:.1}% 已使用 ({}/{})", percent, used, total),
            Language::English => format!("Quota: {:.1}% used ({}/{})", percent, used, total),
        }
    }

    // ========== 按钮悬停提示 ==========
    pub fn select_all_hint(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "选择列表中的所有文件",
            Language::English => "Select all files in the list",
        }
    }

    pub fn invert_selection_hint(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "反转当前选择状态",
            Language::English => "Invert current selection",
        }
    }

    pub fn clear_selection_hint(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "取消选择所有文件",
            Language::English => "Deselect all files",
        }
    }

    pub fn download_hint(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "下载选中的文件到本地",
            Language::English => "Download selected files to local",
        }
    }

    pub fn upload_hint(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "上传文件或文件夹到云端",
            Language::English => "Upload files or folders to cloud",
        }
    }

    pub fn delete_hint(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "从云端和本地删除选中的文件",
            Language::English => "Delete selected files from cloud and local",
        }
    }

    pub fn forget_hint(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "仅从云端移除，保留本地文件",
            Language::English => "Remove from cloud only, keep local files",
        }
    }

    pub fn sync_to_cloud_hint(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "将本地文件同步到云端",
            Language::English => "Sync local files to cloud",
        }
    }

    pub fn connect_hint(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "连接到 Steam 云存储 API",
            Language::English => "Connect to Steam Cloud API",
        }
    }

    pub fn disconnect_hint(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "断开与 Steam 的连接",
            Language::English => "Disconnect from Steam",
        }
    }

    pub fn select_account_hint(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "切换 Steam 账户",
            Language::English => "Switch Steam account",
        }
    }

    pub fn select_game_hint(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "选择要管理云存档的游戏",
            Language::English => "Select game to manage cloud saves",
        }
    }

    // ========== 文件列表面板 ==========
    pub fn local_save_path(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "本地存档路径:",
            Language::English => "Local Save Path:",
        }
    }

    pub fn local_save_path_not_found(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "未找到（可能所有文件都仅在云端）",
            Language::English => "Not found (files may only exist in cloud)",
        }
    }

    pub fn search_files_placeholder(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "搜索文件或文件夹...",
            Language::English => "Search files or folders...",
        }
    }

    pub fn clear(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "清除",
            Language::English => "Clear",
        }
    }

    pub fn only_local(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "仅本地",
            Language::English => "Local Only",
        }
    }

    pub fn only_cloud(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "仅云端",
            Language::English => "Cloud Only",
        }
    }

    pub fn only_local_tooltip(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "只显示仅在本地存在的文件（未同步到云端）",
            Language::English => "Show only files that exist locally but not in cloud",
        }
    }

    pub fn only_cloud_tooltip(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "只显示仅在云端存在的文件（本地不存在）",
            Language::English => "Show only files that exist in cloud but not locally",
        }
    }

    pub fn root_folder(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "根文件夹",
            Language::English => "Root Folder",
        }
    }

    pub fn file_size(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "文件大小",
            Language::English => "File Size",
        }
    }

    pub fn write_date(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "写入日期",
            Language::English => "Write Date",
        }
    }

    pub fn local(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "本地",
            Language::English => "Local",
        }
    }

    pub fn cloud(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "云端",
            Language::English => "Cloud",
        }
    }

    // ========== 文件对比对话框 ==========
    pub fn file_comparison_title(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "文件对比",
            Language::English => "File Comparison",
        }
    }

    pub fn total_files_count(&self, count: usize) -> String {
        match self.lang {
            Language::Chinese => format!("共 {} 个文件", count),
            Language::English => format!("{} files total", count),
        }
    }

    pub fn filter_all(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "全部",
            Language::English => "All",
        }
    }

    pub fn filter_conflicts(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "冲突",
            Language::English => "Conflicts",
        }
    }

    pub fn filter_local_newer(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "本地较新",
            Language::English => "Local Newer",
        }
    }

    pub fn filter_cloud_newer(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "云端较新",
            Language::English => "Cloud Newer",
        }
    }

    pub fn filter_synced(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "已同步",
            Language::English => "Synced",
        }
    }

    pub fn status_local_newer(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "本地新",
            Language::English => "Local↑",
        }
    }

    pub fn status_cloud_newer(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "云端新",
            Language::English => "Cloud↓",
        }
    }

    pub fn status_conflict(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "冲突",
            Language::English => "Conflict",
        }
    }

    pub fn status_local_only(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "仅本地",
            Language::English => "Local",
        }
    }

    pub fn status_cloud_only(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "仅云端",
            Language::English => "Cloud",
        }
    }

    pub fn column_status(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "状态",
            Language::English => "Status",
        }
    }

    pub fn column_filename(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "文件名",
            Language::English => "Filename",
        }
    }

    pub fn column_local_size(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "本地大小",
            Language::English => "Local Size",
        }
    }

    pub fn column_cloud_size(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "云端大小",
            Language::English => "Cloud Size",
        }
    }

    pub fn column_local_time(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "本地时间",
            Language::English => "Local Time",
        }
    }

    pub fn column_cloud_time(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "云端时间",
            Language::English => "Cloud Time",
        }
    }

    pub fn selected_file(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "选中文件:",
            Language::English => "Selected:",
        }
    }

    pub fn local_newer_by(&self, secs: i64) -> String {
        match self.lang {
            Language::Chinese => format!("(本地比云端新 {} 秒)", secs),
            Language::English => format!("(local {} secs newer)", secs),
        }
    }

    pub fn cloud_newer_by(&self, secs: i64) -> String {
        match self.lang {
            Language::Chinese => format!("(云端比本地新 {} 秒)", secs),
            Language::English => format!("(cloud {} secs newer)", secs),
        }
    }

    pub fn conflicts_warning(&self, count: usize) -> String {
        match self.lang {
            Language::Chinese => format!("检测到 {} 个冲突，请手动解决", count),
            Language::English => format!("{} conflicts detected, please resolve manually", count),
        }
    }

    pub fn compare_files(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "对比文件",
            Language::English => "Compare Files",
        }
    }

    pub fn compare_files_hint(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "对比本地和云端文件的差异",
            Language::English => "Compare differences between local and cloud files",
        }
    }

    // ========== 备份功能 ==========

    pub fn backup(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "备份",
            Language::English => "Backup",
        }
    }

    pub fn backup_title(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "备份云存档",
            Language::English => "Backup Cloud Saves",
        }
    }

    pub fn backup_file_count(&self, count: usize) -> String {
        match self.lang {
            Language::Chinese => format!("共 {} 个文件", count),
            Language::English => format!("{} files", count),
        }
    }

    pub fn backup_total_size(&self, size: &str) -> String {
        match self.lang {
            Language::Chinese => format!("总大小: {}", size),
            Language::English => format!("Total size: {}", size),
        }
    }

    pub fn backup_cdp_warning(&self, count: usize) -> String {
        match self.lang {
            Language::Chinese => format!("{} {} 个文件无下载链接，将跳过", icons::WARNING, count),
            Language::English => format!(
                "{} {} files without download URL will be skipped",
                icons::WARNING,
                count
            ),
        }
    }

    pub fn backup_file_list(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "文件列表",
            Language::English => "File List",
        }
    }

    pub fn backup_start(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "开始备份",
            Language::English => "Start Backup",
        }
    }

    pub fn backup_open_dir(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "打开备份目录",
            Language::English => "Open Backup Dir",
        }
    }

    pub fn backup_progress_title(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "备份进度",
            Language::English => "Backup Progress",
        }
    }

    pub fn backup_in_progress(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "正在备份...",
            Language::English => "Backing up...",
        }
    }

    pub fn backup_complete(&self) -> String {
        match self.lang {
            Language::Chinese => format!("{} 备份完成", icons::CHECK),
            Language::English => format!("{} Backup Complete", icons::CHECK),
        }
    }

    pub fn backup_partial(&self) -> String {
        match self.lang {
            Language::Chinese => format!("{} 部分完成", icons::WARNING),
            Language::English => format!("{} Partially Complete", icons::WARNING),
        }
    }

    pub fn backup_result_stats(&self, success: usize, total: usize) -> String {
        match self.lang {
            Language::Chinese => format!("成功: {} / {}", success, total),
            Language::English => format!("Success: {} / {}", success, total),
        }
    }

    pub fn backup_failed_files(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "失败的文件:",
            Language::English => "Failed files:",
        }
    }

    pub fn backup_hint(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "备份当前游戏的所有云存档",
            Language::English => "Backup all cloud saves for current game",
        }
    }

    pub fn backup_dir_label(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "备份目录:",
            Language::English => "Backup Directory:",
        }
    }

    // ========== 下载相关 ==========
    pub fn download_progress_title(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "下载进度",
            Language::English => "Download Progress",
        }
    }

    pub fn download_in_progress(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "正在下载...",
            Language::English => "Downloading...",
        }
    }

    pub fn download_complete(&self) -> String {
        match self.lang {
            Language::Chinese => format!("{} 下载完成", icons::CHECK),
            Language::English => format!("{} Download Complete", icons::CHECK),
        }
    }

    pub fn download_partial_status(&self) -> String {
        match self.lang {
            Language::Chinese => format!("{} 部分完成", icons::WARNING),
            Language::English => format!("{} Partially Complete", icons::WARNING),
        }
    }

    pub fn download_result_stats(&self, success: usize, total: usize) -> String {
        match self.lang {
            Language::Chinese => format!("成功: {} / {}", success, total),
            Language::English => format!("Success: {} / {}", success, total),
        }
    }

    pub fn download_failed_files(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "失败的文件:",
            Language::English => "Failed files:",
        }
    }

    pub fn download_open_dir(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "打开下载目录",
            Language::English => "Open Download Dir",
        }
    }

    // ========== 软链接功能 ==========

    pub fn symlink_title(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "软链接管理 (实验性)",
            Language::English => "Symlink Management (Experimental)",
        }
    }

    pub fn symlink_configured_links(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "已配置的软链接",
            Language::English => "Configured Symlinks",
        }
    }

    pub fn symlink_no_configs(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "暂无软链接配置",
            Language::English => "No symlink configurations",
        }
    }

    pub fn symlink_add_new(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "添加新软链接",
            Language::English => "Add New Symlink",
        }
    }

    pub fn symlink_direction(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "方向:",
            Language::English => "Direction:",
        }
    }

    pub fn symlink_local_path(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "本地路径:",
            Language::English => "Local Path:",
        }
    }

    pub fn symlink_remote_subfolder(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "Remote 子目录:",
            Language::English => "Remote Subfolder:",
        }
    }

    pub fn symlink_browse(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "选择文件夹",
            Language::English => "Browse",
        }
    }

    pub fn symlink_add_config(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "添加配置",
            Language::English => "Add Config",
        }
    }

    pub fn symlink_add_and_create(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "添加并创建链接",
            Language::English => "Add & Create Link",
        }
    }

    pub fn symlink_create(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "创建链接",
            Language::English => "Create Link",
        }
    }

    pub fn symlink_remove_link(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "删除链接",
            Language::English => "Remove Link",
        }
    }

    pub fn symlink_delete_config(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "删除配置",
            Language::English => "Delete Config",
        }
    }

    pub fn symlink_copy_command(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "复制命令",
            Language::English => "Copy Command",
        }
    }

    pub fn symlink_refresh(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "刷新",
            Language::English => "Refresh",
        }
    }

    pub fn symlink_command_copied(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "命令已复制到剪贴板",
            Language::English => "Command copied to clipboard",
        }
    }

    pub fn symlink_config_deleted(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "配置已删除",
            Language::English => "Config deleted",
        }
    }

    pub fn symlink_config_added(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "配置已添加",
            Language::English => "Config added",
        }
    }

    pub fn symlink_created(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "软链接已创建",
            Language::English => "Symlink created",
        }
    }

    pub fn symlink_removed(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "软链接已删除",
            Language::English => "Symlink removed",
        }
    }

    pub fn symlink_create_failed(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "创建失败",
            Language::English => "Create failed",
        }
    }

    pub fn symlink_remove_failed(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "删除失败",
            Language::English => "Remove failed",
        }
    }

    pub fn symlink_add_failed(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "添加配置失败",
            Language::English => "Add config failed",
        }
    }

    pub fn symlink_experimental_title(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "实验性功能",
            Language::English => "Experimental Feature",
        }
    }

    pub fn symlink_experimental_desc(&self) -> &'static str {
        match self.lang {
            Language::Chinese => {
                "创建软链接后会自动同步目录下的文件到云端。点击云端上传按钮可手动同步新增文件。"
            }
            Language::English => {
                "Files in the directory are auto-synced after symlink creation. Use the cloud upload button to manually sync new files."
            }
        }
    }

    pub fn symlink_sync_files(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "同步文件到云端",
            Language::English => "Sync files to cloud",
        }
    }

    pub fn symlink_sync_success(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "同步成功",
            Language::English => "Sync successful",
        }
    }

    pub fn symlink_sync_partial(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "部分同步成功",
            Language::English => "Partially synced",
        }
    }

    pub fn symlink_sync_no_files(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "目录为空，无文件需要同步",
            Language::English => "Directory is empty, no files to sync",
        }
    }

    pub fn symlink_sync_no_manager(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "软链接管理器未初始化",
            Language::English => "Symlink manager not initialized",
        }
    }

    pub fn symlink_sync_no_steam(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "Steam 未连接，无法同步",
            Language::English => "Steam not connected, cannot sync",
        }
    }

    pub fn symlink_sync_scan_failed(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "扫描目录失败",
            Language::English => "Failed to scan directory",
        }
    }

    pub fn files(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "个文件",
            Language::English => "files",
        }
    }

    // ========== AppInfo 对话框和 UFS 配置管理 ==========

    pub fn appinfo_debug_title(&self, app_id: u32) -> String {
        match self.lang {
            Language::Chinese => format!("appinfo.vdf 调试 - App {}", app_id),
            Language::English => format!("appinfo.vdf Debug - App {}", app_id),
        }
    }

    pub fn appinfo_quota(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "配额:",
            Language::English => "Quota:",
        }
    }

    pub fn appinfo_max_files(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "最大文件数:",
            Language::English => "Max Files:",
        }
    }

    pub fn appinfo_current_ufs(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "当前 UFS 云存储配置:",
            Language::English => "Current UFS Cloud Config:",
        }
    }

    pub fn appinfo_custom_ufs(&self) -> String {
        match self.lang {
            Language::Chinese => format!("{} 自定义 UFS 配置 (实验性)", icons::GEAR),
            Language::English => format!("{} Custom UFS Config (Experimental)", icons::GEAR),
        }
    }

    pub fn appinfo_root_type(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "Root 类型:",
            Language::English => "Root Type:",
        }
    }

    pub fn appinfo_relative_path(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "相对路径:",
            Language::English => "Relative Path:",
        }
    }

    pub fn appinfo_pattern(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "文件匹配:",
            Language::English => "Pattern:",
        }
    }

    pub fn appinfo_inject(&self) -> String {
        match self.lang {
            Language::Chinese => format!("{} 注入到 appinfo.vdf", icons::EXPORT),
            Language::English => format!("{} Inject to appinfo.vdf", icons::EXPORT),
        }
    }

    pub fn appinfo_save_config(&self) -> String {
        match self.lang {
            Language::Chinese => format!("{} 保存配置", icons::SAVE),
            Language::English => format!("{} Save Config", icons::SAVE),
        }
    }

    pub fn appinfo_restart_steam(&self) -> String {
        match self.lang {
            Language::Chinese => format!("{} 重启 Steam", icons::REFRESH),
            Language::English => format!("{} Restart Steam", icons::REFRESH),
        }
    }

    pub fn appinfo_saved_configs(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "已保存的配置:",
            Language::English => "Saved Configs:",
        }
    }

    pub fn appinfo_inject_success(&self, root: &str, path: &str) -> String {
        match self.lang {
            Language::Chinese => format!("成功注入 root={} path={}", root, path),
            Language::English => format!("Success: root={} path={}", root, path),
        }
    }

    pub fn appinfo_save_success(&self, root: &str, path: &str) -> String {
        match self.lang {
            Language::Chinese => format!("已保存配置 root={} path={}", root, path),
            Language::English => format!("Saved config: root={} path={}", root, path),
        }
    }

    pub fn appinfo_warning(&self) -> String {
        match self.lang {
            Language::Chinese => {
                format!(
                    "{} 此功能为实验性质。修改 appinfo.vdf 可能被 Steam 覆盖。\n\
                     需要在 Steam 启动前注入，或在注入后立即重启 Steam。",
                    icons::WARNING
                )
            }
            Language::English => {
                format!(
                    "{} This is experimental. appinfo.vdf may be overwritten by Steam.\n\
                     Inject before Steam starts, or restart Steam after injection.",
                    icons::WARNING
                )
            }
        }
    }

    pub fn appinfo_path_hint(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "例如: MyGame/Saves",
            Language::English => "e.g. MyGame/Saves",
        }
    }

    pub fn appinfo_pattern_hint(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "* 或 *.sav",
            Language::English => "* or *.sav",
        }
    }

    pub fn appinfo_delete_config(&self) -> String {
        match self.lang {
            Language::Chinese => format!("{} 删除", icons::TRASH),
            Language::English => format!("{} Delete", icons::TRASH),
        }
    }

    pub fn appinfo_load_config(&self) -> String {
        match self.lang {
            Language::Chinese => format!("{} 加载", icons::DOWNLOAD),
            Language::English => format!("{} Load", icons::DOWNLOAD),
        }
    }

    pub fn appinfo_delete_success(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "配置已删除",
            Language::English => "Config deleted",
        }
    }

    pub fn appinfo_apply_success(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "配置已应用到注入器",
            Language::English => "Config applied to injector",
        }
    }

    pub fn appinfo_no_saved_configs(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "暂无已保存的配置",
            Language::English => "No saved configs",
        }
    }
}
