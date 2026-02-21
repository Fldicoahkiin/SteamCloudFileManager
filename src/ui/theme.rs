use egui::Color32;

// 主题模式
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ThemeMode {
    // 浅色主题
    Light,
    // 深色主题
    Dark,
    // 跟随系统
    #[default]
    System,
}

impl ThemeMode {
    // 获取所有主题模式
    pub fn all() -> &'static [ThemeMode] {
        &[ThemeMode::Dark, ThemeMode::Light, ThemeMode::System]
    }

    // 获取显示名称
    pub fn display_name(&self, i18n: &crate::i18n::I18n) -> &'static str {
        match self {
            ThemeMode::Light => i18n.theme_light(),
            ThemeMode::Dark => i18n.theme_dark(),
            ThemeMode::System => i18n.theme_system(),
        }
    }
}

// 检测系统是否为深色模式
fn detect_system_dark_mode() -> bool {
    #[cfg(target_os = "macos")]
    {
        // macOS: 使用 defaults read 命令检测系统主题
        if let Ok(output) = std::process::Command::new("defaults")
            .args(["read", "-g", "AppleInterfaceStyle"])
            .output()
        {
            // 如果输出包含 "Dark"，说明是深色模式
            // 如果命令失败或输出为空，说明是浅色模式
            let stdout = String::from_utf8_lossy(&output.stdout);
            return stdout.trim().eq_ignore_ascii_case("dark");
        }
        // 默认深色
        true
    }
    #[cfg(target_os = "windows")]
    {
        // Windows: 检查注册表
        // HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize
        // AppsUseLightTheme = 0 表示深色模式
        use std::os::windows::process::CommandExt;
        use std::process::Command;
        if let Ok(output) = Command::new("reg")
            .args([
                "query",
                "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize",
                "/v",
                "AppsUseLightTheme",
            ])
            .creation_flags(winapi::um::winbase::CREATE_NO_WINDOW)
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // 如果包含 "0x0"，说明是深色模式
            return stdout.contains("0x0");
        }
        true
    }
    #[cfg(target_os = "linux")]
    {
        // Linux: 尝试检测 GTK 主题或其他
        // 检查 GTK_THEME 环境变量
        if let Ok(theme) = std::env::var("GTK_THEME") {
            return theme.to_lowercase().contains("dark");
        }
        // 默认深色
        true
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        true
    }
}

// 全局主题模式状态
static CURRENT_MODE: std::sync::OnceLock<std::sync::RwLock<ThemeMode>> = std::sync::OnceLock::new();

// 判断当前是否为深色模式
pub fn is_dark_mode(ctx: &egui::Context) -> bool {
    ctx.style().visuals.dark_mode
}

// 应用主题到 egui Context
pub fn apply_theme(ctx: &egui::Context, mode: ThemeMode) {
    // 更新全局状态
    if let Some(lock) = CURRENT_MODE.get() {
        if let Ok(mut current) = lock.write() {
            *current = mode;
        }
    } else {
        let _ = CURRENT_MODE.set(std::sync::RwLock::new(mode));
    }

    // 决定使用深色还是浅色
    let use_dark = match mode {
        ThemeMode::Dark => true,
        ThemeMode::Light => false,
        ThemeMode::System => detect_system_dark_mode(),
    };

    // 使用 egui 原生主题
    let visuals = if use_dark {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };

    ctx.set_visuals(visuals);
}

// 获取适合当前主题的强调色（Steam Blue）
pub fn accent_color(ctx: &egui::Context) -> Color32 {
    if is_dark_mode(ctx) {
        Color32::from_rgb(102, 192, 244) // Steam Blue
    } else {
        Color32::from_rgb(0, 100, 160) // Darker blue for light mode
    }
}

// 获取适合当前主题的成功/已同步颜色（绿色）
pub fn success_color(ctx: &egui::Context) -> Color32 {
    if is_dark_mode(ctx) {
        Color32::from_rgb(46, 160, 67) // Slightly darker green
    } else {
        Color32::from_rgb(30, 100, 40) // Much darker green for light mode
    }
}

// 获取适合当前主题的警告颜色（黄/橙色）
pub fn warning_color(ctx: &egui::Context) -> Color32 {
    if is_dark_mode(ctx) {
        Color32::from_rgb(230, 160, 0) // Darker amber
    } else {
        Color32::from_rgb(160, 100, 0) // Much darker amber for light mode
    }
}

// 获取适合当前主题的错误颜色（红色）
pub fn error_color(ctx: &egui::Context) -> Color32 {
    if is_dark_mode(ctx) {
        Color32::from_rgb(220, 80, 80) // Red
    } else {
        Color32::from_rgb(180, 40, 40) // Darker red for light mode
    }
}

// 获取适合当前主题的信息颜色（蓝色）
pub fn info_color(ctx: &egui::Context) -> Color32 {
    if is_dark_mode(ctx) {
        Color32::from_rgb(80, 160, 220) // Blue
    } else {
        Color32::from_rgb(20, 100, 180) // Darker blue for light mode
    }
}

// 获取适合当前主题的同步状态颜色（云图标 - 仅云端）
pub fn cloud_only_color(ctx: &egui::Context) -> Color32 {
    if is_dark_mode(ctx) {
        Color32::from_rgb(200, 180, 100) // Darker khaki
    } else {
        Color32::from_rgb(140, 120, 30) // Much darker for light mode
    }
}

// 获取适合当前主题的次要/禁用颜色（灰色）
pub fn muted_color(ctx: &egui::Context) -> Color32 {
    if is_dark_mode(ctx) {
        Color32::from_rgb(120, 120, 120)
    } else {
        Color32::from_rgb(100, 100, 100)
    }
}

// 获取适合当前主题的本地存在标记颜色（深绿色勾号）
pub fn local_exists_color(ctx: &egui::Context) -> Color32 {
    if is_dark_mode(ctx) {
        Color32::from_rgb(0, 180, 80) // Bright green
    } else {
        Color32::from_rgb(0, 120, 50) // Darker green
    }
}

// 获取适合当前主题的云端存在标记颜色（深蓝色勾号）
pub fn cloud_exists_color(ctx: &egui::Context) -> Color32 {
    if is_dark_mode(ctx) {
        Color32::from_rgb(0, 150, 220) // Blue
    } else {
        Color32::from_rgb(0, 100, 170) // Darker blue
    }
}

// 获取适合当前主题的"已安装"标签颜色
pub fn installed_color(ctx: &egui::Context) -> Color32 {
    if is_dark_mode(ctx) {
        Color32::from_rgb(0, 180, 80)
    } else {
        Color32::from_rgb(0, 130, 50)
    }
}

// 获取主要按钮文字颜色（白色）
pub fn primary_button_text_color(_ctx: &egui::Context) -> Color32 {
    Color32::WHITE
}

// 获取透明颜色
pub fn transparent_color() -> Color32 {
    Color32::TRANSPARENT
}
