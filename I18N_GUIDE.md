# i18n 贡献指南 / Internationalization Guide

[English](#english) | [简体中文](#简体中文)

---

## 简体中文

### 概述

本项目使用 Rust 原生的 i18n 实现，所有翻译都在 `src/i18n.rs` 文件中管理。

### 当前支持的语言

- 简体中文 (Chinese Simplified) - 默认
- English

### 如何添加新语言

#### 1. 添加语言枚举

在 `src/i18n.rs` 中的 `Language` 枚举添加新语言：

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Language {
    #[default]
    Chinese,
    English,
    Japanese,  // 新增
}
```

#### 2. 更新 `all()` 方法

```rust
impl Language {
    pub const fn all() -> &'static [Language] {
        &[Language::Chinese, Language::English, Language::Japanese]
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Language::Chinese => "简体中文",
            Language::English => "English",
            Language::Japanese => "日本語",  // 新增
        }
    }
}
```

#### 3. 添加翻译

为 `I18n` 结构体中的每个方法添加新语言的翻译：

```rust
pub fn app_title(&self) -> &'static str {
    match self.lang {
        Language::Chinese => "Steam 云管理器",
        Language::English => "Steam Cloud Manager",
        Language::Japanese => "Steam クラウドマネージャー",  // 新增
    }
}
```

### 翻译函数分类

| 分类        | 说明           | 示例                                             |
| ----------- | -------------- | ------------------------------------------------ |
| UI 通用文本 | 按钮、标签等   | `refresh()`, `confirm()`, `cancel()`             |
| 连接面板    | Steam 连接相关 | `connect()`, `disconnect()`, `steam_running()`   |
| 文件操作    | 文件管理相关   | `download()`, `upload()`, `delete()`, `forget()` |
| 窗口标题    | 各窗口标题     | `about_title()`, `error_title()`                 |
| 状态消息    | 操作状态提示   | `loading()`, `success()`, `failed()`             |
| 错误消息    | 错误提示       | `error_enter_app_id()`, `download_failed()`      |

### 翻译注意事项

1. **保持一致性**：相同概念使用相同的翻译
2. **简洁明了**：按钮文本应简短
3. **上下文准确**：根据实际功能翻译
4. **特殊术语**：
   - `Forget` = 移出云端（从云端删除但保留本地）
   - `Delete` = 删除（同时删除本地和云端）
   - `Cloud` = 云端/云存储

### 测试翻译

1. 编译项目：`cargo build`
2. 运行项目：`cargo run`
3. 在应用中切换语言测试

### 提交 PR

1. Fork 本仓库
2. 创建分支：`git checkout -b feat/i18n-japanese`
3. 提交更改：`git commit -m "feat(i18n): add Japanese translation"`
4. 推送并创建 Pull Request

---

## English

### Overview

This project uses a native Rust i18n implementation. All translations are managed in the `src/i18n.rs` file.

### Currently Supported Languages

- Chinese Simplified - Default
- English

### How to Add a New Language

#### 1. Add Language Enum

Add new language to the `Language` enum in `src/i18n.rs`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Language {
    #[default]
    Chinese,
    English,
    Japanese,  // New
}
```

#### 2. Update `all()` Method

```rust
impl Language {
    pub const fn all() -> &'static [Language] {
        &[Language::Chinese, Language::English, Language::Japanese]
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Language::Chinese => "简体中文",
            Language::English => "English",
            Language::Japanese => "日本語",  // New
        }
    }
}
```

#### 3. Add Translations

Add translations for each method in the `I18n` struct:

```rust
pub fn app_title(&self) -> &'static str {
    match self.lang {
        Language::Chinese => "Steam 云管理器",
        Language::English => "Steam Cloud Manager",
        Language::Japanese => "Steam クラウドマネージャー",  // New
    }
}
```

### Translation Function Categories

| Category         | Description      | Examples                                         |
| ---------------- | ---------------- | ------------------------------------------------ |
| UI Common        | Buttons, labels  | `refresh()`, `confirm()`, `cancel()`             |
| Connection Panel | Steam connection | `connect()`, `disconnect()`, `steam_running()`   |
| File Operations  | File management  | `download()`, `upload()`, `delete()`, `forget()` |
| Window Titles    | Window titles    | `about_title()`, `error_title()`                 |
| Status Messages  | Operation status | `loading()`, `success()`, `failed()`             |
| Error Messages   | Error prompts    | `error_enter_app_id()`, `download_failed()`      |

### Translation Notes

1. **Consistency**: Use the same translation for the same concept
2. **Conciseness**: Button text should be short
3. **Context Accuracy**: Translate according to actual functionality
4. **Special Terms**:
   - `Forget` = Remove from cloud (delete from cloud but keep local)
   - `Delete` = Delete (remove both local and cloud)
   - `Cloud` = Cloud storage

### Testing Translations

1. Build: `cargo build`
2. Run: `cargo run`
3. Switch language in the app to test

### Submitting a PR

1. Fork this repository
2. Create branch: `git checkout -b feat/i18n-japanese`
3. Commit: `git commit -m "feat(i18n): add Japanese translation"`
4. Push and create Pull Request
