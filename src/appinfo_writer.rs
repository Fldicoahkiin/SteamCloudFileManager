// appinfo.vdf 写入器
// 功能：修改 appinfo.vdf 中的 ufs 配置，添加自定义云同步路径

use anyhow::{Result, anyhow};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use sha1::{Digest, Sha1};
use std::collections::HashMap;
use std::fs;
use std::io::{Cursor, Read};
use std::path::PathBuf;

use crate::path_resolver::SaveFileConfig;
use crate::vdf_parser::VdfParser;

// VDF 二进制类型字节常量
const VDF_TYPE_SECTION: u8 = 0x00;
const VDF_TYPE_STRING: u8 = 0x01;
const VDF_TYPE_INT32: u8 = 0x02;
const VDF_TYPE_SECTION_END: u8 = 0x08;

// appinfo.vdf 版本魔数
const APPINFO_V27: u32 = 0x07564427;
const APPINFO_V28: u32 = 0x07564428;
const APPINFO_V29: u32 = 0x07564429;

// UFS 节位置信息（用于减少函数参数数量）
#[derive(Default)]
struct UfsSectionInfo {
    ufs_section_pos: Option<usize>,
    ufs_end_pos: Option<usize>,
    savefiles_section_pos: Option<usize>,
    savefiles_end_pos: Option<usize>,
    last_savefile_index: u32,
    rootoverrides_section_pos: Option<usize>,
    rootoverrides_end_pos: Option<usize>,
    last_rootoverride_index: u32,
    root_end_pos: Option<usize>,
}

pub struct AppInfoWriter {
    steam_path: PathBuf,
}

impl AppInfoWriter {
    pub fn new() -> Result<Self> {
        let steam_path = VdfParser::find_steam_path()?;
        Ok(Self { steam_path })
    }

    // 获取 appinfo.vdf 路径
    fn appinfo_path(&self) -> PathBuf {
        self.steam_path.join("appcache").join("appinfo.vdf")
    }

    // 完整注入 UFS 配置（savefiles + rootoverrides）
    // 使用新版配置格式
    pub fn inject_full_ufs_config(
        &self,
        app_id: u32,
        config: &crate::config::UfsGameConfig,
    ) -> Result<()> {
        let appinfo_path = self.appinfo_path();
        if !appinfo_path.exists() {
            return Err(anyhow!("appinfo.vdf 不存在: {:?}", appinfo_path));
        }

        // 备份原文件（带时间戳，便于追溯问题）
        let timestamp = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
        let backup_filename = format!("appinfo.{}.bak", timestamp);
        let backup_path = appinfo_path.parent().unwrap().join(&backup_filename);
        fs::copy(&appinfo_path, &backup_path)?;
        tracing::info!("已备份 appinfo.vdf 到 {:?}", backup_path);

        let data = fs::read(&appinfo_path)?;

        // 转换 SaveFileEntry 为 SaveFileConfig
        let savefiles: Vec<SaveFileConfig> = config
            .savefiles
            .iter()
            .map(|s| SaveFileConfig {
                root: s.root.clone(),
                root_type: crate::path_resolver::RootType::from_name(&s.root),
                path: s.path.clone(),
                pattern: s.pattern.clone(),
                platforms: s.platforms.clone(),
                recursive: s.recursive,
            })
            .collect();

        let modified_data =
            self.modify_app_ufs_full(&data, app_id, savefiles, &config.root_overrides)?;

        fs::write(&appinfo_path, &modified_data)?;
        tracing::info!(
            "已注入完整 ufs 配置到 app_id {} (savefiles: {}, overrides: {}), 文件大小: {} -> {} bytes",
            app_id,
            config.savefiles.len(),
            config.root_overrides.len(),
            data.len(),
            modified_data.len()
        );

        Ok(())
    }

