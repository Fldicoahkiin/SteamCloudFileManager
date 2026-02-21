// 简体中文翻译

use crate::icons;

pub fn language_label() -> &'static str {
    "语言:"
}

pub fn app_title() -> &'static str {
    "Steam 云存档管理器"
}

pub fn refresh() -> &'static str {
    "刷新"
}

pub fn cancel() -> &'static str {
    "取消"
}

pub fn close() -> &'static str {
    "关闭"
}

pub fn ok() -> &'static str {
    "确定"
}

pub fn logged_in() -> &'static str {
    "已登录"
}

pub fn not_logged_in() -> &'static str {
    "未登录"
}

pub fn connect() -> &'static str {
    "连接"
}

pub fn disconnect() -> &'static str {
    "断开"
}

pub fn disconnect_sync_hint() -> &'static str {
    "断开后 Steam 将自动同步"
}

pub fn refresh_open_url_hint() -> &'static str {
    "在 Steam 中打开云存储页面"
}

pub fn show_appinfo_vdf() -> &'static str {
    "配置 appinfo.vdf"
}

pub fn account_cloud_status() -> &'static str {
    "账户云存储"
}

pub fn select_account() -> &'static str {
    "账户"
}

pub fn select_game() -> &'static str {
    "游戏库"
}

pub fn select_all() -> &'static str {
    "全选"
}

pub fn invert_selection() -> &'static str {
    "反选"
}

pub fn clear_selection() -> &'static str {
    "清除选择"
}

pub fn download() -> &'static str {
    "下载"
}

pub fn upload() -> &'static str {
    "上传"
}

pub fn delete() -> &'static str {
    "删除"
}

pub fn forget() -> &'static str {
    "移出云端"
}

pub fn sync_to_cloud() -> &'static str {
    "同步云端"
}

pub fn file_name() -> &'static str {
    "文件名"
}

pub fn size() -> &'static str {
    "大小"
}

pub fn selected_count(count: usize) -> String {
    format!("已选择 {} 个", count)
}

pub fn select_game_title() -> &'static str {
    "选择游戏"
}

pub fn settings_title() -> &'static str {
    "设置"
}

pub fn settings_log() -> &'static str {
    "日志"
}

pub fn settings_about() -> &'static str {
    "关于"
}

pub fn settings_appearance() -> &'static str {
    "外观"
}

pub fn settings_advanced() -> &'static str {
    "高级"
}

pub fn steam_path_label() -> &'static str {
    "Steam 安装路径"
}

pub fn steam_path_auto_detect() -> &'static str {
    "自动检测"
}

pub fn steam_path_browse() -> &'static str {
    "浏览..."
}

pub fn steam_path_valid(user_count: usize) -> String {
    format!("✓ 路径有效 (检测到 {} 个用户)", user_count)
}

pub fn steam_path_not_exists() -> &'static str {
    "✗ 路径不存在"
}

pub fn steam_path_no_userdata() -> &'static str {
    "✗ 缺少 userdata 目录"
}

pub fn steam_path_no_users() -> &'static str {
    "✗ 未找到用户"
}

pub fn steam_path_hint() -> &'static str {
    "💡 如果 Steam 安装在非标准位置，请手动选择目录"
}

pub fn steam_path_restart_hint() -> &'static str {
    "修改后需要重启应用生效"
}

pub fn reset_all_settings() -> &'static str {
    "恢复默认设置"
}

pub fn reset_confirm() -> &'static str {
    "确定要恢复所有设置为默认值吗？"
}

pub fn config_dir_label() -> &'static str {
    "配置文件:"
}

pub fn open_config_dir() -> &'static str {
    "打开配置目录"
}

pub fn theme_mode_label() -> &'static str {
    "主题模式:"
}

pub fn error_title() -> &'static str {
    "错误"
}

pub fn author() -> &'static str {
    "作者:"
}

pub fn github_repository() -> &'static str {
    "Github仓库:"
}

pub fn connecting() -> &'static str {
    "连接中..."
}

pub fn games_with_cloud(count: usize) -> String {
    format!("{} 个有云存档的游戏", count)
}

pub fn scanning_games() -> &'static str {
    "正在扫描游戏库..."
}

pub fn no_cloud_games_found() -> &'static str {
    "未发现云存档的游戏"
}

