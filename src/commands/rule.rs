use anyhow::Result;
use colored::Colorize;
use std::path::PathBuf;

use crate::cli::{RuleAction, RuleType as CliRuleType};
use crate::config::Config;
use crate::git::GitConfigManager;
use crate::rules::{MatchContext, Rule, RuleEngine, RuleType};

/// Execute rule command
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

/// Add rule
fn add_rule(
    rule_type: CliRuleType,
    pattern: String,
    identity: String,
    priority: u32,
) -> Result<()> {
    let mut config = Config::load()?;

    // Verify identity exists
    if config.find_identity(&identity).is_none() {
        anyhow::bail!("Identity '{identity}' does not exist");
    }

    // Create rule
    let rule = match rule_type {
        CliRuleType::Path => Rule::path(pattern.clone(), identity.clone()),
        CliRuleType::Remote => Rule::remote(pattern.clone(), identity.clone()),
    }
    .with_priority(priority);

    config.add_rule(rule);
    config.save()?;

    let type_name = match rule_type {
        CliRuleType::Path => "Path",
        CliRuleType::Remote => "Remote URL",
    };

    println!(
        "{} Added {} rule: {} -> {}",
        "✓".green(),
        type_name,
        pattern.cyan(),
        format!("[{identity}]").green()
    );

    Ok(())
}

/// List all rules
fn list_rules() -> Result<()> {
    let config = Config::load()?;

    if config.rules.is_empty() {
        println!("{} No rules configured", "!".yellow());
        println!();
        println!("Use {} to add rules", "gid rule add".cyan());
        println!();
        println!("Example:");
        println!(
            "  {} Add path rule",
            "gid rule add -t path -p '~/work/**' -i work".dimmed()
        );
        println!(
            "  {} Add remote rule",
            "gid rule add -t remote -p 'github.com/company/*' -i work".dimmed()
        );
        return Ok(());
    }

    println!("{}", "Configured Rules:".bold());
    println!();

    for (i, rule) in config.rules.iter().enumerate() {
        let type_badge = match &rule.rule_type {
            RuleType::Path { .. } => "[Path]".cyan(),
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

        println!("       Priority: {}", rule.priority.to_string().dimmed());
    }

    println!();
    println!("Total {} rules", config.rules.len());

    Ok(())
}

/// Remove rule
fn remove_rule(index: usize) -> Result<()> {
    let mut config = Config::load()?;

    if index >= config.rules.len() {
        anyhow::bail!(
            "Rule index {} out of range (total {} rules)",
            index,
            config.rules.len()
        );
    }

    let rule = &config.rules[index];
    println!(
        "About to remove rule: {} -> {}",
        rule.pattern().yellow(),
        rule.identity
    );

    let confirm = dialoguer::Confirm::new()
        .with_prompt("Are you sure you want to remove?")
        .default(false)
        .interact()?;

    if !confirm {
        println!("Operation cancelled");
        return Ok(());
    }

    config.remove_rule(index)?;
    config.save()?;

    println!("{} Rule removed", "✓".green());

    Ok(())
}

/// Test rule matching
fn test_rule(path: Option<PathBuf>, remote: Option<String>) -> Result<()> {
    let config = Config::load()?;

    if config.rules.is_empty() {
        println!("{} No rules configured", "!".yellow());
        return Ok(());
    }

    // Build match context
    let mut context = MatchContext::new();

    // Path
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

    println!("{}", "Test Rule Matching:".bold());
    println!();
    println!("  Path: {}", test_path.display().to_string().cyan());
    if let Some(ref remote) = test_remote {
        println!("  Remote: {}", remote.cyan());
    }
    println!();

    let engine = RuleEngine::new(&config.rules);

    // Show all matched rules
    let matched_rules = engine.match_all(&context);

    if matched_rules.is_empty() {
        println!("{} No matching rules", "!".yellow());
    } else {
        println!("Matched Rules:");
        for (i, rule) in matched_rules.iter().enumerate() {
            let marker = if i == 0 { "→".green() } else { " ".into() };
            println!(
                "  {} [{}] {} -> {} (Priority: {})",
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
                    "{} Will use identity: {} {} <{}>",
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
