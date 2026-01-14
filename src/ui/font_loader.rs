use egui;
use std::path::PathBuf;
#[cfg(target_os = "linux")]
use walkdir::WalkDir;

// 设置应用字体
pub fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // 加载符号字体
    load_symbol_fonts(&mut fonts);

    // 加载 CJK 字体
    load_cjk_fonts(&mut fonts);

    // 加载 Phosphor 图标字体
    egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);

    ctx.set_fonts(fonts);

    // 应用默认主题
    crate::ui::theme::apply_theme(ctx, crate::ui::theme::ThemeMode::default());
}

// 加载符号字体
fn load_symbol_fonts(fonts: &mut egui::FontDefinitions) {
    #[cfg(target_os = "windows")]
    {
        if let Ok(windir) = std::env::var("WINDIR") {
            let symbols_path = PathBuf::from(&windir).join("Fonts").join("seguisym.ttf");
            if let Ok(data) = std::fs::read(&symbols_path) {
                fonts.font_data.insert(
                    "symbols".to_owned(),
                    egui::FontData::from_owned(data).into(),
                );
                fonts
                    .families
                    .entry(egui::FontFamily::Proportional)
                    .or_default()
                    .push("symbols".to_owned());
                fonts
                    .families
                    .entry(egui::FontFamily::Monospace)
                    .or_default()
                    .push("symbols".to_owned());
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        let candidates = [
            "/System/Library/Fonts/Apple Symbols.ttf",
            "/System/Library/Fonts/Supplemental/Symbols.ttf",
        ];
        for p in candidates {
            if let Ok(data) = std::fs::read(p) {
                fonts.font_data.insert(
                    "symbols".to_owned(),
                    egui::FontData::from_owned(data).into(),
                );
                fonts
                    .families
                    .entry(egui::FontFamily::Proportional)
                    .or_default()
                    .push("symbols".to_owned());
                fonts
                    .families
                    .entry(egui::FontFamily::Monospace)
                    .or_default()
                    .push("symbols".to_owned());
                break;
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        let candidates = [
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
            "/usr/share/fonts/truetype/dejavu/DejaVuSansCondensed.ttf",
            "/usr/share/fonts/truetype/noto/NotoSansSymbols2-Regular.ttf",
            "/usr/share/fonts/noto/NotoSansSymbols2-Regular.ttf",
        ];
        for p in candidates {
            if let Ok(data) = std::fs::read(p) {
                fonts.font_data.insert(
                    "symbols".to_owned(),
                    egui::FontData::from_owned(data).into(),
                );
                fonts
                    .families
                    .entry(egui::FontFamily::Proportional)
                    .or_default()
                    .push("symbols".to_owned());
                fonts
                    .families
                    .entry(egui::FontFamily::Monospace)
                    .or_default()
                    .push("symbols".to_owned());
                break;
            }
        }
    }
}

// 加载 CJK 字体
fn load_cjk_fonts(fonts: &mut egui::FontDefinitions) {
    let font_paths = find_system_fonts();

    for path in font_paths {
        if let Ok(data) = std::fs::read(&path) {
            tracing::info!("成功加载字体: {:?}", path);
            fonts.font_data.insert(
                "system_cjk".to_owned(),
                egui::FontData::from_owned(data).into(),
            );
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, "system_cjk".to_owned());
            fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .push("system_cjk".to_owned());
            return;
        }
    }
    tracing::warn!("未找到可用的 CJK 字体");
}

pub fn find_system_fonts() -> Vec<PathBuf> {
    let mut font_paths = Vec::new();

    #[cfg(target_os = "macos")]
    {
        let home_font = format!(
            "{}/Library/Fonts",
            std::env::var("HOME").unwrap_or_default()
        );
        let dirs = vec![
            "/System/Library/Fonts",
            "/Library/Fonts",
            home_font.as_str(),
        ];

        for dir in dirs {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("ttf") {
                        font_paths.push(path);
                    }
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        let mut font_dirs = Vec::new();

        if let Ok(windir) = std::env::var("WINDIR") {
            font_dirs.push(PathBuf::from(format!("{}/Fonts", windir)));
        }

        if let Ok(localappdata) = std::env::var("LOCALAPPDATA") {
            font_dirs.push(PathBuf::from(format!(
                "{}\\Microsoft\\Windows\\Fonts",
                localappdata
            )));
        }

        for font_dir in font_dirs {
            if let Ok(entries) = std::fs::read_dir(&font_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if let Some(ext) = path.extension() {
                        let ext_str = ext.to_str().unwrap_or("").to_lowercase();
                        if ext_str == "ttf" || ext_str == "ttc" || ext_str == "otf" {
                            font_paths.push(path);
                        }
                    }
                }
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        let mut font_dirs = Vec::new();
        font_dirs.push("/usr/share/fonts".to_string());
        font_dirs.push("/usr/local/share/fonts".to_string());

        if let Ok(home) = std::env::var("HOME") {
            font_dirs.push(format!("{}/.fonts", home));
            font_dirs.push(format!("{}/.local/share/fonts", home));
        }

        for dir in font_dirs {
            for entry in WalkDir::new(&dir)
                .follow_links(true)
                .max_depth(10)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let path = entry.path();
                if path.is_file()
                    && let Some(ext) = path.extension()
                {
                    let ext_str = ext.to_str().unwrap_or("").to_lowercase();
                    if ext_str == "ttf" || ext_str == "ttc" || ext_str == "otf" {
                        font_paths.push(path.to_path_buf());
                    }
                }
            }
        }
    }

    font_paths.sort_by_key(|p| {
        let name = p
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_lowercase();

        #[cfg(target_os = "windows")]
        {
            if name.contains("msyh") || name.contains("microsoft yahei") {
                0
            } else if name.contains("simsun") {
                1
            } else if name.contains("simhei") {
                2
            } else if name.contains("arial") {
                3
            } else if name.contains("segoe") {
                4
            } else if name.contains("noto") && name.contains("cjk") {
                5
            } else {
                100
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            if name.contains("noto")
                && (name.contains("cjk") || name.contains("sans") && name.contains("sc"))
            {
                0
            } else if name.contains("sarasa") || name.contains("更纱") {
                1
            } else if name.contains("source") && name.contains("han") {
                2
            } else if name.contains("msyh") || name.contains("microsoft yahei") {
                3
            } else if name.contains("simhei") || name.contains("heiti") || name.contains("黑体") {
                4
            } else if name.contains("wenquanyi") || name.contains("文泉驿") {
                5
            } else if name.contains("droid") && name.contains("sans") && name.contains("fallback") {
                6
            } else if name.contains("dejavu") {
                7
            } else if name.contains("liberation") {
                8
            } else if name.contains("arial") {
                9
            } else {
                100
            }
        }
    });

    font_paths
}