pub fn installed() -> &'static str {
    "已安装"
}

pub fn not_installed() -> &'static str {
    "未安装"
}

pub fn select_user() -> &'static str {
    "选择用户"
}

pub fn user_id() -> &'static str {
    "用户 ID"
}

pub fn current_user() -> &'static str {
    "当前用户"
}

pub fn switch() -> &'static str {
    "切换"
}

pub fn steam_users(count: usize) -> String {
    format!("{} 个 Steam 用户", count)
}

pub fn checking_update() -> String {
    format!("{} 检查中...", icons::SPINNER)
}

pub fn check_update_btn() -> String {
    format!("{} 检查更新", icons::REFRESH)
}

pub fn already_latest() -> String {
    format!("{} 当前已是最新版本", icons::CHECK)
}

pub fn new_version_found(version: &str) -> String {
    format!("🎉 发现新版本: {}", version)
}

pub fn new_version_hint() -> &'static str {
    "发现新版本，点击下载并安装："
}

pub fn download_and_install() -> String {
    format!("{} 下载并安装", icons::DOWNLOAD)
}

pub fn view_details() -> String {
    format!("{} 查看详情", icons::GLOBE)
}

pub fn downloading_update() -> String {
    format!("{} 正在下载更新...", icons::DOWNLOAD)
}

pub fn installing_update() -> String {
    format!("{} 正在安装更新...", icons::GEAR)
}

pub fn update_success() -> String {
    format!("{} 更新安装成功！", icons::CHECK)
}

pub fn restart_to_apply() -> &'static str {
    "请重启应用以使用新版本"
}

pub fn restart_now() -> String {
    format!("{} 立即重启", icons::REFRESH)
}

pub fn log_enabled_hint() -> &'static str {
    " 日志存储已启用，重启后生效"
}

pub fn log_disabled_hint() -> &'static str {
    " 日志存储已禁用，重启后生效"
}

pub fn enable_log_storage() -> &'static str {
    "启用日志存储"
}

pub fn open_log_dir() -> &'static str {
    " 打开日志目录"
}

pub fn log_dir_label() -> &'static str {
    "日志目录:"
}

pub fn steam_log_dir_label() -> &'static str {
    "Steam 日志目录:"
}

pub fn open_steam_log_dir() -> &'static str {
    " 打开 Steam 日志目录"
}

pub fn restarting_steam() -> &'static str {
    "正在重启 Steam"
}

pub fn manual_operation_required() -> &'static str {
    "需要手动操作："
}

pub fn i_understand() -> &'static str {
    "我知道了"
}

#[cfg(target_os = "macos")]
pub fn manual_restart_macos_title() -> &'static str {
    "手动重启 Steam (macOS)"
}

#[cfg(target_os = "windows")]
pub fn manual_restart_windows_title() -> &'static str {
    "手动重启 Steam (Windows)"
}

#[cfg(target_os = "linux")]
pub fn manual_restart_linux_title() -> &'static str {
    "手动重启 Steam (Linux)"
}

pub fn prepare_upload() -> &'static str {
    "准备上传"
}

pub fn will_upload_files(count: usize) -> String {
    format!("将要上传 {} 个文件到 Steam Cloud", count)
}

pub fn total_size_label(size: &str) -> String {
    format!("总大小: {}", size)
}

pub fn warning() -> String {
    format!("{} 警告：", icons::WARNING)
}

pub fn overwrite_warning() -> &'static str {
    "• 同名文件将被覆盖"
}

pub fn add_files() -> String {
    format!("{} 添加文件", icons::ADD_FILE)
}

pub fn add_folder() -> String {
    format!("{} 添加文件夹", icons::ADD_FOLDER)
}

pub fn confirm_upload() -> String {
    format!("{} 确认上传", icons::CHECK)
}

pub fn remove_file() -> &'static str {
    "移除"
}

pub fn cloud_path() -> &'static str {
    "云端路径"
}

pub fn edit_path() -> &'static str {
    "编辑路径"
}

pub fn local_file() -> &'static str {
    "本地文件"
}

pub fn no_files_to_upload() -> &'static str {
    "没有文件待上传，请添加文件"
}

pub fn clear_all() -> &'static str {
    "清空列表"
}