    // 修改指定 app 的完整 ufs 配置（savefiles + rootoverrides）
    fn modify_app_ufs_full(
        &self,
        data: &[u8],
        target_app_id: u32,
        custom_savefiles: Vec<SaveFileConfig>,
        custom_overrides: &[crate::config::RootOverrideEntry],
    ) -> Result<Vec<u8>> {
        let mut cursor = Cursor::new(data);

        // 读取文件头
        let magic = cursor.read_u32::<LittleEndian>()?;
        let version = match magic {
            APPINFO_V27 => 27,
            APPINFO_V28 => 28,
            APPINFO_V29 => 29,
            _ => return Err(anyhow!("不支持的 appinfo.vdf 版本: 0x{:X}", magic)),
        };

        let universe = cursor.read_u32::<LittleEndian>()?;

        // V29+ 读取字符串表偏移
        let string_table_offset = if version >= 29 {
            cursor.read_u64::<LittleEndian>()?
        } else {
            0
        };

        // 解析字符串表 (V29+)
        let mut string_table = if version >= 29 && string_table_offset > 0 {
            self.parse_string_table(data, string_table_offset as usize)?
        } else {
            Vec::new()
        };

        // 构建字符串到索引的映射
        let mut string_to_idx: HashMap<String, usize> = HashMap::new();
        for (idx, s) in string_table.iter().enumerate() {
            string_to_idx.insert(s.clone(), idx);
        }

        // 找到目标 app 的条目
        let entries_start = cursor.position() as usize;
        let mut app_start: Option<usize> = None;
        let mut app_end: Option<usize> = None;

        loop {
            let entry_start = cursor.position() as usize;
            let app_id = cursor.read_u32::<LittleEndian>()?;
            if app_id == 0 {
                break;
            }

            let size = cursor.read_u32::<LittleEndian>()?;
            let entry_end = entry_start + 8 + size as usize;

            if app_id == target_app_id {
                app_start = Some(entry_start);
                app_end = Some(entry_end);
                break;
            }

            cursor.set_position(entry_end as u64);
        }

        let (start, end) = match (app_start, app_end) {
            (Some(s), Some(e)) => (s, e),
            _ => return Err(anyhow!("未找到 app_id {} 的配置", target_app_id)),
        };

        // 读取原始条目
        let original_entry = &data[start..end];

        // 修改条目（包含 savefiles 和 rootoverrides）
        let modified_entry = self.modify_entry_full(
            original_entry,
            &custom_savefiles,
            custom_overrides,
            version,
            &mut string_table,
            &mut string_to_idx,
        )?;

        // 构建新的文件数据
        let mut result = Vec::new();

        // 写入文件头
        result.write_u32::<LittleEndian>(magic)?;
        result.write_u32::<LittleEndian>(universe)?;

        // V29+ 写入字符串表偏移占位符（稍后更新）
        let string_table_offset_pos = if version >= 29 {
            let pos = result.len();
            result.write_u64::<LittleEndian>(0)?;
            Some(pos)
        } else {
            None
        };

        // 写入目标条目之前的所有条目
        result.extend_from_slice(&data[entries_start..start]);

        // 写入修改后的条目
        result.extend_from_slice(&modified_entry);

        // 写入目标条目之后的所有条目（不包括字符串表）
        let entries_end = if version >= 29 {
            string_table_offset as usize
        } else {
            data.len()
        };

        if end < entries_end {
            result.extend_from_slice(&data[end..entries_end]);
        }

        // V29+ 更新字符串表
        if version >= 29 {
            // 更新字符串表偏移
            let new_string_table_offset = result.len() as u64;
            if let Some(pos) = string_table_offset_pos {
                let offset_bytes = new_string_table_offset.to_le_bytes();
                result[pos..pos + 8].copy_from_slice(&offset_bytes);
            }

            // 写入更新后的字符串表
            for s in &string_table {
                result.extend_from_slice(s.as_bytes());
                result.push(0); // null terminator
            }
        }

        Ok(result)
    }

    // 修改单个 app 条目（完整版，包含 rootoverrides）
    fn modify_entry_full(
        &self,
        entry: &[u8],
        custom_savefiles: &[SaveFileConfig],
        custom_overrides: &[crate::config::RootOverrideEntry],
        version: u32,
        string_table: &mut Vec<String>,
        string_to_idx: &mut HashMap<String, usize>,
    ) -> Result<Vec<u8>> {
        let mut cursor = Cursor::new(entry);

        // 读取头部
        let app_id = cursor.read_u32::<LittleEndian>()?;
        let _size = cursor.read_u32::<LittleEndian>()?;
        let info_state = cursor.read_u32::<LittleEndian>()?;
        let last_updated = cursor.read_u32::<LittleEndian>()?;
        let access_token = cursor.read_u64::<LittleEndian>()?;

        let mut checksum_text = [0u8; 20];
        cursor.read_exact(&mut checksum_text)?;

        let change_number = cursor.read_u32::<LittleEndian>()?;

        let mut checksum_binary = [0u8; 20];
        if version >= 28 {
            cursor.read_exact(&mut checksum_binary)?;
        }

        // 读取 VDF 数据
        let vdf_start = cursor.position() as usize;
        let vdf_data = &entry[vdf_start..];

        // 修改 VDF 数据，添加自定义 savefiles 和 rootoverrides
        let modified_vdf = self.inject_full_ufs_to_vdf(
            vdf_data,
            custom_savefiles,
            custom_overrides,
            version,
            string_table,
            string_to_idx,
        )?;

        // 计算新的校验和
        let new_checksum_binary = self.calculate_binary_checksum(&modified_vdf);
        let new_checksum_text =
            self.calculate_text_checksum(&modified_vdf, string_table, version)?;

        // 构建新的条目
        let header_size: usize = 4 + 4 + 8 + 20 + 4 + if version >= 28 { 20 } else { 0 };
        let new_size = (header_size + modified_vdf.len()) as u32;

        let mut result = Vec::new();
        result.write_u32::<LittleEndian>(app_id)?;
        result.write_u32::<LittleEndian>(new_size)?;
        result.write_u32::<LittleEndian>(info_state)?;
        result.write_u32::<LittleEndian>(last_updated)?;
        result.write_u64::<LittleEndian>(access_token)?;
        result.extend_from_slice(&new_checksum_text);
        result.write_u32::<LittleEndian>(change_number)?;
        if version >= 28 {
            result.extend_from_slice(&new_checksum_binary);
        }
        result.extend_from_slice(&modified_vdf);

        Ok(result)
    }

