use anyhow::Result;
use colored::Colorize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::config::Config;
use crate::git::GitConfigManager;

/// 审计结果
#[derive(Debug)]
pub struct AuditResult {
    pub repo_path: PathBuf,
    pub total_commits: usize,
    pub issues: Vec<AuditIssue>,
    pub identities_used: HashMap<String, IdentityUsage>,
}

/// 身份使用统计
#[derive(Debug, Default)]
pub struct IdentityUsage {
    pub name: String,
    pub email: String,
    pub commit_count: usize,
    pub is_known: bool,
    pub identity_id: Option<String>,
}

/// 审计问题
#[derive(Debug)]
pub struct AuditIssue {
    pub issue_type: IssueType,
    pub commit_id: String,
    pub message: String,
    pub author_name: String,
    pub author_email: String,
}

/// 问题类型
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IssueType {
    /// 使用了未知的身份
    UnknownIdentity,
    /// 身份不匹配规则
    IdentityMismatch,
    /// 混合使用多个身份
    MixedIdentities,
}

impl std::fmt::Display for IssueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IssueType::UnknownIdentity => write!(f, "未知身份"),
            IssueType::IdentityMismatch => write!(f, "身份不匹配"),
            IssueType::MixedIdentities => write!(f, "混合身份"),
        }
    }
}

/// 审计器
pub struct Auditor {
    config: Config,
}

impl Auditor {
    /// 创建新的审计器
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// 审计单个仓库
    pub fn audit_repo(&self, path: &Path) -> Result<AuditResult> {
        let git = GitConfigManager::from_path(path)?;

        if !git.is_in_repo() {
            anyhow::bail!("{} 不是 Git 仓库", path.display());
        }

        let commits = git.get_commits(1000)?;
        let mut issues = Vec::new();
        let mut identities_used: HashMap<String, IdentityUsage> = HashMap::new();

        // 检查是否应该使用特定身份
        let expected_identity = self.get_expected_identity(path, &git);

        for commit in &commits {
            let key = format!("{} <{}>", commit.author_name, commit.author_email);

            // 统计身份使用
            let usage = identities_used.entry(key.clone()).or_insert_with(|| {
                let (is_known, identity_id) =
                    self.find_matching_identity(&commit.author_name, &commit.author_email);

                IdentityUsage {
                    name: commit.author_name.clone(),
                    email: commit.author_email.clone(),
                    commit_count: 0,
                    is_known,
                    identity_id,
                }
            });
            usage.commit_count += 1;

            // 检查问题
            if !usage.is_known {
                issues.push(AuditIssue {
                    issue_type: IssueType::UnknownIdentity,
                    commit_id: commit.id.clone(),
                    message: commit.message.clone(),
                    author_name: commit.author_name.clone(),
                    author_email: commit.author_email.clone(),
                });
            } else if let Some(ref expected) = expected_identity {
                if usage.identity_id.as_ref() != Some(expected) {
                    issues.push(AuditIssue {
                        issue_type: IssueType::IdentityMismatch,
                        commit_id: commit.id.clone(),
                        message: commit.message.clone(),
                        author_name: commit.author_name.clone(),
                        author_email: commit.author_email.clone(),
                    });
                }
            }
        }

        // 检查是否混合使用了多个已知身份
        let known_identities: Vec<_> = identities_used.values().filter(|u| u.is_known).collect();

        if known_identities.len() > 1 {
            // 找出使用次数最少的身份的提交
            let min_usage = known_identities
                .iter()
                .min_by_key(|u| u.commit_count)
                .unwrap();

            for commit in &commits {
                if commit.author_name == min_usage.name && commit.author_email == min_usage.email {
                    issues.push(AuditIssue {
                        issue_type: IssueType::MixedIdentities,
                        commit_id: commit.id.clone(),
                        message: commit.message.clone(),
                        author_name: commit.author_name.clone(),
                        author_email: commit.author_email.clone(),
                    });
                }
            }
        }

        Ok(AuditResult {
            repo_path: path.to_path_buf(),
            total_commits: commits.len(),
            issues,
            identities_used,
        })
    }

    /// 审计目录下的所有仓库
    pub fn audit_directory(&self, path: &Path) -> Result<Vec<AuditResult>> {
        let mut results = Vec::new();

        // 首先检查当前目录
        if let Ok(result) = self.audit_repo(path) {
            results.push(result);
        }

        // 遍历子目录查找 Git 仓库
        for entry in WalkDir::new(path)
            .min_depth(1)
            .max_depth(3)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_name() == ".git" && entry.file_type().is_dir() {
                if let Some(parent) = entry.path().parent() {
                    if let Ok(result) = self.audit_repo(parent) {
                        results.push(result);
                    }
                }
            }
        }

        Ok(results)
    }

    /// 查找匹配的身份
    fn find_matching_identity(&self, name: &str, email: &str) -> (bool, Option<String>) {
        for identity in &self.config.identities {
            if identity.email == email && identity.name == name {
                return (true, Some(identity.id.clone()));
            }
        }

        // 只匹配邮箱
        for identity in &self.config.identities {
            if identity.email == email {
                return (true, Some(identity.id.clone()));
            }
        }

        (false, None)
    }

    /// 获取期望的身份
    fn get_expected_identity(&self, path: &Path, git: &GitConfigManager) -> Option<String> {
        // 检查 .gid 文件
        if let Ok(Some(project_config)) = crate::config::ProjectConfig::load_from_dir(path) {
            return Some(project_config.identity);
        }

        // 检查规则匹配
        let context = crate::rules::MatchContext::new().with_path(path.to_path_buf());

        let context = if let Some(remote) = git.get_origin_url() {
            context.with_remote(remote)
        } else {
            context
        };

        let engine = crate::rules::RuleEngine::new(&self.config.rules);
        engine.match_context(&context).map(|r| r.identity.clone())
    }
}

impl AuditResult {
    /// 打印审计报告
    pub fn print_report(&self) {
        println!();
        println!("{}", format!("📁 {}", self.repo_path.display()).bold());
        println!("   总提交数: {}", self.total_commits);

        // 身份使用统计
        println!();
        println!("   {}:", "身份使用统计".cyan());
        for usage in self.identities_used.values() {
            let status = if usage.is_known {
                format!("[{}]", usage.identity_id.as_deref().unwrap_or("?")).green()
            } else {
                "[未知]".yellow().to_string().into()
            };
            println!(
                "   {} {} <{}> - {} 次提交",
                status, usage.name, usage.email, usage.commit_count
            );
        }

        // 问题列表
        if self.issues.is_empty() {
            println!();
            println!("   {} 没有发现问题", "✓".green());
        } else {
            println!();
            println!("   {} 发现 {} 个问题:", "⚠".yellow(), self.issues.len());

            // 按类型分组显示
            let mut by_type: HashMap<IssueType, Vec<&AuditIssue>> = HashMap::new();
            for issue in &self.issues {
                by_type
                    .entry(issue.issue_type.clone())
                    .or_default()
                    .push(issue);
            }

            for (issue_type, issues) in by_type {
                println!();
                println!(
                    "   {} ({} 个):",
                    issue_type.to_string().yellow(),
                    issues.len()
                );
                for issue in issues.iter().take(5) {
                    println!(
                        "     {} {} - {} <{}>",
                        issue.commit_id.dimmed(),
                        issue.message.chars().take(40).collect::<String>(),
                        issue.author_name,
                        issue.author_email
                    );
                }
                if issues.len() > 5 {
                    println!("     ... 还有 {} 个", issues.len() - 5);
                }
            }
        }
    }
}
