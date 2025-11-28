use anyhow::Result;
use colored::Colorize;
use std::path::PathBuf;

use crate::cli::{RuleAction, RuleType as CliRuleType};
use crate::config::Config;
use crate::git::GitConfigManager;
use crate::rules::{MatchContext, Rule, RuleEngine, RuleType};

/// 执行规则命令
pub fn execute(action: RuleAction) -> Result<()> {
    match action {
        RuleAction::Add {
            rule_type,
            pattern,
            identity,
            priority,
        } => add_rule(rule_type, pattern, identity, priority),
        RuleAction::List => list_rules(),
        RuleAction::Remove { index } => remove_rule(index),
        RuleAction::Test { path, remote } => test_rule(path, remote),
    }
}

/// 添加规则
fn add_rule(
    rule_type: CliRuleType,
    pattern: String,
    identity: String,
    priority: u32,
) -> Result<()> {
    let mut config = Config::load()?;

    // 验证身份存在
    if config.find_identity(&identity).is_none() {
        anyhow::bail!("身份 '{identity}' 不存在");
    }

    // 创建规则
    let rule = match rule_type {
        CliRuleType::Path => Rule::path(pattern.clone(), identity.clone()),
        CliRuleType::Remote => Rule::remote(pattern.clone(), identity.clone()),
    }
    .with_priority(priority);

    config.add_rule(rule);
    config.save()?;

    let type_name = match rule_type {
        CliRuleType::Path => "路径",
        CliRuleType::Remote => "Remote URL",
    };

    println!(
        "{} 已添加{}规则: {} -> {}",
        "✓".green(),
        type_name,
        pattern.cyan(),
        format!("[{identity}]").green()
    );

    Ok(())
}

/// 列出所有规则
fn list_rules() -> Result<()> {
    let config = Config::load()?;

    if config.rules.is_empty() {
        println!("{} 没有配置任何规则", "!".yellow());
        println!();
        println!("使用 {} 添加规则", "gid rule add".cyan());
        println!();
        println!("示例:");
        println!(
            "  {} 添加路径规则",
            "gid rule add -t path -p '~/work/**' -i work".dimmed()
        );
        println!(
            "  {} 添加 remote 规则",
            "gid rule add -t remote -p 'github.com/company/*' -i work".dimmed()
        );
        return Ok(());
    }

    println!("{}", "已配置的规则:".bold());
    println!();

    for (i, rule) in config.rules.iter().enumerate() {
        let type_badge = match &rule.rule_type {
            RuleType::Path { .. } => "[路径]".cyan(),
            RuleType::Remote { .. } => "[Remote]".magenta(),
        };

        let status = if rule.enabled {
            "✓".green()
        } else {
            "○".dimmed()
        };

        println!(
            "  {} {} {} {} -> {}",
            format!("{i}.").dimmed(),
            status,
            type_badge,
            rule.pattern(),
            format!("[{}]", rule.identity).green()
        );

        if let Some(ref desc) = rule.description {
            println!("       {}", desc.dimmed());
        }

        println!("       优先级: {}", rule.priority.to_string().dimmed());
    }

    println!();
    println!("共 {} 条规则", config.rules.len());

    Ok(())
}

/// 删除规则
fn remove_rule(index: usize) -> Result<()> {
    let mut config = Config::load()?;

    if index >= config.rules.len() {
        anyhow::bail!(
            "规则索引 {} 超出范围 (共 {} 条规则)",
            index,
            config.rules.len()
        );
    }

    let rule = &config.rules[index];
    println!(
        "将要删除规则: {} -> {}",
        rule.pattern().yellow(),
        rule.identity
    );

    let confirm = dialoguer::Confirm::new()
        .with_prompt("确定要删除吗?")
        .default(false)
        .interact()?;

    if !confirm {
        println!("操作已取消");
        return Ok(());
    }

    config.remove_rule(index)?;
    config.save()?;

    println!("{} 规则已删除", "✓".green());

    Ok(())
}

/// 测试规则匹配
fn test_rule(path: Option<PathBuf>, remote: Option<String>) -> Result<()> {
    let config = Config::load()?;

    if config.rules.is_empty() {
        println!("{} 没有配置任何规则", "!".yellow());
        return Ok(());
    }

    // 构建匹配上下文
    let mut context = MatchContext::new();

    // 路径
    let test_path = path.unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
    context = context.with_path(test_path.clone());

    // Remote URL
    let test_remote = if let Some(remote) = remote {
        Some(remote)
    } else {
        let git = GitConfigManager::new()?;
        git.get_origin_url()
    };

    if let Some(ref remote) = test_remote {
        context = context.with_remote(remote.clone());
    }

    println!("{}", "测试规则匹配:".bold());
    println!();
    println!("  路径: {}", test_path.display().to_string().cyan());
    if let Some(ref remote) = test_remote {
        println!("  Remote: {}", remote.cyan());
    }
    println!();

    let engine = RuleEngine::new(&config.rules);

    // 显示所有匹配的规则
    let matched_rules = engine.match_all(&context);

    if matched_rules.is_empty() {
        println!("{} 没有匹配的规则", "!".yellow());
    } else {
        println!("匹配的规则:");
        for (i, rule) in matched_rules.iter().enumerate() {
            let marker = if i == 0 { "→".green() } else { " ".into() };
            println!(
                "  {} [{}] {} -> {} (优先级: {})",
                marker,
                rule.type_name(),
                rule.pattern(),
                format!("[{}]", rule.identity).green(),
                rule.priority
            );
        }

        println!();
        if let Some(first) = matched_rules.first() {
            if let Some(identity) = config.find_identity(&first.identity) {
                println!(
                    "{} 将使用身份: {} {} <{}>",
                    "✓".green(),
                    format!("[{}]", identity.id).green().bold(),
                    identity.name,
                    identity.email.cyan()
                );
            }
        }
    }

    Ok(())
}
