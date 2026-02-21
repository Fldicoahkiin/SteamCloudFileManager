// English translations

use crate::icons;

pub fn language_label() -> &'static str {
    "Language:"
}

pub fn app_title() -> &'static str {
    "Steam Cloud File Manager"
}

pub fn refresh() -> &'static str {
    "Refresh"
}

pub fn cancel() -> &'static str {
    "Cancel"
}

pub fn close() -> &'static str {
    "Close"
}

pub fn ok() -> &'static str {
    "OK"
}

pub fn logged_in() -> &'static str {
    "Logged In"
}

pub fn not_logged_in() -> &'static str {
    "Not Logged In"
}

pub fn connect() -> &'static str {
    "Connect"
}

pub fn disconnect() -> &'static str {
    "Disconnect"
}

pub fn disconnect_sync_hint() -> &'static str {
    "Steam will auto-sync after disconnect"
}

pub fn refresh_open_url_hint() -> &'static str {
    "Open cloud storage page in Steam"
}

pub fn show_appinfo_vdf() -> &'static str {
    "Configure appinfo.vdf"
}

pub fn account_cloud_status() -> &'static str {
    "Account Cloud"
}

pub fn select_account() -> &'static str {
    "Accounts"
}

pub fn select_game() -> &'static str {
    "Game Library"
}

pub fn select_all() -> &'static str {
    "Select All"
}

pub fn invert_selection() -> &'static str {
    "Invert"
}

pub fn clear_selection() -> &'static str {
    "Clear"
}

pub fn download() -> &'static str {
    "Download"
}

pub fn upload() -> &'static str {
    "Upload"
}

pub fn delete() -> &'static str {
    "Delete"
}

pub fn forget() -> &'static str {
    "Forget"
}

pub fn sync_to_cloud() -> &'static str {
    "Sync"
}

pub fn file_name() -> &'static str {
    "File Name"
}

pub fn size() -> &'static str {
    "Size"
}

pub fn selected_count(count: usize) -> String {
    format!("{} selected", count)
}

pub fn select_game_title() -> &'static str {
    "Select Game"
}

pub fn settings_title() -> &'static str {
    "Settings"
}

pub fn settings_log() -> &'static str {
    "Log"
}

pub fn settings_about() -> &'static str {
    "About"
}

pub fn settings_appearance() -> &'static str {
    "Appearance"
}

pub fn settings_advanced() -> &'static str {
    "Advanced"
}

pub fn steam_path_label() -> &'static str {
    "Steam Install Path"
}

pub fn steam_path_auto_detect() -> &'static str {
    "Auto Detect"
}

pub fn steam_path_browse() -> &'static str {
    "Browse..."
}

pub fn steam_path_valid(user_count: usize) -> String {
    format!(
        "✓ Valid path ({} user{} found)",
        user_count,
        if user_count != 1 { "s" } else { "" }
    )
}

pub fn steam_path_not_exists() -> &'static str {
    "✗ Path not exists"
}

pub fn steam_path_no_userdata() -> &'static str {
    "✗ Missing userdata folder"
}

pub fn steam_path_no_users() -> &'static str {
    "✗ No users found"
}

pub fn steam_path_hint() -> &'static str {
    "💡 Select directory manually if Steam is in non-standard location"
}

pub fn steam_path_restart_hint() -> &'static str {
    "Restart required after changing"
}

pub fn reset_all_settings() -> &'static str {
    "Reset All Settings"
}

pub fn reset_confirm() -> &'static str {
    "Reset all settings to default?"
}

pub fn config_dir_label() -> &'static str {
    "Config File:"
}

pub fn open_config_dir() -> &'static str {
    "Open Config Directory"
}

pub fn theme_mode_label() -> &'static str {
    "Theme Mode:"
}

pub fn error_title() -> &'static str {
    "Error"
}

pub fn author() -> &'static str {
    "Author:"
}

pub fn github_repository() -> &'static str {
    "Github Repository:"
}

pub fn connecting() -> &'static str {
    "Connecting..."
}

pub fn games_with_cloud(count: usize) -> String {
    format!(
        "{} game{} with cloud saves",
        count,
        if count != 1 { "s" } else { "" }
    )
}

pub fn scanning_games() -> &'static str {
    "Scanning game library..."
}

pub fn no_cloud_games_found() -> &'static str {
    "No games with cloud saves found"
}

