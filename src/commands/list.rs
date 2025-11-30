use anyhow::Result;
use colored::Colorize;

use crate::config::Config;

/// 列出所有身份
pub fn execute() -> Result<()> {
    let config = Config::load()?;

    if config.identities.is_empty() {
        println!("{} No identities configured", "!".yellow());
        println!();
        println!("Run {} to add a new identity", "gid add".cyan());
        return Ok(());
    }

    println!("{}", "Configured Identities:".bold());
    println!();

    for identity in &config.identities {
        // 身份 ID 和基本信息
        println!(
            "  {} {} <{}>",
            format!("[{}]", identity.id).green().bold(),
            identity.name,
            identity.email.cyan()
        );

        // 描述
        if let Some(ref desc) = identity.description {
            println!("       {}", desc.dimmed());
        }

        // 附加信息
        let mut extras = Vec::new();
        if identity.ssh_key.is_some() {
            extras.push("SSH".to_string());
        }
        if identity.gpg_key.is_some() {
            extras.push("GPG".to_string());
        }
        if !extras.is_empty() {
            println!("       {}", format!("[{}]", extras.join(", ")).dimmed());
        }

        println!();
    }

    println!("Total {} identities", config.identities.len().to_string().bold());

    Ok(())
}