pub fn uploading_files() -> String {
    format!("{} 正在上传文件", icons::UPLOAD)
}

pub fn uploading_file(name: &str) -> String {
    format!("正在上传: {}", name)
}

pub fn upload_progress(current: usize, total: usize) -> String {
    format!("进度: {} / {} 文件", current, total)
}

pub fn speed(speed: &str) -> String {
    format!("速度: {}/s", speed)
}

pub fn upload_complete() -> String {
    format!("{} 上传完成", icons::CHECK)
}

pub fn upload_success(count: usize) -> String {
    format!("{} 成功上传 {} 个文件", icons::ROCKET, count)
}

pub fn upload_partial(success: usize, failed: usize) -> String {
    format!(
        "{} 上传完成：成功 {}，失败 {}",
        icons::WARNING,
        success,
        failed
    )
}

pub fn elapsed_time(secs: u64) -> String {
    format!("用时: {} 秒", secs)
}

pub fn avg_speed(speed: &str) -> String {
    format!("平均速度: {}/s", speed)
}

pub fn failed_files() -> &'static str {
    "失败文件列表："
}

pub fn reason(err: &str) -> String {
    format!("  原因: {}", err)
}

pub fn closing_steam() -> &'static str {
    "正在关闭 Steam..."
}

pub fn starting_steam() -> &'static str {
    "正在启动 Steam..."
}

pub fn steam_restart_success() -> &'static str {
    "Steam 已成功重启!"
}

pub fn user_switched() -> &'static str {
    "已切换用户"
}

pub fn error_enter_app_id() -> &'static str {
    "请输入App ID"
}

pub fn error_invalid_app_id() -> &'static str {
    "无效的 App ID"
}

pub fn status_enter_app_id() -> &'static str {
    "请输入App ID并连接到Steam"
}

pub fn status_loading_files() -> &'static str {
    "正在加载文件列表..."
}

pub fn status_files_loaded(count: usize) -> String {
    format!("已加载 {} 个文件", count)
}

pub fn upload_failed(err: &str) -> String {
    format!("上传失败: {}", err)
}

pub fn error_no_files_selected() -> &'static str {
    "请选择要操作的文件"
}

pub fn error_not_connected() -> &'static str {
    "未连接到 Steam"
}

pub fn hint_you_can() -> &'static str {
    "您可以："
}

pub fn hint_select_game() -> &'static str {
    "点击上方的 '游戏库' 按钮选择游戏"
}

pub fn hint_enter_app_id() -> &'static str {
    "或直接输入 App ID 并点击 '连接'"
}

pub fn no_cloud_files() -> &'static str {
    "没有找到云文件"
}

pub fn no_cloud_files_hint() -> &'static str {
    "该游戏没有云存档文件"
}

pub fn scan_games_failed(err: &str) -> String {
    format!("扫描游戏失败: {}", err)
}

pub fn refresh_files_failed(err: &str) -> String {
    format!("刷新文件列表失败: {}", err)
}

pub fn cdp_no_data_error() -> &'static str {
    "CDP 未获取到游戏数据！\n\n可能原因：\n\
    1. Steam 客户端未响应跳转请求\n\
    2. 页面加载未完成\n\
    3. 未登录 Steam 网页\n\n"
}

pub fn connecting_to_steam(app_id: u32) -> String {
    format!("正在连接到 Steam (App ID: {})...", app_id)
}

pub fn loading_files_for_app(app_id: u32) -> String {
    format!("正在加载文件列表 (App ID: {})...", app_id)
}

pub fn connect_steam_failed(err: &str) -> String {
    format!("连接Steam失败: {}", err)
}

pub fn vdf_parser_not_initialized() -> &'static str {
    "VDF 解析器未初始化"
}

pub fn scanning_game_library() -> &'static str {
    "正在扫描游戏库..."
}

pub fn drop_files_to_upload() -> &'static str {
    "释放文件以上传"
}

pub fn debug_mode_not_enabled() -> String {
    format!("{} Steam 调试模式未启用", icons::WARNING)
}

pub fn steam_running() -> String {
    format!("{} Steam 正在运行", icons::CHECK)
}

pub fn steam_not_running() -> String {
    format!("{} Steam 未运行", icons::CLOSE)
}