    // 将完整 UFS 配置（savefiles + rootoverrides）注入到 VDF 数据中
    //
    // 设计原则：
    // 1. 完整解析 VDF 获取所有关键位置
    // 2. 使用 "复制-跳过-插入" 模式避免数据重复
    // 3. 支持三种模式：创建新节、替换现有节、追加到现有节
    fn inject_full_ufs_to_vdf(
        &self,
        vdf_data: &[u8],
        custom_savefiles: &[SaveFileConfig],
        custom_overrides: &[crate::config::RootOverrideEntry],
        version: u32,
        string_table: &mut Vec<String>,
        string_to_idx: &mut HashMap<String, usize>,
    ) -> Result<Vec<u8>> {
        let mut cursor = Cursor::new(vdf_data);

        // 第一阶段：解析 VDF 结构，收集所有关键位置
        let mut info = UfsSectionInfo::default();
        if version >= 29 {
            self.find_ufs_section_v29_improved(&mut cursor, string_table, &mut info)?;
        } else {
            self.find_ufs_section_v28_improved(&mut cursor, &mut info)?;
        }

        tracing::info!(
            "VDF 解析结果: ufs={:?}..{:?}, savefiles={:?}..{:?}, rootoverrides={:?}..{:?}, root_end={:?}",
            info.ufs_section_pos,
            info.ufs_end_pos,
            info.savefiles_section_pos,
            info.savefiles_end_pos,
            info.rootoverrides_section_pos,
            info.rootoverrides_end_pos,
            info.root_end_pos
        );

        // 第二阶段：根据情况构建新的 VDF 数据
        let mut result = Vec::new();

        // 情况 1：ufs 节存在
        if let Some(ufs_end) = info.ufs_end_pos {
            // 需要处理的位置区间：
            // - 如果 savefiles 存在：需要跳过 savefiles_section..savefiles_end+1（包括 0x08）
            // - 如果 rootoverrides 存在：需要跳过 rootoverrides_section..rootoverrides_end+1

            // 收集需要跳过的区间 (start, end) - end 是排他的
            let mut skip_ranges: Vec<(usize, usize)> = Vec::new();

            if let (Some(sf_start), Some(sf_end)) =
                (info.savefiles_section_pos, info.savefiles_end_pos)
            {
                // 跳过整个 savefiles 节（包括结束标记 0x08）
                skip_ranges.push((sf_start, sf_end + 1));
            }

            if let (Some(ro_start), Some(ro_end)) =
                (info.rootoverrides_section_pos, info.rootoverrides_end_pos)
            {
                // 跳过整个 rootoverrides 节（包括结束标记 0x08）
                skip_ranges.push((ro_start, ro_end + 1));
            }

            // 按起始位置排序
            skip_ranges.sort_by_key(|r| r.0);

            // 复制数据，跳过指定区间，在 ufs_end 前插入新内容
            let mut pos = 0;
            for (skip_start, skip_end) in &skip_ranges {
                if pos < *skip_start {
                    result.extend_from_slice(&vdf_data[pos..*skip_start]);
                }
                pos = *skip_end;
            }

            // 复制到 ufs_end（但不包括 ufs_end 的 0x08）
            if pos < ufs_end {
                result.extend_from_slice(&vdf_data[pos..ufs_end]);
            }

            // 插入新的 savefiles 节（如果有内容）
            if !custom_savefiles.is_empty() {
                self.write_savefiles_section(
                    &mut result,
                    custom_savefiles,
                    version,
                    string_table,
                    string_to_idx,
                );
            }

            // 插入新的 rootoverrides 节（如果有内容）
            if !custom_overrides.is_empty() {
                self.write_rootoverrides_section(
                    &mut result,
                    custom_overrides,
                    version,
                    string_table,
                    string_to_idx,
                );
            }

            // 复制 ufs_end（0x08）及之后的所有内容
            result.extend_from_slice(&vdf_data[ufs_end..]);
        } else if let Some(_ufs_start) = info.ufs_section_pos {
            // 情况 2：ufs 节开始存在但找不到结束位置（异常情况）
            return Err(anyhow!("VDF 结构异常：找到 ufs 节开始但没有结束标记"));
        } else if let Some(root_end) = info.root_end_pos {
            // 情况 3：没有 ufs 节，需要在根节点末尾创建完整的 ufs 结构
            result.extend_from_slice(&vdf_data[..root_end]);

            // 创建 ufs 节
            if version >= 29 {
                let ufs_idx = self.get_or_create_string_index("ufs", string_table, string_to_idx);
                result.push(VDF_TYPE_SECTION);
                result.extend_from_slice(&(ufs_idx as u32).to_le_bytes());
            } else {
                result.push(VDF_TYPE_SECTION);
                result.extend_from_slice(b"ufs\0");
            }

            // 写入 savefiles
            if !custom_savefiles.is_empty() {
                self.write_savefiles_section(
                    &mut result,
                    custom_savefiles,
                    version,
                    string_table,
                    string_to_idx,
                );
            }

            // 写入 rootoverrides
            if !custom_overrides.is_empty() {
                self.write_rootoverrides_section(
                    &mut result,
                    custom_overrides,
                    version,
                    string_table,
                    string_to_idx,
                );
            }

            // ufs 节结束
            result.push(VDF_TYPE_SECTION_END);

            // 复制根节点结束标记及之后的内容
            result.extend_from_slice(&vdf_data[root_end..]);
        } else {
            return Err(anyhow!("无法解析 VDF 结构：找不到根节点结束位置"));
        }

        tracing::info!(
            "已注入完整 UFS 配置: {} savefiles, {} rootoverrides, 大小变化: {} -> {}",
            custom_savefiles.len(),
            custom_overrides.len(),
            vdf_data.len(),
            result.len()
        );

        Ok(result)
    }

