// UFS 配置文本 ↔ 编辑数据双向转换

use crate::config::{PathTransform, RootOverrideEntry, SaveFileEntry};

// 将编辑数据序列化为 VDF 文本格式
pub fn entries_to_ufs_text(savefiles: &[SaveFileEntry], overrides: &[RootOverrideEntry]) -> String {
    let mut lines = Vec::new();
    lines.push("\"ufs\"".to_string());
    lines.push("{".to_string());

    // savefiles section
    if !savefiles.is_empty() {
        lines.push("    \"savefiles\"".to_string());
        lines.push("    {".to_string());
        for (i, sf) in savefiles.iter().enumerate() {
            lines.push(format!("        \"{}\"", i));
            lines.push("        {".to_string());
            lines.push(format!("            \"root\" \"{}\"", sf.root));
            lines.push(format!("            \"path\" \"{}\"", sf.path));
            lines.push(format!("            \"pattern\" \"{}\"", sf.pattern));
            lines.push(format!(
                "            \"recursive\" \"{}\"",
                if sf.recursive { "1" } else { "0" }
            ));
            if !sf.platforms.is_empty() && sf.platforms != vec!["all"] {
                let oslist = platforms_to_oslist(&sf.platforms);
                lines.push(format!("            \"platforms\" \"{}\"", oslist));
            }
            lines.push("        }".to_string());
        }
        lines.push("    }".to_string());
    }

    // rootoverrides section
    if !overrides.is_empty() {
        lines.push("    \"rootoverrides\"".to_string());
        lines.push("    {".to_string());
        for (i, ro) in overrides.iter().enumerate() {
            lines.push(format!("        \"{}\"", i));
            lines.push("        {".to_string());
            lines.push(format!("            \"root\" \"{}\"", ro.original_root));
            lines.push(format!("            \"os\" \"{}\"", ro.os));
            lines.push("            \"oscompare\" \"=\"".to_string());
            lines.push(format!("            \"useinstead\" \"{}\"", ro.new_root));
            if !ro.path_transforms.is_empty() {
                // pathtransforms 模式
                lines.push("            \"pathtransforms\"".to_string());
                lines.push("            {".to_string());
                for (j, pt) in ro.path_transforms.iter().enumerate() {
                    lines.push(format!("                \"{}\"", j));
                    lines.push("                {".to_string());
                    lines.push(format!("                    \"find\" \"{}\"", pt.find));
                    lines.push(format!(
                        "                    \"replace\" \"{}\"",
                        pt.replace
                    ));
                    lines.push("                }".to_string());
                }
                lines.push("            }".to_string());
            } else if !ro.add_path.is_empty() {
                lines.push(format!("            \"addpath\" \"{}\"", ro.add_path));
            }
            lines.push("        }".to_string());
        }
        lines.push("    }".to_string());
    }

    lines.push("}".to_string());
    lines.join("\n")
}

// 从 VDF 文本解析编辑数据
pub fn parse_ufs_text(text: &str) -> (Vec<SaveFileEntry>, Vec<RootOverrideEntry>) {
    let mut savefiles = Vec::new();
    let mut overrides = Vec::new();

    let lines: Vec<&str> = text.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let trimmed = lines[i].trim();

        if trimmed == "\"savefiles\"" {
            i += 1; // skip "{"
            i += 1;
            i = parse_savefiles_section(&lines, i, &mut savefiles);
        } else if trimmed == "\"rootoverrides\"" {
            i += 1; // skip "{"
            i += 1;
            i = parse_overrides_section(&lines, i, &mut overrides);
        } else {
            i += 1;
        }
    }

    (savefiles, overrides)
}

// 解析 savefiles 节内的所有条目
fn parse_savefiles_section(
    lines: &[&str],
    mut i: usize,
    savefiles: &mut Vec<SaveFileEntry>,
) -> usize {
    while i < lines.len() {
        let trimmed = lines[i].trim();
        if trimmed == "}" {
            return i + 1; // 跳过 savefiles 的闭合 }
        }

        // 数字索引行，如 "0"
        if trimmed.starts_with('"')
            && trimmed.ends_with('"')
            && trimmed[1..trimmed.len() - 1]
                .chars()
                .all(|c| c.is_ascii_digit())
        {
            i += 1; // skip "{"
            if i < lines.len() && lines[i].trim() == "{" {
                i += 1;
                let (entry, next_i) = parse_savefile_entry(lines, i);
                savefiles.push(entry);
                i = next_i;
            }
        } else {
            i += 1;
        }
    }
    i
}

// 解析单个 savefile 条目
fn parse_savefile_entry(lines: &[&str], mut i: usize) -> (SaveFileEntry, usize) {
    let mut entry = SaveFileEntry {
        root: String::new(),
        path: String::new(),
        pattern: "*".to_string(),
        platforms: vec!["all".to_string()],
        recursive: true,
    };

    while i < lines.len() {
        let trimmed = lines[i].trim();
        if trimmed == "}" {
            return (entry, i + 1);
        }

        if let Some((key, value)) = extract_kv(trimmed) {
            match key {
                "root" => entry.root = value.to_string(),
                "path" => entry.path = value.to_string(),
                "pattern" => entry.pattern = value.to_string(),
                "recursive" => entry.recursive = value != "0",
                "platforms" => {
                    entry.platforms = oslist_to_platforms(value);
                }
                _ => {}
            }
        }
        i += 1;
    }
    (entry, i)
}

