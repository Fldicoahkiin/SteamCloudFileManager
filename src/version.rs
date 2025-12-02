// 格式: "1.0.0-abc12345" 或 "1.0.0-abc12345-dirty"
pub fn full_version() -> &'static str {
    env!("FULL_VERSION")
}

// 获取 Cargo.toml 中定义的版本号
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

// 获取 git commit hash (短格式，8位)
pub fn git_hash() -> &'static str {
    env!("GIT_HASH")
}

// 获取 git 分支名
pub fn git_branch() -> &'static str {
    env!("GIT_BRANCH")
}

// 检查是否有未提交的修改
pub fn is_dirty() -> bool {
    !env!("GIT_DIRTY").is_empty()
}

// 获取版本信息的详细描述
pub fn version_info() -> String {
    let dirty_marker = if is_dirty() { " (modified)" } else { "" };
    format!(
        "Steam Cloud File Manager v{}\nGit: {} ({}){}",
        version(),
        git_hash(),
        git_branch(),
        dirty_marker
    )
}

// 获取操作系统名称
pub fn os_name() -> &'static str {
    std::env::consts::OS
}

// 获取系统架构
pub fn os_arch() -> &'static str {
    std::env::consts::ARCH
}

// 获取编译时间
pub fn build_time() -> &'static str {
    env!("BUILD_TIME")
}

// 获取编译配置 (debug/release)
pub fn build_profile() -> &'static str {
    if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_info() {
        println!("Full version: {}", full_version());
        println!("Version: {}", version());
        println!("Git hash: {}", git_hash());
        println!("Git branch: {}", git_branch());
        println!("Is dirty: {}", is_dirty());
        println!("\n{}", version_info());
    }
}