    // 写入 savefiles 节（helper 函数）
    fn write_savefiles_section(
        &self,
        result: &mut Vec<u8>,
        savefiles: &[SaveFileConfig],
        version: u32,
        string_table: &mut Vec<String>,
        string_to_idx: &mut HashMap<String, usize>,
    ) {
        // 节头
        if version >= 29 {
            let idx = self.get_or_create_string_index("savefiles", string_table, string_to_idx);
            result.push(VDF_TYPE_SECTION);
            result.extend_from_slice(&(idx as u32).to_le_bytes());
        } else {
            result.push(VDF_TYPE_SECTION);
            result.extend_from_slice(b"savefiles\0");
        }

        // 条目
        for (i, savefile) in savefiles.iter().enumerate() {
            if version >= 29 {
                let encoded =
                    self.encode_savefile_v29(savefile, i as u32, string_table, string_to_idx);
                result.extend_from_slice(&encoded);
            } else {
                let encoded = self.encode_savefile_v28(savefile, i as u32);
                result.extend_from_slice(&encoded);
            }
        }

        // 节结束
        result.push(VDF_TYPE_SECTION_END);
    }

    // 写入 rootoverrides 节（helper 函数）
    fn write_rootoverrides_section(
        &self,
        result: &mut Vec<u8>,
        overrides: &[crate::config::RootOverrideEntry],
        version: u32,
        string_table: &mut Vec<String>,
        string_to_idx: &mut HashMap<String, usize>,
    ) {
        // 节头
        if version >= 29 {
            let idx = self.get_or_create_string_index("rootoverrides", string_table, string_to_idx);
            result.push(VDF_TYPE_SECTION);
            result.extend_from_slice(&(idx as u32).to_le_bytes());
        } else {
            result.push(VDF_TYPE_SECTION);
            result.extend_from_slice(b"rootoverrides\0");
        }

        // 条目 - rootoverrides 索引从 0 开始（与 savefiles 相同）
        for (i, override_entry) in overrides.iter().enumerate() {
            let idx = i as u32;
            if version >= 29 {
                let encoded =
                    self.encode_rootoverride_v29(override_entry, idx, string_table, string_to_idx);
                result.extend_from_slice(&encoded);
            } else {
                let encoded = self.encode_rootoverride_v28(override_entry, idx);
                result.extend_from_slice(&encoded);
            }
        }

        // 节结束
        result.push(VDF_TYPE_SECTION_END);
    }

