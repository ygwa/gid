use anyhow::Result;
use colored::Colorize;
use std::path::PathBuf;

use crate::audit::Auditor;
use crate::config::Config;

/// Audit commit history
pub fn execute(path: Option<PathBuf>, fix: bool) -> Result<()> {
    let config = Config::load()?;
    let auditor = Auditor::new(config);

    let target_path = path.unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

    println!("{}", "Auditing Git commit history...".bold());
    println!("  Target: {}", target_path.display().to_string().cyan());
    println!();

    // Check if single repo or directory
    let results = if target_path.join(".git").exists() {
        vec![auditor.audit_repo(&target_path)?]
    } else {
        auditor.audit_directory(&target_path)?
    };

    if results.is_empty() {
        println!("{} No Git repository found", "!".yellow());
        return Ok(());
    }

    // Show results
    let mut total_issues = 0;
    for result in &results {
        result.print_report();
        total_issues += result.issues.len();
    }

    // Summary
    println!();
    println!("{}", "═".repeat(50));
    println!(
        "Audit complete: {} repositories, {} issues",
        results.len(),
        if total_issues > 0 {
            total_issues.to_string().yellow().to_string()
        } else {
            "0".green().to_string()
        }
    );

    if total_issues > 0 && fix {
        println!();
        println!(
            "{} Automatic fix does not support commit history modification yet",
            "!".yellow()
        );
        println!("  Modifying commit history requires git rebase or git filter-branch");
        println!("  Manual handling or specialized tools like git-filter-repo are recommended");
    }

    Ok(())
}