pub fn debug_mode_hint() -> &'static str {
    "需要启用 Steam 的 CEF 调试模式才能获取到云端数据"
}

pub fn auto_restart_steam() -> &'static str {
    "自动重启 Steam"
}

pub fn start_steam() -> &'static str {
    "启动 Steam"
}

pub fn auto_restart_hint() -> &'static str {
    "自动关闭并重启 Steam，添加调试参数"
}

pub fn start_steam_hint() -> &'static str {
    "以调试模式启动 Steam"
}

pub fn view_manual_steps() -> &'static str {
    "查看手动操作"
}

pub fn manual_steps_hint() -> &'static str {
    "显示如何手动添加启动参数"
}

pub fn dismiss_temporarily() -> String {
    format!("{} 暂时忽略", icons::CLOSE)
}

pub fn dismiss_hint() -> &'static str {
    "隐藏此提示（可在设置中重新显示）"
}

pub fn status_label() -> &'static str {
    "状态:"
}

pub fn cloud_on() -> &'static str {
    "云存储: 开启"
}

pub fn cloud_off() -> &'static str {
    "云存储: 关闭"
}

pub fn quota_usage(percent: f32, used: &str, total: &str) -> String {
    format!("配额: {:.1}% 已使用 ({}/{})", percent, used, total)
}

pub fn select_all_hint() -> &'static str {
    "选择列表中的所有文件"
}

pub fn invert_selection_hint() -> &'static str {
    "反转当前选择状态"
}

pub fn clear_selection_hint() -> &'static str {
    "取消选择所有文件"
}

pub fn download_hint() -> &'static str {
    "下载选中的文件到本地"
}

pub fn upload_hint() -> &'static str {
    "上传文件或文件夹到云端"
}

pub fn delete_hint() -> &'static str {
    "从云端和本地删除选中的文件"
}

pub fn forget_hint() -> &'static str {
    "仅从云端移除，保留本地文件"
}

pub fn sync_to_cloud_hint() -> &'static str {
    "将本地文件同步到云端"
}

pub fn connect_hint() -> &'static str {
    "连接到 Steam 云存储 API"
}

pub fn disconnect_hint() -> &'static str {
    "断开与 Steam 的连接"
}

pub fn select_account_hint() -> &'static str {
    "切换 Steam 账户"
}

pub fn select_game_hint() -> &'static str {
    "选择要管理云存档的游戏"
}

pub fn local_save_path() -> &'static str {
    "本地存档路径:"
}

pub fn local_save_path_not_found() -> &'static str {
    "未找到（可能所有文件都仅在云端）"
}

pub fn search_files_placeholder() -> &'static str {
    "搜索文件或文件夹..."
}

pub fn clear() -> &'static str {
    "清除"
}

pub fn only_local() -> &'static str {
    "仅本地"
}

pub fn only_cloud() -> &'static str {
    "仅云端"
}

pub fn only_local_tooltip() -> &'static str {
    "只显示仅在本地存在的文件（未同步到云端）"
}

pub fn only_cloud_tooltip() -> &'static str {
    "只显示仅在云端存在的文件（本地不存在）"
}

pub fn root_folder() -> &'static str {
    "根文件夹"
}

pub fn file_size() -> &'static str {
    "文件大小"
}

pub fn write_date() -> &'static str {
    "写入日期"
}

pub fn local() -> &'static str {
    "本地"
}

pub fn cloud() -> &'static str {
    "云端"
}

pub fn file_comparison_title() -> &'static str {
    "文件对比"
}

pub fn total_files_count(count: usize) -> String {
    format!("共 {} 个文件", count)
}

pub fn filter_all() -> &'static str {
    "全部"
}

pub fn filter_conflicts() -> &'static str {
    "冲突"
}

pub fn filter_local_newer() -> &'static str {
    "本地较新"
}

pub fn filter_cloud_newer() -> &'static str {
    "云端较新"
}

pub fn filter_synced() -> &'static str {
    "已同步"
}

pub fn status_local_newer() -> &'static str {
    "本地新"
}

pub fn status_cloud_newer() -> &'static str {
    "云端新"
}

pub fn status_conflict() -> &'static str {
    "冲突"
}

pub fn status_local_only() -> &'static str {
    "仅本地"
}

pub fn status_cloud_only() -> &'static str {
    "仅云端"
}

