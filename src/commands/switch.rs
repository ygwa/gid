use anyhow::Result;
use colored::Colorize;

use crate::config::Config;
use crate::git::GitConfigManager;
use crate::ssh::SshManager;

/// 切换身份
pub fn execute(identity_id: &str, global: bool) -> Result<()> {
    let config = Config::load()?;

    // 查找身份
    let identity = config
        .find_identity(identity_id)
        .ok_or_else(|| anyhow::anyhow!("找不到身份 '{identity_id}'"))?;

    let git = GitConfigManager::new()?;

    // 非全局模式需要在 Git 仓库中
    if !global && !git.is_in_repo() {
        anyhow::bail!("当前目录不是 Git 仓库。使用 --global 进行全局切换");
    }

    // 设置 Git 配置
    git.set_user_name(&identity.name, global)?;
    git.set_user_email(&identity.email, global)?;

    // 设置 GPG 签名
    if let Some(ref gpg_key) = identity.gpg_key {
        git.set_signing_key(gpg_key, global)?;
        git.set_gpg_sign(identity.gpg_sign, global)?;
    }

    // 配置 SSH（如果有）
    if let Some(ref ssh_key) = identity.ssh_key {
        let ssh = SshManager::new()?;
        if ssh.key_exists(ssh_key) {
            // 为常见的 Git 托管服务配置 SSH
            let hosts = ["github.com", "gitlab.com", "bitbucket.org"];
            for host in hosts {
                if let Err(e) = ssh.configure_for_identity(identity_id, host, ssh_key) {
                    eprintln!("{} 配置 SSH 失败 ({}): {}", "!".yellow(), host, e);
                }
            }
        }
    }

    // 输出结果
    let scope = if global { "全局" } else { "项目" };
    println!(
        "{} 已切换到{}身份: {} {} <{}>",
        "✓".green(),
        scope,
        format!("[{}]", identity.id).green().bold(),
        identity.name,
        identity.email.cyan()
    );

    if let Some(ref desc) = identity.description {
        println!("  {}", desc.dimmed());
    }

    if identity.ssh_key.is_some() {
        println!("  {} SSH 密钥已配置", "🔑".dimmed());
    }

    if identity.gpg_key.is_some() {
        println!("  {} GPG 签名已启用", "🔏".dimmed());
    }

    Ok(())
}
