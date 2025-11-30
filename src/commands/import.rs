use anyhow::{Context, Result};
use colored::Colorize;
use dialoguer::Confirm;
use std::fs;
use std::path::Path;

use crate::config::Config;

/// 导入配置
pub fn execute(file: &Path) -> Result<()> {
    if !file.exists() {
        anyhow::bail!("File not found: {}", file.display());
    }

    // 读取并解析导入文件
    let content =
        fs::read_to_string(file).with_context(|| format!("Could not read file: {}", file.display()))?;

    let import_config: Config = toml::from_str(&content).with_context(|| "Configuration file format error")?;

    if import_config.identities.is_empty() && import_config.rules.is_empty() {
        println!("{} No valid configuration found in file", "!".yellow());
        return Ok(());
    }

    println!(
        "Found {} identities, {} rules",
        import_config.identities.len(),
        import_config.rules.len()
    );

    // 加载现有配置
    let mut config = Config::load()?;
    let had_existing = !config.identities.is_empty() || !config.rules.is_empty();

    if had_existing {
        println!();
        println!("{}", "Import Options:".cyan());
        println!("  1. Merge (keep existing, add new)");
        println!("  2. Replace (delete existing configuration)");
        println!("  3. Cancel");

        let choice: String = dialoguer::Input::new()
            .with_prompt("Select [1/2/3]")
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
                println!("{} Import complete:", "✓".green());
                println!("  Identities: Added {added_identities}, Skipped {skipped_identities} (already exists)");
                println!("  Rules: Added {added_rules}");
            }
            "2" => {
                // 替换模式
                let confirm = Confirm::new()
                    .with_prompt("Are you sure you want to replace existing configuration? This cannot be undone")
                    .default(false)
                    .interact()?;

                if !confirm {
                    println!("Operation cancelled");
                    return Ok(());
                }

                // 备份现有配置
                let config_path = Config::config_path()?;
                let backup_path = config_path.with_extension("toml.backup");
                if config_path.exists() {
                    fs::copy(&config_path, &backup_path)?;
                    println!("{} Backed up to: {}", "→".blue(), backup_path.display());
                }

                import_config.save()?;

                println!(
                    "{} Configuration replaced: {} identities, {} rules",
                    "✓".green(),
                    import_config.identities.len(),
                    import_config.rules.len()
                );
            }
            _ => {
                println!("Operation cancelled");
                return Ok(());
            }
        }
    } else {
        // 没有现有配置，直接导入
        import_config.save()?;

        println!(
            "{} Configuration imported: {} identities, {} rules",
            "✓".green(),
            import_config.identities.len(),
            import_config.rules.len()
        );
    }

    Ok(())
}
