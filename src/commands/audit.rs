use anyhow::Result;
use colored::Colorize;
use std::path::PathBuf;

use crate::audit::Auditor;
use crate::config::Config;

/// 审计提交历史
pub fn execute(path: Option<PathBuf>, fix: bool) -> Result<()> {
    let config = Config::load()?;
    let auditor = Auditor::new(config);

    let target_path = path.unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

    println!("{}", "审计 Git 提交历史...".bold());
    println!("  目标: {}", target_path.display().to_string().cyan());
    println!();

    // 检查是单个仓库还是目录
    let results = if target_path.join(".git").exists() {
        vec![auditor.audit_repo(&target_path)?]
    } else {
        auditor.audit_directory(&target_path)?
    };

    if results.is_empty() {
        println!("{} 没有找到 Git 仓库", "!".yellow());
        return Ok(());
    }

    // 显示结果
    let mut total_issues = 0;
    for result in &results {
        result.print_report();
        total_issues += result.issues.len();
    }

    // 汇总
    println!();
    println!("{}", "═".repeat(50));
    println!(
        "审计完成: {} 个仓库, {} 个问题",
        results.len(),
        if total_issues > 0 {
            total_issues.to_string().yellow().to_string()
        } else {
            "0".green().to_string()
        }
    );

    if total_issues > 0 && fix {
        println!();
        println!("{} 自动修复暂不支持提交历史修改", "!".yellow());
        println!("  提交历史的修改需要使用 git rebase 或 git filter-branch");
        println!("  建议手动处理或使用专门的工具如 git-filter-repo");
    }

    Ok(())
}