    // V29 格式查找 ufs 节 (改进版，记录 ufs_end)
    fn find_ufs_section_v29_improved(
        &self,
        cursor: &mut Cursor<&[u8]>,
        string_table: &[String],
        info: &mut UfsSectionInfo,
    ) -> Result<()> {
        let mut depth = 0;
        let mut in_ufs = false;
        let mut in_savefiles = false;
        let mut in_rootoverrides = false;
        // 追踪每个节开始时的深度，用于判断什么时候该节真正结束
        let mut ufs_depth = 0;
        let mut savefiles_depth = 0;
        let mut rootoverrides_depth = 0;

        while let Ok(type_byte) = cursor.read_u8() {
            let pos = cursor.position() as usize - 1;

            if type_byte == VDF_TYPE_SECTION_END {
                // 先减少深度
                depth -= 1;

                // 检查是否是各节的真正结束
                if in_savefiles && depth < savefiles_depth {
                    info.savefiles_end_pos = Some(pos);
                    in_savefiles = false;
                }
                if in_rootoverrides && depth < rootoverrides_depth {
                    info.rootoverrides_end_pos = Some(pos);
                    in_rootoverrides = false;
                }
                if in_ufs && depth < ufs_depth {
                    info.ufs_end_pos = Some(pos);
                    in_ufs = false;
                }
                if depth == 0 {
                    info.root_end_pos = Some(pos);
                }
                continue;
            }

            let key_idx = cursor.read_u32::<LittleEndian>()? as usize;
            let key = string_table.get(key_idx).cloned().unwrap_or_default();

            match type_byte {
                VDF_TYPE_SECTION => {
                    depth += 1;
                    if depth == 2 && key == "ufs" {
                        info.ufs_section_pos = Some(pos);
                        in_ufs = true;
                        ufs_depth = depth;
                    } else if in_ufs && key == "savefiles" {
                        info.savefiles_section_pos = Some(pos);
                        in_savefiles = true;
                        savefiles_depth = depth;
                    } else if in_ufs && key == "rootoverrides" {
                        info.rootoverrides_section_pos = Some(pos);
                        in_rootoverrides = true;
                        rootoverrides_depth = depth;
                    } else if in_savefiles
                        && let Ok(idx) = key.parse::<u32>()
                        && idx > info.last_savefile_index
                    {
                        info.last_savefile_index = idx;
                    } else if in_rootoverrides
                        && let Ok(idx) = key.parse::<u32>()
                        && idx > info.last_rootoverride_index
                    {
                        info.last_rootoverride_index = idx;
                    }
                }
                VDF_TYPE_STRING => {
                    self.skip_null_string(cursor);
                }
                VDF_TYPE_INT32 => {
                    cursor.read_i32::<LittleEndian>()?;
                }
                0x07 => {
                    cursor.read_u64::<LittleEndian>()?;
                }
                _ => {}
            }
        }

        Ok(())
    }
    // V28 格式查找 ufs 节
    fn find_ufs_section_v28_improved(
        &self,
        cursor: &mut Cursor<&[u8]>,
        info: &mut UfsSectionInfo,
    ) -> Result<()> {
        let mut depth = 0;
        let mut in_ufs = false;
        let mut in_savefiles = false;
        let mut in_rootoverrides = false;
        let mut ufs_depth = 0;
        let mut savefiles_depth = 0;
        let mut rootoverrides_depth = 0;

        while let Ok(type_byte) = cursor.read_u8() {
            let pos = cursor.position() as usize - 1;

            if type_byte == VDF_TYPE_SECTION_END {
                // 先减少深度
                depth -= 1;

                // 检查是否是各节的真正结束
                if in_savefiles && depth < savefiles_depth {
                    info.savefiles_end_pos = Some(pos);
                    in_savefiles = false;
                }
                if in_rootoverrides && depth < rootoverrides_depth {
                    info.rootoverrides_end_pos = Some(pos);
                    in_rootoverrides = false;
                }
                if in_ufs && depth < ufs_depth {
                    info.ufs_end_pos = Some(pos);
                    in_ufs = false;
                }
                if depth == 0 {
                    info.root_end_pos = Some(pos);
                }
                continue;
            }

            let key = self.read_null_string(cursor);

            match type_byte {
                VDF_TYPE_SECTION => {
                    depth += 1;
                    if depth == 2 && key == "ufs" {
                        info.ufs_section_pos = Some(pos);
                        in_ufs = true;
                        ufs_depth = depth;
                    } else if in_ufs && key == "savefiles" {
                        info.savefiles_section_pos = Some(pos);
                        in_savefiles = true;
                        savefiles_depth = depth;
                    } else if in_ufs && key == "rootoverrides" {
                        info.rootoverrides_section_pos = Some(pos);
                        in_rootoverrides = true;
                        rootoverrides_depth = depth;
                    } else if in_savefiles
                        && let Ok(idx) = key.parse::<u32>()
                        && idx > info.last_savefile_index
                    {
                        info.last_savefile_index = idx;
                    } else if in_rootoverrides
                        && let Ok(idx) = key.parse::<u32>()
                        && idx > info.last_rootoverride_index
                    {
                        info.last_rootoverride_index = idx;
                    }
                }
                VDF_TYPE_STRING => {
                    self.skip_null_string(cursor);
                }
                VDF_TYPE_INT32 => {
                    cursor.read_i32::<LittleEndian>()?;
                }
                0x07 => {
                    cursor.read_u64::<LittleEndian>()?;
                }
                _ => {}
            }
        }

        Ok(())
    }

    // 读取 null 结尾字符串
    fn read_null_string(&self, cursor: &mut Cursor<&[u8]>) -> String {
        let mut bytes = Vec::new();
        loop {
            match cursor.read_u8() {
                Ok(0) => break,
                Ok(b) => bytes.push(b),
                Err(_) => break,
            }
        }
        String::from_utf8(bytes).unwrap_or_default()
    }

    // 跳过 null 结尾字符串
    fn skip_null_string(&self, cursor: &mut Cursor<&[u8]>) {
        loop {
            match cursor.read_u8() {
                Ok(0) | Err(_) => break,
                _ => {}
            }
        }
    }

