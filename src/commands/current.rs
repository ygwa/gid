use anyhow::Result;
use colored::Colorize;

use crate::config::Config;
use crate::git::GitConfigManager;

/// 显示当前身份
pub fn execute() -> Result<()> {
    let config = Config::load()?;
    let git = GitConfigManager::new()?;

    println!("{}", "当前 Git 身份:".bold());
    println!();

    // 项目级配置
    let local_name = git.get_user_name(false);
    let local_email = git.get_user_email(false);

    if local_name.is_some() || local_email.is_some() {
        println!(
            "  {} {} <{}>",
            "项目级:".green(),
            local_name.as_deref().unwrap_or("未设置"),
            local_email.as_deref().unwrap_or("未设置").cyan()
        );
    } else {
        println!("  {} {}", "项目级:".dimmed(), "未设置".dimmed());
    }

    // 全局配置
    let global_name = git.get_user_name(true);
    let global_email = git.get_user_email(true);

    if global_name.is_some() || global_email.is_some() {
        println!(
            "  {} {} <{}>",
            "全局级:".green(),
            global_name.as_deref().unwrap_or("未设置"),
            global_email.as_deref().unwrap_or("未设置").cyan()
        );
    } else {
        println!("  {} {}", "全局级:".dimmed(), "未设置".dimmed());
    }

    println!();

    // 实际使用的配置
    let effective_name = git.get_effective_user_name();
    let effective_email = git.get_effective_user_email();

    if let (Some(name), Some(email)) = (&effective_name, &effective_email) {
        println!("  {} {} <{}>", "实际使用:".bold(), name, email.cyan());

        // 尝试匹配已知身份
        let matched = config
            .identities
            .iter()
            .find(|i| &i.name == name && &i.email == email);

        if let Some(identity) = matched {
            println!(
                "  {} {}",
                "身份 ID:".green(),
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
                    "可能是:".yellow(),
                    format!("[{}]", identity.id).yellow(),
                    "(名称不匹配)".dimmed()
                );
            } else {
                println!("  {} {}", "⚠".yellow(), "未匹配到已配置的身份".yellow());
            }
        }
    } else {
        println!("{} 没有找到有效的 Git 用户配置", "⚠".yellow());
        println!();
        println!("运行 {} 添加身份", "gid add".cyan());
        println!("运行 {} 切换身份", "gid switch <id>".cyan());
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