pub fn installed() -> &'static str {
    "Installed"
}

pub fn not_installed() -> &'static str {
    "Not Installed"
}

pub fn select_user() -> &'static str {
    "Select User"
}

pub fn user_id() -> &'static str {
    "User ID"
}

pub fn current_user() -> &'static str {
    "Current User"
}

pub fn switch() -> &'static str {
    "Switch"
}

pub fn steam_users(count: usize) -> String {
    format!("{} Steam user{}", count, if count != 1 { "s" } else { "" })
}

pub fn checking_update() -> String {
    format!("{} Checking...", icons::SPINNER)
}

pub fn check_update_btn() -> String {
    format!("{} Check Update", icons::REFRESH)
}

pub fn already_latest() -> String {
    format!("{} Already up to date", icons::CHECK)
}

pub fn new_version_found(version: &str) -> String {
    format!("🎉 New version available: {}", version)
}

pub fn new_version_hint() -> &'static str {
    "New version found, click to download and install:"
}

pub fn download_and_install() -> String {
    format!("{} Download & Install", icons::DOWNLOAD)
}

pub fn view_details() -> String {
    format!("{} View Details", icons::GLOBE)
}

pub fn downloading_update() -> String {
    format!("{} Downloading update...", icons::DOWNLOAD)
}

pub fn installing_update() -> String {
    format!("{} Installing update...", icons::GEAR)
}

pub fn update_success() -> String {
    format!("{} Update installed successfully!", icons::CHECK)
}

pub fn restart_to_apply() -> &'static str {
    "Please restart the app to use the new version"
}

pub fn restart_now() -> String {
    format!("{} Restart Now", icons::REFRESH)
}

pub fn log_enabled_hint() -> &'static str {
    " Log storage enabled, restart to apply"
}

pub fn log_disabled_hint() -> &'static str {
    " Log storage disabled, restart to apply"
}

pub fn enable_log_storage() -> &'static str {
    "Enable Log Storage"
}

pub fn open_log_dir() -> &'static str {
    " Open Log Directory"
}

pub fn log_dir_label() -> &'static str {
    "Log Directory:"
}

pub fn steam_log_dir_label() -> &'static str {
    "Steam Log Directory:"
}

pub fn open_steam_log_dir() -> &'static str {
    " Open Steam Log Directory"
}

pub fn restarting_steam() -> &'static str {
    "Restarting Steam"
}

pub fn manual_operation_required() -> &'static str {
    "Manual operation required:"
}

pub fn i_understand() -> &'static str {
    "I Understand"
}

#[cfg(target_os = "macos")]
pub fn manual_restart_macos_title() -> &'static str {
    "Manual Restart Steam (macOS)"
}

#[cfg(target_os = "windows")]
pub fn manual_restart_windows_title() -> &'static str {
    "Manual Restart Steam (Windows)"
}

#[cfg(target_os = "linux")]
pub fn manual_restart_linux_title() -> &'static str {
    "Manual Restart Steam (Linux)"
}

pub fn prepare_upload() -> &'static str {
    "Prepare Upload"
}

pub fn will_upload_files(count: usize) -> String {
    format!(
        "Will upload {} file{} to Steam Cloud",
        count,
        if count != 1 { "s" } else { "" }
    )
}

pub fn total_size_label(size: &str) -> String {
    format!("Total size: {}", size)
}

pub fn warning() -> String {
    format!("{} Warning:", icons::WARNING)
}

pub fn overwrite_warning() -> &'static str {
    "• Files with same name will be overwritten"
}

pub fn add_files() -> String {
    format!("{} Add Files", icons::ADD_FILE)
}

pub fn add_folder() -> String {
    format!("{} Add Folder", icons::ADD_FOLDER)
}

pub fn confirm_upload() -> String {
    format!("{} Confirm Upload", icons::CHECK)
}

pub fn remove_file() -> &'static str {
    "Remove"
}

pub fn cloud_path() -> &'static str {
    "Cloud Path"
}

pub fn edit_path() -> &'static str {
    "Edit Path"
}

pub fn local_file() -> &'static str {
    "Local File"
}

pub fn no_files_to_upload() -> &'static str {
    "No files to upload, please add files"
}

pub fn clear_all() -> &'static str {
    "Clear All"
}

