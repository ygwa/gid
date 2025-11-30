use anyhow::Result;
use colored::Colorize;

use crate::config::Config;
use crate::git::GitConfigManager;
use crate::rules::{MatchContext, RuleEngine};

/// 根据规则自动切换身份
pub fn execute() -> Result<()> {
    let config = Config::load()?;
    let git = GitConfigManager::new()?;

    if !git.is_in_repo() {
        anyhow::bail!("Current directory is not a Git repository");
    }

    let current_dir = std::env::current_dir()?;

    // 1. 首先检查 .gid 项目配置
    if let Ok(Some(project_config)) = crate::config::ProjectConfig::load_from_dir(&current_dir) {
        let project_identity = project_config.identity;
        if config.find_identity(&project_identity).is_some() {
            println!(
                "{} Using project config (.gid): {}",
                "→".blue(),
                format!("[{}]", project_identity).cyan()
            );
            return crate::commands::switch::execute(&project_identity, false);
        }
    }

    // 2. 检查规则匹配
    if config.rules.is_empty() {
        println!("{} No rules configured", "!".yellow());
        println!();
        println!("Use {} to add rules", "gid rule add".cyan());
        println!("Or create a .gid file in the project root to specify identity");
        return Ok(());
    }

    let mut context = MatchContext::new().with_path(current_dir);

    if let Some(remote) = git.get_origin_url() {
        context = context.with_remote(remote);
    }

    let engine = RuleEngine::new(&config.rules);

    if let Some(matched_rule) = engine.match_context(&context) {
        println!(
            "{} Matched rule: {} -> {}",
            "→".blue(),
            matched_rule.pattern().dimmed(),
            format!("[{}]", matched_rule.identity).cyan()
        );
        return crate::commands::switch::execute(&matched_rule.identity, false);
    }

    // 3. 没有匹配的规则
    println!("{} No matching rules", "!".yellow());

    // 显示当前身份
    let current_name = git.get_effective_user_name();
    let current_email = git.get_effective_user_email();

    if let (Some(name), Some(email)) = (current_name, current_email) {
        println!("  Current identity: {name} <{email}>");
    }

    Ok(())
}
