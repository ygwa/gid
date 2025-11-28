use anyhow::{Context, Result};
use colored::Colorize;
use dialoguer::Confirm;
use std::fs;
use std::path::Path;

use crate::config::Config;

/// 导入配置
pub fn execute(file: &Path) -> Result<()> {
    if !file.exists() {
        anyhow::bail!("文件不存在: {}", file.display());
    }
    
    // 读取并解析导入文件
    let content = fs::read_to_string(file)
        .with_context(|| format!("无法读取文件: {}", file.display()))?;
    
    let import_config: Config = toml::from_str(&content)
        .with_context(|| "配置文件格式错误")?;
    
    if import_config.identities.is_empty() && import_config.rules.is_empty() {
        println!("{} 文件中没有找到有效的配置", "!".yellow());
        return Ok(());
    }
    
    println!(
        "发现 {} 个身份, {} 条规则",
        import_config.identities.len(),
        import_config.rules.len()
    );
    
    // 加载现有配置
    let mut config = Config::load()?;
    let had_existing = !config.identities.is_empty() || !config.rules.is_empty();
    
    if had_existing {
        println!();
        println!("{}", "导入选项:".cyan());
        println!("  1. 合并（保留现有，添加新的）");
        println!("  2. 替换（删除现有配置）");
        println!("  3. 取消");
        
        let choice: String = dialoguer::Input::new()
            .with_prompt("选择 [1/2/3]")
            .default("1".to_string())
            .interact_text()?;
        
        match choice.trim() {
            "1" => {
                // 合并模式
                let mut added_identities = 0;
                let mut skipped_identities = 0;
                
                for identity in import_config.identities {
                    if config.find_identity(&identity.id).is_none() {
                        config.identities.push(identity);
                        added_identities += 1;
                    } else {
                        skipped_identities += 1;
                    }
                }
                
                let added_rules = import_config.rules.len();
                for rule in import_config.rules {
                    config.add_rule(rule);
                }
                
                config.save()?;
                
                println!();
                println!("{} 导入完成:", "✓".green());
                println!("  身份: 添加 {}, 跳过 {} (已存在)", added_identities, skipped_identities);
                println!("  规则: 添加 {}", added_rules);
            }
            "2" => {
                // 替换模式
                let confirm = Confirm::new()
                    .with_prompt("确定要替换现有配置吗? 此操作不可撤销")
                    .default(false)
                    .interact()?;
                
                if !confirm {
                    println!("操作已取消");
                    return Ok(());
                }
                
                // 备份现有配置
                let config_path = Config::config_path()?;
                let backup_path = config_path.with_extension("toml.backup");
                if config_path.exists() {
                    fs::copy(&config_path, &backup_path)?;
                    println!("{} 已备份到: {}", "→".blue(), backup_path.display());
                }
                
                import_config.save()?;
                
                println!(
                    "{} 配置已替换: {} 个身份, {} 条规则",
                    "✓".green(),
                    import_config.identities.len(),
                    import_config.rules.len()
                );
            }
            _ => {
                println!("操作已取消");
                return Ok(());
            }
        }
    } else {
        // 没有现有配置，直接导入
        import_config.save()?;
        
        println!(
            "{} 配置已导入: {} 个身份, {} 条规则",
            "✓".green(),
            import_config.identities.len(),
            import_config.rules.len()
        );
    }
    
    Ok(())
}

