use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Language {
    #[default]
    English,
    Chinese,
}

impl Language {
    // 返回所有支持的语言列表
    pub const fn all() -> &'static [Language] {
        &[Language::English, Language::Chinese]
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Language::English => "English",
            Language::Chinese => "简体中文",
        }
    }

    // 从配置字符串解析语言设置
    pub fn from_config(value: &str) -> Self {
        let result = match value {
            "en" => Language::English,
            "zh" => Language::Chinese,
            "auto" => Self::detect_system_language(),
            _ => Language::English,
        };
        tracing::info!("语言配置: {} -> {:?}", value, result);
        result
    }

    // 转换为配置字符串
    pub fn to_config(self) -> &'static str {
        match self {
            Language::Chinese => "zh",
            Language::English => "en",
        }
    }

    // 检测系统语言
    pub fn detect_system_language() -> Self {
        // 尝试读取环境变量
        let lang_env = std::env::var("LANG")
            .or_else(|_| std::env::var("LC_ALL"))
            .or_else(|_| std::env::var("LC_MESSAGES"))
            .or_else(|_| std::env::var("LANGUAGE"))
            .unwrap_or_default();

        // 检查环境变量是否为中文
        if lang_env.starts_with("zh")
            || lang_env.contains("CN")
            || lang_env.contains("TW")
            || lang_env.contains("HK")
        {
            tracing::info!("系统语言检测: 中文 (环境变量: {})", lang_env);
            return Language::Chinese;
        }

        // macOS: 使用 defaults 命令检测
        #[cfg(target_os = "macos")]
        if let Ok(output) = std::process::Command::new("defaults")
            .args(["read", "-g", "AppleLanguages"])
            .output()
            && let Ok(stdout) = String::from_utf8(output.stdout)
            && stdout.contains("zh")
        {
            tracing::info!("系统语言检测: 中文 (macOS AppleLanguages)");
            return Language::Chinese;
        }

        // Windows: 使用 PowerShell 检测系统区域设置
        #[cfg(target_os = "windows")]
        if let Ok(output) = std::process::Command::new("powershell")
            .args(["-Command", "(Get-Culture).Name"])
            .output()
            && let Ok(stdout) = String::from_utf8(output.stdout)
            && stdout.starts_with("zh")
        {
            tracing::info!("系统语言检测: 中文 (Windows Culture)");
            return Language::Chinese;
        }

        // 默认使用英文
        tracing::info!("系统语言检测: 英文 (默认)");
        Language::English
    }
}

pub struct I18n {
    lang: Language,
}

