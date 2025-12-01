use std::process::Command;

fn main() {
    // 获取 git commit hash
    let output = Command::new("git")
        .args(["rev-parse", "--short=8", "HEAD"])
        .output();

    let git_hash = match output {
        Ok(output) if output.status.success() => String::from_utf8(output.stdout)
            .unwrap_or_else(|_| "unknown".to_string())
            .trim()
            .to_string(),
        _ => "unknown".to_string(),
    };

    // 获取 git branch
    let branch_output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output();

    let git_branch = match branch_output {
        Ok(output) if output.status.success() => String::from_utf8(output.stdout)
            .unwrap_or_else(|_| "unknown".to_string())
            .trim()
            .to_string(),
        _ => "unknown".to_string(),
    };

    // 检查是否有未提交的修改
    let dirty_output = Command::new("git").args(["status", "--porcelain"]).output();

    let is_dirty = match dirty_output {
        Ok(output) if output.status.success() => !output.stdout.is_empty(),
        _ => false,
    };

    let dirty_suffix = if is_dirty { "-dirty" } else { "" };

    // 设置环境变量供编译时使用
    println!("cargo:rustc-env=GIT_HASH={}", git_hash);
    println!("cargo:rustc-env=GIT_BRANCH={}", git_branch);
    println!("cargo:rustc-env=GIT_DIRTY={}", dirty_suffix);

    // 构建完整版本字符串
    let version = env!("CARGO_PKG_VERSION");
    let full_version = format!("{}-{}{}", version, git_hash, dirty_suffix);
    println!("cargo:rustc-env=FULL_VERSION={}", full_version);

    // 当 git 状态改变时重新运行
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/index");
}