    // 编码 savefile 条目 (V28 格式 - 直接字符串)
    fn encode_savefile_v28(&self, savefile: &SaveFileConfig, index: u32) -> Vec<u8> {
        let mut result = Vec::new();

        // Section 开始: 0x00 + index\0
        result.push(VDF_TYPE_SECTION);
        result.extend_from_slice(index.to_string().as_bytes());
        result.push(0);

        // root 字段
        result.push(VDF_TYPE_STRING);
        result.extend_from_slice(b"root\0");
        result.extend_from_slice(savefile.root.as_bytes());
        result.push(0);

        // path 字段
        result.push(VDF_TYPE_STRING);
        result.extend_from_slice(b"path\0");
        result.extend_from_slice(savefile.path.as_bytes());
        result.push(0);

        // pattern 字段
        result.push(VDF_TYPE_STRING);
        result.extend_from_slice(b"pattern\0");
        result.extend_from_slice(savefile.pattern.as_bytes());
        result.push(0);

        // platforms 字段 (可选，如果不是 "all")
        if !savefile.platforms.is_empty()
            && !savefile.platforms.iter().any(|p| p.to_lowercase() == "all")
        {
            result.push(VDF_TYPE_STRING);
            result.extend_from_slice(b"platforms\0");
            let platforms_str = self.platforms_to_oslist(&savefile.platforms);
            result.extend_from_slice(platforms_str.as_bytes());
            result.push(0);
        }

        // Section 结束
        result.push(VDF_TYPE_SECTION_END);

        result
    }

    // 将平台列表转换为 oslist 格式 (Steam 格式)
    fn platforms_to_oslist(&self, platforms: &[String]) -> String {
        let mut parts = Vec::new();
        for p in platforms {
            let p_lower = p.to_lowercase();
            if p_lower.contains("windows") || p_lower.contains("win") {
                parts.push("windows");
            } else if p_lower.contains("macos")
                || p_lower.contains("mac")
                || p_lower.contains("osx")
            {
                parts.push("macos");
            } else if p_lower.contains("linux") {
                parts.push("linux");
            }
        }
        parts.join(",")
    }

    // 获取或创建字符串索引 (V29+)
    fn get_or_create_string_index(
        &self,
        s: &str,
        string_table: &mut Vec<String>,
        string_to_idx: &mut HashMap<String, usize>,
    ) -> usize {
        if let Some(&idx) = string_to_idx.get(s) {
            idx
        } else {
            let idx = string_table.len();
            string_table.push(s.to_string());
            string_to_idx.insert(s.to_string(), idx);
            idx
        }
    }

    // 编码单个 savefile 条目为二进制 VDF (V29 格式)
    fn encode_savefile_v29(
        &self,
        savefile: &SaveFileConfig,
        index: u32,
        string_table: &mut Vec<String>,
        string_to_idx: &mut HashMap<String, usize>,
    ) -> Vec<u8> {
        let mut result = Vec::new();

        // Section 开始: 0x00 + index_key
        let index_str = index.to_string();
        let index_idx = self.get_or_create_string_index(&index_str, string_table, string_to_idx);
        result.push(VDF_TYPE_SECTION);
        result.extend_from_slice(&(index_idx as u32).to_le_bytes());

        // root 字段
        let root_idx = self.get_or_create_string_index("root", string_table, string_to_idx);
        result.push(VDF_TYPE_STRING);
        result.extend_from_slice(&(root_idx as u32).to_le_bytes());
        result.extend_from_slice(savefile.root.as_bytes());
        result.push(0);

        // path 字段
        let path_idx = self.get_or_create_string_index("path", string_table, string_to_idx);
        result.push(VDF_TYPE_STRING);
        result.extend_from_slice(&(path_idx as u32).to_le_bytes());
        result.extend_from_slice(savefile.path.as_bytes());
        result.push(0);

        // pattern 字段
        let pattern_idx = self.get_or_create_string_index("pattern", string_table, string_to_idx);
        result.push(VDF_TYPE_STRING);
        result.extend_from_slice(&(pattern_idx as u32).to_le_bytes());
        result.extend_from_slice(savefile.pattern.as_bytes());
        result.push(0);

        // platforms 字段 (可选，如果不是 "all")
        if !savefile.platforms.is_empty()
            && !savefile.platforms.iter().any(|p| p.to_lowercase() == "all")
        {
            let platforms_idx =
                self.get_or_create_string_index("platforms", string_table, string_to_idx);
            result.push(VDF_TYPE_STRING);
            result.extend_from_slice(&(platforms_idx as u32).to_le_bytes());
            let platforms_str = self.platforms_to_oslist(&savefile.platforms);
            result.extend_from_slice(platforms_str.as_bytes());
            result.push(0);
        }

        // Section 结束
        result.push(VDF_TYPE_SECTION_END);

        result
    }

