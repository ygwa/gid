use anyhow::Result;
use colored::Colorize;

use crate::config::Config;
use crate::git::GitConfigManager;
use crate::rules::{load_project_config, MatchContext, RuleEngine};

/// 根据规则自动切换身份
pub fn execute() -> Result<()> {
    let config = Config::load()?;
    let git = GitConfigManager::new()?;
    
    if !git.is_in_repo() {
        anyhow::bail!("当前目录不是 Git 仓库");
    }
    
    let current_dir = std::env::current_dir()?;
    
    // 1. 首先检查 .gid 项目配置
    if let Ok(Some(project_identity)) = load_project_config(&current_dir) {
        if config.find_identity(&project_identity).is_some() {
            println!(
                "{} 使用项目配置 (.gid): {}",
                "→".blue(),
                format!("[{}]", project_identity).cyan()
            );
            return crate::commands::switch::execute(&project_identity, false);
        }
    }
    
    // 2. 检查规则匹配
    if config.rules.is_empty() {
        println!("{} 没有配置规则", "!".yellow());
        println!();
        println!("使用 {} 添加规则", "gid rule add".cyan());
        println!("或在项目根目录创建 .gid 文件指定身份");
        return Ok(());
    }
    
    let mut context = MatchContext::new().with_path(current_dir);
    
    if let Some(remote) = git.get_origin_url() {
        context = context.with_remote(remote);
    }
    
    let engine = RuleEngine::new(&config.rules);
    
    if let Some(matched_rule) = engine.match_context(&context) {
        println!(
            "{} 匹配规则: {} -> {}",
            "→".blue(),
            matched_rule.pattern().dimmed(),
            format!("[{}]", matched_rule.identity).cyan()
        );
        return crate::commands::switch::execute(&matched_rule.identity, false);
    }
    
    // 3. 没有匹配的规则
    println!("{} 没有匹配的规则", "!".yellow());
    
    // 显示当前身份
    let current_name = git.get_effective_user_name();
    let current_email = git.get_effective_user_email();
    
    if let (Some(name), Some(email)) = (current_name, current_email) {
        println!("  当前身份: {} <{}>", name, email);
    }
    
    Ok(())
}

