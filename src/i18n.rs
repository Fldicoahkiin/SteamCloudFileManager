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

#[allow(dead_code)]
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

    // UI 通用文本
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

    pub fn confirm(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "确认",
            Language::English => "Confirm",
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

    // 连接面板
    pub fn steam_client_status(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "Steam 客户端",
            Language::English => "Steam Client",
        }
    }

    pub fn running(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "运行中",
            Language::English => "Running",
        }
    }

    pub fn not_running(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "未运行",
            Language::English => "Not Running",
        }
    }

    pub fn api_connection(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "API 连接",
            Language::English => "API Connection",
        }
    }

    pub fn connected(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "已连接",
            Language::English => "Connected",
        }
    }

    pub fn disconnected(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "未连接",
            Language::English => "Disconnected",
        }
    }

    pub fn login_status(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "登录状态",
            Language::English => "Login Status",
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

    pub fn open_cloud_page(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "打开云存储页",
            Language::English => "Open Cloud Page",
        }
    }

    pub fn restart_steam(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "重启 Steam",
            Language::English => "Restart Steam",
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

    // 账户和游戏选择
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

    pub fn current_game(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "当前游戏",
            Language::English => "Current Game",
        }
    }

    pub fn load_files(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "加载文件",
            Language::English => "Load Files",
        }
    }

    // 文件操作
    pub fn file_list(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "文件列表",
            Language::English => "File List",
        }
    }

    pub fn list_view(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "列表",
            Language::English => "List",
        }
    }

    pub fn tree_view(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "树状",
            Language::English => "Tree",
        }
    }

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

    pub fn timestamp(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "时间戳",
            Language::English => "Timestamp",
        }
    }

    pub fn platforms(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "平台",
            Language::English => "Platforms",
        }
    }

    pub fn no_files(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "无文件",
            Language::English => "No files",
        }
    }

    pub fn files_count(&self, count: usize) -> String {
        match self.lang {
            Language::Chinese => format!("{} 个文件", count),
            Language::English => format!("{} file{}", count, if count != 1 { "s" } else { "" }),
        }
    }

    pub fn selected_count(&self, count: usize) -> String {
        match self.lang {
            Language::Chinese => format!("已选择 {} 个", count),
            Language::English => format!("{} selected", count),
        }
    }

    // 窗口标题
    pub fn select_account_title(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "选择账户",
            Language::English => "Select Account",
        }
    }

    pub fn select_game_title(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "选择游戏",
            Language::English => "Select Game",
        }
    }

    pub fn about_title(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "关于",
            Language::English => "About",
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

    pub fn error_title(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "错误",
            Language::English => "Error",
        }
    }

    // About 窗口内容
    pub fn version(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "版本",
            Language::English => "Version",
        }
    }

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

    pub fn description(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "描述",
            Language::English => "Description",
        }
    }

    pub fn app_description(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "一个用于管理 Steam 云存档的工具",
            Language::English => "A tool for managing Steam cloud saves",
        }
    }

    // 游戏选择器
    pub fn game_name(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "游戏名称",
            Language::English => "Game Name",
        }
    }

    pub fn app_id(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "应用 ID",
            Language::English => "App ID",
        }
    }

    pub fn file_count(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "文件数",
            Language::English => "Files",
        }
    }

    pub fn total_size(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "总大小",
            Language::English => "Total Size",
        }
    }

    pub fn last_played(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "最后游玩",
            Language::English => "Last Played",
        }
    }

    pub fn search_placeholder(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "搜索游戏...",
            Language::English => "Search games...",
        }
    }

    pub fn no_games_found(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "未找到游戏",
            Language::English => "No games found",
        }
    }

    // 引导对话框
    pub fn guide_title(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "操作引导",
            Language::English => "Guide",
        }
    }

    pub fn steam_restart_guide_title(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "重启 Steam 引导",
            Language::English => "Steam Restart Guide",
        }
    }

    pub fn manual_operation_guide_title(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "手动操作引导",
            Language::English => "Manual Operation Guide",
        }
    }

    // 状态消息
    pub fn loading(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "加载中...",
            Language::English => "Loading...",
        }
    }

    pub fn connecting(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "连接中...",
            Language::English => "Connecting...",
        }
    }

    pub fn downloading(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "下载中...",
            Language::English => "Downloading...",
        }
    }

    pub fn uploading(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "上传中...",
            Language::English => "Uploading...",
        }
    }

    pub fn deleting(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "删除中...",
            Language::English => "Deleting...",
        }
    }

    pub fn processing(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "处理中...",
            Language::English => "Processing...",
        }
    }

    pub fn success(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "成功",
            Language::English => "Success",
        }
    }

    pub fn failed(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "失败",
            Language::English => "Failed",
        }
    }

    // 操作确认
    pub fn confirm_delete(&self, count: usize) -> String {
        match self.lang {
            Language::Chinese => format!("确认删除 {} 个文件?", count),
            Language::English => format!(
                "Confirm delete {} file{}?",
                count,
                if count != 1 { "s" } else { "" }
            ),
        }
    }

    pub fn confirm_forget(&self, count: usize) -> String {
        match self.lang {
            Language::Chinese => format!("确认将 {} 个文件移出云端？\n（本地副本将保留）", count),
            Language::English => format!(
                "Forget {} file{} from cloud?\n(Local copy will be kept)",
                count,
                if count != 1 { "s" } else { "" }
            ),
        }
    }

    // 文件夹相关
    pub fn folder(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "文件夹",
            Language::English => "Folder",
        }
    }

    pub fn file(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "文件",
            Language::English => "File",
        }
    }

    // 游戏选择器相关
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

    pub fn never_played(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "从未游玩",
            Language::English => "Never played",
        }
    }

    // About 窗口
    pub fn check_update(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "检查更新",
            Language::English => "Check Update",
        }
    }

    pub fn checking(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "检查中...",
            Language::English => "Checking...",
        }
    }

    pub fn up_to_date(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "已是最新版本",
            Language::English => "Up to date",
        }
    }

    pub fn new_version_available(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "发现新版本",
            Language::English => "New version available",
        }
    }

    pub fn download_update(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "下载更新",
            Language::English => "Download Update",
        }
    }

    pub fn view_release_notes(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "查看更新说明",
            Language::English => "View Release Notes",
        }
    }

    pub fn license(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "许可证",
            Language::English => "License",
        }
    }

    // 用户选择器
    pub fn select_user(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "选择用户",
            Language::English => "Select User",
        }
    }

    pub fn user_name(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "用户名",
            Language::English => "Username",
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

    // About 窗口更多翻译
    pub fn checking_update(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "检查中...",
            Language::English => "Checking...",
        }
    }

    pub fn check_update_btn(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "🔄 检查更新",
            Language::English => "🔄 Check Update",
        }
    }

    pub fn already_latest(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "✅ 当前已是最新版本",
            Language::English => "✅ Already up to date",
        }
    }

    pub fn new_version_found(&self, version: &str) -> String {
        match self.lang {
            Language::Chinese => format!("🎉 发现新版本: {}", version),
            Language::English => format!("🎉 New version available: {}", version),
        }
    }

    pub fn new_version_macos_hint(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "发现新版本，macOS 需要手动安装：",
            Language::English => "New version found, manual installation required on macOS:",
        }
    }

    pub fn new_version_hint(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "发现新版本，可以进行更新操作：",
            Language::English => "New version available, you can update now:",
        }
    }

    pub fn download_package(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "📥 下载安装包",
            Language::English => "📥 Download Package",
        }
    }

    pub fn download_and_install(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "📥 下载并安装",
            Language::English => "📥 Download & Install",
        }
    }

    pub fn view_details(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "🌐 查看详情",
            Language::English => "🌐 View Details",
        }
    }

    pub fn download_location(&self, path: &str) -> String {
        match self.lang {
            Language::Chinese => format!("下载位置: {}", path),
            Language::English => format!("Download location: {}", path),
        }
    }

    pub fn downloading_update(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "📥 正在下载更新...",
            Language::English => "📥 Downloading update...",
        }
    }

    pub fn installing_update(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "⚙️ 正在安装更新...",
            Language::English => "⚙️ Installing update...",
        }
    }

    pub fn update_success(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "✅ 更新安装成功！",
            Language::English => "✅ Update installed successfully!",
        }
    }

    pub fn restart_to_apply(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "请重启应用以使用新版本",
            Language::English => "Please restart the app to use the new version",
        }
    }

    pub fn restart_now(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "🔄 立即重启",
            Language::English => "🔄 Restart Now",
        }
    }

    pub fn retry(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "🔄 重试",
            Language::English => "🔄 Retry",
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

    pub fn log_location(&self, path: &str) -> String {
        match self.lang {
            Language::Chinese => format!("日志位置: {}", path),
            Language::English => format!("Log location: {}", path),
        }
    }

    // Guide 对话框
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

    pub fn manual_restart_macos_title(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "手动重启 Steam (macOS)",
            Language::English => "Manual Restart Steam (macOS)",
        }
    }

    pub fn manual_restart_windows_title(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "手动重启 Steam (Windows)",
            Language::English => "Manual Restart Steam (Windows)",
        }
    }

    pub fn manual_restart_linux_title(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "手动重启 Steam (Linux)",
            Language::English => "Manual Restart Steam (Linux)",
        }
    }

    // Upload 对话框
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

    pub fn warning(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "⚠️ 警告：",
            Language::English => "⚠️ Warning:",
        }
    }

    pub fn overwrite_warning(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "• 同名文件将被覆盖",
            Language::English => "• Files with same name will be overwritten",
        }
    }

    pub fn add_files(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "📄 添加文件",
            Language::English => "📄 Add Files",
        }
    }

    pub fn add_folder(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "📁 添加文件夹",
            Language::English => "📁 Add Folder",
        }
    }

    pub fn confirm_upload(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "✓ 确认上传",
            Language::English => "✓ Confirm Upload",
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

    pub fn uploading_files(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "📤 正在上传文件",
            Language::English => "📤 Uploading Files",
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

    pub fn upload_complete(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "✓ 上传完成",
            Language::English => "✓ Upload Complete",
        }
    }

    pub fn upload_success(&self, count: usize) -> String {
        match self.lang {
            Language::Chinese => format!("🎉 成功上传 {} 个文件", count),
            Language::English => format!(
                "🎉 Successfully uploaded {} file{}",
                count,
                if count != 1 { "s" } else { "" }
            ),
        }
    }

    pub fn upload_partial(&self, success: usize, failed: usize) -> String {
        match self.lang {
            Language::Chinese => format!("⚠️ 上传完成：成功 {}，失败 {}", success, failed),
            Language::English => format!(
                "⚠️ Upload complete: {} succeeded, {} failed",
                success, failed
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

    pub fn select(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "选择",
            Language::English => "Select",
        }
    }

    // Steam 重启状态消息
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

    // 错误消息
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

    pub fn status_connecting(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "正在连接...",
            Language::English => "Connecting...",
        }
    }

    pub fn status_connected(&self, app_id: u32) -> String {
        match self.lang {
            Language::Chinese => format!("已连接到 App ID: {}", app_id),
            Language::English => format!("Connected to App ID: {}", app_id),
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

    // 下载相关
    pub fn download_success(&self, count: usize) -> String {
        match self.lang {
            Language::Chinese => format!("成功下载 {} 个文件", count),
            Language::English => format!(
                "Successfully downloaded {} file{}",
                count,
                if count != 1 { "s" } else { "" }
            ),
        }
    }

    pub fn download_partial(&self, success: usize, failed: usize, files: &str) -> String {
        match self.lang {
            Language::Chinese => format!(
                "下载完成：成功 {} 个，失败 {} 个\n失败文件：{}",
                success, failed, files
            ),
            Language::English => format!(
                "Download complete: {} succeeded, {} failed\nFailed files: {}",
                success, failed, files
            ),
        }
    }

    pub fn download_failed(&self, err: &str) -> String {
        match self.lang {
            Language::Chinese => format!("下载失败: {}", err),
            Language::English => format!("Download failed: {}", err),
        }
    }

    pub fn error_not_connected(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "未连接到 Steam",
            Language::English => "Not connected to Steam",
        }
    }

    pub fn error_select_files(&self, err: &str) -> String {
        match self.lang {
            Language::Chinese => format!("选择文件失败: {}", err),
            Language::English => format!("Failed to select files: {}", err),
        }
    }

    // 删除和遗忘相关
    pub fn forget_success(&self, count: usize) -> String {
        match self.lang {
            Language::Chinese => format!("已取消 {} 个文件的云同步", count),
            Language::English => format!(
                "Removed {} file{} from cloud sync",
                count,
                if count != 1 { "s" } else { "" }
            ),
        }
    }

    pub fn delete_success(&self, count: usize) -> String {
        match self.lang {
            Language::Chinese => format!("已删除 {} 个文件", count),
            Language::English => format!(
                "Deleted {} file{}",
                count,
                if count != 1 { "s" } else { "" }
            ),
        }
    }

    // 提示文本
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

    // 调试模式警告
    pub fn debug_mode_not_enabled(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "⚠ Steam 调试模式未启用",
            Language::English => "⚠ Steam Debug Mode Not Enabled",
        }
    }

    pub fn steam_running(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "✓ Steam 正在运行",
            Language::English => "✓ Steam is running",
        }
    }

    pub fn steam_not_running(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "✗ Steam 未运行",
            Language::English => "✗ Steam is not running",
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

    pub fn dismiss_temporarily(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "✕ 暂时忽略",
            Language::English => "✕ Dismiss",
        }
    }

    pub fn dismiss_hint(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "隐藏此提示（可在设置中重新显示）",
            Language::English => "Hide this hint (can be re-enabled in settings)",
        }
    }

    // 状态栏
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

    pub fn upload_tooltip(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "上传文件或文件夹",
            Language::English => "Upload file or folder",
        }
    }

    // 按钮悬停提示
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

    pub fn about_hint(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "查看软件版本和作者信息",
            Language::English => "View software version and author info",
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

    // 文件列表面板
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
            Language::Chinese => "只显示本地存在的文件",
            Language::English => "Show only files that exist locally",
        }
    }

    pub fn only_cloud_tooltip(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "只显示云端存在的文件",
            Language::English => "Show only files that exist in cloud",
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

    // 文件对比对话框
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

    pub fn action_upload_to_cloud(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "上传到云端",
            Language::English => "Upload to Cloud",
        }
    }

    pub fn action_upload_hint(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "用本地文件覆盖云端版本",
            Language::English => "Overwrite cloud version with local file",
        }
    }

    pub fn action_download_to_local(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "下载到本地",
            Language::English => "Download to Local",
        }
    }

    pub fn action_download_hint(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "用云端文件覆盖本地版本",
            Language::English => "Overwrite local version with cloud file",
        }
    }

    pub fn action_keep_both(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "保留两者",
            Language::English => "Keep Both",
        }
    }

    pub fn action_keep_both_hint(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "重命名本地文件并保留云端版本",
            Language::English => "Rename local file and keep cloud version",
        }
    }

    pub fn conflicts_warning(&self, count: usize) -> String {
        match self.lang {
            Language::Chinese => format!("检测到 {} 个冲突，请手动解决", count),
            Language::English => format!("{} conflicts detected, please resolve manually", count),
        }
    }

    pub fn upload_all_local_newer(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "上传所有本地较新",
            Language::English => "Upload All Local Newer",
        }
    }

    pub fn upload_all_hint(&self, count: usize) -> String {
        match self.lang {
            Language::Chinese => format!("将 {} 个本地较新的文件上传到云端", count),
            Language::English => format!("Upload {} local newer files to cloud", count),
        }
    }

    pub fn download_all_cloud_newer(&self) -> &'static str {
        match self.lang {
            Language::Chinese => "下载所有云端较新",
            Language::English => "Download All Cloud Newer",
        }
    }

    pub fn download_all_hint(&self, count: usize) -> String {
        match self.lang {
            Language::Chinese => format!("将 {} 个云端较新的文件下载到本地", count),
            Language::English => format!("Download {} cloud newer files to local", count),
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
}
