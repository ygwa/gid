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
        println!("{} 已创建配置文件: {}", "→".blue(), config_path.display());
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
    
    println!("{} 使用 {} 编辑配置文件...", "→".blue(), editor);
    println!("  {}", config_path.display().to_string().dimmed());
    
    // 打开编辑器
    let status = Command::new(&editor)
        .arg(&config_path)
        .status()
        .with_context(|| format!("无法启动编辑器: {}", editor))?;
    
    if !status.success() {
        anyhow::bail!("编辑器退出异常");
    }
    
    // 验证配置文件
    match Config::load() {
        Ok(config) => {
            println!(
                "{} 配置文件有效，包含 {} 个身份，{} 条规则",
                "✓".green(),
                config.identities.len(),
                config.rules.len()
            );
        }
        Err(e) => {
            println!("{} 配置文件格式错误: {}", "✗".red(), e);
            println!("请修复配置文件后重试");
        }
    }
    
    Ok(())
}