pub fn column_status() -> &'static str {
    "状态"
}

pub fn column_filename() -> &'static str {
    "文件名"
}

pub fn column_local_size() -> &'static str {
    "本地大小"
}

pub fn column_cloud_size() -> &'static str {
    "云端大小"
}

pub fn column_local_time() -> &'static str {
    "本地时间"
}

pub fn column_cloud_time() -> &'static str {
    "云端时间"
}

pub fn selected_file() -> &'static str {
    "选中文件:"
}

pub fn local_newer_by_minutes(mins: i64) -> String {
    format!("(本地比云端新 {} 分钟)", mins)
}

pub fn cloud_newer_by_minutes(mins: i64) -> String {
    format!("(云端比本地新 {} 分钟)", mins)
}

pub fn conflicts_warning(count: usize) -> String {
    format!("检测到 {} 个冲突，请手动解决", count)
}

pub fn compare_files() -> &'static str {
    "对比文件"
}

pub fn compare_files_hint() -> &'static str {
    "对比本地和云端文件的差异"
}

pub fn backup() -> &'static str {
    "备份"
}

pub fn backup_title() -> &'static str {
    "备份云存档"
}

pub fn backup_file_count(count: usize) -> String {
    format!("共 {} 个文件", count)
}

pub fn backup_total_size(size: &str) -> String {
    format!("总大小: {}", size)
}

pub fn backup_cdp_warning(count: usize) -> String {
    format!("{} {} 个文件无下载链接，将跳过", icons::WARNING, count)
}

pub fn backup_file_list() -> &'static str {
    "文件列表"
}

pub fn backup_start() -> &'static str {
    "开始备份"
}

pub fn backup_open_dir() -> &'static str {
    "打开备份目录"
}

pub fn backup_progress_title() -> &'static str {
    "备份进度"
}

pub fn backup_in_progress() -> &'static str {
    "正在备份..."
}

pub fn backup_complete() -> String {
    format!("{} 备份完成", icons::CHECK)
}

pub fn backup_partial() -> String {
    format!("{} 部分完成", icons::WARNING)
}

pub fn backup_result_stats(success: usize, total: usize) -> String {
    format!("成功: {} / {}", success, total)
}

pub fn backup_failed_files() -> &'static str {
    "失败的文件:"
}

pub fn backup_hint() -> &'static str {
    "备份当前游戏的所有云存档"
}

pub fn backup_dir_label() -> &'static str {
    "备份目录:"
}

pub fn download_progress_title() -> &'static str {
    "下载进度"
}

pub fn download_in_progress() -> &'static str {
    "正在下载..."
}

pub fn download_complete() -> String {
    format!("{} 下载完成", icons::CHECK)
}

pub fn download_partial_status() -> String {
    format!("{} 部分完成", icons::WARNING)
}

pub fn download_result_stats(success: usize, total: usize) -> String {
    format!("成功: {} / {}", success, total)
}

pub fn download_failed_files() -> &'static str {
    "失败的文件:"
}

pub fn download_open_dir() -> &'static str {
    "打开下载目录"
}

pub fn symlink_title() -> &'static str {
    "软链接管理 (实验性)"
}

pub fn symlink_configured_links() -> &'static str {
    "已配置的软链接"
}

pub fn symlink_no_configs() -> &'static str {
    "暂无软链接配置"
}

pub fn symlink_add_new() -> &'static str {
    "添加新软链接"
}

pub fn symlink_direction() -> &'static str {
    "方向:"
}

pub fn symlink_local_path() -> &'static str {
    "本地路径:"
}

pub fn symlink_remote_subfolder() -> &'static str {
    "Remote 子目录:"
}

pub fn symlink_browse() -> &'static str {
    "选择文件夹"
}

pub fn symlink_add_config() -> &'static str {
    "添加配置"
}

pub fn symlink_add_and_create() -> &'static str {
    "添加并创建链接"
}

pub fn symlink_create() -> &'static str {
    "创建链接"
}

pub fn symlink_remove_link() -> &'static str {
    "删除链接"
}

pub fn symlink_delete_config() -> &'static str {
    "删除配置"
}

pub fn symlink_copy_command() -> &'static str {
    "复制命令"
}

pub fn symlink_refresh() -> &'static str {
    "刷新"
}

