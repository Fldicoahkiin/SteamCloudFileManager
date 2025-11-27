use std::path::PathBuf;

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
        font_dirs.push("/usr/share/fonts/truetype".to_string());

        if let Ok(home) = std::env::var("HOME") {
            font_dirs.push(format!("{}/.fonts", home));
            font_dirs.push(format!("{}/.local/share/fonts", home));
        }

        for dir in font_dirs {
            if let Ok(walker) = std::fs::read_dir(&dir) {
                for entry in walker.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        if let Ok(sub_entries) = std::fs::read_dir(&path) {
                            for sub_entry in sub_entries.flatten() {
                                let sub_path = sub_entry.path();
                                if sub_path.extension().and_then(|s| s.to_str()) == Some("ttf") {
                                    font_paths.push(sub_path);
                                }
                            }
                        }
                    } else if path.extension().and_then(|s| s.to_str()) == Some("ttf") {
                        font_paths.push(path);
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
            if name.contains("msyh") || name.contains("microsoft yahei") {
                0
            } else if name.contains("simhei") || name.contains("heiti") {
                1
            } else if name.contains("arial") {
                2
            } else if name.contains("noto") && name.contains("cjk") {
                3
            } else if name.contains("sarasa") {
                4
            } else if name.contains("source") && name.contains("han") {
                5
            } else if name.contains("wenquanyi") {
                10
            } else {
                100
            }
        }
    });

    font_paths
}
