use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;
use std::path::PathBuf;

use crate::config::Config;

/// 导出配置
pub fn execute(file: PathBuf) -> Result<()> {
    let config = Config::load()?;

    if config.identities.is_empty() && config.rules.is_empty() {
        println!("{} 没有配置可导出", "!".yellow());
        return Ok(());
    }

    let content = toml::to_string_pretty(&config).context("无法序列化配置")?;

    fs::write(&file, content).with_context(|| format!("无法写入文件: {}", file.display()))?;

    println!("{} 配置已导出到: {}", "✓".green(), file.display());
    println!(
        "  {} 个身份, {} 条规则",
        config.identities.len(),
        config.rules.len()
    );

    Ok(())
}
