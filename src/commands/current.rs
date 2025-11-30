use anyhow::Result;
use colored::Colorize;

use crate::config::Config;
use crate::git::GitConfigManager;

/// 显示当前身份
pub fn execute() -> Result<()> {
    let config = Config::load()?;
    let git = GitConfigManager::new()?;

    println!("{}", "Current Git Identity:".bold());
    println!();

    // 项目级配置
    let local_name = git.get_user_name(false);
    let local_email = git.get_user_email(false);

    if local_name.is_some() || local_email.is_some() {
        println!(
            "  {} {} <{}>",
            "Project:".green(),
            local_name.as_deref().unwrap_or("Not set"),
            local_email.as_deref().unwrap_or("Not set").cyan()
        );
    } else {
        println!("  {} {}", "Project:".dimmed(), "Not set".dimmed());
    }

    // 全局配置
    let global_name = git.get_user_name(true);
    let global_email = git.get_user_email(true);

    if global_name.is_some() || global_email.is_some() {
        println!(
            "  {} {} <{}>",
            "Global:".green(),
            global_name.as_deref().unwrap_or("Not set"),
            global_email.as_deref().unwrap_or("Not set").cyan()
        );
    } else {
        println!("  {} {}", "Global:".dimmed(), "Not set".dimmed());
    }

    println!();

    // 实际使用的配置
    let effective_name = git.get_effective_user_name();
    let effective_email = git.get_effective_user_email();

    if let (Some(name), Some(email)) = (&effective_name, &effective_email) {
        println!("  {} {} <{}>", "Effective:".bold(), name, email.cyan());

        // 尝试匹配已知身份
        let matched = config
            .identities
            .iter()
            .find(|i| &i.name == name && &i.email == email);

        if let Some(identity) = matched {
            println!(
                "  {} {}",
                "Identity ID:".green(),
                format!("[{}]", identity.id).green().bold()
            );
            if let Some(ref desc) = identity.description {
                println!("  {}", desc.dimmed());
            }
        } else {
            // 只匹配邮箱
            let email_matched = config.identities.iter().find(|i| &i.email == email);
            if let Some(identity) = email_matched {
                println!(
                    "  {} {} {}",
                    "Possibly:".yellow(),
                    format!("[{}]", identity.id).yellow(),
                    "(Name mismatch)".dimmed()
                );
            } else {
                println!("  {} {}", "⚠".yellow(), "No configured identity matched".yellow());
            }
        }
    } else {
        println!("{} No valid Git user configuration found", "⚠".yellow());
        println!();
        println!("Run {} to add identity", "gid add".cyan());
        println!("Run {} to switch identity", "gid switch <id>".cyan());
    }

    // 显示仓库信息
    if git.is_in_repo() {
        println!();
        if let Some(remote) = git.get_origin_url() {
            println!("  {} {}", "Remote:".dimmed(), remote.dimmed());
        }
    }

    Ok(())
}