pub fn symlink_command_copied() -> &'static str {
    "命令已复制到剪贴板"
}

pub fn symlink_config_deleted() -> &'static str {
    "配置已删除"
}

pub fn symlink_config_added() -> &'static str {
    "配置已添加"
}

pub fn symlink_created() -> &'static str {
    "软链接已创建"
}

pub fn symlink_removed() -> &'static str {
    "软链接已删除"
}

pub fn symlink_create_failed() -> &'static str {
    "创建失败"
}

pub fn symlink_remove_failed() -> &'static str {
    "删除失败"
}

pub fn symlink_add_failed() -> &'static str {
    "添加配置失败"
}

pub fn symlink_experimental_title() -> &'static str {
    "实验性功能"
}

pub fn symlink_experimental_desc() -> &'static str {
    "创建软链接后会自动同步目录下的文件到云端。点击云端上传按钮可手动同步新增文件。"
}

pub fn symlink_sync_files() -> &'static str {
    "同步文件到云端"
}

pub fn symlink_sync_success() -> &'static str {
    "同步成功"
}

pub fn symlink_sync_partial() -> &'static str {
    "部分同步成功"
}

pub fn symlink_sync_no_files() -> &'static str {
    "目录为空，无文件需要同步"
}

pub fn symlink_sync_no_manager() -> &'static str {
    "软链接管理器未初始化"
}

pub fn symlink_sync_no_steam() -> &'static str {
    "Steam 未连接，无法同步"
}

pub fn symlink_sync_scan_failed() -> &'static str {
    "扫描目录失败"
}

pub fn files() -> &'static str {
    "个文件"
}

pub fn appinfo_tab_local_ufs() -> &'static str {
    "本地 UFS 配置"
}

pub fn appinfo_tab_custom_config() -> &'static str {
    "自定义配置"
}

pub fn appinfo_debug_title(app_id: u32) -> String {
    format!("appinfo.vdf 调试 - App {}", app_id)
}

pub fn appinfo_quota() -> &'static str {
    "配额:"
}

pub fn appinfo_max_files() -> &'static str {
    "最大文件数:"
}

pub fn appinfo_current_ufs() -> &'static str {
    "当前 UFS 云存储配置:"
}

pub fn appinfo_restart_steam() -> String {
    format!("{} 重启 Steam", icons::REFRESH)
}

pub fn appinfo_warning() -> String {
    format!(
        "{} 此功能为实验性质。修改 appinfo.vdf 可能被 Steam 覆盖。\n\
    需要在 Steam 启动前注入，或在注入后立即重启 Steam。",
        icons::WARNING
    )
}

pub fn appinfo_path_hint() -> &'static str {
    "例如: MyGame/Saves"
}

pub fn appinfo_pattern_hint() -> &'static str {
    "* 或 *.sav"
}

pub fn ufs_savefiles_header(count: usize) -> String {
    format!("存档文件配置 ({})", count)
}

pub fn ufs_overrides_header(count: usize) -> String {
    format!("跨平台路径映射 ({})", count)
}

pub fn ufs_add_savefile() -> &'static str {
    "添加存档路径"
}

pub fn ufs_add_override() -> &'static str {
    "添加路径映射"
}

pub fn ufs_no_savefiles() -> &'static str {
    "暂无存档配置 — 点击添加创建"
}

pub fn ufs_no_overrides() -> &'static str {
    "暂无路径映射 — 添加以支持跨平台"
}

pub fn ufs_label_root() -> &'static str {
    "根目录"
}

pub fn ufs_label_path() -> &'static str {
    "路径"
}

pub fn ufs_label_pattern() -> &'static str {
    "匹配"
}

pub fn ufs_label_platforms() -> &'static str {
    "平台"
}

pub fn ufs_label_recursive() -> &'static str {
    "递归搜索子目录"
}

pub fn ufs_label_actions() -> &'static str {
    "操作"
}

pub fn ufs_label_original_root() -> &'static str {
    "原始根目录"
}

pub fn ufs_label_target_os() -> &'static str {
    "目标系统"
}

pub fn ufs_label_new_root() -> &'static str {
    "新根目录"
}

pub fn ufs_label_add_path() -> &'static str {
    "追加路径"
}