pub fn uploading_files() -> String {
    format!("{} Uploading Files", icons::UPLOAD)
}

pub fn uploading_file(name: &str) -> String {
    format!("Uploading: {}", name)
}

pub fn upload_progress(current: usize, total: usize) -> String {
    format!("Progress: {} / {} files", current, total)
}

pub fn speed(speed: &str) -> String {
    format!("Speed: {}/s", speed)
}

pub fn upload_complete() -> String {
    format!("{} Upload Complete", icons::CHECK)
}

pub fn upload_success(count: usize) -> String {
    format!(
        "{} Successfully uploaded {} file{}",
        icons::ROCKET,
        count,
        if count != 1 { "s" } else { "" }
    )
}

pub fn upload_partial(success: usize, failed: usize) -> String {
    format!(
        "{} Upload complete: {} succeeded, {} failed",
        icons::WARNING,
        success,
        failed
    )
}

pub fn elapsed_time(secs: u64) -> String {
    format!("Time: {} second{}", secs, if secs != 1 { "s" } else { "" })
}

pub fn avg_speed(speed: &str) -> String {
    format!("Avg speed: {}/s", speed)
}

pub fn failed_files() -> &'static str {
    "Failed files:"
}

pub fn reason(err: &str) -> String {
    format!("  Reason: {}", err)
}

pub fn closing_steam() -> &'static str {
    "Closing Steam..."
}

pub fn starting_steam() -> &'static str {
    "Starting Steam..."
}

pub fn steam_restart_success() -> &'static str {
    "Steam restarted successfully!"
}

pub fn user_switched() -> &'static str {
    "User switched"
}

pub fn error_enter_app_id() -> &'static str {
    "Please enter App ID"
}

pub fn error_invalid_app_id() -> &'static str {
    "Invalid App ID"
}

pub fn status_enter_app_id() -> &'static str {
    "Enter App ID and connect to Steam"
}

pub fn status_loading_files() -> &'static str {
    "Loading file list..."
}

pub fn status_files_loaded(count: usize) -> String {
    format!("Loaded {} file{}", count, if count != 1 { "s" } else { "" })
}

pub fn upload_failed(err: &str) -> String {
    format!("Upload failed: {}", err)
}

pub fn error_no_files_selected() -> &'static str {
    "Please select files to operate"
}

pub fn error_not_connected() -> &'static str {
    "Not connected to Steam"
}

pub fn hint_you_can() -> &'static str {
    "You can:"
}

pub fn hint_select_game() -> &'static str {
    "Click 'Game Library' button above to choose a game"
}

pub fn hint_enter_app_id() -> &'static str {
    "Or enter App ID directly and click 'Connect'"
}

pub fn no_cloud_files() -> &'static str {
    "No cloud files found"
}

pub fn no_cloud_files_hint() -> &'static str {
    "This game has no cloud save files"
}

pub fn scan_games_failed(err: &str) -> String {
    format!("Failed to scan games: {}", err)
}

pub fn refresh_files_failed(err: &str) -> String {
    format!("Failed to refresh file list: {}", err)
}

pub fn cdp_no_data_error() -> &'static str {
    "CDP failed to get game data!\n\nPossible reasons:\n\
    1. Steam client not responding to redirect request\n\
    2. Page not fully loaded\n\
    3. Not logged into Steam web\n\n"
}

pub fn connecting_to_steam(app_id: u32) -> String {
    format!("Connecting to Steam (App ID: {})...", app_id)
}

pub fn loading_files_for_app(app_id: u32) -> String {
    format!("Loading file list (App ID: {})...", app_id)
}

pub fn connect_steam_failed(err: &str) -> String {
    format!("Failed to connect to Steam: {}", err)
}

pub fn vdf_parser_not_initialized() -> &'static str {
    "VDF parser not initialized"
}

pub fn scanning_game_library() -> &'static str {
    "Scanning game library..."
}

pub fn drop_files_to_upload() -> &'static str {
    "Drop files to upload"
}

pub fn debug_mode_not_enabled() -> String {
    format!("{} Steam Debug Mode Not Enabled", icons::WARNING)
}

pub fn steam_running() -> String {
    format!("{} Steam is running", icons::CHECK)
}

pub fn steam_not_running() -> String {
    format!("{} Steam is not running", icons::CLOSE)
}

pub fn debug_mode_hint() -> &'static str {
    "CEF debug mode is required to access cloud data"
}

