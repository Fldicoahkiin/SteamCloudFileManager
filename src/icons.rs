//! Phosphor 图标常量定义
//!
//! 统一管理所有 UI 中使用的图标，方便维护和替换

#![allow(dead_code)]

use egui_phosphor::regular as ph;

// 通用操作图标
pub const COPY: &str = ph::COPY;
pub const TRASH: &str = ph::TRASH;
pub const REFRESH: &str = ph::ARROWS_CLOCKWISE;
pub const CLOSE: &str = ph::X;
pub const CHECK: &str = ph::CHECK;
pub const WARNING: &str = ph::WARNING;
pub const ERROR: &str = ph::X_CIRCLE;
pub const INFO: &str = ph::INFO;

// 文件操作图标
pub const FOLDER: &str = ph::FOLDER;
pub const FOLDER_OPEN: &str = ph::FOLDER_OPEN;
pub const FILE: &str = ph::FILE;
pub const DOWNLOAD: &str = ph::DOWNLOAD_SIMPLE;
pub const UPLOAD: &str = ph::UPLOAD_SIMPLE;
pub const ADD_FILE: &str = ph::FILE_PLUS;
pub const ADD_FOLDER: &str = ph::FOLDER_PLUS;

// 云同步图标
pub const CLOUD: &str = ph::CLOUD;
pub const CLOUD_UPLOAD: &str = ph::CLOUD_ARROW_UP;
pub const CLOUD_DOWNLOAD: &str = ph::CLOUD_ARROW_DOWN;
pub const CLOUD_CHECK: &str = ph::CLOUD_CHECK;

// 连接/链接图标
pub const LINK: &str = ph::LINK;
pub const UNLINK: &str = ph::LINK_BREAK;
pub const PLUG: &str = ph::PLUG;
pub const PLUGS_CONNECTED: &str = ph::PLUGS_CONNECTED;

// 状态图标
pub const SPINNER: &str = ph::CIRCLE_NOTCH;
pub const HOURGLASS: &str = ph::HOURGLASS;
pub const CLOCK: &str = ph::CLOCK;

// 箭头图标
pub const ARROW_UP: &str = ph::ARROW_UP;
pub const ARROW_DOWN: &str = ph::ARROW_DOWN;
pub const ARROW_LEFT: &str = ph::ARROW_LEFT;
pub const ARROW_RIGHT: &str = ph::ARROW_RIGHT;
pub const ARROW_SYNC: &str = ph::ARROWS_CLOCKWISE;

// 游戏/Steam 相关
pub const GAME: &str = ph::GAME_CONTROLLER;
pub const GEAR: &str = ph::GEAR;
pub const WRENCH: &str = ph::WRENCH;

// 其他
pub const MAGNIFYING_GLASS: &str = ph::MAGNIFYING_GLASS;
pub const QUESTION: &str = ph::QUESTION;
pub const SLIDERS: &str = ph::SLIDERS;
pub const EXPORT: &str = ph::EXPORT;
pub const SCISSORS: &str = ph::SCISSORS;
pub const GLOBE: &str = ph::GLOBE;
pub const ROCKET: &str = ph::ROCKET;
