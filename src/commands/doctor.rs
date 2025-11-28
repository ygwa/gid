use anyhow::Result;
use colored::Colorize;

use crate::config::Config;
use crate::git::GitConfigManager;
use crate::rules::{load_project_config, MatchContext, RuleEngine};

/// 检查当前目录的身份配置
pub fn execute(fix: bool) -> Result<()> {
    let config = Config::load()?;
    let git = GitConfigManager::new()?;

    println!("{}", "检查 Git 身份配置...".bold());
    println!();

    let mut issues = Vec::new();
    let mut suggestions = Vec::new();

    // 1. 检查是否在 Git 仓库中
    if !git.is_in_repo() {
        println!("{} 当前目录不是 Git 仓库", "!".yellow());
        return Ok(());
    }

    let current_dir = std::env::current_dir()?;

    // 2. 获取当前配置
    let current_name = git.get_effective_user_name();
    let current_email = git.get_effective_user_email();

    println!("当前身份:");
    if let (Some(ref name), Some(ref email)) = (&current_name, &current_email) {
        println!("  {} <{}>", name, email.cyan());

        // 检查是否是已知身份
        let known = config
            .identities
            .iter()
            .find(|i| &i.name == name && &i.email == email);

        if let Some(identity) = known {
            println!(
                "  {} {}",
                "身份:".green(),
                format!("[{}]", identity.id).green()
            );
        } else {
            issues.push("当前身份不在配置列表中".to_string());
        }
    } else {
        issues.push("没有配置 Git 用户信息".to_string());
    }

    println!();

    // 3. 检查 .gid 项目配置
    if let Ok(Some(project_identity)) = load_project_config(&current_dir) {
        println!("项目配置 (.gid):");
        println!("  期望身份: {}", format!("[{project_identity}]").cyan());

        if let Some(identity) = config.find_identity(&project_identity) {
            // 检查是否匹配
            if current_name.as_ref() != Some(&identity.name)
                || current_email.as_ref() != Some(&identity.email)
            {
                issues.push(format!(
                    "当前身份与项目配置不匹配 (期望: [{project_identity}])"
                ));
                suggestions.push(format!("gid switch {project_identity}"));
            } else {
                println!("  {} 身份匹配", "✓".green());
            }
        } else {
            issues.push(format!("项目配置的身份 '{project_identity}' 不存在"));
        }
        println!();
    }

    // 4. 检查规则匹配
    if !config.rules.is_empty() {
        let mut context = MatchContext::new().with_path(current_dir.clone());

        if let Some(remote) = git.get_origin_url() {
            context = context.with_remote(remote.clone());
            println!("Remote URL:");
            println!("  {}", remote.dimmed());
        }

        let engine = RuleEngine::new(&config.rules);

        if let Some(matched_rule) = engine.match_context(&context) {
            println!();
            println!("匹配的规则:");
            println!(
                "  {} -> {}",
                matched_rule.pattern().cyan(),
                format!("[{}]", matched_rule.identity).green()
            );

            if let Some(identity) = config.find_identity(&matched_rule.identity) {
                if current_name.as_ref() != Some(&identity.name)
                    || current_email.as_ref() != Some(&identity.email)
                {
                    issues.push(format!(
                        "当前身份与规则不匹配 (期望: [{}])",
                        matched_rule.identity
                    ));
                    suggestions.push(format!("gid switch {}", matched_rule.identity));
                } else {
                    println!("  {} 身份匹配", "✓".green());
                }
            }
        }
    }

    // 5. 检查 SSH 配置
    if let Some(ref email) = current_email {
        let identity = config.identities.iter().find(|i| &i.email == email);
        if let Some(identity) = identity {
            if let Some(ref ssh_key) = identity.ssh_key {
                let ssh = crate::ssh::SshManager::new()?;
                if !ssh.key_exists(ssh_key) {
                    issues.push(format!("SSH 密钥文件不存在: {}", ssh_key.display()));
                }
            }
        }
    }

    // 6. 输出结果
    println!();

    if issues.is_empty() {
        println!("{} 没有发现问题", "✓".green().bold());
    } else {
        println!("{} 发现 {} 个问题:", "⚠".yellow().bold(), issues.len());
        for issue in &issues {
            println!("  {} {}", "•".red(), issue);
        }

        if !suggestions.is_empty() && fix {
            println!();
            println!("正在修复...");

            // 执行第一个建议
            if let Some(suggestion) = suggestions.first() {
                if suggestion.starts_with("gid switch ") {
                    let identity_id = suggestion.trim_start_matches("gid switch ");
                    crate::commands::switch::execute(identity_id, false)?;
                }
            }
        } else if !suggestions.is_empty() {
            println!();
            println!("建议操作:");
            for suggestion in &suggestions {
                println!("  {} {}", "→".blue(), suggestion.cyan());
            }
            println!();
            println!("使用 {} 自动修复", "gid doctor --fix".cyan());
        }
    }

    Ok(())
}