mod en;
mod zh;

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

    pub fn language_label(&self) -> &'static str {
        match self.lang {
            Language::English => en::language_label(),
            Language::Chinese => zh::language_label(),
        }
    }

    pub fn app_title(&self) -> &'static str {
        match self.lang {
            Language::English => en::app_title(),
            Language::Chinese => zh::app_title(),
        }
    }

    pub fn refresh(&self) -> &'static str {
        match self.lang {
            Language::English => en::refresh(),
            Language::Chinese => zh::refresh(),
        }
    }

    pub fn cancel(&self) -> &'static str {
        match self.lang {
            Language::English => en::cancel(),
            Language::Chinese => zh::cancel(),
        }
    }

    pub fn close(&self) -> &'static str {
        match self.lang {
            Language::English => en::close(),
            Language::Chinese => zh::close(),
        }
    }

    pub fn ok(&self) -> &'static str {
        match self.lang {
            Language::English => en::ok(),
            Language::Chinese => zh::ok(),
        }
    }

    pub fn logged_in(&self) -> &'static str {
        match self.lang {
            Language::English => en::logged_in(),
            Language::Chinese => zh::logged_in(),
        }
    }

    pub fn not_logged_in(&self) -> &'static str {
        match self.lang {
            Language::English => en::not_logged_in(),
            Language::Chinese => zh::not_logged_in(),
        }
    }

    pub fn connect(&self) -> &'static str {
        match self.lang {
            Language::English => en::connect(),
            Language::Chinese => zh::connect(),
        }
    }

    pub fn disconnect(&self) -> &'static str {
        match self.lang {
            Language::English => en::disconnect(),
            Language::Chinese => zh::disconnect(),
        }
    }

    pub fn disconnect_sync_hint(&self) -> &'static str {
        match self.lang {
            Language::English => en::disconnect_sync_hint(),
            Language::Chinese => zh::disconnect_sync_hint(),
        }
    }

    pub fn refresh_open_url_hint(&self) -> &'static str {
        match self.lang {
            Language::English => en::refresh_open_url_hint(),
            Language::Chinese => zh::refresh_open_url_hint(),
        }
    }

    pub fn show_appinfo_vdf(&self) -> &'static str {
        match self.lang {
            Language::English => en::show_appinfo_vdf(),
            Language::Chinese => zh::show_appinfo_vdf(),
        }
    }

    // ========== 账户和游戏选择 ==========

    pub fn account_cloud_status(&self) -> &'static str {
        match self.lang {
            Language::English => en::account_cloud_status(),
            Language::Chinese => zh::account_cloud_status(),
        }
    }

    pub fn select_account(&self) -> &'static str {
        match self.lang {
            Language::English => en::select_account(),
            Language::Chinese => zh::select_account(),
        }
    }

    pub fn select_game(&self) -> &'static str {
        match self.lang {
            Language::English => en::select_game(),
            Language::Chinese => zh::select_game(),
        }
    }

    // ========== 文件操作 ==========

    pub fn select_all(&self) -> &'static str {
        match self.lang {
            Language::English => en::select_all(),
            Language::Chinese => zh::select_all(),
        }
    }

    pub fn invert_selection(&self) -> &'static str {
        match self.lang {
            Language::English => en::invert_selection(),
            Language::Chinese => zh::invert_selection(),
        }
    }

    pub fn clear_selection(&self) -> &'static str {
        match self.lang {
            Language::English => en::clear_selection(),
            Language::Chinese => zh::clear_selection(),
        }
    }

    pub fn download(&self) -> &'static str {
        match self.lang {
            Language::English => en::download(),
            Language::Chinese => zh::download(),
        }
    }

    pub fn upload(&self) -> &'static str {
        match self.lang {
            Language::English => en::upload(),
            Language::Chinese => zh::upload(),
        }
    }

    pub fn delete(&self) -> &'static str {
        match self.lang {
            Language::English => en::delete(),
            Language::Chinese => zh::delete(),
        }
    }

    pub fn forget(&self) -> &'static str {
        match self.lang {
            Language::English => en::forget(),
            Language::Chinese => zh::forget(),
        }
    }

    pub fn sync_to_cloud(&self) -> &'static str {
        match self.lang {
            Language::English => en::sync_to_cloud(),
            Language::Chinese => zh::sync_to_cloud(),
        }
    }

    pub fn file_name(&self) -> &'static str {
        match self.lang {
            Language::English => en::file_name(),
            Language::Chinese => zh::file_name(),
        }
    }

    pub fn size(&self) -> &'static str {
        match self.lang {
            Language::English => en::size(),
            Language::Chinese => zh::size(),
        }
    }

    pub fn selected_count(&self, count: usize) -> String {
        match self.lang {
            Language::English => en::selected_count(count),
            Language::Chinese => zh::selected_count(count),
        }
    }

    // ========== 窗口标题 ==========

    pub fn select_game_title(&self) -> &'static str {
        match self.lang {
            Language::English => en::select_game_title(),
            Language::Chinese => zh::select_game_title(),
        }
    }

    pub fn settings_title(&self) -> &'static str {
        match self.lang {
            Language::English => en::settings_title(),
            Language::Chinese => zh::settings_title(),
        }
    }

    pub fn settings_log(&self) -> &'static str {
        match self.lang {
            Language::English => en::settings_log(),
            Language::Chinese => zh::settings_log(),
        }
    }

    pub fn settings_about(&self) -> &'static str {
        match self.lang {
            Language::English => en::settings_about(),
            Language::Chinese => zh::settings_about(),
        }
    }

    pub fn settings_appearance(&self) -> &'static str {
        match self.lang {
            Language::English => en::settings_appearance(),
            Language::Chinese => zh::settings_appearance(),
        }
    }

    pub fn settings_advanced(&self) -> &'static str {
        match self.lang {
            Language::English => en::settings_advanced(),
            Language::Chinese => zh::settings_advanced(),
        }
    }

    // ========== 高级设置 ==========

    pub fn steam_path_label(&self) -> &'static str {
        match self.lang {
            Language::English => en::steam_path_label(),
            Language::Chinese => zh::steam_path_label(),
        }
    }

    pub fn steam_path_auto_detect(&self) -> &'static str {
        match self.lang {
            Language::English => en::steam_path_auto_detect(),
            Language::Chinese => zh::steam_path_auto_detect(),
        }
    }

    pub fn steam_path_browse(&self) -> &'static str {
        match self.lang {
            Language::English => en::steam_path_browse(),
            Language::Chinese => zh::steam_path_browse(),
        }
    }

    pub fn steam_path_valid(&self, user_count: usize) -> String {
        match self.lang {
            Language::English => en::steam_path_valid(user_count),
            Language::Chinese => zh::steam_path_valid(user_count),
        }
    }

    pub fn steam_path_not_exists(&self) -> &'static str {
        match self.lang {
            Language::English => en::steam_path_not_exists(),
            Language::Chinese => zh::steam_path_not_exists(),
        }
    }

    pub fn steam_path_no_userdata(&self) -> &'static str {
        match self.lang {
            Language::English => en::steam_path_no_userdata(),
            Language::Chinese => zh::steam_path_no_userdata(),
        }
    }

    pub fn steam_path_no_users(&self) -> &'static str {
        match self.lang {
            Language::English => en::steam_path_no_users(),
            Language::Chinese => zh::steam_path_no_users(),
        }
    }

    pub fn steam_path_hint(&self) -> &'static str {
        match self.lang {
            Language::English => en::steam_path_hint(),
            Language::Chinese => zh::steam_path_hint(),
        }
    }

    pub fn steam_path_restart_hint(&self) -> &'static str {
        match self.lang {
            Language::English => en::steam_path_restart_hint(),
            Language::Chinese => zh::steam_path_restart_hint(),
        }
    }

    pub fn reset_all_settings(&self) -> &'static str {
        match self.lang {
            Language::English => en::reset_all_settings(),
            Language::Chinese => zh::reset_all_settings(),
        }
    }

    pub fn reset_confirm(&self) -> &'static str {
        match self.lang {
            Language::English => en::reset_confirm(),
            Language::Chinese => zh::reset_confirm(),
        }
    }

    pub fn config_dir_label(&self) -> &'static str {
        match self.lang {
            Language::English => en::config_dir_label(),
            Language::Chinese => zh::config_dir_label(),
        }
    }

    pub fn open_config_dir(&self) -> &'static str {
        match self.lang {
            Language::English => en::open_config_dir(),
            Language::Chinese => zh::open_config_dir(),
        }
    }

    pub fn theme_mode_label(&self) -> &'static str {
        match self.lang {
            Language::English => en::theme_mode_label(),
            Language::Chinese => zh::theme_mode_label(),
        }
    }

    pub fn error_title(&self) -> &'static str {
        match self.lang {
            Language::English => en::error_title(),
            Language::Chinese => zh::error_title(),
        }
    }

    // ========== About 窗口内容 ==========

    pub fn author(&self) -> &'static str {
        match self.lang {
            Language::English => en::author(),
            Language::Chinese => zh::author(),
        }
    }

    pub fn github_repository(&self) -> &'static str {
        match self.lang {
            Language::English => en::github_repository(),
            Language::Chinese => zh::github_repository(),
        }
    }

    pub fn connecting(&self) -> &'static str {
        match self.lang {
            Language::English => en::connecting(),
            Language::Chinese => zh::connecting(),
        }
    }

    // ========== 游戏选择器相关 ==========

    pub fn games_with_cloud(&self, count: usize) -> String {
        match self.lang {
            Language::English => en::games_with_cloud(count),
            Language::Chinese => zh::games_with_cloud(count),
        }
    }

    pub fn scanning_games(&self) -> &'static str {
        match self.lang {
            Language::English => en::scanning_games(),
            Language::Chinese => zh::scanning_games(),
        }
    }

    pub fn no_cloud_games_found(&self) -> &'static str {
        match self.lang {
            Language::English => en::no_cloud_games_found(),
            Language::Chinese => zh::no_cloud_games_found(),
        }
    }

    pub fn installed(&self) -> &'static str {
        match self.lang {
            Language::English => en::installed(),
            Language::Chinese => zh::installed(),
        }
    }

    pub fn not_installed(&self) -> &'static str {
        match self.lang {
            Language::English => en::not_installed(),
            Language::Chinese => zh::not_installed(),
        }
    }

    // ========== 用户选择器 ==========

    pub fn select_user(&self) -> &'static str {
        match self.lang {
            Language::English => en::select_user(),
            Language::Chinese => zh::select_user(),
        }
    }

    pub fn user_id(&self) -> &'static str {
        match self.lang {
            Language::English => en::user_id(),
            Language::Chinese => zh::user_id(),
        }
    }

    pub fn current_user(&self) -> &'static str {
        match self.lang {
            Language::English => en::current_user(),
            Language::Chinese => zh::current_user(),
        }
    }

    pub fn switch(&self) -> &'static str {
        match self.lang {
            Language::English => en::switch(),
            Language::Chinese => zh::switch(),
        }
    }

    pub fn steam_users(&self, count: usize) -> String {
        match self.lang {
            Language::English => en::steam_users(count),
            Language::Chinese => zh::steam_users(count),
        }
    }

    // ========== About 窗口更多翻译 ==========

    pub fn checking_update(&self) -> String {
        match self.lang {
            Language::English => en::checking_update(),
            Language::Chinese => zh::checking_update(),
        }
    }

    pub fn check_update_btn(&self) -> String {
        match self.lang {
            Language::English => en::check_update_btn(),
            Language::Chinese => zh::check_update_btn(),
        }
    }

    pub fn already_latest(&self) -> String {
        match self.lang {
            Language::English => en::already_latest(),
            Language::Chinese => zh::already_latest(),
        }
    }

    pub fn new_version_found(&self, version: &str) -> String {
        match self.lang {
            Language::English => en::new_version_found(version),
            Language::Chinese => zh::new_version_found(version),
        }
    }

    pub fn new_version_hint(&self) -> &'static str {
        match self.lang {
            Language::English => en::new_version_hint(),
            Language::Chinese => zh::new_version_hint(),
        }
    }

    pub fn download_and_install(&self) -> String {
        match self.lang {
            Language::English => en::download_and_install(),
            Language::Chinese => zh::download_and_install(),
        }
    }

    pub fn view_details(&self) -> String {
        match self.lang {
            Language::English => en::view_details(),
            Language::Chinese => zh::view_details(),
        }
    }

    pub fn downloading_update(&self) -> String {
        match self.lang {
            Language::English => en::downloading_update(),
            Language::Chinese => zh::downloading_update(),
        }
    }

    pub fn installing_update(&self) -> String {
        match self.lang {
            Language::English => en::installing_update(),
            Language::Chinese => zh::installing_update(),
        }
    }

    pub fn update_success(&self) -> String {
        match self.lang {
            Language::English => en::update_success(),
            Language::Chinese => zh::update_success(),
        }
    }

    pub fn restart_to_apply(&self) -> &'static str {
        match self.lang {
            Language::English => en::restart_to_apply(),
            Language::Chinese => zh::restart_to_apply(),
        }
    }

    pub fn restart_now(&self) -> String {
        match self.lang {
            Language::English => en::restart_now(),
            Language::Chinese => zh::restart_now(),
        }
    }

    pub fn log_enabled_hint(&self) -> &'static str {
        match self.lang {
            Language::English => en::log_enabled_hint(),
            Language::Chinese => zh::log_enabled_hint(),
        }
    }

    pub fn log_disabled_hint(&self) -> &'static str {
        match self.lang {
            Language::English => en::log_disabled_hint(),
            Language::Chinese => zh::log_disabled_hint(),
        }
    }

    pub fn enable_log_storage(&self) -> &'static str {
        match self.lang {
            Language::English => en::enable_log_storage(),
            Language::Chinese => zh::enable_log_storage(),
        }
    }

    pub fn open_log_dir(&self) -> &'static str {
        match self.lang {
            Language::English => en::open_log_dir(),
            Language::Chinese => zh::open_log_dir(),
        }
    }

    pub fn log_dir_label(&self) -> &'static str {
        match self.lang {
            Language::English => en::log_dir_label(),
            Language::Chinese => zh::log_dir_label(),
        }
    }

    pub fn steam_log_dir_label(&self) -> &'static str {
        match self.lang {
            Language::English => en::steam_log_dir_label(),
            Language::Chinese => zh::steam_log_dir_label(),
        }
    }

    pub fn open_steam_log_dir(&self) -> &'static str {
        match self.lang {
            Language::English => en::open_steam_log_dir(),
            Language::Chinese => zh::open_steam_log_dir(),
        }
    }

    // ========== Guide 对话框 ==========

    pub fn restarting_steam(&self) -> &'static str {
        match self.lang {
            Language::English => en::restarting_steam(),
            Language::Chinese => zh::restarting_steam(),
        }
    }

    pub fn manual_operation_required(&self) -> &'static str {
        match self.lang {
            Language::English => en::manual_operation_required(),
            Language::Chinese => zh::manual_operation_required(),
        }
    }

    pub fn i_understand(&self) -> &'static str {
        match self.lang {
            Language::English => en::i_understand(),
            Language::Chinese => zh::i_understand(),
        }
    }

    #[cfg(target_os = "macos")]
    pub fn manual_restart_macos_title(&self) -> &'static str {
        match self.lang {
            Language::English => en::manual_restart_macos_title(),
            Language::Chinese => zh::manual_restart_macos_title(),
        }
    }

    #[cfg(target_os = "windows")]
    pub fn manual_restart_windows_title(&self) -> &'static str {
        match self.lang {
            Language::English => en::manual_restart_windows_title(),
            Language::Chinese => zh::manual_restart_windows_title(),
        }
    }

    #[cfg(target_os = "linux")]
    pub fn manual_restart_linux_title(&self) -> &'static str {
        match self.lang {
            Language::English => en::manual_restart_linux_title(),
            Language::Chinese => zh::manual_restart_linux_title(),
        }
    }

    // ========== Upload 对话框 ==========

    pub fn prepare_upload(&self) -> &'static str {
        match self.lang {
            Language::English => en::prepare_upload(),
            Language::Chinese => zh::prepare_upload(),
        }
    }

    pub fn will_upload_files(&self, count: usize) -> String {
        match self.lang {
            Language::English => en::will_upload_files(count),
            Language::Chinese => zh::will_upload_files(count),
        }
    }

    pub fn total_size_label(&self, size: &str) -> String {
        match self.lang {
            Language::English => en::total_size_label(size),
            Language::Chinese => zh::total_size_label(size),
        }
    }

    pub fn warning(&self) -> String {
        match self.lang {
            Language::English => en::warning(),
            Language::Chinese => zh::warning(),
        }
    }

    pub fn overwrite_warning(&self) -> &'static str {
        match self.lang {
            Language::English => en::overwrite_warning(),
            Language::Chinese => zh::overwrite_warning(),
        }
    }

    pub fn add_files(&self) -> String {
        match self.lang {
            Language::English => en::add_files(),
            Language::Chinese => zh::add_files(),
        }
    }

    pub fn add_folder(&self) -> String {
        match self.lang {
            Language::English => en::add_folder(),
            Language::Chinese => zh::add_folder(),
        }
    }

    pub fn confirm_upload(&self) -> String {
        match self.lang {
            Language::English => en::confirm_upload(),
            Language::Chinese => zh::confirm_upload(),
        }
    }

    pub fn remove_file(&self) -> &'static str {
        match self.lang {
            Language::English => en::remove_file(),
            Language::Chinese => zh::remove_file(),
        }
    }

    pub fn cloud_path(&self) -> &'static str {
        match self.lang {
            Language::English => en::cloud_path(),
            Language::Chinese => zh::cloud_path(),
        }
    }

    pub fn edit_path(&self) -> &'static str {
        match self.lang {
            Language::English => en::edit_path(),
            Language::Chinese => zh::edit_path(),
        }
    }

    pub fn local_file(&self) -> &'static str {
        match self.lang {
            Language::English => en::local_file(),
            Language::Chinese => zh::local_file(),
        }
    }

    pub fn no_files_to_upload(&self) -> &'static str {
        match self.lang {
            Language::English => en::no_files_to_upload(),
            Language::Chinese => zh::no_files_to_upload(),
        }
    }

    pub fn clear_all(&self) -> &'static str {
        match self.lang {
            Language::English => en::clear_all(),
            Language::Chinese => zh::clear_all(),
        }
    }

    pub fn uploading_files(&self) -> String {
        match self.lang {
            Language::English => en::uploading_files(),
            Language::Chinese => zh::uploading_files(),
        }
    }

    pub fn uploading_file(&self, name: &str) -> String {
        match self.lang {
            Language::English => en::uploading_file(name),
            Language::Chinese => zh::uploading_file(name),
        }
    }

    pub fn upload_progress(&self, current: usize, total: usize) -> String {
        match self.lang {
            Language::English => en::upload_progress(current, total),
            Language::Chinese => zh::upload_progress(current, total),
        }
    }

    pub fn speed(&self, speed: &str) -> String {
        match self.lang {
            Language::English => en::speed(speed),
            Language::Chinese => zh::speed(speed),
        }
    }

    pub fn upload_complete(&self) -> String {
        match self.lang {
            Language::English => en::upload_complete(),
            Language::Chinese => zh::upload_complete(),
        }
    }

    pub fn upload_success(&self, count: usize) -> String {
        match self.lang {
            Language::English => en::upload_success(count),
            Language::Chinese => zh::upload_success(count),
        }
    }

    pub fn upload_partial(&self, success: usize, failed: usize) -> String {
        match self.lang {
            Language::English => en::upload_partial(success, failed),
            Language::Chinese => zh::upload_partial(success, failed),
        }
    }

    pub fn elapsed_time(&self, secs: u64) -> String {
        match self.lang {
            Language::English => en::elapsed_time(secs),
            Language::Chinese => zh::elapsed_time(secs),
        }
    }

    pub fn avg_speed(&self, speed: &str) -> String {
        match self.lang {
            Language::English => en::avg_speed(speed),
            Language::Chinese => zh::avg_speed(speed),
        }
    }

    pub fn failed_files(&self) -> &'static str {
        match self.lang {
            Language::English => en::failed_files(),
            Language::Chinese => zh::failed_files(),
        }
    }

    pub fn reason(&self, err: &str) -> String {
        match self.lang {
            Language::English => en::reason(err),
            Language::Chinese => zh::reason(err),
        }
    }

    // ========== Steam 重启状态消息 ==========

    pub fn closing_steam(&self) -> &'static str {
        match self.lang {
            Language::English => en::closing_steam(),
            Language::Chinese => zh::closing_steam(),
        }
    }

    pub fn starting_steam(&self) -> &'static str {
        match self.lang {
            Language::English => en::starting_steam(),
            Language::Chinese => zh::starting_steam(),
        }
    }

    pub fn steam_restart_success(&self) -> &'static str {
        match self.lang {
            Language::English => en::steam_restart_success(),
            Language::Chinese => zh::steam_restart_success(),
        }
    }

    pub fn user_switched(&self) -> &'static str {
        match self.lang {
            Language::English => en::user_switched(),
            Language::Chinese => zh::user_switched(),
        }
    }

    // ========== 错误消息 ==========

    pub fn error_enter_app_id(&self) -> &'static str {
        match self.lang {
            Language::English => en::error_enter_app_id(),
            Language::Chinese => zh::error_enter_app_id(),
        }
    }

    pub fn error_invalid_app_id(&self) -> &'static str {
        match self.lang {
            Language::English => en::error_invalid_app_id(),
            Language::Chinese => zh::error_invalid_app_id(),
        }
    }

    pub fn status_enter_app_id(&self) -> &'static str {
        match self.lang {
            Language::English => en::status_enter_app_id(),
            Language::Chinese => zh::status_enter_app_id(),
        }
    }

    pub fn status_loading_files(&self) -> &'static str {
        match self.lang {
            Language::English => en::status_loading_files(),
            Language::Chinese => zh::status_loading_files(),
        }
    }

    pub fn status_files_loaded(&self, count: usize) -> String {
        match self.lang {
            Language::English => en::status_files_loaded(count),
            Language::Chinese => zh::status_files_loaded(count),
        }
    }

    pub fn upload_failed(&self, err: &str) -> String {
        match self.lang {
            Language::English => en::upload_failed(err),
            Language::Chinese => zh::upload_failed(err),
        }
    }

    pub fn error_no_files_selected(&self) -> &'static str {
        match self.lang {
            Language::English => en::error_no_files_selected(),
            Language::Chinese => zh::error_no_files_selected(),
        }
    }

    pub fn error_not_connected(&self) -> &'static str {
        match self.lang {
            Language::English => en::error_not_connected(),
            Language::Chinese => zh::error_not_connected(),
        }
    }

    // ========== 提示文本 ==========

    pub fn hint_you_can(&self) -> &'static str {
        match self.lang {
            Language::English => en::hint_you_can(),
            Language::Chinese => zh::hint_you_can(),
        }
    }

    pub fn hint_select_game(&self) -> &'static str {
        match self.lang {
            Language::English => en::hint_select_game(),
            Language::Chinese => zh::hint_select_game(),
        }
    }

    pub fn hint_enter_app_id(&self) -> &'static str {
        match self.lang {
            Language::English => en::hint_enter_app_id(),
            Language::Chinese => zh::hint_enter_app_id(),
        }
    }

    pub fn no_cloud_files(&self) -> &'static str {
        match self.lang {
            Language::English => en::no_cloud_files(),
            Language::Chinese => zh::no_cloud_files(),
        }
    }

    pub fn no_cloud_files_hint(&self) -> &'static str {
        match self.lang {
            Language::English => en::no_cloud_files_hint(),
            Language::Chinese => zh::no_cloud_files_hint(),
        }
    }

    pub fn scan_games_failed(&self, err: &str) -> String {
        match self.lang {
            Language::English => en::scan_games_failed(err),
            Language::Chinese => zh::scan_games_failed(err),
        }
    }

    pub fn refresh_files_failed(&self, err: &str) -> String {
        match self.lang {
            Language::English => en::refresh_files_failed(err),
            Language::Chinese => zh::refresh_files_failed(err),
        }
    }

    pub fn cdp_no_data_error(&self) -> &'static str {
        match self.lang {
            Language::English => en::cdp_no_data_error(),
            Language::Chinese => zh::cdp_no_data_error(),
        }
    }

    pub fn connecting_to_steam(&self, app_id: u32) -> String {
        match self.lang {
            Language::English => en::connecting_to_steam(app_id),
            Language::Chinese => zh::connecting_to_steam(app_id),
        }
    }

    pub fn loading_files_for_app(&self, app_id: u32) -> String {
        match self.lang {
            Language::English => en::loading_files_for_app(app_id),
            Language::Chinese => zh::loading_files_for_app(app_id),
        }
    }

    pub fn connect_steam_failed(&self, err: &str) -> String {
        match self.lang {
            Language::English => en::connect_steam_failed(err),
            Language::Chinese => zh::connect_steam_failed(err),
        }
    }

    pub fn vdf_parser_not_initialized(&self) -> &'static str {
        match self.lang {
            Language::English => en::vdf_parser_not_initialized(),
            Language::Chinese => zh::vdf_parser_not_initialized(),
        }
    }

    pub fn scanning_game_library(&self) -> &'static str {
        match self.lang {
            Language::English => en::scanning_game_library(),
            Language::Chinese => zh::scanning_game_library(),
        }
    }

    pub fn drop_files_to_upload(&self) -> &'static str {
        match self.lang {
            Language::English => en::drop_files_to_upload(),
            Language::Chinese => zh::drop_files_to_upload(),
        }
    }

    // ========== 调试模式警告 ==========

    pub fn debug_mode_not_enabled(&self) -> String {
        match self.lang {
            Language::English => en::debug_mode_not_enabled(),
            Language::Chinese => zh::debug_mode_not_enabled(),
        }
    }

    pub fn steam_running(&self) -> String {
        match self.lang {
            Language::English => en::steam_running(),
            Language::Chinese => zh::steam_running(),
        }
    }

    pub fn steam_not_running(&self) -> String {
        match self.lang {
            Language::English => en::steam_not_running(),
            Language::Chinese => zh::steam_not_running(),
        }
    }

    pub fn debug_mode_hint(&self) -> &'static str {
        match self.lang {
            Language::English => en::debug_mode_hint(),
            Language::Chinese => zh::debug_mode_hint(),
        }
    }

    pub fn auto_restart_steam(&self) -> &'static str {
        match self.lang {
            Language::English => en::auto_restart_steam(),
            Language::Chinese => zh::auto_restart_steam(),
        }
    }

    pub fn start_steam(&self) -> &'static str {
        match self.lang {
            Language::English => en::start_steam(),
            Language::Chinese => zh::start_steam(),
        }
    }

    pub fn auto_restart_hint(&self) -> &'static str {
        match self.lang {
            Language::English => en::auto_restart_hint(),
            Language::Chinese => zh::auto_restart_hint(),
        }
    }

    pub fn start_steam_hint(&self) -> &'static str {
        match self.lang {
            Language::English => en::start_steam_hint(),
            Language::Chinese => zh::start_steam_hint(),
        }
    }

    pub fn view_manual_steps(&self) -> &'static str {
        match self.lang {
            Language::English => en::view_manual_steps(),
            Language::Chinese => zh::view_manual_steps(),
        }
    }

    pub fn manual_steps_hint(&self) -> &'static str {
        match self.lang {
            Language::English => en::manual_steps_hint(),
            Language::Chinese => zh::manual_steps_hint(),
        }
    }

    pub fn dismiss_temporarily(&self) -> String {
        match self.lang {
            Language::English => en::dismiss_temporarily(),
            Language::Chinese => zh::dismiss_temporarily(),
        }
    }

    pub fn dismiss_hint(&self) -> &'static str {
        match self.lang {
            Language::English => en::dismiss_hint(),
            Language::Chinese => zh::dismiss_hint(),
        }
    }

    // ========== 状态栏 ==========

    pub fn status_label(&self) -> &'static str {
        match self.lang {
            Language::English => en::status_label(),
            Language::Chinese => zh::status_label(),
        }
    }

    pub fn cloud_on(&self) -> &'static str {
        match self.lang {
            Language::English => en::cloud_on(),
            Language::Chinese => zh::cloud_on(),
        }
    }

    pub fn cloud_off(&self) -> &'static str {
        match self.lang {
            Language::English => en::cloud_off(),
            Language::Chinese => zh::cloud_off(),
        }
    }

    pub fn quota_usage(&self, percent: f32, used: &str, total: &str) -> String {
        match self.lang {
            Language::English => en::quota_usage(percent, used, total),
            Language::Chinese => zh::quota_usage(percent, used, total),
        }
    }

    // ========== 按钮悬停提示 ==========

    pub fn select_all_hint(&self) -> &'static str {
        match self.lang {
            Language::English => en::select_all_hint(),
            Language::Chinese => zh::select_all_hint(),
        }
    }

    pub fn invert_selection_hint(&self) -> &'static str {
        match self.lang {
            Language::English => en::invert_selection_hint(),
            Language::Chinese => zh::invert_selection_hint(),
        }
    }

    pub fn clear_selection_hint(&self) -> &'static str {
        match self.lang {
            Language::English => en::clear_selection_hint(),
            Language::Chinese => zh::clear_selection_hint(),
        }
    }

    pub fn download_hint(&self) -> &'static str {
        match self.lang {
            Language::English => en::download_hint(),
            Language::Chinese => zh::download_hint(),
        }
    }

    pub fn upload_hint(&self) -> &'static str {
        match self.lang {
            Language::English => en::upload_hint(),
            Language::Chinese => zh::upload_hint(),
        }
    }

    pub fn delete_hint(&self) -> &'static str {
        match self.lang {
            Language::English => en::delete_hint(),
            Language::Chinese => zh::delete_hint(),
        }
    }

    pub fn forget_hint(&self) -> &'static str {
        match self.lang {
            Language::English => en::forget_hint(),
            Language::Chinese => zh::forget_hint(),
        }
    }

    pub fn sync_to_cloud_hint(&self) -> &'static str {
        match self.lang {
            Language::English => en::sync_to_cloud_hint(),
            Language::Chinese => zh::sync_to_cloud_hint(),
        }
    }

    pub fn connect_hint(&self) -> &'static str {
        match self.lang {
            Language::English => en::connect_hint(),
            Language::Chinese => zh::connect_hint(),
        }
    }

    pub fn disconnect_hint(&self) -> &'static str {
        match self.lang {
            Language::English => en::disconnect_hint(),
            Language::Chinese => zh::disconnect_hint(),
        }
    }

    pub fn select_account_hint(&self) -> &'static str {
        match self.lang {
            Language::English => en::select_account_hint(),
            Language::Chinese => zh::select_account_hint(),
        }
    }

    pub fn select_game_hint(&self) -> &'static str {
        match self.lang {
            Language::English => en::select_game_hint(),
            Language::Chinese => zh::select_game_hint(),
        }
    }

    // ========== 文件列表面板 ==========

    pub fn local_save_path(&self) -> &'static str {
        match self.lang {
            Language::English => en::local_save_path(),
            Language::Chinese => zh::local_save_path(),
        }
    }

    pub fn local_save_path_not_found(&self) -> &'static str {
        match self.lang {
            Language::English => en::local_save_path_not_found(),
            Language::Chinese => zh::local_save_path_not_found(),
        }
    }

    pub fn folder_not_exist(&self, path: &str) -> String {
        match self.lang {
            Language::English => en::folder_not_exist(path),
            Language::Chinese => zh::folder_not_exist(path),
        }
    }

    pub fn search_files_placeholder(&self) -> &'static str {
        match self.lang {
            Language::English => en::search_files_placeholder(),
            Language::Chinese => zh::search_files_placeholder(),
        }
    }

    pub fn clear(&self) -> &'static str {
        match self.lang {
            Language::English => en::clear(),
            Language::Chinese => zh::clear(),
        }
    }

    pub fn only_local(&self) -> &'static str {
        match self.lang {
            Language::English => en::only_local(),
            Language::Chinese => zh::only_local(),
        }
    }

    pub fn only_cloud(&self) -> &'static str {
        match self.lang {
            Language::English => en::only_cloud(),
            Language::Chinese => zh::only_cloud(),
        }
    }

    pub fn only_local_tooltip(&self) -> &'static str {
        match self.lang {
            Language::English => en::only_local_tooltip(),
            Language::Chinese => zh::only_local_tooltip(),
        }
    }

    pub fn only_cloud_tooltip(&self) -> &'static str {
        match self.lang {
            Language::English => en::only_cloud_tooltip(),
            Language::Chinese => zh::only_cloud_tooltip(),
        }
    }

    pub fn root_folder(&self) -> &'static str {
        match self.lang {
            Language::English => en::root_folder(),
            Language::Chinese => zh::root_folder(),
        }
    }

    pub fn file_size(&self) -> &'static str {
        match self.lang {
            Language::English => en::file_size(),
            Language::Chinese => zh::file_size(),
        }
    }

    pub fn write_date(&self) -> &'static str {
        match self.lang {
            Language::English => en::write_date(),
            Language::Chinese => zh::write_date(),
        }
    }

    pub fn local(&self) -> &'static str {
        match self.lang {
            Language::English => en::local(),
            Language::Chinese => zh::local(),
        }
    }

    pub fn cloud(&self) -> &'static str {
        match self.lang {
            Language::English => en::cloud(),
            Language::Chinese => zh::cloud(),
        }
    }

    // ========== 文件对比对话框 ==========

    pub fn file_comparison_title(&self) -> &'static str {
        match self.lang {
            Language::English => en::file_comparison_title(),
            Language::Chinese => zh::file_comparison_title(),
        }
    }

    pub fn total_files_count(&self, count: usize) -> String {
        match self.lang {
            Language::English => en::total_files_count(count),
            Language::Chinese => zh::total_files_count(count),
        }
    }

    pub fn filter_all(&self) -> &'static str {
        match self.lang {
            Language::English => en::filter_all(),
            Language::Chinese => zh::filter_all(),
        }
    }

    pub fn filter_conflicts(&self) -> &'static str {
        match self.lang {
            Language::English => en::filter_conflicts(),
            Language::Chinese => zh::filter_conflicts(),
        }
    }

    pub fn filter_local_newer(&self) -> &'static str {
        match self.lang {
            Language::English => en::filter_local_newer(),
            Language::Chinese => zh::filter_local_newer(),
        }
    }

    pub fn filter_cloud_newer(&self) -> &'static str {
        match self.lang {
            Language::English => en::filter_cloud_newer(),
            Language::Chinese => zh::filter_cloud_newer(),
        }
    }

    pub fn filter_synced(&self) -> &'static str {
        match self.lang {
            Language::English => en::filter_synced(),
            Language::Chinese => zh::filter_synced(),
        }
    }

    pub fn status_local_newer(&self) -> &'static str {
        match self.lang {
            Language::English => en::status_local_newer(),
            Language::Chinese => zh::status_local_newer(),
        }
    }

    pub fn status_cloud_newer(&self) -> &'static str {
        match self.lang {
            Language::English => en::status_cloud_newer(),
            Language::Chinese => zh::status_cloud_newer(),
        }
    }

    pub fn status_conflict(&self) -> &'static str {
        match self.lang {
            Language::English => en::status_conflict(),
            Language::Chinese => zh::status_conflict(),
        }
    }

    pub fn status_local_only(&self) -> &'static str {
        match self.lang {
            Language::English => en::status_local_only(),
            Language::Chinese => zh::status_local_only(),
        }
    }

    pub fn status_cloud_only(&self) -> &'static str {
        match self.lang {
            Language::English => en::status_cloud_only(),
            Language::Chinese => zh::status_cloud_only(),
        }
    }

    pub fn column_status(&self) -> &'static str {
        match self.lang {
            Language::English => en::column_status(),
            Language::Chinese => zh::column_status(),
        }
    }

    pub fn column_filename(&self) -> &'static str {
        match self.lang {
            Language::English => en::column_filename(),
            Language::Chinese => zh::column_filename(),
        }
    }

    pub fn column_local_size(&self) -> &'static str {
        match self.lang {
            Language::English => en::column_local_size(),
            Language::Chinese => zh::column_local_size(),
        }
    }

    pub fn column_cloud_size(&self) -> &'static str {
        match self.lang {
            Language::English => en::column_cloud_size(),
            Language::Chinese => zh::column_cloud_size(),
        }
    }

    pub fn column_local_time(&self) -> &'static str {
        match self.lang {
            Language::English => en::column_local_time(),
            Language::Chinese => zh::column_local_time(),
        }
    }

    pub fn column_cloud_time(&self) -> &'static str {
        match self.lang {
            Language::English => en::column_cloud_time(),
            Language::Chinese => zh::column_cloud_time(),
        }
    }

    pub fn selected_file(&self) -> &'static str {
        match self.lang {
            Language::English => en::selected_file(),
            Language::Chinese => zh::selected_file(),
        }
    }

    pub fn local_newer_by_minutes(&self, mins: i64) -> String {
        match self.lang {
            Language::English => en::local_newer_by_minutes(mins),
            Language::Chinese => zh::local_newer_by_minutes(mins),
        }
    }

    pub fn cloud_newer_by_minutes(&self, mins: i64) -> String {
        match self.lang {
            Language::English => en::cloud_newer_by_minutes(mins),
            Language::Chinese => zh::cloud_newer_by_minutes(mins),
        }
    }

    pub fn conflicts_warning(&self, count: usize) -> String {
        match self.lang {
            Language::English => en::conflicts_warning(count),
            Language::Chinese => zh::conflicts_warning(count),
        }
    }

    pub fn compare_files(&self) -> &'static str {
        match self.lang {
            Language::English => en::compare_files(),
            Language::Chinese => zh::compare_files(),
        }
    }

    pub fn compare_files_hint(&self) -> &'static str {
        match self.lang {
            Language::English => en::compare_files_hint(),
            Language::Chinese => zh::compare_files_hint(),
        }
    }

    // ========== 备份功能 ==========

    pub fn backup(&self) -> &'static str {
        match self.lang {
            Language::English => en::backup(),
            Language::Chinese => zh::backup(),
        }
    }

    pub fn backup_title(&self) -> &'static str {
        match self.lang {
            Language::English => en::backup_title(),
            Language::Chinese => zh::backup_title(),
        }
    }

    pub fn backup_file_count(&self, count: usize) -> String {
        match self.lang {
            Language::English => en::backup_file_count(count),
            Language::Chinese => zh::backup_file_count(count),
        }
    }

    pub fn backup_total_size(&self, size: &str) -> String {
        match self.lang {
            Language::English => en::backup_total_size(size),
            Language::Chinese => zh::backup_total_size(size),
        }
    }

    pub fn backup_cdp_warning(&self, count: usize) -> String {
        match self.lang {
            Language::English => en::backup_cdp_warning(count),
            Language::Chinese => zh::backup_cdp_warning(count),
        }
    }

    pub fn backup_file_list(&self) -> &'static str {
        match self.lang {
            Language::English => en::backup_file_list(),
            Language::Chinese => zh::backup_file_list(),
        }
    }

    pub fn backup_start(&self) -> &'static str {
        match self.lang {
            Language::English => en::backup_start(),
            Language::Chinese => zh::backup_start(),
        }
    }

    pub fn backup_open_dir(&self) -> &'static str {
        match self.lang {
            Language::English => en::backup_open_dir(),
            Language::Chinese => zh::backup_open_dir(),
        }
    }

    pub fn backup_progress_title(&self) -> &'static str {
        match self.lang {
            Language::English => en::backup_progress_title(),
            Language::Chinese => zh::backup_progress_title(),
        }
    }

    pub fn backup_in_progress(&self) -> &'static str {
        match self.lang {
            Language::English => en::backup_in_progress(),
            Language::Chinese => zh::backup_in_progress(),
        }
    }

    pub fn backup_complete(&self) -> String {
        match self.lang {
            Language::English => en::backup_complete(),
            Language::Chinese => zh::backup_complete(),
        }
    }

    pub fn backup_partial(&self) -> String {
        match self.lang {
            Language::English => en::backup_partial(),
            Language::Chinese => zh::backup_partial(),
        }
    }

    pub fn backup_result_stats(&self, success: usize, total: usize) -> String {
        match self.lang {
            Language::English => en::backup_result_stats(success, total),
            Language::Chinese => zh::backup_result_stats(success, total),
        }
    }

    pub fn backup_failed_files(&self) -> &'static str {
        match self.lang {
            Language::English => en::backup_failed_files(),
            Language::Chinese => zh::backup_failed_files(),
        }
    }

    pub fn backup_hint(&self) -> &'static str {
        match self.lang {
            Language::English => en::backup_hint(),
            Language::Chinese => zh::backup_hint(),
        }
    }

    pub fn backup_dir_label(&self) -> &'static str {
        match self.lang {
            Language::English => en::backup_dir_label(),
            Language::Chinese => zh::backup_dir_label(),
        }
    }

    // ========== 下载相关 ==========

    pub fn download_progress_title(&self) -> &'static str {
        match self.lang {
            Language::English => en::download_progress_title(),
            Language::Chinese => zh::download_progress_title(),
        }
    }

    pub fn download_in_progress(&self) -> &'static str {
        match self.lang {
            Language::English => en::download_in_progress(),
            Language::Chinese => zh::download_in_progress(),
        }
    }

    pub fn download_complete(&self) -> String {
        match self.lang {
            Language::English => en::download_complete(),
            Language::Chinese => zh::download_complete(),
        }
    }

    pub fn download_partial_status(&self) -> String {
        match self.lang {
            Language::English => en::download_partial_status(),
            Language::Chinese => zh::download_partial_status(),
        }
    }

    pub fn download_result_stats(&self, success: usize, total: usize) -> String {
        match self.lang {
            Language::English => en::download_result_stats(success, total),
            Language::Chinese => zh::download_result_stats(success, total),
        }
    }

    pub fn download_failed_files(&self) -> &'static str {
        match self.lang {
            Language::English => en::download_failed_files(),
            Language::Chinese => zh::download_failed_files(),
        }
    }

    pub fn download_open_dir(&self) -> &'static str {
        match self.lang {
            Language::English => en::download_open_dir(),
            Language::Chinese => zh::download_open_dir(),
        }
    }

    // ========== 软链接功能 ==========

    pub fn symlink_title(&self) -> &'static str {
        match self.lang {
            Language::English => en::symlink_title(),
            Language::Chinese => zh::symlink_title(),
        }
    }

    pub fn symlink_configured_links(&self) -> &'static str {
        match self.lang {
            Language::English => en::symlink_configured_links(),
            Language::Chinese => zh::symlink_configured_links(),
        }
    }

    pub fn symlink_no_configs(&self) -> &'static str {
        match self.lang {
            Language::English => en::symlink_no_configs(),
            Language::Chinese => zh::symlink_no_configs(),
        }
    }

    pub fn symlink_add_new(&self) -> &'static str {
        match self.lang {
            Language::English => en::symlink_add_new(),
            Language::Chinese => zh::symlink_add_new(),
        }
    }

    pub fn symlink_direction(&self) -> &'static str {
        match self.lang {
            Language::English => en::symlink_direction(),
            Language::Chinese => zh::symlink_direction(),
        }
    }

    pub fn symlink_local_path(&self) -> &'static str {
        match self.lang {
            Language::English => en::symlink_local_path(),
            Language::Chinese => zh::symlink_local_path(),
        }
    }

    pub fn symlink_remote_subfolder(&self) -> &'static str {
        match self.lang {
            Language::English => en::symlink_remote_subfolder(),
            Language::Chinese => zh::symlink_remote_subfolder(),
        }
    }

    pub fn symlink_browse(&self) -> &'static str {
        match self.lang {
            Language::English => en::symlink_browse(),
            Language::Chinese => zh::symlink_browse(),
        }
    }

    pub fn symlink_add_config(&self) -> &'static str {
        match self.lang {
            Language::English => en::symlink_add_config(),
            Language::Chinese => zh::symlink_add_config(),
        }
    }

    pub fn symlink_add_and_create(&self) -> &'static str {
        match self.lang {
            Language::English => en::symlink_add_and_create(),
            Language::Chinese => zh::symlink_add_and_create(),
        }
    }

    pub fn symlink_create(&self) -> &'static str {
        match self.lang {
            Language::English => en::symlink_create(),
            Language::Chinese => zh::symlink_create(),
        }
    }

    pub fn symlink_remove_link(&self) -> &'static str {
        match self.lang {
            Language::English => en::symlink_remove_link(),
            Language::Chinese => zh::symlink_remove_link(),
        }
    }

    pub fn symlink_delete_config(&self) -> &'static str {
        match self.lang {
            Language::English => en::symlink_delete_config(),
            Language::Chinese => zh::symlink_delete_config(),
        }
    }

    pub fn symlink_copy_command(&self) -> &'static str {
        match self.lang {
            Language::English => en::symlink_copy_command(),
            Language::Chinese => zh::symlink_copy_command(),
        }
    }

    pub fn symlink_refresh(&self) -> &'static str {
        match self.lang {
            Language::English => en::symlink_refresh(),
            Language::Chinese => zh::symlink_refresh(),
        }
    }

    pub fn symlink_command_copied(&self) -> &'static str {
        match self.lang {
            Language::English => en::symlink_command_copied(),
            Language::Chinese => zh::symlink_command_copied(),
        }
    }

    pub fn symlink_config_deleted(&self) -> &'static str {
        match self.lang {
            Language::English => en::symlink_config_deleted(),
            Language::Chinese => zh::symlink_config_deleted(),
        }
    }

    pub fn symlink_config_added(&self) -> &'static str {
        match self.lang {
            Language::English => en::symlink_config_added(),
            Language::Chinese => zh::symlink_config_added(),
        }
    }

    pub fn symlink_created(&self) -> &'static str {
        match self.lang {
            Language::English => en::symlink_created(),
            Language::Chinese => zh::symlink_created(),
        }
    }

    pub fn symlink_removed(&self) -> &'static str {
        match self.lang {
            Language::English => en::symlink_removed(),
            Language::Chinese => zh::symlink_removed(),
        }
    }

    pub fn symlink_create_failed(&self) -> &'static str {
        match self.lang {
            Language::English => en::symlink_create_failed(),
            Language::Chinese => zh::symlink_create_failed(),
        }
    }

    pub fn symlink_remove_failed(&self) -> &'static str {
        match self.lang {
            Language::English => en::symlink_remove_failed(),
            Language::Chinese => zh::symlink_remove_failed(),
        }
    }

    pub fn symlink_add_failed(&self) -> &'static str {
        match self.lang {
            Language::English => en::symlink_add_failed(),
            Language::Chinese => zh::symlink_add_failed(),
        }
    }

    pub fn symlink_experimental_title(&self) -> &'static str {
        match self.lang {
            Language::English => en::symlink_experimental_title(),
            Language::Chinese => zh::symlink_experimental_title(),
        }
    }

    pub fn symlink_experimental_desc(&self) -> &'static str {
        match self.lang {
            Language::English => en::symlink_experimental_desc(),
            Language::Chinese => zh::symlink_experimental_desc(),
        }
    }

    pub fn symlink_sync_files(&self) -> &'static str {
        match self.lang {
            Language::English => en::symlink_sync_files(),
            Language::Chinese => zh::symlink_sync_files(),
        }
    }

    pub fn symlink_sync_success(&self) -> &'static str {
        match self.lang {
            Language::English => en::symlink_sync_success(),
            Language::Chinese => zh::symlink_sync_success(),
        }
    }

    pub fn symlink_sync_partial(&self) -> &'static str {
        match self.lang {
            Language::English => en::symlink_sync_partial(),
            Language::Chinese => zh::symlink_sync_partial(),
        }
    }

    pub fn symlink_sync_no_files(&self) -> &'static str {
        match self.lang {
            Language::English => en::symlink_sync_no_files(),
            Language::Chinese => zh::symlink_sync_no_files(),
        }
    }

    pub fn symlink_sync_no_manager(&self) -> &'static str {
        match self.lang {
            Language::English => en::symlink_sync_no_manager(),
            Language::Chinese => zh::symlink_sync_no_manager(),
        }
    }

    pub fn symlink_sync_no_steam(&self) -> &'static str {
        match self.lang {
            Language::English => en::symlink_sync_no_steam(),
            Language::Chinese => zh::symlink_sync_no_steam(),
        }
    }

    pub fn symlink_sync_scan_failed(&self) -> &'static str {
        match self.lang {
            Language::English => en::symlink_sync_scan_failed(),
            Language::Chinese => zh::symlink_sync_scan_failed(),
        }
    }

    pub fn files(&self) -> &'static str {
        match self.lang {
            Language::English => en::files(),
            Language::Chinese => zh::files(),
        }
    }

    // ========== AppInfo 对话框和 UFS 配置管理 ==========

    pub fn appinfo_tab_local_ufs(&self) -> &'static str {
        match self.lang {
            Language::English => en::appinfo_tab_local_ufs(),
            Language::Chinese => zh::appinfo_tab_local_ufs(),
        }
    }

    pub fn appinfo_tab_custom_config(&self) -> &'static str {
        match self.lang {
            Language::English => en::appinfo_tab_custom_config(),
            Language::Chinese => zh::appinfo_tab_custom_config(),
        }
    }

    pub fn appinfo_debug_title(&self, app_id: u32) -> String {
        match self.lang {
            Language::English => en::appinfo_debug_title(app_id),
            Language::Chinese => zh::appinfo_debug_title(app_id),
        }
    }

    pub fn appinfo_quota(&self) -> &'static str {
        match self.lang {
            Language::English => en::appinfo_quota(),
            Language::Chinese => zh::appinfo_quota(),
        }
    }

    pub fn appinfo_max_files(&self) -> &'static str {
        match self.lang {
            Language::English => en::appinfo_max_files(),
            Language::Chinese => zh::appinfo_max_files(),
        }
    }

    pub fn appinfo_current_ufs(&self) -> &'static str {
        match self.lang {
            Language::English => en::appinfo_current_ufs(),
            Language::Chinese => zh::appinfo_current_ufs(),
        }
    }

    pub fn appinfo_restart_steam(&self) -> String {
        match self.lang {
            Language::English => en::appinfo_restart_steam(),
            Language::Chinese => zh::appinfo_restart_steam(),
        }
    }

    pub fn appinfo_warning(&self) -> String {
        match self.lang {
            Language::English => en::appinfo_warning(),
            Language::Chinese => zh::appinfo_warning(),
        }
    }

    pub fn appinfo_path_hint(&self) -> &'static str {
        match self.lang {
            Language::English => en::appinfo_path_hint(),
            Language::Chinese => zh::appinfo_path_hint(),
        }
    }

    pub fn appinfo_pattern_hint(&self) -> &'static str {
        match self.lang {
            Language::English => en::appinfo_pattern_hint(),
            Language::Chinese => zh::appinfo_pattern_hint(),
        }
    }

    // ========== UFS 配置表格标签 ==========

    pub fn ufs_savefiles_header(&self, count: usize) -> String {
        match self.lang {
            Language::English => en::ufs_savefiles_header(count),
            Language::Chinese => zh::ufs_savefiles_header(count),
        }
    }

    pub fn ufs_overrides_header(&self, count: usize) -> String {
        match self.lang {
            Language::English => en::ufs_overrides_header(count),
            Language::Chinese => zh::ufs_overrides_header(count),
        }
    }

    pub fn ufs_add_savefile(&self) -> &'static str {
        match self.lang {
            Language::English => en::ufs_add_savefile(),
            Language::Chinese => zh::ufs_add_savefile(),
        }
    }

    pub fn ufs_add_override(&self) -> &'static str {
        match self.lang {
            Language::English => en::ufs_add_override(),
            Language::Chinese => zh::ufs_add_override(),
        }
    }

    pub fn ufs_no_savefiles(&self) -> &'static str {
        match self.lang {
            Language::English => en::ufs_no_savefiles(),
            Language::Chinese => zh::ufs_no_savefiles(),
        }
    }

    pub fn ufs_no_overrides(&self) -> &'static str {
        match self.lang {
            Language::English => en::ufs_no_overrides(),
            Language::Chinese => zh::ufs_no_overrides(),
        }
    }

    pub fn ufs_label_root(&self) -> &'static str {
        match self.lang {
            Language::English => en::ufs_label_root(),
            Language::Chinese => zh::ufs_label_root(),
        }
    }

    pub fn ufs_label_path(&self) -> &'static str {
        match self.lang {
            Language::English => en::ufs_label_path(),
            Language::Chinese => zh::ufs_label_path(),
        }
    }

    pub fn ufs_label_pattern(&self) -> &'static str {
        match self.lang {
            Language::English => en::ufs_label_pattern(),
            Language::Chinese => zh::ufs_label_pattern(),
        }
    }

    pub fn ufs_label_platforms(&self) -> &'static str {
        match self.lang {
            Language::English => en::ufs_label_platforms(),
            Language::Chinese => zh::ufs_label_platforms(),
        }
    }

    pub fn ufs_label_recursive(&self) -> &'static str {
        match self.lang {
            Language::English => en::ufs_label_recursive(),
            Language::Chinese => zh::ufs_label_recursive(),
        }
    }

    pub fn ufs_label_actions(&self) -> &'static str {
        match self.lang {
            Language::English => en::ufs_label_actions(),
            Language::Chinese => zh::ufs_label_actions(),
        }
    }

    pub fn ufs_label_original_root(&self) -> &'static str {
        match self.lang {
            Language::English => en::ufs_label_original_root(),
            Language::Chinese => zh::ufs_label_original_root(),
        }
    }

    pub fn ufs_label_target_os(&self) -> &'static str {
        match self.lang {
            Language::English => en::ufs_label_target_os(),
            Language::Chinese => zh::ufs_label_target_os(),
        }
    }

    pub fn ufs_label_new_root(&self) -> &'static str {
        match self.lang {
            Language::English => en::ufs_label_new_root(),
            Language::Chinese => zh::ufs_label_new_root(),
        }
    }

    pub fn ufs_label_add_path(&self) -> &'static str {
        match self.lang {
            Language::English => en::ufs_label_add_path(),
            Language::Chinese => zh::ufs_label_add_path(),
        }
    }

    pub fn ufs_label_replace_path(&self) -> &'static str {
        match self.lang {
            Language::English => en::ufs_label_replace_path(),
            Language::Chinese => zh::ufs_label_replace_path(),
        }
    }

    pub fn ufs_label_replace_with(&self) -> &'static str {
        match self.lang {
            Language::English => en::ufs_label_replace_with(),
            Language::Chinese => zh::ufs_label_replace_with(),
        }
    }

    pub fn ufs_label_find_path(&self) -> &'static str {
        match self.lang {
            Language::English => en::ufs_label_find_path(),
            Language::Chinese => zh::ufs_label_find_path(),
        }
    }

    pub fn ufs_hint_auto_fill(&self) -> &'static str {
        match self.lang {
            Language::English => en::ufs_hint_auto_fill(),
            Language::Chinese => zh::ufs_hint_auto_fill(),
        }
    }

    pub fn ufs_refresh(&self) -> &'static str {
        match self.lang {
            Language::English => en::ufs_refresh(),
            Language::Chinese => zh::ufs_refresh(),
        }
    }

    pub fn ufs_clear_all(&self) -> &'static str {
        match self.lang {
            Language::English => en::ufs_clear_all(),
            Language::Chinese => zh::ufs_clear_all(),
        }
    }

    pub fn ufs_clear_all_tooltip(&self) -> &'static str {
        match self.lang {
            Language::English => en::ufs_clear_all_tooltip(),
            Language::Chinese => zh::ufs_clear_all_tooltip(),
        }
    }

    pub fn ufs_save_config(&self) -> String {
        match self.lang {
            Language::English => en::ufs_save_config(),
            Language::Chinese => zh::ufs_save_config(),
        }
    }

    pub fn ufs_inject_to_vdf(&self) -> String {
        match self.lang {
            Language::English => en::ufs_inject_to_vdf(),
            Language::Chinese => zh::ufs_inject_to_vdf(),
        }
    }

    pub fn ufs_inject_success(&self, savefiles: usize, overrides: usize) -> String {
        match self.lang {
            Language::English => en::ufs_inject_success(savefiles, overrides),
            Language::Chinese => zh::ufs_inject_success(savefiles, overrides),
        }
    }

    pub fn ufs_inject_empty(&self) -> &'static str {
        match self.lang {
            Language::English => en::ufs_inject_empty(),
            Language::Chinese => zh::ufs_inject_empty(),
        }
    }

    pub fn ufs_inject_error(&self, error: &str) -> String {
        match self.lang {
            Language::English => en::ufs_inject_error(error),
            Language::Chinese => zh::ufs_inject_error(error),
        }
    }

    pub fn ufs_writer_init_error(&self, error: &str) -> String {
        match self.lang {
            Language::English => en::ufs_writer_init_error(error),
            Language::Chinese => zh::ufs_writer_init_error(error),
        }
    }

    pub fn ufs_save_success(&self, savefiles: usize, overrides: usize) -> String {
        match self.lang {
            Language::English => en::ufs_save_success(savefiles, overrides),
            Language::Chinese => zh::ufs_save_success(savefiles, overrides),
        }
    }

    pub fn ufs_save_error(&self, error: &str) -> String {
        match self.lang {
            Language::English => en::ufs_save_error(error),
            Language::Chinese => zh::ufs_save_error(error),
        }
    }

    pub fn ufs_clear_success(&self) -> &'static str {
        match self.lang {
            Language::English => en::ufs_clear_success(),
            Language::Chinese => zh::ufs_clear_success(),
        }
    }

    pub fn ufs_clear_error(&self, error: &str) -> String {
        match self.lang {
            Language::English => en::ufs_clear_error(error),
            Language::Chinese => zh::ufs_clear_error(error),
        }
    }

    // ========== 错误弹窗消息 ==========

    pub fn error_get_appinfo(&self, error: &str) -> String {
        match self.lang {
            Language::English => en::error_get_appinfo(error),
            Language::Chinese => zh::error_get_appinfo(error),
        }
    }

    pub fn error_vdf_parser_init(&self, error: &str) -> String {
        match self.lang {
            Language::English => en::error_vdf_parser_init(error),
            Language::Chinese => zh::error_vdf_parser_init(error),
        }
    }

    pub fn error_load_timeout(&self) -> &'static str {
        match self.lang {
            Language::English => en::error_load_timeout(),
            Language::Chinese => zh::error_load_timeout(),
        }
    }

    pub fn disconnected(&self) -> &'static str {
        match self.lang {
            Language::English => en::disconnected(),
            Language::Chinese => zh::disconnected(),
        }
    }

    pub fn error_install_failed(&self, error: &str) -> String {
        match self.lang {
            Language::English => en::error_install_failed(error),
            Language::Chinese => zh::error_install_failed(error),
        }
    }

    pub fn error_download_failed(&self, error: &str) -> String {
        match self.lang {
            Language::English => en::error_download_failed(error),
            Language::Chinese => zh::error_download_failed(error),
        }
    }

    // ========== 文件操作结果消息 ==========

    pub fn error_select_files_to_forget(&self) -> &'static str {
        match self.lang {
            Language::English => en::error_select_files_to_forget(),
            Language::Chinese => zh::error_select_files_to_forget(),
        }
    }

    pub fn error_local_only_no_forget(&self, count: usize) -> String {
        match self.lang {
            Language::English => en::error_local_only_no_forget(count),
            Language::Chinese => zh::error_local_only_no_forget(count),
        }
    }

    pub fn forgotten_files(&self, count: usize) -> String {
        match self.lang {
            Language::English => en::forgotten_files(count),
            Language::Chinese => zh::forgotten_files(count),
        }
    }

    pub fn ufs_forget_failed(&self, count: usize) -> String {
        match self.lang {
            Language::English => en::ufs_forget_failed(count),
            Language::Chinese => zh::ufs_forget_failed(count),
        }
    }

    pub fn forget_failed_files(&self, count: usize, names: &str) -> String {
        match self.lang {
            Language::English => en::forget_failed_files(count, names),
            Language::Chinese => zh::forget_failed_files(count, names),
        }
    }

    pub fn skipped_local_only_files(&self, count: usize) -> String {
        match self.lang {
            Language::English => en::skipped_local_only_files(count),
            Language::Chinese => zh::skipped_local_only_files(count),
        }
    }

    pub fn no_files_forgotten(&self) -> &'static str {
        match self.lang {
            Language::English => en::no_files_forgotten(),
            Language::Chinese => zh::no_files_forgotten(),
        }
    }

    pub fn error_select_files_to_delete(&self) -> &'static str {
        match self.lang {
            Language::English => en::error_select_files_to_delete(),
            Language::Chinese => zh::error_select_files_to_delete(),
        }
    }

    pub fn deleted_files(&self, count: usize) -> String {
        match self.lang {
            Language::English => en::deleted_files(count),
            Language::Chinese => zh::deleted_files(count),
        }
    }

    pub fn ufs_cloud_sync_hint(&self) -> &'static str {
        match self.lang {
            Language::English => en::ufs_cloud_sync_hint(),
            Language::Chinese => zh::ufs_cloud_sync_hint(),
        }
    }

    pub fn ufs_delete_failed(&self, count: usize) -> String {
        match self.lang {
            Language::English => en::ufs_delete_failed(count),
            Language::Chinese => zh::ufs_delete_failed(count),
        }
    }

    pub fn delete_failed_files(&self, count: usize) -> String {
        match self.lang {
            Language::English => en::delete_failed_files(count),
            Language::Chinese => zh::delete_failed_files(count),
        }
    }

    pub fn no_files_deleted(&self) -> &'static str {
        match self.lang {
            Language::English => en::no_files_deleted(),
            Language::Chinese => zh::no_files_deleted(),
        }
    }

    pub fn error_select_files_to_sync(&self) -> &'static str {
        match self.lang {
            Language::English => en::error_select_files_to_sync(),
            Language::Chinese => zh::error_select_files_to_sync(),
        }
    }

    pub fn synced_files_to_cloud(&self, count: usize) -> String {
        match self.lang {
            Language::English => en::synced_files_to_cloud(count),
            Language::Chinese => zh::synced_files_to_cloud(count),
        }
    }

    pub fn all_files_in_cloud(&self, count: usize) -> String {
        match self.lang {
            Language::English => en::all_files_in_cloud(count),
            Language::Chinese => zh::all_files_in_cloud(count),
        }
    }

    pub fn no_files_synced(&self) -> &'static str {
        match self.lang {
            Language::English => en::no_files_synced(),
            Language::Chinese => zh::no_files_synced(),
        }
    }

    pub fn partial_sync_failed(&self, names: &str) -> String {
        match self.lang {
            Language::English => en::partial_sync_failed(names),
            Language::Chinese => zh::partial_sync_failed(names),
        }
    }

    // ========== 同步状态显示 ==========

    pub fn sync_status_synced(&self) -> String {
        match self.lang {
            Language::English => en::sync_status_synced(),
            Language::Chinese => zh::sync_status_synced(),
        }
    }

    pub fn sync_status_local_newer(&self) -> String {
        match self.lang {
            Language::English => en::sync_status_local_newer(),
            Language::Chinese => zh::sync_status_local_newer(),
        }
    }

    pub fn sync_status_cloud_newer(&self) -> String {
        match self.lang {
            Language::English => en::sync_status_cloud_newer(),
            Language::Chinese => zh::sync_status_cloud_newer(),
        }
    }

    pub fn sync_status_conflict(&self) -> String {
        match self.lang {
            Language::English => en::sync_status_conflict(),
            Language::Chinese => zh::sync_status_conflict(),
        }
    }

    pub fn sync_status_local_only(&self) -> String {
        match self.lang {
            Language::English => en::sync_status_local_only(),
            Language::Chinese => zh::sync_status_local_only(),
        }
    }

    pub fn sync_status_cloud_only(&self) -> String {
        match self.lang {
            Language::English => en::sync_status_cloud_only(),
            Language::Chinese => zh::sync_status_cloud_only(),
        }
    }

    pub fn sync_status_unknown(&self) -> String {
        match self.lang {
            Language::English => en::sync_status_unknown(),
            Language::Chinese => zh::sync_status_unknown(),
        }
    }

    // ========== Hash 状态显示 ==========

    pub fn hash_status_pending(&self) -> String {
        match self.lang {
            Language::English => en::hash_status_pending(),
            Language::Chinese => zh::hash_status_pending(),
        }
    }

    pub fn hash_status_skipped(&self) -> String {
        match self.lang {
            Language::English => en::hash_status_skipped(),
            Language::Chinese => zh::hash_status_skipped(),
        }
    }

    pub fn hash_status_checking(&self) -> String {
        match self.lang {
            Language::English => en::hash_status_checking(),
            Language::Chinese => zh::hash_status_checking(),
        }
    }

    pub fn hash_status_match(&self) -> String {
        match self.lang {
            Language::English => en::hash_status_match(),
            Language::Chinese => zh::hash_status_match(),
        }
    }

    pub fn hash_status_mismatch(&self) -> String {
        match self.lang {
            Language::English => en::hash_status_mismatch(),
            Language::Chinese => zh::hash_status_mismatch(),
        }
    }

    pub fn hash_status_error(&self) -> String {
        match self.lang {
            Language::English => en::hash_status_error(),
            Language::Chinese => zh::hash_status_error(),
        }
    }

    // ========== 文件对比对话框详细文本 ==========

    pub fn size_diff_label(&self) -> &'static str {
        match self.lang {
            Language::English => en::size_diff_label(),
            Language::Chinese => zh::size_diff_label(),
        }
    }

    pub fn local_larger_bytes(&self, bytes: i64) -> String {
        match self.lang {
            Language::English => en::local_larger_bytes(bytes),
            Language::Chinese => zh::local_larger_bytes(bytes),
        }
    }

    pub fn cloud_larger_bytes(&self, bytes: i64) -> String {
        match self.lang {
            Language::English => en::cloud_larger_bytes(bytes),
            Language::Chinese => zh::cloud_larger_bytes(bytes),
        }
    }

    pub fn diff_items_label(&self) -> &'static str {
        match self.lang {
            Language::English => en::diff_items_label(),
            Language::Chinese => zh::diff_items_label(),
        }
    }

    pub fn diff_exists(&self) -> &'static str {
        match self.lang {
            Language::English => en::diff_exists(),
            Language::Chinese => zh::diff_exists(),
        }
    }

    pub fn diff_sync(&self) -> &'static str {
        match self.lang {
            Language::English => en::diff_sync(),
            Language::Chinese => zh::diff_sync(),
        }
    }

    pub fn diff_size(&self) -> &'static str {
        match self.lang {
            Language::English => en::diff_size(),
            Language::Chinese => zh::diff_size(),
        }
    }

    pub fn diff_time(&self) -> &'static str {
        match self.lang {
            Language::English => en::diff_time(),
            Language::Chinese => zh::diff_time(),
        }
    }

    pub fn hash_status_label(&self) -> &'static str {
        match self.lang {
            Language::English => en::hash_status_label(),
            Language::Chinese => zh::hash_status_label(),
        }
    }

    pub fn retry_hash_check(&self) -> &'static str {
        match self.lang {
            Language::English => en::retry_hash_check(),
            Language::Chinese => zh::retry_hash_check(),
        }
    }

    pub fn local_hash_label(&self) -> &'static str {
        match self.lang {
            Language::English => en::local_hash_label(),
            Language::Chinese => zh::local_hash_label(),
        }
    }

    pub fn cloud_hash_label(&self) -> &'static str {
        match self.lang {
            Language::English => en::cloud_hash_label(),
            Language::Chinese => zh::cloud_hash_label(),
        }
    }

    pub fn not_calculated(&self) -> &'static str {
        match self.lang {
            Language::English => en::not_calculated(),
            Language::Chinese => zh::not_calculated(),
        }
    }

    // ========== Symlink 对话框 ==========

    pub fn error_delete_config(&self, error: &str) -> String {
        match self.lang {
            Language::English => en::error_delete_config(error),
            Language::Chinese => zh::error_delete_config(error),
        }
    }

    pub fn remote_dir_label(&self) -> &'static str {
        match self.lang {
            Language::English => en::remote_dir_label(),
            Language::Chinese => zh::remote_dir_label(),
        }
    }

    pub fn copy_path(&self) -> &'static str {
        match self.lang {
            Language::English => en::copy_path(),
            Language::Chinese => zh::copy_path(),
        }
    }

    pub fn symlink_conflict_label(&self) -> &'static str {
        match self.lang {
            Language::English => en::symlink_conflict_label(),
            Language::Chinese => zh::symlink_conflict_label(),
        }
    }

    // ========== Settings 对话框 ==========

    pub fn steam_path_hint_text(&self) -> &'static str {
        match self.lang {
            Language::English => en::steam_path_hint_text(),
            Language::Chinese => zh::steam_path_hint_text(),
        }
    }

    // ========== 状态面板 ==========

    pub fn cloud_status_not_ready(&self) -> &'static str {
        match self.lang {
            Language::English => en::cloud_status_not_ready(),
            Language::Chinese => zh::cloud_status_not_ready(),
        }
    }

    // ========== 游戏选择器详情 ==========

    pub fn game_file_info(&self, count: usize, size: &str) -> String {
        match self.lang {
            Language::English => en::game_file_info(count, size),
            Language::Chinese => zh::game_file_info(count, size),
        }
    }

    pub fn install_dir_label(&self, dir: &str) -> String {
        match self.lang {
            Language::English => en::install_dir_label(dir),
            Language::Chinese => zh::install_dir_label(dir),
        }
    }

    pub fn tags_label(&self, tags: &str) -> String {
        match self.lang {
            Language::English => en::tags_label(tags),
            Language::Chinese => zh::tags_label(tags),
        }
    }

    pub fn playtime_label(&self, hours: f64) -> String {
        match self.lang {
            Language::English => en::playtime_label(hours),
            Language::Chinese => zh::playtime_label(hours),
        }
    }

    pub fn last_played_label(&self, time: &str) -> String {
        match self.lang {
            Language::English => en::last_played_label(time),
            Language::Chinese => zh::last_played_label(time),
        }
    }

    pub fn select_button(&self) -> &'static str {
        match self.lang {
            Language::English => en::select_button(),
            Language::Chinese => zh::select_button(),
        }
    }

    pub fn check_update_failed(&self, error: &str) -> String {
        match self.lang {
            Language::English => en::check_update_failed(error),
            Language::Chinese => zh::check_update_failed(error),
        }
    }

    // ========== 主题名称 ==========

    pub fn theme_light(&self) -> &'static str {
        match self.lang {
            Language::English => en::theme_light(),
            Language::Chinese => zh::theme_light(),
        }
    }

    pub fn theme_dark(&self) -> &'static str {
        match self.lang {
            Language::English => en::theme_dark(),
            Language::Chinese => zh::theme_dark(),
        }
    }

    pub fn theme_system(&self) -> &'static str {
        match self.lang {
            Language::English => en::theme_system(),
            Language::Chinese => zh::theme_system(),
        }
    }
}
