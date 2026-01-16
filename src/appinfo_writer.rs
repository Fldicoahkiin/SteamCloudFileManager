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

    // 注入自定义 ufs 配置（别名）
    pub fn inject_ufs(&self, app_id: u32, savefiles: &[SaveFileConfig]) -> Result<()> {
        self.inject_savefiles(app_id, savefiles.to_vec())
    }

    // 获取 appinfo.vdf 路径
    fn appinfo_path(&self) -> PathBuf {
        self.steam_path.join("appcache").join("appinfo.vdf")
    }

    // 注入自定义 ufs 配置
    // 将在现有 savefiles 列表末尾添加新的条目
    pub fn inject_savefiles(
        &self,
        app_id: u32,
        custom_savefiles: Vec<SaveFileConfig>,
    ) -> Result<()> {
        let appinfo_path = self.appinfo_path();
        if !appinfo_path.exists() {
            return Err(anyhow!("appinfo.vdf 不存在: {:?}", appinfo_path));
        }

        // 备份原文件
        let backup_path = appinfo_path.with_extension("vdf.bak");
        fs::copy(&appinfo_path, &backup_path)?;
        tracing::info!("已备份 appinfo.vdf 到 {:?}", backup_path);

        let data = fs::read(&appinfo_path)?;
        let modified_data = self.modify_app_ufs(&data, app_id, custom_savefiles)?;

        fs::write(&appinfo_path, &modified_data)?;
        tracing::info!(
            "已注入 ufs 配置到 app_id {}, 文件大小: {} -> {} bytes",
            app_id,
            data.len(),
            modified_data.len()
        );

        Ok(())
    }

    // 修改指定 app 的 ufs 配置
    fn modify_app_ufs(
        &self,
        data: &[u8],
        target_app_id: u32,
        custom_savefiles: Vec<SaveFileConfig>,
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

        // 修改条目
        let modified_entry = self.modify_entry(
            original_entry,
            &custom_savefiles,
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

    // 修改单个 app 条目
    fn modify_entry(
        &self,
        entry: &[u8],
        custom_savefiles: &[SaveFileConfig],
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

        // 修改 VDF 数据，添加自定义 savefiles
        let modified_vdf = self.inject_savefiles_to_vdf(
            vdf_data,
            custom_savefiles,
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

    // 将自定义 savefiles 注入到 VDF 数据中
    fn inject_savefiles_to_vdf(
        &self,
        vdf_data: &[u8],
        custom_savefiles: &[SaveFileConfig],
        version: u32,
        string_table: &mut Vec<String>,
        string_to_idx: &mut HashMap<String, usize>,
    ) -> Result<Vec<u8>> {
        // 简化策略：
        // 1. 如果有 savefiles 节，在其末尾添加新条目
        // 2. 如果没有 ufs 节，在根节点末尾创建完整的 ufs/savefiles 结构
        // 3. 如果有 ufs 节但没有 savefiles，暂时不支持（需要更复杂的处理）

        let mut cursor = Cursor::new(vdf_data);

        // 解析 VDF 结构找到关键位置
        let mut info = UfsSectionInfo::default();

        // 遍历根节点找 ufs
        if version >= 29 {
            self.find_ufs_section_v29_improved(&mut cursor, string_table, &mut info)?;
        } else {
            self.find_ufs_section_v28_improved(&mut cursor, &mut info)?;
        }

        tracing::info!(
            "VDF 解析结果: ufs={:?}..{:?}, savefiles={:?}..{:?}, last_idx={}, root_end={:?}",
            info.ufs_section_pos,
            info.ufs_end_pos,
            info.savefiles_section_pos,
            info.savefiles_end_pos,
            info.last_savefile_index,
            info.root_end_pos
        );

        // 构建新的 VDF 数据
        let mut result = Vec::new();

        if let Some(savefiles_end) = info.savefiles_end_pos {
            // 情况1: 有 savefiles 节，在其末尾插入新条目
            result.extend_from_slice(&vdf_data[..savefiles_end]);

            for (i, savefile) in custom_savefiles.iter().enumerate() {
                let index = info.last_savefile_index + 1 + i as u32;
                if version >= 29 {
                    let encoded =
                        self.encode_savefile_v29(savefile, index, string_table, string_to_idx);
                    result.extend_from_slice(&encoded);
                } else {
                    let encoded = self.encode_savefile_v28(savefile, index);
                    result.extend_from_slice(&encoded);
                }
            }

            result.extend_from_slice(&vdf_data[savefiles_end..]);
            tracing::info!(
                "在现有 savefiles 末尾添加了 {} 个条目",
                custom_savefiles.len()
            );
        } else if info.ufs_section_pos.is_none() {
            // 情况2: 没有 ufs 节，在根节点末尾创建完整结构
            let root_end = info
                .root_end_pos
                .ok_or_else(|| anyhow!("无法找到 VDF 根节点结束位置"))?;
            result.extend_from_slice(&vdf_data[..root_end]);

            // 创建 ufs 节
            if version >= 29 {
                let ufs_idx = self.get_or_create_string_index("ufs", string_table, string_to_idx);
                result.push(VDF_TYPE_SECTION);
                result.extend_from_slice(&(ufs_idx as u32).to_le_bytes());

                let savefiles_idx =
                    self.get_or_create_string_index("savefiles", string_table, string_to_idx);
                result.push(VDF_TYPE_SECTION);
                result.extend_from_slice(&(savefiles_idx as u32).to_le_bytes());
            } else {
                result.push(VDF_TYPE_SECTION);
                result.extend_from_slice(b"ufs\0");
                result.push(VDF_TYPE_SECTION);
                result.extend_from_slice(b"savefiles\0");
            }

            for (i, savefile) in custom_savefiles.iter().enumerate() {
                if version >= 29 {
                    let encoded =
                        self.encode_savefile_v29(savefile, i as u32, string_table, string_to_idx);
                    result.extend_from_slice(&encoded);
                } else {
                    let encoded = self.encode_savefile_v28(savefile, i as u32);
                    result.extend_from_slice(&encoded);
                }
            }

            // savefiles 节结束
            result.push(VDF_TYPE_SECTION_END);
            // ufs 节结束
            result.push(VDF_TYPE_SECTION_END);

            result.extend_from_slice(&vdf_data[root_end..]);
            tracing::info!(
                "创建了新的 ufs/savefiles 结构，包含 {} 个条目",
                custom_savefiles.len()
            );
        } else if let (Some(_ufs_start), Some(ufs_end)) = (info.ufs_section_pos, info.ufs_end_pos) {
            // 情况3: 有 ufs 节但没有 savefiles，在 ufs 节内添加 savefiles
            // 策略：复制到 ufs 结束标记之前，插入 savefiles，然后复制结束标记

            // 复制到 ufs 结束之前（不包括最后的 0x08）
            result.extend_from_slice(&vdf_data[..ufs_end]);

            // 添加 savefiles 节
            if version >= 29 {
                let savefiles_idx =
                    self.get_or_create_string_index("savefiles", string_table, string_to_idx);
                result.push(VDF_TYPE_SECTION);
                result.extend_from_slice(&(savefiles_idx as u32).to_le_bytes());
            } else {
                result.push(VDF_TYPE_SECTION);
                result.extend_from_slice(b"savefiles\0");
            }

            for (i, savefile) in custom_savefiles.iter().enumerate() {
                if version >= 29 {
                    let encoded =
                        self.encode_savefile_v29(savefile, i as u32, string_table, string_to_idx);
                    result.extend_from_slice(&encoded);
                } else {
                    let encoded = self.encode_savefile_v28(savefile, i as u32);
                    result.extend_from_slice(&encoded);
                }
            }

            // savefiles 节结束
            result.push(VDF_TYPE_SECTION_END);

            // 复制 ufs 结束标记及之后的内容
            result.extend_from_slice(&vdf_data[ufs_end..]);
            tracing::info!(
                "在现有 ufs 节内创建了 savefiles，包含 {} 个条目",
                custom_savefiles.len()
            );
        } else {
            return Err(anyhow!("无法解析 VDF 结构"));
        }

        Ok(result)
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

        while let Ok(type_byte) = cursor.read_u8() {
            let pos = cursor.position() as usize - 1;

            if type_byte == VDF_TYPE_SECTION_END {
                if in_savefiles {
                    info.savefiles_end_pos = Some(pos);
                    in_savefiles = false;
                } else if in_ufs {
                    info.ufs_end_pos = Some(pos);
                    in_ufs = false;
                }
                depth -= 1;
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
                    } else if in_ufs && key == "savefiles" {
                        info.savefiles_section_pos = Some(pos);
                        in_savefiles = true;
                    } else if in_savefiles
                        && let Ok(idx) = key.parse::<u32>()
                        && idx > info.last_savefile_index
                    {
                        info.last_savefile_index = idx;
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

    // V28 格式查找 ufs 节 (改进版)
    fn find_ufs_section_v28_improved(
        &self,
        cursor: &mut Cursor<&[u8]>,
        info: &mut UfsSectionInfo,
    ) -> Result<()> {
        let mut depth = 0;
        let mut in_ufs = false;
        let mut in_savefiles = false;

        while let Ok(type_byte) = cursor.read_u8() {
            let pos = cursor.position() as usize - 1;

            if type_byte == VDF_TYPE_SECTION_END {
                if in_savefiles {
                    info.savefiles_end_pos = Some(pos);
                    in_savefiles = false;
                } else if in_ufs {
                    info.ufs_end_pos = Some(pos);
                    in_ufs = false;
                }
                depth -= 1;
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
                    } else if in_ufs && key == "savefiles" {
                        info.savefiles_section_pos = Some(pos);
                        in_savefiles = true;
                    } else if in_savefiles
                        && let Ok(idx) = key.parse::<u32>()
                        && idx > info.last_savefile_index
                    {
                        info.last_savefile_index = idx;
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