pub fn auto_restart_steam() -> &'static str {
    "Auto Restart Steam"
}

pub fn start_steam() -> &'static str {
    "Start Steam"
}

pub fn auto_restart_hint() -> &'static str {
    "Automatically restart Steam with debug parameters"
}

pub fn start_steam_hint() -> &'static str {
    "Start Steam in debug mode"
}

pub fn view_manual_steps() -> &'static str {
    "View Manual Steps"
}

pub fn manual_steps_hint() -> &'static str {
    "Show how to manually add startup parameters"
}

pub fn dismiss_temporarily() -> String {
    format!("{} Dismiss", icons::CLOSE)
}

pub fn dismiss_hint() -> &'static str {
    "Hide this hint (can be re-enabled in settings)"
}

pub fn status_label() -> &'static str {
    "Status:"
}

pub fn cloud_on() -> &'static str {
    "Cloud: On"
}

pub fn cloud_off() -> &'static str {
    "Cloud: Off"
}

pub fn quota_usage(percent: f32, used: &str, total: &str) -> String {
    format!("Quota: {:.1}% used ({}/{})", percent, used, total)
}

pub fn select_all_hint() -> &'static str {
    "Select all files in the list"
}

pub fn invert_selection_hint() -> &'static str {
    "Invert current selection"
}

pub fn clear_selection_hint() -> &'static str {
    "Deselect all files"
}

pub fn download_hint() -> &'static str {
    "Download selected files to local"
}

pub fn upload_hint() -> &'static str {
    "Upload files or folders to cloud"
}

pub fn delete_hint() -> &'static str {
    "Delete selected files from cloud and local"
}

pub fn forget_hint() -> &'static str {
    "Remove from cloud only, keep local files"
}

pub fn sync_to_cloud_hint() -> &'static str {
    "Sync local files to cloud"
}

pub fn connect_hint() -> &'static str {
    "Connect to Steam Cloud API"
}

pub fn disconnect_hint() -> &'static str {
    "Disconnect from Steam"
}

pub fn select_account_hint() -> &'static str {
    "Switch Steam account"
}

pub fn select_game_hint() -> &'static str {
    "Select game to manage cloud saves"
}

pub fn local_save_path() -> &'static str {
    "Local Save Path:"
}

pub fn local_save_path_not_found() -> &'static str {
    "Not found (files may only exist in cloud)"
}

pub fn search_files_placeholder() -> &'static str {
    "Search files or folders..."
}

pub fn clear() -> &'static str {
    "Clear"
}

pub fn only_local() -> &'static str {
    "Local Only"
}

pub fn only_cloud() -> &'static str {
    "Cloud Only"
}

pub fn only_local_tooltip() -> &'static str {
    "Show only files that exist locally but not in cloud"
}

pub fn only_cloud_tooltip() -> &'static str {
    "Show only files that exist in cloud but not locally"
}

pub fn root_folder() -> &'static str {
    "Root Folder"
}

pub fn file_size() -> &'static str {
    "File Size"
}

pub fn write_date() -> &'static str {
    "Write Date"
}

pub fn local() -> &'static str {
    "Local"
}

pub fn cloud() -> &'static str {
    "Cloud"
}

pub fn file_comparison_title() -> &'static str {
    "File Comparison"
}

pub fn total_files_count(count: usize) -> String {
    format!("{} files total", count)
}

pub fn filter_all() -> &'static str {
    "All"
}

pub fn filter_conflicts() -> &'static str {
    "Conflicts"
}

pub fn filter_local_newer() -> &'static str {
    "Local Newer"
}

pub fn filter_cloud_newer() -> &'static str {
    "Cloud Newer"
}

pub fn filter_synced() -> &'static str {
    "Synced"
}

pub fn status_local_newer() -> &'static str {
    "Local↑"
}

pub fn status_cloud_newer() -> &'static str {
    "Cloud↓"
}

pub fn status_conflict() -> &'static str {
    "Conflict"
}

pub fn status_local_only() -> &'static str {
    "Local"
}

pub fn status_cloud_only() -> &'static str {
    "Cloud"
}

pub fn column_status() -> &'static str {
    "Status"
}

pub fn column_filename() -> &'static str {
    "Filename"
}

pub fn column_local_size() -> &'static str {
    "Local Size"
}

