use anyhow::Result;
use colored::Colorize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::config::Config;
use crate::git::GitConfigManager;

/// Audit Result
#[derive(Debug)]
pub struct AuditResult {
    pub repo_path: PathBuf,
    pub total_commits: usize,
    pub issues: Vec<AuditIssue>,
    pub identities_used: HashMap<String, IdentityUsage>,
}

/// Identity Usage Statistics
#[derive(Debug, Default)]
pub struct IdentityUsage {
    pub name: String,
    pub email: String,
    pub commit_count: usize,
    pub is_known: bool,
    pub identity_id: Option<String>,
}

/// Audit Issue
#[derive(Debug)]
pub struct AuditIssue {
    pub issue_type: IssueType,
    pub commit_id: String,
    pub message: String,
    pub author_name: String,
    pub author_email: String,
}

/// Issue Type
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IssueType {
    /// Unknown identity used
    UnknownIdentity,
    /// Identity mismatch with rules
    IdentityMismatch,
    /// Mixed identities used
    MixedIdentities,
}

impl std::fmt::Display for IssueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IssueType::UnknownIdentity => write!(f, "Unknown Identity"),
            IssueType::IdentityMismatch => write!(f, "Identity Mismatch"),
            IssueType::MixedIdentities => write!(f, "Mixed Identities"),
        }
    }
}

/// Auditor
pub struct Auditor {
    config: Config,
}

impl Auditor {
    /// Create new auditor
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Audit single repository
    pub fn audit_repo(&self, path: &Path) -> Result<AuditResult> {
        let git = GitConfigManager::from_path(path)?;

        if !git.is_in_repo() {
            anyhow::bail!("{} is not a Git repository", path.display());
        }

        let commits = git.get_commits(1000)?;
        let mut issues = Vec::new();
        let mut identities_used: HashMap<String, IdentityUsage> = HashMap::new();

        // Check if specific identity should be used
        let expected_identity = self.get_expected_identity(path, &git);

        for commit in &commits {
            let key = format!("{} <{}>", commit.author_name, commit.author_email);

            // Track identity usage
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

            // Check for issues
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

        // Check for mixed usage of multiple known identities
        let known_identities: Vec<_> = identities_used.values().filter(|u| u.is_known).collect();

        if known_identities.len() > 1 {
            // Find commits with least used identity
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

    /// Audit all repositories in directory
    pub fn audit_directory(&self, path: &Path) -> Result<Vec<AuditResult>> {
        let mut results = Vec::new();

        // Check current directory first
        if let Ok(result) = self.audit_repo(path) {
            results.push(result);
        }

        // Walk subdirectories to find Git repositories
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

    /// Find matching identity
    fn find_matching_identity(&self, name: &str, email: &str) -> (bool, Option<String>) {
        for identity in &self.config.identities {
            if identity.email == email && identity.name == name {
                return (true, Some(identity.id.clone()));
            }
        }

        // Match email only
        for identity in &self.config.identities {
            if identity.email == email {
                return (true, Some(identity.id.clone()));
            }
        }

        (false, None)
    }

    /// Get expected identity
    fn get_expected_identity(&self, path: &Path, git: &GitConfigManager) -> Option<String> {
        // Check .gid file
        if let Ok(Some(project_config)) = crate::config::ProjectConfig::load_from_dir(path) {
            return Some(project_config.identity);
        }

        // Check rule matching
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
    /// Print audit report
    pub fn print_report(&self) {
        println!();
        println!("{}", format!("📁 {}", self.repo_path.display()).bold());
        println!("   Total Commits: {}", self.total_commits);

        // Identity usage statistics
        println!();
        println!("   {}:", "Identity Usage Statistics".cyan());
        for usage in self.identities_used.values() {
            let status = if usage.is_known {
                format!("[{}]", usage.identity_id.as_deref().unwrap_or("?")).green()
            } else {
                "[Unknown]".yellow().to_string().into()
            };
            println!(
                "   {} {} <{}> - {} commits",
                status, usage.name, usage.email, usage.commit_count
            );
        }

        // Issue list
        if self.issues.is_empty() {
            println!();
            println!("   {} No issues found", "✓".green());
        } else {
            println!();
            println!("   {} Found {} issues:", "⚠".yellow(), self.issues.len());

            // Group by type
            let mut by_type: HashMap<IssueType, Vec<&AuditIssue>> = HashMap::new();
            for issue in &self.issues {
                by_type
                    .entry(issue.issue_type.clone())
                    .or_default()
                    .push(issue);
            }

            for (issue_type, issues) in by_type {
                println!();
                println!("   {} ({}):", issue_type.to_string().yellow(), issues.len());
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
                    println!("     ... and {} more", issues.len() - 5);
                }
            }
        }
    }
}