pub fn ufs_label_replace_path() -> &'static str {
    "替换路径"
}

pub fn ufs_label_replace_with() -> &'static str {
    "替换为:"
}

pub fn ufs_label_find_path() -> &'static str {
    "原始路径"
}

pub fn ufs_hint_auto_fill() -> &'static str {
    "留空自动填充"
}

pub fn ufs_refresh() -> &'static str {
    "刷新"
}

pub fn ufs_clear_all() -> &'static str {
    "清空全部"
}

pub fn ufs_clear_all_tooltip() -> &'static str {
    "清空所有自定义存档路径和路径映射"
}

pub fn ufs_save_config() -> String {
    format!("{} 保存配置", icons::SAVE)
}

pub fn ufs_inject_to_vdf() -> String {
    format!("{} 注入到 VDF", icons::CLOUD_UPLOAD)
}

pub fn ufs_inject_success(savefiles: usize, overrides: usize) -> String {
    format!("已注入 {} 个存档路径, {} 个路径映射", savefiles, overrides)
}

pub fn ufs_inject_empty() -> &'static str {
    "无存档路径或路径映射可注入"
}

pub fn ufs_inject_error(error: &str) -> String {
    format!("注入失败: {}", error)
}

pub fn ufs_writer_init_error(error: &str) -> String {
    format!("写入器初始化失败: {}", error)
}

pub fn ufs_save_success(savefiles: usize, overrides: usize) -> String {
    format!("已保存 {} 个存档路径, {} 个路径映射", savefiles, overrides)
}

pub fn ufs_save_error(error: &str) -> String {
    format!("保存失败: {}", error)
}

pub fn ufs_clear_success() -> &'static str {
    "已清除所有自定义配置"
}

pub fn ufs_clear_error(error: &str) -> String {
    format!("清除失败: {}", error)
}

pub fn error_get_appinfo(error: &str) -> String {
    format!("无法获取 appinfo: {}", error)
}

pub fn error_vdf_parser_init(error: &str) -> String {
    format!("VDF 解析器初始化失败: {}", error)
}

pub fn error_load_timeout() -> &'static str {
    "加载超时，请重试"
}

pub fn disconnected() -> &'static str {
    "已断开连接"
}

pub fn error_install_failed(error: &str) -> String {
    format!("安装失败: {}\n\n请手动下载更新", error)
}

pub fn error_download_failed(error: &str) -> String {
    format!("下载失败: {}\n\n请手动下载更新", error)
}

pub fn error_select_files_to_forget() -> &'static str {
    "请选择要移出云端的文件"
}

pub fn error_local_only_no_forget(count: usize) -> String {
    format!("所选 {} 个文件仅存在于本地，云端无记录，无需移出", count)
}

pub fn forgotten_files(count: usize) -> String {
    format!("已移出云端 {} 个文件", count)
}

pub fn ufs_forget_failed(count: usize) -> String {
    format!(
        "{} 个自动云同步文件无法通过 API 移出，请尝试使用「删除」功能",
        count
    )
}

pub fn forget_failed_files(count: usize, names: &str) -> String {
    format!("{} 个文件移出失败: {}", count, names)
}

pub fn skipped_local_only_files(count: usize) -> String {
    format!("跳过 {} 个本地独有文件", count)
}

pub fn no_files_forgotten() -> &'static str {
    "没有文件被移出云端"
}

pub fn error_select_files_to_delete() -> &'static str {
    "请选择要删除的文件"
}

pub fn deleted_files(count: usize) -> String {
    format!("已删除 {} 个文件", count)
}

pub fn ufs_cloud_sync_hint() -> &'static str {
    "自动云同步文件的云端副本将在 Steam 同步后自动删除，请稍后刷新确认"
}

pub fn ufs_delete_failed(count: usize) -> String {
    format!(
        "{} 个自动云同步文件无法删除（游戏未安装且 API 不支持，请安装游戏后重试）",
        count
    )
}

pub fn delete_failed_files(count: usize) -> String {
    format!("{} 个文件删除失败", count)
}

pub fn no_files_deleted() -> &'static str {
    "没有文件被删除"
}

pub fn error_select_files_to_sync() -> &'static str {
    "请选择要同步的文件"
}

pub fn synced_files_to_cloud(count: usize) -> String {
    format!("已同步 {} 个文件到云端", count)
}

