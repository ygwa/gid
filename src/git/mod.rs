use anyhow::{Context, Result};
use git2::{Config as GitConfig, Repository};
use std::path::Path;

/// Git 配置操作
pub struct GitConfigManager {
    repo: Option<Repository>,
}

impl GitConfigManager {
    /// 创建新的配置管理器
    pub fn new() -> Result<Self> {
        let repo = Repository::discover(".").ok();
        Ok(Self { repo })
    }

    /// 从指定路径创建
    pub fn from_path(path: &Path) -> Result<Self> {
        let repo = Repository::discover(path).ok();
        Ok(Self { repo })
    }

    /// 检查是否在 Git 仓库中
    pub fn is_in_repo(&self) -> bool {
        self.repo.is_some()
    }

    /// 获取仓库路径
    pub fn repo_path(&self) -> Option<&Path> {
        self.repo.as_ref().map(|r| r.path())
    }

    /// 设置用户名
    pub fn set_user_name(&self, name: &str, global: bool) -> Result<()> {
        if global {
            let mut config = GitConfig::open_default().context("无法打开全局 Git 配置")?;
            config
                .set_str("user.name", name)
                .context("无法设置 user.name")?;
        } else {
            let repo = self
                .repo
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("当前目录不是 Git 仓库"))?;
            let mut config = repo.config().context("无法打开仓库配置")?;
            config
                .set_str("user.name", name)
                .context("无法设置 user.name")?;
        }
        Ok(())
    }

    /// 设置邮箱
    pub fn set_user_email(&self, email: &str, global: bool) -> Result<()> {
        if global {
            let mut config = GitConfig::open_default().context("无法打开全局 Git 配置")?;
            config
                .set_str("user.email", email)
                .context("无法设置 user.email")?;
        } else {
            let repo = self
                .repo
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("当前目录不是 Git 仓库"))?;
            let mut config = repo.config().context("无法打开仓库配置")?;
            config
                .set_str("user.email", email)
                .context("无法设置 user.email")?;
        }
        Ok(())
    }

    /// 设置 GPG 签名密钥
    pub fn set_signing_key(&self, key: &str, global: bool) -> Result<()> {
        if global {
            let mut config = GitConfig::open_default().context("无法打开全局 Git 配置")?;
            config
                .set_str("user.signingkey", key)
                .context("无法设置 user.signingkey")?;
        } else {
            let repo = self
                .repo
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("当前目录不是 Git 仓库"))?;
            let mut config = repo.config().context("无法打开仓库配置")?;
            config
                .set_str("user.signingkey", key)
                .context("无法设置 user.signingkey")?;
        }
        Ok(())
    }

    /// 启用/禁用 GPG 签名
    pub fn set_gpg_sign(&self, enabled: bool, global: bool) -> Result<()> {
        if global {
            let mut config = GitConfig::open_default().context("无法打开全局 Git 配置")?;
            config
                .set_bool("commit.gpgsign", enabled)
                .context("无法设置 commit.gpgsign")?;
        } else {
            let repo = self
                .repo
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("当前目录不是 Git 仓库"))?;
            let mut config = repo.config().context("无法打开仓库配置")?;
            config
                .set_bool("commit.gpgsign", enabled)
                .context("无法设置 commit.gpgsign")?;
        }
        Ok(())
    }

    /// 获取当前用户名
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

    /// 获取当前邮箱
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

    /// 获取有效的用户名（优先本地，然后全局）
    pub fn get_effective_user_name(&self) -> Option<String> {
        self.get_user_name(false)
            .or_else(|| self.get_user_name(true))
    }

    /// 获取有效的邮箱（优先本地，然后全局）
    pub fn get_effective_user_email(&self) -> Option<String> {
        self.get_user_email(false)
            .or_else(|| self.get_user_email(true))
    }

    /// 获取 origin remote URL
    pub fn get_origin_url(&self) -> Option<String> {
        let repo = self.repo.as_ref()?;
        let remote = repo.find_remote("origin").ok()?;
        remote.url().map(|s| s.to_string())
    }

    /// 获取提交历史
    pub fn get_commits(&self, max_count: usize) -> Result<Vec<CommitInfo>> {
        let repo = self
            .repo
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("当前目录不是 Git 仓库"))?;

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

/// 提交信息
#[derive(Debug)]
pub struct CommitInfo {
    pub id: String,
    pub message: String,
    pub author_name: String,
    pub author_email: String,
}