pub fn column_cloud_size() -> &'static str {
    "Cloud Size"
}

pub fn column_local_time() -> &'static str {
    "Local Time"
}

pub fn column_cloud_time() -> &'static str {
    "Cloud Time"
}

pub fn selected_file() -> &'static str {
    "Selected:"
}

pub fn local_newer_by_minutes(mins: i64) -> String {
    format!("(local {} mins newer)", mins)
}

pub fn cloud_newer_by_minutes(mins: i64) -> String {
    format!("(cloud {} mins newer)", mins)
}

pub fn conflicts_warning(count: usize) -> String {
    format!("{} conflicts detected, please resolve manually", count)
}

pub fn compare_files() -> &'static str {
    "Compare Files"
}

pub fn compare_files_hint() -> &'static str {
    "Compare differences between local and cloud files"
}

pub fn backup() -> &'static str {
    "Backup"
}

pub fn backup_title() -> &'static str {
    "Backup Cloud Saves"
}

pub fn backup_file_count(count: usize) -> String {
    format!("{} files", count)
}

pub fn backup_total_size(size: &str) -> String {
    format!("Total size: {}", size)
}

pub fn backup_cdp_warning(count: usize) -> String {
    format!(
        "{} {} files without download URL will be skipped",
        icons::WARNING,
        count
    )
}

pub fn backup_file_list() -> &'static str {
    "File List"
}

pub fn backup_start() -> &'static str {
    "Start Backup"
}

pub fn backup_open_dir() -> &'static str {
    "Open Backup Dir"
}

pub fn backup_progress_title() -> &'static str {
    "Backup Progress"
}

pub fn backup_in_progress() -> &'static str {
    "Backing up..."
}

pub fn backup_complete() -> String {
    format!("{} Backup Complete", icons::CHECK)
}

pub fn backup_partial() -> String {
    format!("{} Partially Complete", icons::WARNING)
}

pub fn backup_result_stats(success: usize, total: usize) -> String {
    format!("Success: {} / {}", success, total)
}

pub fn backup_failed_files() -> &'static str {
    "Failed files:"
}

pub fn backup_hint() -> &'static str {
    "Backup all cloud saves for current game"
}

pub fn backup_dir_label() -> &'static str {
    "Backup Directory:"
}

pub fn download_progress_title() -> &'static str {
    "Download Progress"
}

pub fn download_in_progress() -> &'static str {
    "Downloading..."
}

pub fn download_complete() -> String {
    format!("{} Download Complete", icons::CHECK)
}

pub fn download_partial_status() -> String {
    format!("{} Partially Complete", icons::WARNING)
}

pub fn download_result_stats(success: usize, total: usize) -> String {
    format!("Success: {} / {}", success, total)
}

pub fn download_failed_files() -> &'static str {
    "Failed files:"
}

pub fn download_open_dir() -> &'static str {
    "Open Download Dir"
}

pub fn symlink_title() -> &'static str {
    "Symlink Management (Experimental)"
}

pub fn symlink_configured_links() -> &'static str {
    "Configured Symlinks"
}

pub fn symlink_no_configs() -> &'static str {
    "No symlink configurations"
}

pub fn symlink_add_new() -> &'static str {
    "Add New Symlink"
}

pub fn symlink_direction() -> &'static str {
    "Direction:"
}

pub fn symlink_local_path() -> &'static str {
    "Local Path:"
}

pub fn symlink_remote_subfolder() -> &'static str {
    "Remote Subfolder:"
}

pub fn symlink_browse() -> &'static str {
    "Browse"
}

pub fn symlink_add_config() -> &'static str {
    "Add Config"
}

pub fn symlink_add_and_create() -> &'static str {
    "Add & Create Link"
}

pub fn symlink_create() -> &'static str {
    "Create Link"
}

pub fn symlink_remove_link() -> &'static str {
    "Remove Link"
}

pub fn symlink_delete_config() -> &'static str {
    "Delete Config"
}

pub fn symlink_copy_command() -> &'static str {
    "Copy Command"
}

pub fn symlink_refresh() -> &'static str {
    "Refresh"
}

pub fn symlink_command_copied() -> &'static str {
    "Command copied to clipboard"
}

pub fn symlink_config_deleted() -> &'static str {
    "Config deleted"
}

pub fn symlink_config_added() -> &'static str {
    "Config added"
}