    // 编码 rootoverride 条目 (V28 格式 - 直接字符串)
    // VDF 格式: root, os, oscompare, useinstead, addpath 或 pathtransforms
    // 注意：pathtransforms 和 addpath 互斥，有 pathtransforms 时不输出 addpath
    fn encode_rootoverride_v28(
        &self,
        override_entry: &crate::config::RootOverrideEntry,
        index: u32,
    ) -> Vec<u8> {
        let mut result = Vec::new();

        // Section 开始: 0x00 + index\0
        result.push(VDF_TYPE_SECTION);
        result.extend_from_slice(index.to_string().as_bytes());
        result.push(0);

        // root 字段 (要覆盖的原始根)
        result.push(VDF_TYPE_STRING);
        result.extend_from_slice(b"root\0");
        result.extend_from_slice(override_entry.original_root.as_bytes());
        result.push(0);

        // os 字段 (目标操作系统)
        result.push(VDF_TYPE_STRING);
        result.extend_from_slice(b"os\0");
        result.extend_from_slice(override_entry.os.as_bytes());
        result.push(0);

        // oscompare 字段 (比较符，已知值: "="，可能存在其他值如 "!=")
        result.push(VDF_TYPE_STRING);
        result.extend_from_slice(b"oscompare\0");
        result.extend_from_slice(b"=");
        result.push(0);

        // useinstead 字段 (新的根目录名称)
        result.push(VDF_TYPE_STRING);
        result.extend_from_slice(b"useinstead\0");
        result.extend_from_slice(override_entry.new_root.as_bytes());
        result.push(0);

        // pathtransforms 和 addpath 互斥
        if !override_entry.path_transforms.is_empty() {
            // 有 pathtransforms 时，编码 pathtransforms 结构
            result.push(VDF_TYPE_SECTION);
            result.extend_from_slice(b"pathtransforms\0");

            for (i, transform) in override_entry.path_transforms.iter().enumerate() {
                // 每个 transform 是一个子节
                result.push(VDF_TYPE_SECTION);
                result.extend_from_slice(i.to_string().as_bytes());
                result.push(0);

                // find 字段
                result.push(VDF_TYPE_STRING);
                result.extend_from_slice(b"find\0");
                result.extend_from_slice(transform.find.as_bytes());
                result.push(0);

                // replace 字段
                result.push(VDF_TYPE_STRING);
                result.extend_from_slice(b"replace\0");
                result.extend_from_slice(transform.replace.as_bytes());
                result.push(0);

                // transform 子节结束
                result.push(VDF_TYPE_SECTION_END);
            }

            // pathtransforms 节结束
            result.push(VDF_TYPE_SECTION_END);
        } else if !override_entry.add_path.is_empty() {
            // 无 pathtransforms 时，输出 addpath
            result.push(VDF_TYPE_STRING);
            result.extend_from_slice(b"addpath\0");
            result.extend_from_slice(override_entry.add_path.as_bytes());
            result.push(0);
        }

        // Section 结束
        result.push(VDF_TYPE_SECTION_END);

        result
    }

    // 编码 rootoverride 条目 (V29 格式 - 字符串表索引)
    // VDF 格式: root, os, oscompare, useinstead, addpath 或 pathtransforms
    // 注意：pathtransforms 和 addpath 互斥，有 pathtransforms 时不输出 addpath
    fn encode_rootoverride_v29(
        &self,
        override_entry: &crate::config::RootOverrideEntry,
        index: u32,
        string_table: &mut Vec<String>,
        string_to_idx: &mut HashMap<String, usize>,
    ) -> Vec<u8> {
        let mut result = Vec::new();

        // Section 开始: 0x00 + index_key
        let index_str = index.to_string();
        let index_idx = self.get_or_create_string_index(&index_str, string_table, string_to_idx);
        result.push(VDF_TYPE_SECTION);
        result.extend_from_slice(&(index_idx as u32).to_le_bytes());

        // root 字段 (要覆盖的原始根)
        let root_idx = self.get_or_create_string_index("root", string_table, string_to_idx);
        result.push(VDF_TYPE_STRING);
        result.extend_from_slice(&(root_idx as u32).to_le_bytes());
        result.extend_from_slice(override_entry.original_root.as_bytes());
        result.push(0);

        // os 字段 (目标操作系统)
        let os_idx = self.get_or_create_string_index("os", string_table, string_to_idx);
        result.push(VDF_TYPE_STRING);
        result.extend_from_slice(&(os_idx as u32).to_le_bytes());
        result.extend_from_slice(override_entry.os.as_bytes());
        result.push(0);

        // oscompare 字段 (比较符，已知值: "="，可能存在其他值如 "!=")
        let oscompare_idx =
            self.get_or_create_string_index("oscompare", string_table, string_to_idx);
        result.push(VDF_TYPE_STRING);
        result.extend_from_slice(&(oscompare_idx as u32).to_le_bytes());
        result.extend_from_slice(b"=");
        result.push(0);

        // useinstead 字段 (新的根目录名称，字符串类型)
        let useinstead_idx =
            self.get_or_create_string_index("useinstead", string_table, string_to_idx);
        result.push(VDF_TYPE_STRING);
        result.extend_from_slice(&(useinstead_idx as u32).to_le_bytes());
        result.extend_from_slice(override_entry.new_root.as_bytes());
        result.push(0);

        // pathtransforms 和 addpath 互斥
        if !override_entry.path_transforms.is_empty() {
            // 有 pathtransforms 时，编码 pathtransforms 结构
            let pathtransforms_idx =
                self.get_or_create_string_index("pathtransforms", string_table, string_to_idx);
            result.push(VDF_TYPE_SECTION);
            result.extend_from_slice(&(pathtransforms_idx as u32).to_le_bytes());

            for (i, transform) in override_entry.path_transforms.iter().enumerate() {
                // 每个 transform 是一个子节
                let transform_idx_str = i.to_string();
                let transform_idx = self.get_or_create_string_index(
                    &transform_idx_str,
                    string_table,
                    string_to_idx,
                );
                result.push(VDF_TYPE_SECTION);
                result.extend_from_slice(&(transform_idx as u32).to_le_bytes());

                // find 字段
                let find_idx = self.get_or_create_string_index("find", string_table, string_to_idx);
                result.push(VDF_TYPE_STRING);
                result.extend_from_slice(&(find_idx as u32).to_le_bytes());
                result.extend_from_slice(transform.find.as_bytes());
                result.push(0);

                // replace 字段
                let replace_idx =
                    self.get_or_create_string_index("replace", string_table, string_to_idx);
                result.push(VDF_TYPE_STRING);
                result.extend_from_slice(&(replace_idx as u32).to_le_bytes());
                result.extend_from_slice(transform.replace.as_bytes());
                result.push(0);

                // transform 子节结束
                result.push(VDF_TYPE_SECTION_END);
            }

            // pathtransforms 节结束
            result.push(VDF_TYPE_SECTION_END);
        } else if !override_entry.add_path.is_empty() {
            // 无 pathtransforms 时，输出 addpath
            let addpath_idx =
                self.get_or_create_string_index("addpath", string_table, string_to_idx);
            result.push(VDF_TYPE_STRING);
            result.extend_from_slice(&(addpath_idx as u32).to_le_bytes());
            result.extend_from_slice(override_entry.add_path.as_bytes());
            result.push(0);
        }

        // Section 结束
        result.push(VDF_TYPE_SECTION_END);

        result
    }