pub fn all_files_in_cloud(count: usize) -> String {
    format!("所有 {} 个文件已在云端，无需同步", count)
}

pub fn no_files_synced() -> &'static str {
    "没有文件被同步"
}

pub fn partial_sync_failed(names: &str) -> String {
    format!("部分文件同步失败: {}", names)
}

pub fn sync_status_synced() -> String {
    format!("{} 已同步", crate::icons::CHECK)
}

pub fn sync_status_local_newer() -> String {
    format!("{} 本地较新", crate::icons::ARROW_UP)
}

pub fn sync_status_cloud_newer() -> String {
    format!("{} 云端较新", crate::icons::ARROW_DOWN)
}

pub fn sync_status_conflict() -> String {
    format!("{} 冲突", crate::icons::WARNING)
}

pub fn sync_status_local_only() -> String {
    format!("{} 仅本地", crate::icons::FILE)
}

pub fn sync_status_cloud_only() -> String {
    format!("{} 仅云端", crate::icons::CLOUD)
}

pub fn sync_status_unknown() -> String {
    format!("{} 检测中", crate::icons::QUESTION)
}

pub fn hash_status_pending() -> String {
    format!("{} 等待", crate::icons::HOURGLASS)
}

pub fn hash_status_skipped() -> String {
    format!("{} 已跳过", crate::icons::CHECK)
}

pub fn hash_status_checking() -> String {
    format!("{} 检测中", crate::icons::SPINNER)
}

pub fn hash_status_match() -> String {
    format!("{} 一致", crate::icons::CHECK)
}

pub fn hash_status_mismatch() -> String {
    format!("{} 不一致", crate::icons::ERROR)
}

pub fn hash_status_error() -> String {
    format!("{} 错误", crate::icons::WARNING)
}

pub fn size_diff_label() -> &'static str {
    "大小差异:"
}

pub fn local_larger_bytes(bytes: i64) -> String {
    format!("本地大 {} bytes", bytes)
}

pub fn cloud_larger_bytes(bytes: i64) -> String {
    format!("云端大 {} bytes", bytes)
}

pub fn diff_items_label() -> &'static str {
    "差异项:"
}

pub fn diff_exists() -> &'static str {
    "存在"
}

pub fn diff_sync() -> &'static str {
    "同步"
}

pub fn diff_size() -> &'static str {
    "大小"
}

pub fn diff_time() -> &'static str {
    "时间"
}

pub fn hash_status_label() -> &'static str {
    "Hash 状态:"
}

pub fn retry_hash_check() -> &'static str {
    "重新检测 Hash"
}

pub fn local_hash_label() -> &'static str {
    "本地 Hash:"
}

pub fn cloud_hash_label() -> &'static str {
    "云端 Hash:"
}

pub fn not_calculated() -> &'static str {
    "未计算"
}

pub fn error_delete_config(error: &str) -> String {
    format!("删除配置失败: {}", error)
}

pub fn remote_dir_label() -> &'static str {
    "Remote 目录:"
}

pub fn copy_path() -> &'static str {
    "复制路径"
}

pub fn symlink_conflict_label() -> &'static str {
    "冲突"
}

pub fn steam_path_hint_text() -> &'static str {
    "Steam 安装路径"
}

pub fn cloud_status_not_ready() -> &'static str {
    "云存储状态: 未就绪"
}

pub fn game_file_info(count: usize, size: &str) -> String {
    format!("{} 个文件 | {}", count, size)
}

pub fn install_dir_label(dir: &str) -> String {
    format!("安装目录: {}", dir)
}

pub fn tags_label(tags: &str) -> String {
    format!("标签: {}", tags)
}

pub fn playtime_label(hours: f64) -> String {
    format!("游戏时间: {:.2} 小时", hours)
}

pub fn last_played_label(time: &str) -> String {
    format!("最后运行: {}", time)
}

pub fn select_button() -> &'static str {
    "选择"
}

pub fn check_update_failed(error: &str) -> String {
    format!("检查更新失败: {}", error)
}

pub fn theme_light() -> &'static str {
    "浅色"
}

pub fn theme_dark() -> &'static str {
    "深色"
}

pub fn theme_system() -> &'static str {
    "跟随系统"
}
