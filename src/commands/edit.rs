use anyhow::{Context, Result};
use colored::Colorize;
use std::process::Command;

use crate::config::Config;

/// 编辑配置文件
pub fn execute() -> Result<()> {
    let config_path = Config::config_path()?;

    // 确保配置文件存在
    if !config_path.exists() {
        let config = Config::default();
        config.save()?;
        println!(
            "{} Configuration file created: {}",
            "→".blue(),
            config_path.display()
        );
    }

    // 获取编辑器
    let editor = std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| {
            if cfg!(target_os = "windows") {
                "notepad".to_string()
            } else {
                "vi".to_string()
            }
        });

    println!(
        "{} Editing configuration file using {}...",
        "→".blue(),
        editor
    );
    println!("  {}", config_path.display().to_string().dimmed());

    // 打开编辑器
    let status = Command::new(&editor)
        .arg(&config_path)
        .status()
        .with_context(|| format!("Failed to start editor: {editor}"))?;

    if !status.success() {
        anyhow::bail!("Editor exited abnormally");
    }

    // 验证配置文件
    match Config::load() {
        Ok(config) => {
            println!(
                "{} Configuration valid, contains {} identities, {} rules",
                "✓".green(),
                config.identities.len(),
                config.rules.len()
            );
        }
        Err(e) => {
            println!("{} Configuration format error: {}", "✗".red(), e);
            println!("Please fix the configuration file and try again");
        }
    }

    Ok(())
}