    // 计算二进制校验和 (SHA-1)
    fn calculate_binary_checksum(&self, data: &[u8]) -> [u8; 20] {
        let mut hasher = Sha1::new();
        hasher.update(data);
        hasher.finalize().into()
    }

    // 计算文本校验和 (SHA-1 of VDF text format)
    // 根据 Steam-Metadata-Editor 的分析，需要将二进制 VDF 转为文本格式再计算
    fn calculate_text_checksum(
        &self,
        vdf_data: &[u8],
        string_table: &[String],
        version: u32,
    ) -> Result<[u8; 20]> {
        let mut text_output = String::new();
        let mut cursor = Cursor::new(vdf_data);

        self.format_vdf_to_text(&mut cursor, string_table, version, 0, &mut text_output)?;

        // 重要：反斜杠需要双写
        let escaped = text_output.replace('\\', "\\\\");

        let mut hasher = Sha1::new();
        hasher.update(escaped.as_bytes());
        Ok(hasher.finalize().into())
    }

    // 将二进制 VDF 转换为文本格式
    fn format_vdf_to_text(
        &self,
        cursor: &mut Cursor<&[u8]>,
        string_table: &[String],
        version: u32,
        indent: usize,
        output: &mut String,
    ) -> Result<()> {
        let indent_str = "\t".repeat(indent);

        while let Ok(type_byte) = cursor.read_u8() {
            if type_byte == VDF_TYPE_SECTION_END {
                break;
            }

            // 读取 key
            let key = if version >= 29 {
                let idx = cursor.read_u32::<LittleEndian>()? as usize;
                string_table.get(idx).cloned().unwrap_or_default()
            } else {
                self.read_null_string(cursor)
            };

            match type_byte {
                VDF_TYPE_SECTION => {
                    output.push_str(&format!("{}\"{}\"\\n", indent_str, key));
                    output.push_str(&format!("{}{{\\n", indent_str));
                    self.format_vdf_to_text(cursor, string_table, version, indent + 1, output)?;
                    output.push_str(&format!("{}}}\\n", indent_str));
                }
                VDF_TYPE_STRING => {
                    let value = self.read_null_string(cursor);
                    output.push_str(&format!("{}\"{}\"\\t\\t\"{}\"\\n", indent_str, key, value));
                }
                VDF_TYPE_INT32 => {
                    let value = cursor.read_i32::<LittleEndian>()?;
                    output.push_str(&format!("{}\"{}\"\\t\\t\"{}\"\\n", indent_str, key, value));
                }
                0x07 => {
                    let value = cursor.read_u64::<LittleEndian>()?;
                    output.push_str(&format!("{}\"{}\"\\t\\t\"{}\"\\n", indent_str, key, value));
                }
                _ => {
                    // 跳过未知类型
                }
            }
        }

        Ok(())
    }

    // 解析字符串表
    fn parse_string_table(&self, data: &[u8], offset: usize) -> Result<Vec<String>> {
        if offset >= data.len() {
            return Ok(Vec::new());
        }

        let mut strings = Vec::new();
        let mut pos = offset;

        while pos < data.len() {
            let start = pos;
            while pos < data.len() && data[pos] != 0 {
                pos += 1;
            }

            if pos > start
                && let Ok(s) = String::from_utf8(data[start..pos].to_vec())
            {
                strings.push(s);
            }

            pos += 1;

            if strings.len() > 50000 {
                break;
            }
        }

        Ok(strings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_checksum() {
        let writer = AppInfoWriter {
            steam_path: PathBuf::new(),
        };
        let data = b"test data";
        let checksum = writer.calculate_binary_checksum(data);
        assert_eq!(checksum.len(), 20);
    }
}
