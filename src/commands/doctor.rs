use anyhow::Result;
use colored::Colorize;

use crate::config::Config;
use crate::git::GitConfigManager;
use crate::rules::{MatchContext, RuleEngine};

/// Check identity configuration in current directory
pub fn execute(fix: bool) -> Result<()> {
    let config = Config::load()?;
    let git = GitConfigManager::new()?;

    println!("{}", "Checking Git identity configuration...".bold());
    println!();

    let mut issues = Vec::new();
    let mut suggestions = Vec::new();

    // 1. Check if in Git repository
    if !git.is_in_repo() {
        println!("{} Current directory is not a Git repository", "!".yellow());
        return Ok(());
    }

    let current_dir = std::env::current_dir()?;

    // 2. Get current configuration
    let current_name = git.get_effective_user_name();
    let current_email = git.get_effective_user_email();

    println!("Current Identity:");
    if let (Some(ref name), Some(ref email)) = (&current_name, &current_email) {
        println!("  {} <{}>", name, email.cyan());

        // Check if it is a known identity
        let known = config
            .identities
            .iter()
            .find(|i| &i.name == name && &i.email == email);

        if let Some(identity) = known {
            println!(
                "  {} {}",
                "Identity:".green(),
                format!("[{}]", identity.id).green()
            );
        } else {
            issues.push("Current identity is not in the configuration list".to_string());
        }
    } else {
        issues.push("Git user information not configured".to_string());
    }

    println!();

    // 3. Check .gid project config
    if let Ok(Some(project_config)) = crate::config::ProjectConfig::load_from_dir(&current_dir) {
        let project_identity = project_config.identity;
        println!("Project Config (.gid):");
        println!(
            "  Expected Identity: {}",
            format!("[{project_identity}]").cyan()
        );

        if let Some(identity) = config.find_identity(&project_identity) {
            // Check if matches
            if current_name.as_ref() != Some(&identity.name)
                || current_email.as_ref() != Some(&identity.email)
            {
                issues.push(format!(
                    "Current identity does not match project config (expected: [{project_identity}])"
                ));
                suggestions.push(format!("gid switch {project_identity}"));
            } else {
                println!("  {} Identity matches", "✓".green());
            }
        } else {
            issues.push(format!(
                "Project configured identity '{project_identity}' does not exist"
            ));
        }
        println!();
    }

    // 4. Check rule matching
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
            println!("Matched Rules:");
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
                        "Current identity does not match rule (expected: [{}])",
                        matched_rule.identity
                    ));
                    suggestions.push(format!("gid switch {}", matched_rule.identity));
                } else {
                    println!("  {} Identity matches", "✓".green());
                }
            }
        }
    }

    // 5. Check SSH configuration
    if let Some(ref email) = current_email {
        let identity = config.identities.iter().find(|i| &i.email == email);
        if let Some(identity) = identity {
            if let Some(ref ssh_key) = identity.ssh_key {
                let ssh = crate::ssh::SshManager::new()?;
                if !ssh.key_exists(ssh_key) {
                    issues.push(format!(
                        "SSH key file does not exist: {}",
                        ssh_key.display()
                    ));
                }
            }
        }
    }

    // 6. Output results
    println!();

    if issues.is_empty() {
        println!("{} No issues found", "✓".green().bold());
    } else {
        println!("{} Found {} issues:", "⚠".yellow().bold(), issues.len());
        for issue in &issues {
            println!("  {} {}", "•".red(), issue);
        }

        if !suggestions.is_empty() && fix {
            println!();
            println!("Fixing...");

            // Execute first suggestion
            if let Some(suggestion) = suggestions.first() {
                if suggestion.starts_with("gid switch ") {
                    let identity_id = suggestion.trim_start_matches("gid switch ");
                    crate::commands::switch::execute(identity_id, false)?;
                }
            }
        } else if !suggestions.is_empty() {
            println!();
            println!("Suggested actions:");
            for suggestion in &suggestions {
                println!("  {} {}", "→".blue(), suggestion.cyan());
            }
            println!();
            println!("Use {} to fix automatically", "gid doctor --fix".cyan());
        }
    }

    Ok(())
}
