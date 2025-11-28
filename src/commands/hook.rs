use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use crate::cli::HookAction;
use crate::git::GitConfigManager;

/// Git hook 脚本内容
const PRE_COMMIT_HOOK: &str = r#"#!/bin/sh
# gid pre-commit hook
# 检查 Git 身份是否符合规则

# 允许跳过检查
if [ "$GID_SKIP" = "1" ]; then
    exit 0
fi

# 检查 gid 是否可用
if ! command -v gid &> /dev/null; then
    echo "Warning: gid not found, skipping identity check"
    exit 0
fi

# 运行检查
output=$(gid doctor 2>&1)
exit_code=$?

if [ $exit_code -ne 0 ]; then
    echo ""
    echo "╭─────────────────────────────────────────────────╮"
    echo "│  ⚠️  Git Identity Check Failed                  │"
    echo "╰─────────────────────────────────────────────────╯"
    echo ""
    echo "$output"
    echo ""
    echo "To fix: gid doctor --fix"
    echo "To skip: GID_SKIP=1 git commit"
    echo "Or:      git commit --no-verify"
    echo ""
    exit 1
fi

exit 0
"#;

/// 执行 hook 命令
pub fn execute(action: HookAction) -> Result<()> {
    match action {
        HookAction::Install { global } => install_hook(global),
        HookAction::Uninstall { global } => uninstall_hook(global),
        HookAction::Status => show_status(),
    }
}

/// 安装 hook
fn install_hook(global: bool) -> Result<()> {
    if global {
        install_global_hook()
    } else {
        install_local_hook()
    }
}

/// 安装本地 hook
fn install_local_hook() -> Result<()> {
    let git = GitConfigManager::new()?;

    if !git.is_in_repo() {
        anyhow::bail!("当前目录不是 Git 仓库");
    }

    let hooks_dir = git
        .repo_path()
        .ok_or_else(|| anyhow::anyhow!("无法获取仓库路径"))?
        .join("hooks");

    fs::create_dir_all(&hooks_dir).context("无法创建 hooks 目录")?;

    let hook_path = hooks_dir.join("pre-commit");

    // 检查是否已存在 hook
    if hook_path.exists() {
        let content = fs::read_to_string(&hook_path)?;
        if !content.contains("gid") {
            println!("{} 已存在 pre-commit hook", "!".yellow());
            println!("  {}", hook_path.display().to_string().dimmed());

            let confirm = dialoguer::Confirm::new()
                .with_prompt("是否覆盖?")
                .default(false)
                .interact()?;

            if !confirm {
                println!("操作已取消");
                return Ok(());
            }
        }
    }

    // 写入 hook
    fs::write(&hook_path, PRE_COMMIT_HOOK).context("无法写入 hook 文件")?;

    // 设置可执行权限
    #[cfg(unix)]
    fs::set_permissions(&hook_path, fs::Permissions::from_mode(0o755))?;

    println!("{} 已安装 pre-commit hook", "✓".green());
    println!("  {}", hook_path.display().to_string().dimmed());

    Ok(())
}

/// 安装全局 hook
fn install_global_hook() -> Result<()> {
    let home = home::home_dir().ok_or_else(|| anyhow::anyhow!("无法获取主目录"))?;

    let hooks_dir = home.join(".config").join("git").join("hooks");

    fs::create_dir_all(&hooks_dir).context("无法创建全局 hooks 目录")?;

    let hook_path = hooks_dir.join("pre-commit");

    // 写入 hook
    fs::write(&hook_path, PRE_COMMIT_HOOK).context("无法写入 hook 文件")?;

    // 设置可执行权限
    #[cfg(unix)]
    fs::set_permissions(&hook_path, fs::Permissions::from_mode(0o755))?;

    // 设置 Git 全局配置
    let output = std::process::Command::new("git")
        .args([
            "config",
            "--global",
            "core.hooksPath",
            hooks_dir.to_str().unwrap(),
        ])
        .output()
        .context("无法设置 core.hooksPath")?;

    if !output.status.success() {
        anyhow::bail!("设置 core.hooksPath 失败");
    }

    println!("{} 已安装全局 pre-commit hook", "✓".green());
    println!("  {}", hook_path.display().to_string().dimmed());
    println!();
    println!(
        "已设置 {} = {}",
        "core.hooksPath".cyan(),
        hooks_dir.display()
    );

    Ok(())
}