pub fn symlink_created() -> &'static str {
    "Symlink created"
}

pub fn symlink_removed() -> &'static str {
    "Symlink removed"
}

pub fn symlink_create_failed() -> &'static str {
    "Create failed"
}

pub fn symlink_remove_failed() -> &'static str {
    "Remove failed"
}

pub fn symlink_add_failed() -> &'static str {
    "Add config failed"
}

pub fn symlink_experimental_title() -> &'static str {
    "Experimental Feature"
}

pub fn symlink_experimental_desc() -> &'static str {
    "Files in the directory are auto-synced after symlink creation. Use the cloud upload button to manually sync new files."
}

pub fn symlink_sync_files() -> &'static str {
    "Sync files to cloud"
}

pub fn symlink_sync_success() -> &'static str {
    "Sync successful"
}

pub fn symlink_sync_partial() -> &'static str {
    "Partially synced"
}

pub fn symlink_sync_no_files() -> &'static str {
    "Directory is empty, no files to sync"
}

pub fn symlink_sync_no_manager() -> &'static str {
    "Symlink manager not initialized"
}

pub fn symlink_sync_no_steam() -> &'static str {
    "Steam not connected, cannot sync"
}

pub fn symlink_sync_scan_failed() -> &'static str {
    "Failed to scan directory"
}

pub fn files() -> &'static str {
    "files"
}

pub fn appinfo_tab_local_ufs() -> &'static str {
    "Local UFS Config"
}

pub fn appinfo_tab_custom_config() -> &'static str {
    "Custom Config"
}

pub fn appinfo_debug_title(app_id: u32) -> String {
    format!("appinfo.vdf Debug - App {}", app_id)
}

pub fn appinfo_quota() -> &'static str {
    "Quota:"
}

pub fn appinfo_max_files() -> &'static str {
    "Max Files:"
}

pub fn appinfo_current_ufs() -> &'static str {
    "Current UFS Cloud Config:"
}

pub fn appinfo_restart_steam() -> String {
    format!("{} Restart Steam", icons::REFRESH)
}

pub fn appinfo_warning() -> String {
    format!(
        "{} This is experimental. appinfo.vdf may be overwritten by Steam.\n\
    Inject before Steam starts, or restart Steam after injection.",
        icons::WARNING
    )
}

pub fn appinfo_path_hint() -> &'static str {
    "e.g. MyGame/Saves"
}

pub fn appinfo_pattern_hint() -> &'static str {
    "* or *.sav"
}

pub fn ufs_savefiles_header(count: usize) -> String {
    format!("Savefiles Configuration ({})", count)
}

pub fn ufs_overrides_header(count: usize) -> String {
    format!("Root Overrides ({})", count)
}

pub fn ufs_add_savefile() -> &'static str {
    "Add Savefile"
}

pub fn ufs_add_override() -> &'static str {
    "Add Override"
}

pub fn ufs_no_savefiles() -> &'static str {
    "No savefiles configured — click Add to create"
}

pub fn ufs_no_overrides() -> &'static str {
    "No overrides — add for cross-platform support"
}

pub fn ufs_label_root() -> &'static str {
    "Root"
}

pub fn ufs_label_path() -> &'static str {
    "Path"
}

pub fn ufs_label_pattern() -> &'static str {
    "Pattern"
}

pub fn ufs_label_platforms() -> &'static str {
    "Platforms"
}

pub fn ufs_label_recursive() -> &'static str {
    "Recursive (search subdirectories)"
}

pub fn ufs_label_actions() -> &'static str {
    "Actions"
}

pub fn ufs_label_original_root() -> &'static str {
    "Original Root"
}

pub fn ufs_label_target_os() -> &'static str {
    "Target OS"
}

pub fn ufs_label_new_root() -> &'static str {
    "New Root"
}

pub fn ufs_label_add_path() -> &'static str {
    "Add Path"
}

pub fn ufs_label_replace_path() -> &'static str {
    "Replace Path"
}

pub fn ufs_label_replace_with() -> &'static str {
    "Replace:"
}

pub fn ufs_label_find_path() -> &'static str {
    "Find Path"
}

pub fn ufs_hint_auto_fill() -> &'static str {
    "Auto-fill if empty"
}

pub fn ufs_refresh() -> &'static str {
    "Refresh"
}

