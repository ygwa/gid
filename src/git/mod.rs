use anyhow::{Context, Result};
use git2::{Config as GitConfig, Repository};
use std::path::Path;

/// Git Configuration Manager
pub struct GitConfigManager {
    repo: Option<Repository>,
}

impl GitConfigManager {
    /// Create new configuration manager
    pub fn new() -> Result<Self> {
        let repo = Repository::discover(".").ok();
        Ok(Self { repo })
    }

    /// Create from specified path
    pub fn from_path(path: &Path) -> Result<Self> {
        let repo = Repository::discover(path).ok();
        Ok(Self { repo })
    }

    /// Check if in Git repository
    pub fn is_in_repo(&self) -> bool {
        self.repo.is_some()
    }

    /// Get repository path
    pub fn repo_path(&self) -> Option<&Path> {
        self.repo.as_ref().map(|r| r.path())
    }

    /// Set user name
    pub fn set_user_name(&self, name: &str, global: bool) -> Result<()> {
        if global {
            let mut config =
                GitConfig::open_default().context("Could not open global Git config")?;
            config
                .set_str("user.name", name)
                .context("Could not set user.name")?;
        } else {
            let repo = self
                .repo
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("Current directory is not a Git repository"))?;
            let mut config = repo.config().context("Could not open repository config")?;
            config
                .set_str("user.name", name)
                .context("Could not set user.name")?;
        }
        Ok(())
    }

    /// Set user email
    pub fn set_user_email(&self, email: &str, global: bool) -> Result<()> {
        if global {
            let mut config =
                GitConfig::open_default().context("Could not open global Git config")?;
            config
                .set_str("user.email", email)
                .context("Could not set user.email")?;
        } else {
            let repo = self
                .repo
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("Current directory is not a Git repository"))?;
            let mut config = repo.config().context("Could not open repository config")?;
            config
                .set_str("user.email", email)
                .context("Could not set user.email")?;
        }
        Ok(())
    }

    /// Set GPG signing key
    pub fn set_signing_key(&self, key: &str, global: bool) -> Result<()> {
        if global {
            let mut config =
                GitConfig::open_default().context("Could not open global Git config")?;
            config
                .set_str("user.signingkey", key)
                .context("Could not set user.signingkey")?;
        } else {
            let repo = self
                .repo
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("Current directory is not a Git repository"))?;
            let mut config = repo.config().context("Could not open repository config")?;
            config
                .set_str("user.signingkey", key)
                .context("Could not set user.signingkey")?;
        }
        Ok(())
    }

    /// Enable/Disable GPG signing
    pub fn set_gpg_sign(&self, enabled: bool, global: bool) -> Result<()> {
        if global {
            let mut config =
                GitConfig::open_default().context("Could not open global Git config")?;
            config
                .set_bool("commit.gpgsign", enabled)
                .context("Could not set commit.gpgsign")?;
        } else {
            let repo = self
                .repo
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("Current directory is not a Git repository"))?;
            let mut config = repo.config().context("Could not open repository config")?;
            config
                .set_bool("commit.gpgsign", enabled)
                .context("Could not set commit.gpgsign")?;
        }
        Ok(())
    }

    /// Get current user name
    pub fn get_user_name(&self, global: bool) -> Option<String> {
        if global {
            GitConfig::open_default()
                .ok()
                .and_then(|c| c.get_string("user.name").ok())
        } else {
            self.repo
                .as_ref()
                .and_then(|r| r.config().ok())
                .and_then(|c| c.get_string("user.name").ok())
        }
    }

    /// Get current user email
    pub fn get_user_email(&self, global: bool) -> Option<String> {
        if global {
            GitConfig::open_default()
                .ok()
                .and_then(|c| c.get_string("user.email").ok())
        } else {
            self.repo
                .as_ref()
                .and_then(|r| r.config().ok())
                .and_then(|c| c.get_string("user.email").ok())
        }
    }

    /// Get effective user name (local first, then global)
    pub fn get_effective_user_name(&self) -> Option<String> {
        self.get_user_name(false)
            .or_else(|| self.get_user_name(true))
    }

    /// Get effective user email (local first, then global)
    pub fn get_effective_user_email(&self) -> Option<String> {
        self.get_user_email(false)
            .or_else(|| self.get_user_email(true))
    }

    /// Get origin remote URL
    pub fn get_origin_url(&self) -> Option<String> {
        let repo = self.repo.as_ref()?;
        let remote = repo.find_remote("origin").ok()?;
        remote.url().map(|s| s.to_string())
    }

    /// Get commit history
    pub fn get_commits(&self, max_count: usize) -> Result<Vec<CommitInfo>> {
        let repo = self
            .repo
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Current directory is not a Git repository"))?;

        let mut revwalk = repo.revwalk()?;
        revwalk.push_head()?;

        let mut commits = Vec::new();
        for (i, oid) in revwalk.enumerate() {
            if i >= max_count {
                break;
            }

            let oid = oid?;
            let commit = repo.find_commit(oid)?;
            let author = commit.author();

            commits.push(CommitInfo {
                id: oid.to_string()[..7].to_string(),
                message: commit
                    .message()
                    .unwrap_or("")
                    .lines()
                    .next()
                    .unwrap_or("")
                    .to_string(),
                author_name: author.name().unwrap_or("").to_string(),
                author_email: author.email().unwrap_or("").to_string(),
            });
        }

        Ok(commits)
    }
}

/// Commit Information
#[derive(Debug)]
pub struct CommitInfo {
    pub id: String,
    pub message: String,
    pub author_name: String,
    pub author_email: String,
}
