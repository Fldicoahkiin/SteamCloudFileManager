# i18n 贡献指南 / Internationalization Guide

[English](#english) | [简体中文](#简体中文)

---

## 简体中文

### 概述

本项目使用 Rust 原生的 i18n 实现，翻译按语言拆分为独立文件，便于维护和添加新语言。

### 文件结构

```
src/i18n/
├── mod.rs    # Language 枚举、I18n 结构体、语言分发逻辑
├── en.rs     # English 翻译
└── zh.rs     # 简体中文翻译
```

- `mod.rs` 定义 `Language` 枚举和 `I18n` 结构体，每个翻译方法通过 `match self.lang` 分发到对应的语言模块。
- 每个语言文件导出与 `I18n` 方法同名的函数，返回对应语言的翻译文本。

### 当前支持的语言

- English
- 简体中文 (Chinese Simplified)

### 如何添加新语言

以添加日语 (Japanese) 为例：

#### 1. 在 `mod.rs` 中添加语言枚举

```rust
pub enum Language {
    #[default]
    English,
    Chinese,
    Japanese,  // 新增
}
```

#### 2. 更新 `mod.rs` 中的 `Language` 方法

```rust
pub const fn all() -> &'static [Language] {
    &[Language::English, Language::Chinese, Language::Japanese]
}

pub fn display_name(&self) -> &'static str {
    match self {
        Language::English => "English",
        Language::Chinese => "简体中文",
        Language::Japanese => "日本語",
    }
}

pub fn from_config(value: &str) -> Self {
    match value {
        "en" => Language::English,
        "zh" => Language::Chinese,
        "ja" => Language::Japanese,
        "auto" => Self::detect_system_language(),
        _ => Language::English,
    }
}

pub fn to_config(self) -> &'static str {
    match self {
        Language::Chinese => "zh",
        Language::English => "en",
        Language::Japanese => "ja",
    }
}
```

#### 3. 创建新的语言文件 `src/i18n/ja.rs`

参考 `en.rs` 或 `zh.rs`，实现所有同名函数：

```rust
// 日本語翻訳

use crate::icons;

pub fn app_title() -> &'static str {
    "Steam クラウドセーブマネージャー"
}

pub fn refresh() -> &'static str {
    "更新"
}

// ... 所有其他翻译函数
```

#### 4. 在 `mod.rs` 中注册模块并更新分发

```rust
mod en;
mod ja;  // 新增
mod zh;
```

在每个 `I18n` 方法的 `match` 中添加新语言分支：

```rust
pub fn app_title(&self) -> &'static str {
    match self.lang {
        Language::English => en::app_title(),
        Language::Chinese => zh::app_title(),
        Language::Japanese => ja::app_title(),
    }
}
```

> **提示**：可以使用编辑器的全局替换功能批量添加新语言分支。

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

This project uses a native Rust i18n implementation. Translations are split into separate files per language for easy maintenance and extensibility.

### File Structure

```
src/i18n/
├── mod.rs    # Language enum, I18n struct, dispatch logic
├── en.rs     # English translations
└── zh.rs     # Chinese Simplified translations
```

- `mod.rs` defines the `Language` enum and `I18n` struct. Each translation method dispatches to the corresponding language module via `match self.lang`.
- Each language file exports functions with the same names as `I18n` methods, returning translated text.

### Currently Supported Languages

- English
- 简体中文 (Chinese Simplified)

### How to Add a New Language

Using Japanese as an example:

#### 1. Add Language Enum in `mod.rs`

```rust
pub enum Language {
    #[default]
    English,
    Chinese,
    Japanese,  // New
}
```

#### 2. Update `Language` Methods in `mod.rs`

```rust
pub const fn all() -> &'static [Language] {
    &[Language::English, Language::Chinese, Language::Japanese]
}

pub fn display_name(&self) -> &'static str {
    match self {
        Language::English => "English",
        Language::Chinese => "简体中文",
        Language::Japanese => "日本語",
    }
}

pub fn from_config(value: &str) -> Self {
    match value {
        "en" => Language::English,
        "zh" => Language::Chinese,
        "ja" => Language::Japanese,
        "auto" => Self::detect_system_language(),
        _ => Language::English,
    }
}

pub fn to_config(self) -> &'static str {
    match self {
        Language::Chinese => "zh",
        Language::English => "en",
        Language::Japanese => "ja",
    }
}
```

#### 3. Create New Language File `src/i18n/ja.rs`

Use `en.rs` or `zh.rs` as a reference, implementing all functions with the same names:

```rust
// Japanese translations

use crate::icons;

pub fn app_title() -> &'static str {
    "Steam クラウドセーブマネージャー"
}

pub fn refresh() -> &'static str {
    "更新"
}

// ... all other translation functions
```

#### 4. Register Module and Update Dispatch in `mod.rs`

```rust
mod en;
mod ja;  // New
mod zh;
```

Add the new language branch to every `I18n` method's `match`:

```rust
pub fn app_title(&self) -> &'static str {
    match self.lang {
        Language::English => en::app_title(),
        Language::Chinese => zh::app_title(),
        Language::Japanese => ja::app_title(),
    }
}
```

> **Tip**: Use your editor's global find-and-replace to batch-add new language branches.

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