pub fn ufs_clear_all() -> &'static str {
    "Clear All"
}

pub fn ufs_clear_all_tooltip() -> &'static str {
    "Clear all custom savefiles and root overrides"
}

pub fn ufs_save_config() -> String {
    format!("{} Save Config", icons::SAVE)
}

pub fn ufs_inject_to_vdf() -> String {
    format!("{} Inject to VDF", icons::CLOUD_UPLOAD)
}

pub fn ufs_inject_success(savefiles: usize, overrides: usize) -> String {
    format!("Injected {} savefiles, {} overrides", savefiles, overrides)
}

pub fn ufs_inject_empty() -> &'static str {
    "No savefiles or overrides to inject"
}

pub fn ufs_inject_error(error: &str) -> String {
    format!("Inject error: {}", error)
}

pub fn ufs_writer_init_error(error: &str) -> String {
    format!("Writer init error: {}", error)
}

pub fn ufs_save_success(savefiles: usize, overrides: usize) -> String {
    format!("Saved {} savefiles, {} overrides", savefiles, overrides)
}

pub fn ufs_save_error(error: &str) -> String {
    format!("Save error: {}", error)
}

pub fn ufs_clear_success() -> &'static str {
    "Cleared all custom configurations"
}

pub fn ufs_clear_error(error: &str) -> String {
    format!("Clear error: {}", error)
}

pub fn error_get_appinfo(error: &str) -> String {
    format!("Failed to get appinfo: {}", error)
}

pub fn error_vdf_parser_init(error: &str) -> String {
    format!("VDF parser init failed: {}", error)
}

pub fn error_load_timeout() -> &'static str {
    "Loading timed out, please retry"
}

pub fn disconnected() -> &'static str {
    "Disconnected"
}

pub fn error_install_failed(error: &str) -> String {
    format!("Install failed: {}\n\nPlease download manually", error)
}

pub fn error_download_failed(error: &str) -> String {
    format!("Download failed: {}\n\nPlease download manually", error)
}

pub fn error_select_files_to_forget() -> &'static str {
    "Please select files to forget"
}

pub fn error_local_only_no_forget(count: usize) -> String {
    format!(
        "Selected {} file{} only exist locally, no cloud record to forget",
        count,
        if count != 1 { "s" } else { "" }
    )
}

pub fn forgotten_files(count: usize) -> String {
    format!(
        "Removed {} file{} from cloud",
        count,
        if count != 1 { "s" } else { "" }
    )
}

pub fn ufs_forget_failed(count: usize) -> String {
    format!(
        "{} auto-cloud file{} cannot be removed via API, try \"Delete\" instead",
        count,
        if count != 1 { "s" } else { "" }
    )
}

pub fn forget_failed_files(count: usize, names: &str) -> String {
    format!(
        "{} file{} failed to forget: {}",
        count,
        if count != 1 { "s" } else { "" },
        names
    )
}

pub fn skipped_local_only_files(count: usize) -> String {
    format!(
        "Skipped {} local-only file{}",
        count,
        if count != 1 { "s" } else { "" }
    )
}

pub fn no_files_forgotten() -> &'static str {
    "No files were removed from cloud"
}

pub fn error_select_files_to_delete() -> &'static str {
    "Please select files to delete"
}

pub fn deleted_files(count: usize) -> String {
    format!(
        "Deleted {} file{}",
        count,
        if count != 1 { "s" } else { "" }
    )
}

pub fn ufs_cloud_sync_hint() -> &'static str {
    "Cloud copies of auto-sync files will be removed after Steam syncs, please refresh later"
}

pub fn ufs_delete_failed(count: usize) -> String {
    format!(
        "{} auto-cloud file{} cannot be deleted (game not installed, please install and retry)",
        count,
        if count != 1 { "s" } else { "" }
    )
}

pub fn delete_failed_files(count: usize) -> String {
    format!(
        "{} file{} failed to delete",
        count,
        if count != 1 { "s" } else { "" }
    )
}

pub fn no_files_deleted() -> &'static str {
    "No files were deleted"
}

pub fn error_select_files_to_sync() -> &'static str {
    "Please select files to sync"
}

pub fn synced_files_to_cloud(count: usize) -> String {
    format!(
        "Synced {} file{} to cloud",
        count,
        if count != 1 { "s" } else { "" }
    )
}