// 解析 rootoverrides 节内的所有条目
fn parse_overrides_section(
    lines: &[&str],
    mut i: usize,
    overrides: &mut Vec<RootOverrideEntry>,
) -> usize {
    while i < lines.len() {
        let trimmed = lines[i].trim();
        if trimmed == "}" {
            return i + 1;
        }

        if trimmed.starts_with('"')
            && trimmed.ends_with('"')
            && trimmed[1..trimmed.len() - 1]
                .chars()
                .all(|c| c.is_ascii_digit())
        {
            i += 1;
            if i < lines.len() && lines[i].trim() == "{" {
                i += 1;
                let (entry, next_i) = parse_override_entry(lines, i);
                overrides.push(entry);
                i = next_i;
            }
        } else {
            i += 1;
        }
    }
    i
}

// 解析单个 rootoverride 条目
fn parse_override_entry(lines: &[&str], mut i: usize) -> (RootOverrideEntry, usize) {
    let mut entry = RootOverrideEntry {
        original_root: String::new(),
        os: String::new(),
        new_root: String::new(),
        add_path: String::new(),
        path_transforms: Vec::new(),
    };

    while i < lines.len() {
        let trimmed = lines[i].trim();
        if trimmed == "}" {
            return (entry, i + 1);
        }

        if trimmed == "\"pathtransforms\"" {
            i += 1;
            if i < lines.len() && lines[i].trim() == "{" {
                i += 1;
                i = parse_pathtransforms(lines, i, &mut entry.path_transforms);
            }
            continue;
        }

        if let Some((key, value)) = extract_kv(trimmed) {
            match key {
                "root" => entry.original_root = value.to_string(),
                "os" => entry.os = value.to_string(),
                "useinstead" => entry.new_root = value.to_string(),
                "addpath" => entry.add_path = value.to_string(),
                // oscompare 忽略，固定为 "="
                _ => {}
            }
        }
        i += 1;
    }
    (entry, i)
}

// 解析 pathtransforms 子节
fn parse_pathtransforms(
    lines: &[&str],
    mut i: usize,
    transforms: &mut Vec<PathTransform>,
) -> usize {
    while i < lines.len() {
        let trimmed = lines[i].trim();
        if trimmed == "}" {
            return i + 1;
        }

        // 数字索引
        if trimmed.starts_with('"')
            && trimmed.ends_with('"')
            && trimmed[1..trimmed.len() - 1]
                .chars()
                .all(|c| c.is_ascii_digit())
        {
            i += 1;
            if i < lines.len() && lines[i].trim() == "{" {
                i += 1;
                let mut pt = PathTransform::default();
                while i < lines.len() {
                    let t = lines[i].trim();
                    if t == "}" {
                        i += 1;
                        break;
                    }
                    if let Some((key, value)) = extract_kv(t) {
                        match key {
                            "find" => pt.find = value.to_string(),
                            "replace" => pt.replace = value.to_string(),
                            _ => {}
                        }
                    }
                    i += 1;
                }
                transforms.push(pt);
                continue;
            }
        }
        i += 1;
    }
    i
}

// 提取 "key" "value" 对
fn extract_kv(line: &str) -> Option<(&str, &str)> {
    let mut parts = line.split('"');
    parts.next()?; // 前导空白
    let key = parts.next()?;
    parts.next()?; // 中间空白
    let value = parts.next()?;
    Some((key, value))
}

// 平台列表转 oslist 字符串 (Steam 格式)
fn platforms_to_oslist(platforms: &[String]) -> String {
    platforms
        .iter()
        .map(|p| match p.as_str() {
            "windows" => "windows",
            "macos" => "macos",
            "linux" => "linux",
            "all" => "all",
            other => other,
        })
        .collect::<Vec<_>>()
        .join(",")
}

// oslist 字符串转平台列表
fn oslist_to_platforms(oslist: &str) -> Vec<String> {
    oslist
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_basic() {
        let savefiles = vec![SaveFileEntry {
            root: "MacAppSupport".to_string(),
            path: "GameSaves".to_string(),
            pattern: "*.sav".to_string(),
            platforms: vec!["all".to_string()],
            recursive: true,
        }];
        let overrides = vec![RootOverrideEntry {
            original_root: "WinAppDataLocal".to_string(),
            os: "macos".to_string(),
            new_root: "MacAppSupport".to_string(),
            add_path: "saves".to_string(),
            path_transforms: Vec::new(),
        }];

        let text = entries_to_ufs_text(&savefiles, &overrides);
        let (parsed_sf, parsed_ro) = parse_ufs_text(&text);

        assert_eq!(parsed_sf.len(), 1);
        assert_eq!(parsed_sf[0].root, "MacAppSupport");
        assert_eq!(parsed_sf[0].path, "GameSaves");
        assert_eq!(parsed_sf[0].pattern, "*.sav");
        assert!(parsed_sf[0].recursive);

        assert_eq!(parsed_ro.len(), 1);
        assert_eq!(parsed_ro[0].original_root, "WinAppDataLocal");
        assert_eq!(parsed_ro[0].os, "macos");
        assert_eq!(parsed_ro[0].new_root, "MacAppSupport");
        assert_eq!(parsed_ro[0].add_path, "saves");
    }

    #[test]
    fn roundtrip_with_pathtransforms() {
        let overrides = vec![RootOverrideEntry {
            original_root: "WinMyDocuments".to_string(),
            os: "macos".to_string(),
            new_root: "MacHome".to_string(),
            add_path: String::new(),
            path_transforms: vec![PathTransform {
                find: "My Games/TestGame".to_string(),
                replace: "Library/TestGame".to_string(),
            }],
        }];

        let text = entries_to_ufs_text(&[], &overrides);
        let (_, parsed_ro) = parse_ufs_text(&text);

        assert_eq!(parsed_ro.len(), 1);
        assert_eq!(parsed_ro[0].path_transforms.len(), 1);
        assert_eq!(parsed_ro[0].path_transforms[0].find, "My Games/TestGame");
        assert_eq!(parsed_ro[0].path_transforms[0].replace, "Library/TestGame");
    }
}
