use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;
use std::path::PathBuf;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use crate::cli::HookAction;
use crate::git::GitConfigManager;

/// Git hook 脚本内容
const PRE_COMMIT_HOOK: &str = r#"#!/bin/sh
# gid pre-commit hook
# Check if Git identity matches rules

# Allow skipping check
if [ "$GID_SKIP" = "1" ]; then
    exit 0
fi

# Check if gid is available
if ! command -v gid &> /dev/null; then
    echo "Warning: gid not found, skipping identity check"
    exit 0
fi

# Run check
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
        anyhow::bail!("Current directory is not a Git repository");
    }

    let hooks_dir = git
        .repo_path()
        .ok_or_else(|| anyhow::anyhow!("Could not get repository path"))?
        .join("hooks");

    fs::create_dir_all(&hooks_dir).context("Could not create hooks directory")?;

    let hook_path = hooks_dir.join("pre-commit");

    // 检查是否已存在 hook
    if hook_path.exists() {
        let content = fs::read_to_string(&hook_path)?;
        if !content.contains("gid") {
            println!("{} pre-commit hook already exists", "!".yellow());
            println!("  {}", hook_path.display().to_string().dimmed());

            let confirm = dialoguer::Confirm::new()
                .with_prompt("Overwrite?")
                .default(false)
                .interact()?;

            if !confirm {
                println!("Operation cancelled");
                return Ok(());
            }
        }
    }

    // 写入 hook
    fs::write(&hook_path, PRE_COMMIT_HOOK).context("Could not write hook file")?;

    // 设置可执行权限 (仅 Unix)
    #[cfg(unix)]
    fs::set_permissions(&hook_path, fs::Permissions::from_mode(0o755))?;

    println!("{} pre-commit hook installed", "✓".green());
    println!("  {}", hook_path.display().to_string().dimmed());

    Ok(())
}

/// 安装全局 hook
fn install_global_hook() -> Result<()> {
    let home = home::home_dir().ok_or_else(|| anyhow::anyhow!("Could not get home directory"))?;

    let hooks_dir = home.join(".config").join("git").join("hooks");

    fs::create_dir_all(&hooks_dir).context("Could not create global hooks directory")?;

    let hook_path = hooks_dir.join("pre-commit");

    // 写入 hook
    fs::write(&hook_path, PRE_COMMIT_HOOK).context("Could not write hook file")?;

    // 设置可执行权限 (仅 Unix)
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
        .context("Could not set core.hooksPath")?;

    if !output.status.success() {
        anyhow::bail!("Failed to set core.hooksPath");
    }

    println!("{} Global pre-commit hook installed", "✓".green());
    println!("  {}", hook_path.display().to_string().dimmed());
    println!();
    println!(
        "Set {} = {}",
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
        anyhow::bail!("Current directory is not a Git repository");
    }

    let hook_path = git
        .repo_path()
        .ok_or_else(|| anyhow::anyhow!("Could not get repository path"))?
        .join("hooks")
        .join("pre-commit");

    if !hook_path.exists() {
        println!("{} hook does not exist", "!".yellow());
        return Ok(());
    }

    // 检查是否是 gid 的 hook
    let content = fs::read_to_string(&hook_path)?;
    if !content.contains("gid") {
        println!("{} This is not a gid hook, skipping removal", "!".yellow());
        return Ok(());
    }

    fs::remove_file(&hook_path).context("Could not remove hook file")?;

    println!("{} pre-commit hook uninstalled", "✓".green());

    Ok(())
}

/// 卸载全局 hook
fn uninstall_global_hook() -> Result<()> {
    let home = home::home_dir().ok_or_else(|| anyhow::anyhow!("Could not get home directory"))?;

    let hook_path = home
        .join(".config")
        .join("git")
        .join("hooks")
        .join("pre-commit");

    if hook_path.exists() {
        let content = fs::read_to_string(&hook_path)?;
        if content.contains("gid") {
            fs::remove_file(&hook_path)?;
            println!("{} Global hook removed", "✓".green());
        }
    }

    // 移除 Git 全局配置
    let _ = std::process::Command::new("git")
        .args(["config", "--global", "--unset", "core.hooksPath"])
        .output();

    println!("{} core.hooksPath configuration removed", "✓".green());

    Ok(())
}

/// 显示 hook 状态
fn show_status() -> Result<()> {
    println!("{}", "Git Hook Status:".bold());
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
                    println!("  {} Local hook: {}", "✓".green(), "Installed (gid)".green());
                } else {
                    println!(
                        "  {} Local hook: {}",
                        "!".yellow(),
                        "Exists (non-gid)".yellow()
                    );
                }
                println!("    {}", hook_path.display().to_string().dimmed());
            } else {
                println!("  {} Local hook: {}", "○".dimmed(), "Not installed".dimmed());
            }
        }
    } else {
        println!(
            "  {} Local hook: {}",
            "○".dimmed(),
            "Not in a Git repository".dimmed()
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
                println!("  {} Global hook: {}", "✓".green(), "Installed (gid)".green());
            } else {
                println!(
                    "  {} Global hook: {}",
                    "!".yellow(),
                    "Exists (non-gid)".yellow()
                );
            }
            println!("    {}", hook_path.display().to_string().dimmed());
        } else {
            println!("  {} Global hook: {}", "○".dimmed(), "Not installed".dimmed());
        }
        println!("    core.hooksPath = {}", hooks_path.dimmed());
    } else {
        println!("  {} Global hook: {}", "○".dimmed(), "Not configured".dimmed());
    }

    Ok(())
}