pub fn all_files_in_cloud(count: usize) -> String {
    format!(
        "All {} file{} already in cloud, no sync needed",
        count,
        if count != 1 { "s" } else { "" }
    )
}

pub fn no_files_synced() -> &'static str {
    "No files were synced"
}

pub fn partial_sync_failed(names: &str) -> String {
    format!("Some files failed to sync: {}", names)
}

pub fn sync_status_synced() -> String {
    format!("{} Synced", crate::icons::CHECK)
}

pub fn sync_status_local_newer() -> String {
    format!("{} Local Newer", crate::icons::ARROW_UP)
}

pub fn sync_status_cloud_newer() -> String {
    format!("{} Cloud Newer", crate::icons::ARROW_DOWN)
}

pub fn sync_status_conflict() -> String {
    format!("{} Conflict", crate::icons::WARNING)
}

pub fn sync_status_local_only() -> String {
    format!("{} Local Only", crate::icons::FILE)
}

pub fn sync_status_cloud_only() -> String {
    format!("{} Cloud Only", crate::icons::CLOUD)
}

pub fn sync_status_unknown() -> String {
    format!("{} Checking", crate::icons::QUESTION)
}

pub fn hash_status_pending() -> String {
    format!("{} Pending", crate::icons::HOURGLASS)
}

pub fn hash_status_skipped() -> String {
    format!("{} Skipped", crate::icons::CHECK)
}

pub fn hash_status_checking() -> String {
    format!("{} Checking", crate::icons::SPINNER)
}

pub fn hash_status_match() -> String {
    format!("{} Match", crate::icons::CHECK)
}

pub fn hash_status_mismatch() -> String {
    format!("{} Mismatch", crate::icons::ERROR)
}

pub fn hash_status_error() -> String {
    format!("{} Error", crate::icons::WARNING)
}

pub fn size_diff_label() -> &'static str {
    "Size diff:"
}

pub fn local_larger_bytes(bytes: i64) -> String {
    format!("Local larger by {} bytes", bytes)
}

pub fn cloud_larger_bytes(bytes: i64) -> String {
    format!("Cloud larger by {} bytes", bytes)
}

pub fn diff_items_label() -> &'static str {
    "Differences:"
}

pub fn diff_exists() -> &'static str {
    "Exists"
}

pub fn diff_sync() -> &'static str {
    "Sync"
}

pub fn diff_size() -> &'static str {
    "Size"
}

pub fn diff_time() -> &'static str {
    "Time"
}

pub fn hash_status_label() -> &'static str {
    "Hash Status:"
}

pub fn retry_hash_check() -> &'static str {
    "Retry Hash Check"
}

pub fn local_hash_label() -> &'static str {
    "Local Hash:"
}

pub fn cloud_hash_label() -> &'static str {
    "Cloud Hash:"
}

pub fn not_calculated() -> &'static str {
    "Not calculated"
}

pub fn error_delete_config(error: &str) -> String {
    format!("Failed to delete config: {}", error)
}

pub fn remote_dir_label() -> &'static str {
    "Remote Dir:"
}

pub fn copy_path() -> &'static str {
    "Copy Path"
}

pub fn symlink_conflict_label() -> &'static str {
    "Conflict"
}

pub fn steam_path_hint_text() -> &'static str {
    "Steam Install Path"
}

pub fn cloud_status_not_ready() -> &'static str {
    "Cloud Status: Not Ready"
}

pub fn game_file_info(count: usize, size: &str) -> String {
    format!(
        "{} file{} | {}",
        count,
        if count != 1 { "s" } else { "" },
        size
    )
}

pub fn install_dir_label(dir: &str) -> String {
    format!("Install dir: {}", dir)
}

pub fn tags_label(tags: &str) -> String {
    format!("Tags: {}", tags)
}

pub fn playtime_label(hours: f64) -> String {
    format!("Playtime: {:.2} hours", hours)
}

pub fn last_played_label(time: &str) -> String {
    format!("Last played: {}", time)
}

pub fn select_button() -> &'static str {
    "Select"
}

pub fn check_update_failed(error: &str) -> String {
    format!("Check update failed: {}", error)
}

pub fn theme_light() -> &'static str {
    "Light"
}

pub fn theme_dark() -> &'static str {
    "Dark"
}

pub fn theme_system() -> &'static str {
    "System"
}