/// 卸载 hook
fn uninstall_hook(global: bool) -> Result<()> {
    if global {
        uninstall_global_hook()
    } else {
        uninstall_local_hook()
    }
}

/// 卸载本地 hook
fn uninstall_local_hook() -> Result<()> {
    let git = GitConfigManager::new()?;

    if !git.is_in_repo() {
        anyhow::bail!("当前目录不是 Git 仓库");
    }

    let hook_path = git
        .repo_path()
        .ok_or_else(|| anyhow::anyhow!("无法获取仓库路径"))?
        .join("hooks")
        .join("pre-commit");

    if !hook_path.exists() {
        println!("{} hook 不存在", "!".yellow());
        return Ok(());
    }

    // 检查是否是 gid 的 hook
    let content = fs::read_to_string(&hook_path)?;
    if !content.contains("gid") {
        println!("{} 这不是 gid 的 hook，跳过删除", "!".yellow());
        return Ok(());
    }

    fs::remove_file(&hook_path).context("无法删除 hook 文件")?;

    println!("{} 已卸载 pre-commit hook", "✓".green());

    Ok(())
}

/// 卸载全局 hook
fn uninstall_global_hook() -> Result<()> {
    let home = home::home_dir().ok_or_else(|| anyhow::anyhow!("无法获取主目录"))?;

    let hook_path = home
        .join(".config")
        .join("git")
        .join("hooks")
        .join("pre-commit");

    if hook_path.exists() {
        let content = fs::read_to_string(&hook_path)?;
        if content.contains("gid") {
            fs::remove_file(&hook_path)?;
            println!("{} 已删除全局 hook", "✓".green());
        }
    }

    // 移除 Git 全局配置
    let _ = std::process::Command::new("git")
        .args(["config", "--global", "--unset", "core.hooksPath"])
        .output();

    println!("{} 已移除 core.hooksPath 配置", "✓".green());

    Ok(())
}

/// 显示 hook 状态
fn show_status() -> Result<()> {
    println!("{}", "Git Hook 状态:".bold());
    println!();

    // 检查本地 hook
    let git = GitConfigManager::new()?;

    if git.is_in_repo() {
        let local_hook = git.repo_path().map(|p| p.join("hooks").join("pre-commit"));

        if let Some(hook_path) = local_hook {
            if hook_path.exists() {
                let content = fs::read_to_string(&hook_path).unwrap_or_default();
                let is_gid = content.contains("gid");

                if is_gid {
                    println!("  {} 本地 hook: {}", "✓".green(), "已安装 (gid)".green());
                } else {
                    println!(
                        "  {} 本地 hook: {}",
                        "!".yellow(),
                        "已存在 (非 gid)".yellow()
                    );
                }
                println!("    {}", hook_path.display().to_string().dimmed());
            } else {
                println!("  {} 本地 hook: {}", "○".dimmed(), "未安装".dimmed());
            }
        }
    } else {
        println!(
            "  {} 本地 hook: {}",
            "○".dimmed(),
            "不在 Git 仓库中".dimmed()
        );
    }

    println!();

    // 检查全局 hook
    let global_hooks_path = std::process::Command::new("git")
        .args(["config", "--global", "--get", "core.hooksPath"])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
            } else {
                None
            }
        });

    if let Some(ref hooks_path) = global_hooks_path {
        let hook_path = PathBuf::from(hooks_path).join("pre-commit");

        if hook_path.exists() {
            let content = fs::read_to_string(&hook_path).unwrap_or_default();
            let is_gid = content.contains("gid");

            if is_gid {
                println!("  {} 全局 hook: {}", "✓".green(), "已安装 (gid)".green());
            } else {
                println!(
                    "  {} 全局 hook: {}",
                    "!".yellow(),
                    "已存在 (非 gid)".yellow()
                );
            }
            println!("    {}", hook_path.display().to_string().dimmed());
        } else {
            println!("  {} 全局 hook: {}", "○".dimmed(), "未安装".dimmed());
        }
        println!("    core.hooksPath = {}", hooks_path.dimmed());
    } else {
        println!("  {} 全局 hook: {}", "○".dimmed(), "未配置".dimmed());
    }

    Ok(())
}
